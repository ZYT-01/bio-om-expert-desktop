"""
Report Generator · 研究报告生成器
接收研究技能输出的原始数据，按标准模板生成 Markdown 格式研究报告。

用途: report-generator 技能 · 核心脚本
输入: JSON 原始数据 + 报告主题 + 来源类型
输出: Markdown 报告文件路径
"""

import json
import sys
import os
from datetime import datetime


def load_template(source_type: str) -> str:
    """加载对应类型的报告模板"""
    template_map = {
        "web": "../templates/research_report_template.md",
        "url": "../templates/url_research_report_template.md",
        "local": "../templates/local_research_report_template.md",
    }
    template_path = template_map.get(source_type, template_map["web"])

    # 尝试从本技能模板加载，失败则从对应研究技能模板加载
    local_template = f"../templates/{source_type}_report_template.md"
    try:
        if os.path.exists(local_template):
            with open(local_template, "r", encoding="utf-8") as f:
                return f.read()
    except Exception:
        pass

    try:
        with open(template_path, "r", encoding="utf-8") as f:
            return f.read()
    except FileNotFoundError:
        return _default_template()


def _default_template() -> str:
    """内置最小模板（兜底）"""
    return """# {topic} · 研究报告

**报告类型**：{source_type_label}
**生成时间**：{generated_at}
**信息来源**：{sources_count} 个

## 执行摘要
{executive_summary}

## 一、信息来源
{sources_section}

## 二、核心发现
{findings_section}

## 三、关键数据汇总
{data_table}

## 四、归纳总结
{conclusions_section}

## 附录
{appendix_section}
"""


def fill_template(template: str, data: dict) -> str:
    """
    将数据填入模板占位符。
    替换 {变量名} 格式的占位符。
    """
    # 基础替换
    replacements = {
        "{topic}": data.get("topic", ""),
        "{source_type_label}": data.get("source_type_label", ""),
        "{generated_at}": datetime.now().strftime("%Y-%m-%d %H:%M"),
        "{sources_count}": str(data.get("sources_count", 0)),
        "{executive_summary}": data.get("executive_summary", "(待生成)"),
        "{sources_section}": data.get("sources_section", "(待生成)"),
        "{findings_section}": data.get("findings_section", "(待生成)"),
        "{data_table}": data.get("data_table", "(待生成)"),
        "{conclusions_section}": data.get("conclusions_section", "(待生成)"),
        "{appendix_section}": data.get("appendix_section", "(待生成)"),
        "{YYYY-MM-DD HH:mm}": datetime.now().strftime("%Y-%m-%d %H:%M"),
        "{YYYY-MM-DD}": datetime.now().strftime("%Y-%m-%d"),
    }

    report = template
    for placeholder, value in replacements.items():
        report = report.replace(placeholder, value)

    return report


def save_report(report: str, topic: str, output_dir: str = "../../output/reports") -> str:
    """
    保存报告到文件。

    Args:
        report: Markdown 报告内容
        topic: 报告主题（用于生成文件名）
        output_dir: 输出目录

    Returns:
        报告文件路径
    """
    # 确保输出目录存在
    os.makedirs(output_dir, exist_ok=True)

    # 生成文件名: YYYY-MM-DD_{topic}_report.md
    date_str = datetime.now().strftime("%Y-%m-%d")
    safe_topic = topic.replace("/", "-").replace("\\", "-")[:100]
    filename = f"{date_str}_{safe_topic}_report.md"
    filepath = os.path.join(output_dir, filename)

    with open(filepath, "w", encoding="utf-8") as f:
        f.write(report)

    return filepath


def generate(raw_data: dict, topic: str, source_type: str = "web") -> str:
    """
    主入口：生成研究报告。

    Args:
        raw_data: 研究技能输出的原始数据
        topic: 报告主题
        source_type: 来源类型 (web/url/local)

    Returns:
        生成的报告文件路径
    """
    source_type_labels = {
        "web": "全网搜索研究",
        "url": "URL来源分析",
        "local": "本地文档分析",
    }

    # 加载模板
    template = load_template(source_type)

    # 准备模板填充数据
    fill_data = {
        "topic": topic,
        "source_type_label": source_type_labels.get(source_type, source_type),
        "sources_count": raw_data.get("sources_count", 0),
        "executive_summary": raw_data.get("executive_summary", "(待生成)"),
        "sources_section": raw_data.get("sources_section", "(待生成)"),
        "findings_section": raw_data.get("findings_section", "(待生成)"),
        "data_table": raw_data.get("data_table", "(待生成)"),
        "conclusions_section": raw_data.get("conclusions_section", "(待生成)"),
        "appendix_section": raw_data.get("appendix_section", "(待生成)"),
    }

    # 填充模板
    report = fill_template(template, fill_data)

    # 保存报告
    filepath = save_report(report, topic)

    return filepath


if __name__ == "__main__":
    data_file = sys.argv[1] if len(sys.argv) > 1 else ""
    topic = sys.argv[2] if len(sys.argv) > 2 else "未命名报告"
    source_type = sys.argv[3] if len(sys.argv) > 3 else "web"

    with open(data_file, "r", encoding="utf-8") as f:
        raw_data = json.load(f)

    report_path = generate(raw_data, topic, source_type)
    print(json.dumps({"report_path": report_path}, ensure_ascii=False))
