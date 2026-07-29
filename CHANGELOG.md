# Changelog

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
