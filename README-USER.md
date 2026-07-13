# Bio-OM Expert 内容运营工作台

面向生物科技行业的 AI 内容运营桌面工具。自然语言输入，自动编排研究→报告→推文→视频脚本全流程。

## 安装要求

| 依赖 | 用途 | 安装方式 |
|------|------|----------|
| **Node.js** ≥18 | Claude CLI 运行环境 | [nodejs.org](https://nodejs.org/) 下载安装 |
| **Claude Code CLI** | 执行 skill 的 AI 引擎 | `npm install -g @anthropic-ai/claude-code` |
| **Python3** | Word 文档生成 | macOS 自带，Windows 需下载 |
| **python-docx** | .docx 格式支持 | `pip3 install python-docx` |

## 安装步骤

### 1. 安装 Node.js

从 [nodejs.org](https://nodejs.org/) 下载并安装（选择 LTS 版本）。

### 2. 安装 Claude Code CLI

```bash
npm install -g @anthropic-ai/claude-code
```

配置 API Key：
```bash
export ANTHROPIC_API_KEY="sk-..."
```
建议写入 `~/.zshrc` 或 `~/.bashrc` 永久生效。

### 3. 安装 Bio-OM Expert 应用

**macOS:**
- 下载 `Bio-OM Expert_1.0.0_aarch64.dmg`
- 双击打开，拖入 Applications 文件夹

**Windows:**
- 下载 `Bio-OM Expert_1.0.0_x64.msi`
- 双击安装

### 4. 安装 Skills

```bash
cd /Applications/Bio-OM\ Expert.app/Contents/Resources/
bash setup.sh
```

或从 bio-om-expert 项目目录运行：
```bash
bash setup.sh
```

## 使用方式

1. 打开 Bio-OM Expert
2. 输入需求，如"写一篇关于 SOD 抗氧化机制的科普推文"
3. 系统自动识别意图、编排 skill、执行全流程
4. 右侧预览面板查看产出文件
5. 不满意？输入"太技术了，改通俗一点"自动修改

## 常见问题

**Q: 点运行没反应？**
确认 Claude Code CLI 已安装：`claude --version`

**Q: 提示 API Key 错误？**
确认环境变量 `ANTHROPIC_API_KEY` 已设置

**Q: Word 文档没生成？**
确认 Python3 和 python-docx 已安装：`pip3 install python-docx`

## 版本

v1.0.0 — 2026-07-13
