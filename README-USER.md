# Bio-OM Expert 内容运营工作台

面向生物科技行业的 AI 内容运营桌面工具。自然语言输入，自动编排研究→报告→推文→视频脚本全流程。

## 版本

**v1.3.0** — 2026-07-30

## 安装要求

| 依赖 | 用途 | 安装方式 |
|------|------|----------|
| **Node.js** ≥18 | Claude CLI 运行环境 | [nodejs.org](https://nodejs.org/) 下载 LTS 版本 |
| **Claude Code CLI** | 执行 Skill 的 AI 引擎 | `npm install -g @anthropic-ai/claude-code` |
| **Python3** | Word 文档生成 | macOS 自带; Windows 从 [python.org](https://python.org/) 下载 |
| **python-docx** | .docx 格式支持 | `pip3 install python-docx` |
| **Anthropic API Key** | Claude API 鉴权 | 控制台获取 `sk-ant-...` 密钥 |

> ⚠️ **重要**: Claude Code CLI 是必要依赖。Bio-OM Expert 通过本机 Claude Code 命令行工具执行 AI Skills，不内置 API 客户端。

## 快速安装

### 方法一：使用安装脚本（推荐）

```bash
# macOS / Linux
cd Bio-OM-Expert_v1.3.0_完整安装包/
bash setup.sh
```

```cmd
REM Windows — 右键 "以管理员身份运行" 或直接在 cmd 中执行
cd Bio-OM-Expert_v1.3.0_完整安装包
setup.bat
```

脚本会自动检查并安装 Node.js、Claude CLI、Python3、python-docx 和 5 个 Skills。

### 方法二：手动安装

#### macOS

1. 双击 `Bio-OM Expert_1.3.0_aarch64.dmg` 打开安装镜像
2. 将 `Bio-OM Expert.app` 拖入 `Applications` 文件夹
3. 首次打开时，如果提示"无法验证开发者":
   - 打开 **系统设置 → 隐私与安全性**
   - 找到 "Bio-OM Expert" 并点击 **"仍要打开"**
4. 运行安装脚本安装 Skills:
   ```bash
   bash /Volumes/Bio-OM\ Expert/setup.sh
   ```

#### Windows

1. 双击 `Bio-OM Expert_1.3.0_x64_zh-CN.msi` 启动安装
2. 按照安装向导完成安装
3. 安装完成后，运行 `setup.bat` 安装 Skills:
   ```cmd
   cd "C:\Program Files\Bio-OM Expert"
   setup.bat
   ```

## 配置 API Key

```bash
# macOS / Linux — 写入 shell 配置文件
echo 'export ANTHROPIC_API_KEY="sk-ant-..."' >> ~/.zshrc
source ~/.zshrc
```

```cmd
REM Windows — 设置永久环境变量
setx ANTHROPIC_API_KEY "sk-ant-..."
```

## 使用方式

1. 打开 Bio-OM Expert
2. 输入需求，如"写一篇关于 SOD 抗氧化机制的科普推文"
3. 系统自动识别意图、编排 Skill、执行全流程
4. 右侧预览面板查看产出文件
5. **📊 仪表盘** — 点击右上角按钮进入全屏仪表盘，浏览所有历史产出，支持搜索、分类筛选、全文预览
6. 不满意？输入"太技术了，改通俗一点"自动修改

## 目录结构

```
~/.bio-om-expert/
├── output/         ← 所有产出文件（永久存储）
│   ├── report/     ← 研究报告
│   ├── article/    ← 推文草稿
│   ├── script/     ← 视频脚本
│   └── image/      ← 配图建议
└── skills/         ← 用户自定义 Skills
```

## v1.3.0 新功能

- **🖥️ 资产仪表盘** — 全屏暗色主题，实时扫描所有产出文件，统计分类、搜索过滤、展开预览、双击沉浸阅读
- **🐛 修复** — 仪表盘中文字符崩溃、文件时间戳异常、浮点数精度、修改任务卡死
- **📂 永久存储** — 输出目录从系统临时目录改为 `~/.bio-om-expert/output/`，不再丢失文件

## 常见问题

**Q: 点运行没反应？**
确认 Claude Code CLI 已安装: `claude --version`

**Q: 提示 API Key 错误？**
确认环境变量 `ANTHROPIC_API_KEY` 已设置，注意 macOS 应用从 GUI 启动时不会读取 `.zshrc`，建议用 `launchctl setenv` 或设置系统级环境变量。

**Q: macOS 提示应用"已损坏"？**
这是 macOS Gatekeeper 对非 App Store 应用的限制，运行:
```bash
xattr -cr /Applications/Bio-OM\ Expert.app
```

**Q: Word 文档没生成？**
确认 Python3 和 python-docx: `pip3 install python-docx`

**Q: 仪表盘显示空白？**
确认 `~/.bio-om-expert/output/` 目录存在且有产出文件。首次使用需至少运行一次任务。

---

v1.3.0 — 2026-07-30
