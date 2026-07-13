"""
Doc Parser · 文档解析主调度器
根据文件扩展名匹配对应的解析器，批量提取文档中的文本内容。

用途: local-research 技能 · 步骤2-3
输入: list[dict] 文件索引列表
输出: list[dict{path, text, metadata, parse_status, error}]
"""

import json
import sys
import os

# 子解析器导入（实际使用时取消注释）
# from pdf_extractor import extract_pdf
# from office_parser import extract_office


def extract_text_markdown(filepath: str) -> dict:
    """提取 Markdown/TXT 文件的文本"""
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            text = f.read()
        return {"text": text, "metadata": {"encoding": "utf-8"}}
    except UnicodeDecodeError:
        with open(filepath, "r", encoding="gbk") as f:
            text = f.read()
        return {"text": text, "metadata": {"encoding": "gbk"}}
    except Exception as e:
        return {"text": "", "metadata": {}, "error": str(e)}


def extract_text_pdf(filepath: str) -> dict:
    """提取 PDF 文本（委托 pdf_extractor.py）"""
    # TODO: 实际调用 pdf_extractor.extract_pdf(filepath)
    return {"text": f"[PDF内容待提取: {os.path.basename(filepath)}]", "metadata": {}}


def extract_text_office(filepath: str) -> dict:
    """提取 Office 文档文本（委托 office_parser.py）"""
    # TODO: 实际调用 office_parser.extract_office(filepath)
    return {"text": f"[Office文档内容待提取: {os.path.basename(filepath)}]", "metadata": {}}


def extract_text_generic(filepath: str) -> dict:
    """通用文本提取（尝试作为文本文件读取）"""
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            text = f.read()
        return {"text": text, "metadata": {"method": "generic_text"}}
    except Exception as e:
        return {"text": "", "metadata": {}, "error": f"Generic extraction failed: {e}"}


# 解析器路由表
PARSER_MAP = {
    "markdown": extract_text_markdown,
    "text": extract_text_markdown,
    "pdf": extract_text_pdf,
    "office": extract_text_office,
    "data": extract_text_markdown,  # .json/.csv 等仍由 markdown/text 函数覆盖
    "web": extract_text_markdown,   # .html 尝试文本读取
}


def parse_files(file_index: list[dict]) -> list[dict]:
    """
    根据文件索引批量解析文档。

    Args:
        file_index: file_indexer 输出的文件元数据列表

    Returns:
        解析结果列表，每项含 path, text, metadata, parse_status, error
    """
    results = []

    for file_entry in file_index:
        filepath = file_entry["path"]
        file_type = file_entry["type"]

        parser = PARSER_MAP.get(file_type, extract_text_generic)

        try:
            extracted = parser(filepath)
            parse_status = "success" if extracted.get("text") else "empty"
        except Exception as e:
            extracted = {"text": "", "metadata": {}}
            parse_status = "error"

        results.append({
            "path": filepath,
            "relative_path": file_entry.get("relative_path", filepath),
            "name": file_entry["name"],
            "type": file_type,
            "text": extracted.get("text", ""),
            "metadata": extracted.get("metadata", {}),
            "parse_status": parse_status,
            "error": extracted.get("error", ""),
        })

    return results


if __name__ == "__main__":
    input_data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else []
    result = parse_files(input_data)
    print(json.dumps(result, ensure_ascii=False, indent=2))
