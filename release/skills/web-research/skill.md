# Skill: web-research（全网搜索研究）

## 用途
根据给定的主题或商务目标，在全网范围内收集、整理、归纳相关信息，生成结构化研究报告。

## 触发条件
- 用户输入：`/web-research "主题"` 或 `/web-research "主题" --goal "商务目标"`
- 示例：`/web-research "AAV基因治疗商业化进展" --goal "评估行业竞争格局"`

## 输入字段
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| topic | string | 是 | 研究主题 |
| business_goal | string | 否 | 商务目标/研究目的 |
| depth | enum | 否 | 搜索深度：basic / standard / deep（默认standard） |

## 执行步骤
1. **关键词扩展**：调用 `scripts/keyword_expander.py` 基于主题生成搜索关键词列表
2. **多渠道搜索**：调用 `scripts/web_scraper.py` 对各大搜索引擎/平台进行搜索
   - 搜索引擎：百度、Google、Bing
   - 行业平台：生物谷、医药魔方、BioRxiv、Nature Biotechnology
   - 社交媒体：微信公众号（搜一搜）、知乎、微博
3. **内容解析**：调用 `scripts/content_parser.py` 提取正文、标题、发布时间、作者等
4. **去重与过滤**：按内容相似度去重，过滤低质量/无关内容
5. **信息归纲**：按主题聚类，提取核心观点、数据、案例
6. **报告生成**：调用 `report-generator` 技能生成Markdown报告

## 输出字段
| 字段 | 类型 | 说明 |
|------|------|------|
| report_path | string | 报告文件路径 |
| sources_count | int | 信息来源数量 |
| topics_clustered | array | 归纲后的主题簇列表 |
| key_findings | array | 核心发现列表 |

## 依赖清单
- **scripts/**:
  - `web_scraper.py` — 步骤2调用，执行网络爬取
  - `content_parser.py` — 步骤3调用，解析页面内容
  - `keyword_expander.py` — 步骤1调用，生成扩展关键词
- **assets/**:
  - `search_engines.json` — 配置搜索源和API
  - `industry_keywords.md` — 生物科技行业专业术语库
- **templates/**:
  - `research_report_template.md` — 报告输出骨架
- **references/**:
  - `search_strategy.md` — 搜索策略与技巧
  - `source_credibility.md` — 来源可信度评估方法