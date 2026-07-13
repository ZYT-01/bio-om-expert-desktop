# Skill: local-research（本地文档分析研究）

## 用途
对指定目录内的所有可读文档（Markdown、PDF、Office文档、TXT等）进行批量信息提取、整理与归纲，生成结构化研究报告。

## 触发条件
- 用户输入：`/local-research --dir "/path/to/folder"`
- 示例：`/local-research --dir "./project_materials"`

## 输入字段
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| directory | string | 是 | 待扫描的本地目录路径 |
| recursive | boolean | 否 | 是否递归遍历子目录（默认true） |
| file_types | array | 否 | 限定扫描的文件类型，如 `[".md", ".pdf"]` |

## 执行步骤
1. **文件扫描**：递归遍历目录，生成文件清单，记录文件元数据（大小、修改时间）
2. **格式识别**：按扩展名识别文件类型，匹配对应的解析器
3. **内容提取**：调用对应解析脚本提取文本内容
   - `.md/.txt`：直接读取
   - `.pdf`：调用 `scripts/pdf_extractor.py`
   - `.docx/.xlsx/.pptx`：调用 `scripts/office_parser.py`
   - 其他格式：尝试通用提取，失败则跳过并记录日志
4. **内容聚合**：将所有提取的文本合并，构建文档语料库
5. **信息归纲**：按主题聚类，提炼关键信息（实体、数据、观点）
6. **报告生成**：调用 `report-generator` 技能生成Markdown报告

## 输出字段
| 字段 | 类型 | 说明 |
|------|------|------|
| report_path | string | 报告文件路径 |
| total_files | int | 扫描文件总数 |
| parsed_files | int | 成功解析文件数 |
| skipped_files | array | 跳过文件列表（含原因） |
| clustered_topics | array | 归纲后的主题簇 |
| key_findings | array | 核心发现列表 |

## 依赖清单
- **scripts/**:
  - `doc_parser.py` — 文档解析主调度脚本
  - `pdf_extractor.py` — PDF文本提取
  - `office_parser.py` — Office文档解析
  - `file_indexer.py` — 文件索引生成
- **assets/**:
  - `supported_formats.json` — 支持的文件格式配置及对应解析器
- **templates/**:
  - `local_research_report_template.md` — 本地研究报告输出骨架
- **references/**:
  - `parsing_troubleshooting.md` — 解析问题排查指南
  - `encoding_guide.md` — 编码与字符集处理指南