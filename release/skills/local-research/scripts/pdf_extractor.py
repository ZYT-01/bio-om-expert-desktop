"""
PDF Extractor · PDF文本提取器
从PDF文件中提取文本内容，支持文字型PDF和扫描型PDF。

用途: local-research 技能 · 步骤3（PDF子调用）
输入: filepath (str)
输出: dict{text, metadata{pages, has_ocr, producer, ...}}
"""

import json
import sys
import os


def extract_text_layer(pdf_path: str) -> tuple[str, dict]:
    """
    提取文字层（适用于文字型PDF）。
    使用 pdfplumber / PyMuPDF 提取。
    """
    # TODO: 实际使用 pdfplumber 或 PyMuPDF (fitz)
    # import fitz
    # doc = fitz.open(pdf_path)
    # text = "\n".join([page.get_text() for page in doc])
    return ("", {"method": "text_layer"})


def extract_with_ocr(pdf_path: str) -> tuple[str, dict]:
    """
    OCR提取（适用于扫描型PDF）。
    使用 Tesseract OCR / PaddleOCR 提取。
    """
    # TODO: 将PDF页面渲染为图片后OCR
    # from pdf2image import convert_from_path
    # images = convert_from_path(pdf_path)
    # text = ocr_engine.process(images)
    return ("", {"method": "ocr"})


def extract_metadata(pdf_path: str) -> dict:
    """提取PDF元数据"""
    # TODO: 提取标题、作者、创建日期、页数、生成器
    return {
        "pages": 0,
        "title": "",
        "author": "",
        "producer": "",
        "has_ocr": False,
    }


def extract_pdf(filepath: str) -> dict:
    """
    提取PDF文件文本和元数据。

    Returns:
        {
            "text": str,
            "metadata": {
                "pages": int,
                "title": str,
                "author": str,
                "method": "text_layer" | "ocr",
                "has_ocr": bool,
            }
        }
    """
    if not os.path.exists(filepath):
        return {"text": "", "metadata": {}, "error": f"File not found: {filepath}"}

    meta = extract_metadata(filepath)

    # 优先尝试文字层提取
    text, extraction_meta = extract_text_layer(filepath)
    meta.update(extraction_meta)

    # 如果文字层为空，回退到OCR
    if not text.strip():
        meta["has_ocr"] = True
        text, ocr_meta = extract_with_ocr(filepath)
        meta.update(ocr_meta)

    return {"text": text, "metadata": meta}


if __name__ == "__main__":
    pdf_path = sys.argv[1] if len(sys.argv) > 1 else ""
    result = extract_pdf(pdf_path)
    print(json.dumps(result, ensure_ascii=False, indent=2))
