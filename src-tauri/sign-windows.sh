#!/bin/bash
# Bio-OM Expert - Windows MSI/EXE 签名脚本
# 使用 osslsigncode 在 macOS 上对 Windows 安装包进行 Authenticode 签名
# 用法: bash sign-windows.sh <待签名文件路径>
#
# 正式使用时：
#   1. 将正式证书的 cert.pem 和 key.pem 放入 certs/ 目录
#   2. 或设置环境变量 WINDOWS_CERT_PATH / WINDOWS_KEY_PATH

set -e

TARGET_FILE="$1"

if [ -z "$TARGET_FILE" ] || [ ! -f "$TARGET_FILE" ]; then
    echo "[签名] ❌ 文件不存在: ${TARGET_FILE:-<未指定>}"
    echo "[签名] 用法: bash sign-windows.sh <待签名文件路径>"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CERT_DIR="$SCRIPT_DIR/../certs"

# 优先级: 环境变量 > certs/ 目录 > 测试证书
if [ -n "$WINDOWS_CERT_PATH" ] && [ -n "$WINDOWS_KEY_PATH" ]; then
    CERT="$WINDOWS_CERT_PATH"
    KEY="$WINDOWS_KEY_PATH"
    echo "[签名] 使用环境变量指定的证书"
elif [ -f "$CERT_DIR/cert.pem" ] && [ -f "$CERT_DIR/key.pem" ]; then
    CERT="$CERT_DIR/cert.pem"
    KEY="$CERT_DIR/key.pem"
    echo "[签名] 使用 certs/ 目录中的正式证书"
elif [ -f "$CERT_DIR/test-cert.pem" ] && [ -f "$CERT_DIR/test-key.pem" ]; then
    CERT="$CERT_DIR/test-cert.pem"
    KEY="$CERT_DIR/test-key.pem"
    echo "[签名] ⚠️ 使用测试自签名证书 (仅用于本地测试)"
else
    echo "[签名] ❌ 未找到任何证书文件"
    echo "[签名] 请在 certs/ 目录放入 cert.pem 和 key.pem"
    echo "[签名] 或设置 WINDOWS_CERT_PATH / WINDOWS_KEY_PATH 环境变量"
    exit 1
fi

# 检查 osslsigncode
if ! command -v osslsigncode &>/dev/null; then
    echo "[签名] ❌ osslsigncode 未安装"
    echo "[签名] 安装: brew install osslsigncode"
    exit 1
fi

echo "[签名] 文件: $TARGET_FILE"
echo "[签名] 证书: $CERT"
echo "[签名] 开始签名..."

osslsigncode sign \
    -certs "$CERT" \
    -key "$KEY" \
    -h sha256 \
    -t http://timestamp.digicert.com \
    -in "$TARGET_FILE" \
    -out "${TARGET_FILE}.signed"

mv "${TARGET_FILE}.signed" "$TARGET_FILE"

echo "[签名] ✅ 完成: $TARGET_FILE"

# 可选验证
if [ "${2:-}" = "--verify" ]; then
    echo ""
    echo "[签名] 验证签名结果:"
    osslsigncode verify "$TARGET_FILE" 2>&1 || true
fi
