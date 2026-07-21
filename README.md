# Bio-OM Expert — 内容运营工作台

面向生物科技内容运营团队的桌面应用。输入自然语言需求，自动编排多个 AI Skills 完成内容生产全流程：研究 → 报告 → 推文 → 视频脚本。

## 功能

- **自然语言编排** — 输入"写一篇 SOD 抗氧化机制科普推文"，系统自动识别意图并编排 Skills
- **5 个内置 Skills** — 全网搜索、链接分析、本地资料查询、报告生成、文案撰写
- **流式进度展示** — 实时查看每个 Skill 的执行进度
- **Markdown 预览** — 富文本查看产出内容，支持源码切换
- **对话式迭代** — 产出后可自然语言修改："太技术了，改通俗一点"
- **Word 文档导出** — 自动生成合集 Word 文档

## 安装

从 [GitHub Releases](https://github.com/ZYT-01/bio-om-expert-desktop/releases) 下载最新版本：

- **macOS**: 下载 `.dmg` 文件，双击安装
- **Windows**: 下载 `.msi` 文件，运行安装

### 前置依赖

| 依赖 | 用途 | 安装方式 |
|------|------|---------|
| Claude Code CLI | Skills 运行时 | `npm install -g @anthropic-ai/claude-code` |
| Python 3 | Word 文档生成 | macOS 已内置；Windows 从 python.org 下载 |
| Node.js | Claude Code 依赖 | 从 nodejs.org 下载 |

启动应用后自动检测前置依赖是否就绪。

## 添加自定义 Skill

Bio-OM Expert 支持通过 JSON 清单文件添加新的 Skill。详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

快速开始：
```bash
# 1. 创建新 Skill 模板
./bin/bio-om-skill new "活动策划"

# 2. 编辑生成的 JSON 文件，填写触发词和调用参数
# 3. 验证格式是否正确
./bin/bio-om-skill validate ~/.bio-om-expert/skills/活动策划.json

# 4. 启动应用 — Skill 自动生效
```

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

## 文档

- [贡献指南](CONTRIBUTING.md) — 如何添加 Skill 或贡献代码
- [Skill 清单格式](skills-manifest/schema.json) — JSON Schema 定义
- [CLI 工具](bin/bio-om-skill) — Skill 脚手架和验证工具
