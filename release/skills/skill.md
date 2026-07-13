# Skills · 技能总览说明书

## 技能能力矩阵

| 技能 | 触发命令 | 输入 | 输出 | 调用场景 |
|------|----------|------|------|----------|
| **web-research** | /web-research | 主题 + 商务目标（可选） | 研究报告 (Markdown) | 需要全网范围信息收集时 |
| **url-research** | /url-research | 主题 + URL列表 或 Excel文件 | 研究报告 (Markdown) | 已有明确来源链接或Excel记录时 |
| **local-research** | /local-research | 指定目录路径 | 研究报告 (Markdown) | 本地已有大量文档需整合时 |
| **report-generator** | /report-generator | 原始数据 + 模板 | 格式化报告 (Markdown) | 三个研究技能完成数据采集后调用 |
| **content-writing** | **/content-writing** | **研究报告 + 选题** | **推文草稿 + 视频脚本 (Markdown)** | **研究报告完成后进行内容创作时** |

### url-research 调用方式（v1.3 更新）

| 方式 | 命令示例 | 场景 |
|------|----------|------|
| 手动URL列表 | `/url-research "CRISPR临床进展" --urls "url1,url2"` | 临时分析少量链接 |
| Excel批量 | `/url-research --xlsx "URL.xlsx"` | 加载Excel中所有记录批量处理 |
| Excel+筛选 | `/url-research --xlsx "URL.xlsx" --topic "AAV空壳率"` | 仅分析Excel中匹配主题的记录 |
| Excel+Sheet | `/url-research --xlsx "URL.xlsx" --sheet "基因治疗"` | 指定Sheet页 |

## 技能目录映射

skills/
├── web-research/ # 全网搜索研究
├── url-research/ # 指定链接分析（支持 Excel 批量加载）
├── local-research/ # 本地文档分析
├── report-generator/ # 报告生成（共用）
└── content-writing/ # 文案撰写与视频脚本


## 调用优先级
1. 若用户提供Excel文件 → 优先使用 `url-research --xlsx`（批量模式）
2. 若用户提供URL列表 → 使用 `url-research`
3. 若用户指定本地目录 → 使用 `local-research`
4. 若仅有主题/目标 → 使用 `web-research`
5. 所有研究技能完成后 → 统一调用 `report-generator` 生成最终报告
6. 研究报告生成后 → 调用 `content-writing` 撰写推文和视频脚本

## 技能间数据流
```
URL.xlsx ──→ url-research ──┐
web-research ───────────────┤
                             ├──→ 原始数据 ──→ report-generator ──→ 研究报告 ──→ content-writing ──→ 推文 + 视频脚本
url-research (手动URL) ─────┤
                             │
local-research ──────────────┘
```
