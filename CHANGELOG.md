# Changelog

## [1.3.1] — 2026-08-03

### Added
- **5 个真正的 Claude Code Skill 实现文件** — 不再仅有 JSON 编排清单，Claude Code 可直接发现并执行
  - `web-research`：全网搜索研究，生成结构化研究报告
  - `url-research`：链接分析研究，按 URL 分节分析
  - `local-research`：本地文档分析，支持 PDF/Word/Excel
  - `report-generator`：报告整合，统一格式化已有研究素材
  - `content-writing`：文案撰写与视频脚本，产出 9 种文件类型
- **4 个内容创作智能体角色**（content-writing 内）：内容架构师、技术转译者、标题打磨师、视听转化师
- **`CLAUDE.md` 全局配置** — 文件命名约定，确保产出符合仪表盘分类要求
- **setup.sh 第 6 步**：自动挂载 DMG → 复制 App → `xattr -cr` 清除 Gatekeeper 隔离属性
- **setup.sh 第 7 步**：CLAUDE.md 安装
- **setup.sh Skills 双目录安装**：JSON → `~/Library/Application Support/`；SKILL.md → `~/.claude/skills/`
- **`LSEnvironment` PATH 注入** — Info.plist 中配置 PATH，解决 GUI 启动找不到 `claude` 的问题
- `rename_to_chinese` 新增 `research_report.md` → `研究报告.md` 映射
- `ExecutionStep` 新增 `description` 和 `produces` 字段
- **仪表盘 `.json` 文件扫描** — `scan_dashboard` 同时扫描 `.md` 和 `.json`
- content-writing 同步输出 `image_suggestions.md`（Markdown）+ `配图建议.json`

### Fixed
- **仪表盘"研究报告"始终为 0** — 文件名含中文时无法匹配英文关键字。修复：增强中文关键字检测
- **仪表盘"配图建议"始终为 0** — `.json` 文件被跳过。修复：支持 `.json` 扫描
- **仪表盘分类缺失** — `interaction_design_video` 等文件名无法匹配。修复：提取 `classify_asset` 独立函数，添加 `video` 等关键字，移除父目录路径泄漏
- **研究报告重复输出** — Claude 同时输出中英文版本。修复：CLAUDE.md 限定只输出英文文件名
- **Skill 调用错位** — prompt 仅传 "运行 X skill" 导致 Claude 降级使用已有 skill。修复：embed skill 完整描述到 prompt
- **Orchestrator prompt 信息不足** — 修复：构造富 prompt，含 skill 描述 + 预期产出 + 依赖规则
- **setup.sh Skills 安装路径错误** — JSON 装到 `~/.claude/skills/` 但 App 读取路径不同
- **GUI App 找不到 `claude`** — 修复：Info.plist `LSEnvironment` 注入 PATH
- **编译后 App 白屏** — 改用 `npx tauri build` 完整打包
- **二进制替换后代码签名失效** — 修复：`codesign --force --deep --sign -` 重签

### Changed
- `scan_dashboard` 分类逻辑重写为独立 `classify_asset` 函数，只检查文件名不含父目录路径
- `run_pipeline` prompt 增强：嵌入 skill 描述 + 预期产出 + 执行角色
- `orchestrate` 关键词路径构造富 prompt 替代 "运行 X skill"
- Skill 输出文件名策略：保留英文关键字供仪表盘分类识别
- Info.plist：版本号 → 1.3.1，新增 `LSEnvironment` 字典

## [1.3.0] — 2026-07-29

### Added
- **资产仪表盘** — 全屏暗色主题仪表盘，实时扫描 `~/.bio-om-expert/output/` 所有产出文件
  - 统计栏：按报告/推文/脚本/配图分类计数，点击筛选
  - 卡片网格：标题、摘要、大小、日期，展开预览全文
  - 沉浸阅读器：双击卡片进入全屏 Markdown 阅读
  - 搜索过滤、刷新同步、Escape 键关闭
- 新增 `scan_dashboard` 和 `get_base_output_dir` Rust 命令

### Fixed
- 仪表盘崩溃：中文正文按字节切片改为按字符截断
- 仪表盘崩溃：异常文件时间戳改为 UTC 安全转换
- 仪表盘崩溃：文件扫描加 200 上限保护
- 文件大小浮点数显示精度修复

## [1.2.0] — 2026-07-29

### Added
- 输出目录改为永久存储 (`~/.bio-om-expert/output/`)，不再因 macOS 清理 `/tmp` 丢失文件
- 新增 Rust 命令 `check_path_exists`，打开文件夹前验证目录是否存在
- 预览面板新增空状态引导：无可预览文件时显示「打开文件夹」按钮
- 关闭预览后可通过底部 `📄 预览` 按钮重新打开
- 预览面板宽度改为弹性布局 (`max-width: 42vw`)，避免窄窗口遮挡主内容

### Fixed
- **关键修复**: 修改任务 (`🔧 修改`) 完成后进程不结束 — `doneFiredRef` 未重置导致 `skill-done` 事件被误拦截
- **关键修复**: 预览面板 X 按钮无效 — 改为控制 `previewVisible` 状态真正关闭面板
- **关键修复**: 「全部文件」按钮静默失败 — 现在先检查目录存在性，清理后显示红色错误提示
- 历史记录选择时正确重置预览状态，避免旧文件残留
- `loadPreviewFiles` 静默吞错 → 改为 console.error 输出，方便调试

### Changed
- `open_output_folder` 命令增加路径存在性校验，目录不存在时返回错误

## [1.1.0] — 2026-07-22

### Added
- macOS 代码签名支持 (Entitlements.plist, 签名脚本)
- Windows WiX 安装器启动条件检查

### Fixed
- IME 输入法兼容 (compositionStart/End 跨浏览器兼容)
- 取消按钮立即可用 + 输出立即停止
- React StrictMode 重复输出修复
- stderr 去重输出

## [1.0.0] — 2026-07-13

- 初始版本：Go/No-Go Gate、Orchestrator、Pipeline 多 Skill 串行执行、流式输出、Word 文档生成、历史记录、对话式迭代修改、Markdown 预览面板
