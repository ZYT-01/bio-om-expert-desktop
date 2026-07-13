"""
HTML Extractor · 结构化信息提取器
从HTML中提取正文、标题、关键数据、表格、列表、引用文献等。

用途: url-research 技能 · 步骤3
输入: list[dict{url, html}]
输出: list[dict{url, title, body, tables, lists, references, external_links}]
"""

import json
import sys
import re


def extract_title(html: str) -> str:
    """提取文章标题"""
    # TODO: 解析 <title> / <h1> / og:title / meta title
    return ""


def extract_main_body(html: str) -> str:
    """提取正文（去除导航、广告、侧栏等）"""
    # TODO: 使用 readability/trafilatura 提取正文
    return ""


def extract_tables(html: str) -> list[dict]:
    """提取HTML表格为结构化数据"""
    # TODO: 解析 <table> 标签为 list[dict]
    return []


def extract_lists(html: str) -> list[dict]:
    """提取结构化列表（<ul>/<ol>/<dl>）"""
    # TODO: 解析列表标签
    return []


def extract_references(html: str) -> list[dict]:
    """提取引用文献"""
    # TODO: 解析引用区块，识别DOI/PMID/标准引用格式
    return []


def extract_external_links(html: str, base_url: str) -> list[str]:
    """提取外部链接"""
    # TODO: 解析 <a href="...">，过滤同域链接
    return []


def extract_key_data_points(text: str) -> list[dict]:
    """
    提取关键数据点：
    - 数字+单位（如 1.2×10^6 vg/mL）
    - 百分比
    - 金额
    - 日期
    """
    # TODO: 基于正则 + 领域词典提取结构化数据
    return []


def extract(html_pages: list[dict]) -> list[dict]:
    """
    批量提取结构化信息。
    """
    extracted = []
    for page in html_pages:
        html = page.get("html", "")
        url = page.get("url", "")

        if not html:
            extracted.append({"url": url, "error": "no_html_content"})
            continue

        entry = {
            "url": url,
            "title": extract_title(html),
            "body": extract_main_body(html),
            "tables": extract_tables(html),
            "lists": extract_lists(html),
            "references": extract_references(html),
            "external_links": extract_external_links(html, url),
            "key_data_points": extract_key_data_points(extract_main_body(html)),
        }
        extracted.append(entry)

    return extracted


if __name__ == "__main__":
    input_data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else []
    result = extract(input_data)
    print(json.dumps(result, ensure_ascii=False, indent=2))
