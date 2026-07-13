"""
Office Parser · Office文档解析器
解析 .docx, .xlsx, .pptx 文件，提取文本、表格、幻灯片内容。

用途: local-research 技能 · 步骤3（Office子调用）
输入: filepath (str)
输出: dict{text, metadata, tables, slides}
"""

import json
import sys
import os


def extract_docx(filepath: str) -> dict:
    """
    提取 .docx 文件文本。
    使用 python-docx。
    """
    # TODO: 实际实现
    # from docx import Document
    # doc = Document(filepath)
    # text = "\n".join([p.text for p in doc.paragraphs])
    # tables = [[cell.text for cell in row.cells] for table in doc.tables for row in table.rows]
    return {"text": "", "tables": []}


def extract_xlsx(filepath: str) -> dict:
    """
    提取 .xlsx 文件表格数据。
    使用 openpyxl。
    """
    # TODO: 实际实现
    # from openpyxl import load_workbook
    # wb = load_workbook(filepath, data_only=True)
    # sheets = {}
    # for name in wb.sheetnames:
    #     ws = wb[name]
    #     sheets[name] = [[cell.value for cell in row] for row in ws.iter_rows()]
    return {"text": "", "sheets": {}}


def extract_pptx(filepath: str) -> dict:
    """
    提取 .pptx 文件幻灯片文本。
    使用 python-pptx。
    """
    # TODO: 实际实现
    # from pptx import Presentation
    # prs = Presentation(filepath)
    # slides = []
    # for slide in prs.slides:
    #     slide_text = " ".join([shape.text for shape in slide.shapes if shape.has_text_frame])
    #     slides.append(slide_text)
    return {"text": "", "slides": []}


EXTRACTOR_MAP = {
    ".docx": extract_docx,
    ".xlsx": extract_xlsx,
    ".pptx": extract_pptx,
}


def extract_office(filepath: str) -> dict:
    """
    自动识别 Office 文件类型并调用对应解析器。

    Returns:
        {
            "text": str,
            "metadata": {"type": "docx|xlsx|pptx"},
            "tables": list | None,
            "slides": list | None,
            "sheets": dict | None,
        }
    """
    if not os.path.exists(filepath):
        return {"text": "", "metadata": {}, "error": f"File not found: {filepath}"}

    ext = os.path.splitext(filepath)[1].lower()
    extractor = EXTRACTOR_MAP.get(ext)

    if extractor is None:
        return {"text": "", "metadata": {}, "error": f"Unsupported office format: {ext}"}

    result = extractor(filepath)
    result["metadata"] = {"type": ext[1:]}
    return result


if __name__ == "__main__":
    office_path = sys.argv[1] if len(sys.argv) > 1 else ""
    result = extract_office(office_path)
    print(json.dumps(result, ensure_ascii=False, indent=2))
