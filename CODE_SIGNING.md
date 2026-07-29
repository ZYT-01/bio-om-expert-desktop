# Bio-OM Expert 代码签名指南

> 最后更新: 2026-07-24 | 当前环境: macOS (Apple Silicon)

---

## 当前状态

| 平台 | 已配置 | 证书状态 | 工具 |
|------|--------|---------|------|
| macOS | ✅ Hardened Runtime + Entitlements.plist | ❌ 无 Developer ID 证书 | codesign + notarytool |
| Windows | ⚠️ 测试证书 + signCommand 脚本 | ⚠️ 仅有自签名测试证书 | osslsigncode (需安装) |

**已变更文件：**
- `src-tauri/Entitlements.plist` — macOS Hardened Runtime 权限
- `src-tauri/certs/test-cert.pem` — 测试用自签名证书
- `src-tauri/certs/test-cert.pfx` — PKCS12 格式测试证书 (密码: bioom123)
- `src-tauri/tauri.conf.json` — 已添加 `entitlements` 配置

---

## macOS: 无证书方案（当前可用）

不需要 $99/年，最小配置也能减少用户端警告：

```json
// tauri.conf.json 已配置
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "11.0",
      "entitlements": "Entitlements.plist"
    }
  }
}
```

`Entitlements.plist` 启用了：
- JIT 编译（WebView 需要）
- 网络访问（联网调用 Claude API）
- 文件读写（产出物保存）
- 动态库验证豁免（Tauri/WebView 兼容）

**效果：** 用户仍会看到「无法验证开发者」提示，但右键→打开后应用正常运行，不会出现「已损坏」错误。

**获取正式证书后**，只需要在 `tauri.conf.json` 中加一行：
```json
"signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
```
然后执行构建 + 公证即可。

---

## macOS: 正式签名（有证书后）

```bash
# 1. 安装证书
security import dev-id.p12 -k ~/Library/Keychains/login.keychain -P "密码"

# 2. 存公证凭据（一次性）
xcrun notarytool store-credentials "BioOMNotary" \
  --apple-id your@email.com \
  --team-id YOUR_TEAM_ID \
  --password "@keychain:AC_PASSWORD"

# 3. 构建 + 签名（tauri.conf.json 已配置 signingIdentity）
npx tauri build --bundles dmg

# 4. 公证 + 装订
xcrun notarytool submit target/release/bundle/dmg/*.dmg \
  --keychain-profile BioOMNotary --wait
xcrun stapler staple target/release/bundle/dmg/*.dmg
```

---

## Windows: 跨平台签名方案（macOS 上签名 Windows 包）

### 安装 osslsigncode
```bash
brew install osslsigncode
```

### 配置 signCommand

在 `tauri.conf.json` → `bundle.windows` 中添加：

```json
"signCommand": "bash src-tauri/sign-windows.sh %1"
```

### 签名脚本 `src-tauri/sign-windows.sh`

```bash
#!/bin/bash
# 使用 osslsigncode 在 macOS 上签名 Windows PE/MSI 文件
# 用法: sign-windows.sh <待签名文件路径>

CERT_DIR="$(cd "$(dirname "$0")" && pwd)/certs"
CERT="$CERT_DIR/cert.pem"
KEY="$CERT_DIR/key.pem"

# 检查证书
if [ ! -f "$CERT" ] || [ ! -f "$KEY" ]; then
    echo "[签名] ⚠️ 证书文件不存在: $CERT / $KEY"
    echo "[签名] 请在 certs/ 目录放入正式的代码签名证书"
    echo "[签名] 当前使用自签名证书仅用于测试"
    CERT="$CERT_DIR/test-cert.pem"
    KEY="$CERT_DIR/test-key.pem"
fi

# 签名
osslsigncode sign \
    -certs "$CERT" \
    -key "$KEY" \
    -h sha256 \
    -t http://timestamp.digicert.com \
    -in "$1" \
    -out "${1}.signed"

# 替换原文件
mv "${1}.signed" "$1"
echo "[签名] ✅ 已签名: $1"
```

### 本地测试签名流程

```bash
# 1. 安装工具
brew install osslsigncode

# 2. 构建 MSI
npx tauri build --bundles msi

# 3. 手动验证签名（可选）
osslsigncode verify target/release/bundle/msi/*.msi
```

---

## GitHub Actions 自动签名

```yaml
# .github/workflows/release.yml
jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Import Apple cert
        env:
          APPLE_CERT_BASE64: ${{ secrets.APPLE_DEVELOPER_CERT_BASE64 }}
          APPLE_CERT_PWD: ${{ secrets.APPLE_CERT_PASSWORD }}
        run: |
          echo "$APPLE_CERT_BASE64" | base64 -d > cert.p12
          security import cert.p12 -k ~/Library/Keychains/login.keychain \
            -P "$APPLE_CERT_PWD" -T /usr/bin/codesign
      - name: Build & Sign DMG
        run: npx tauri build --bundles dmg
      - name: Notarize
        env:
          NOTARY_PWD: ${{ secrets.APPLE_NOTARY_PASSWORD }}
        run: |
          xcrun notarytool submit src-tauri/target/release/bundle/dmg/*.dmg \
            --apple-id "${{ secrets.APPLE_ID }}" \
            --team-id "${{ secrets.APPLE_TEAM_ID }}" \
            --password "$NOTARY_PWD" --wait

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Import Windows cert
        shell: pwsh
        run: |
          $cert = [Convert]::FromBase64String("${{ secrets.WINDOWS_CERT_BASE64 }}")
          [IO.File]::WriteAllBytes("cert.pfx", $cert)
      - name: Build & Sign MSI
        shell: pwsh
        env:
          CERT_PASSWORD: ${{ secrets.WINDOWS_CERT_PASSWORD }}
        run: npx tauri build --bundles msi
```

在 GitHub 仓库 Settings → Secrets 中配置 4 个密钥即可启用。

---

## 证书成本对比

| 方案 | 费用 | 用户效果 |
|------|------|---------|
| Hardened Runtime only（当前） | $0 | macOS: 右键打开即可；Windows: SmartScreen 警告 |
| Apple Developer ID | $99/年 | macOS: 无警告，正常安装 |
| Windows OV 证书 | $200-400/年 | Windows: SmartScreen 信任逐渐建立 |
| Windows EV 证书 | $400-600/年 | Windows: SmartScreen 立即信任 |
| Apple + EV 双证 | $500-700/年 | 双端零警告，企业级分发 |

---

## 下一步操作

1. [ ] `brew install osslsigncode` — 安装 Windows 签名工具
2. [ ] 将 `src-tauri/sign-windows.sh` 创建为可执行文件
3. [ ] 如已获得正式证书，在 `tauri.conf.json` 中添加 `signingIdentity` 和 `signCommand`
4. [ ] 将正式证书（.p12/.pfx）放入 `src-tauri/certs/` 目录
5. [ ] 在 GitHub 设置签名 Secrets 启用 CI 自动签名
