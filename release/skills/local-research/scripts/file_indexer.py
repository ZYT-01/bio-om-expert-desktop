"""
File Indexer · 文件索引生成器
递归扫描目录，生成文件清单及元数据（大小、修改时间、类型）。

用途: local-research 技能 · 步骤1
输入: 目录路径 (str) + 递归标志 (bool) + 文件类型过滤 (list[str]|None)
输出: list[dict{path, name, ext, size, mtime, type}]
"""

import os
import json
import sys
from datetime import datetime


# 尝试加载支持的文件格式配置
try:
    with open("../assets/supported_formats.json", "r", encoding="utf-8") as f:
        FORMATS_CONFIG = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    FORMATS_CONFIG = {
        "formats": {
            ".md": "markdown",
            ".txt": "text",
            ".pdf": "pdf",
            ".docx": "office",
            ".xlsx": "office",
            ".pptx": "office",
            ".csv": "data",
            ".json": "data",
            ".html": "web",
            ".xml": "data",
        }
    }


def classify_file(ext: str) -> str:
    """按扩展名分类文件类型"""
    return FORMATS_CONFIG.get("formats", {}).get(ext.lower(), "unknown")


def index_directory(
    directory: str,
    recursive: bool = True,
    file_types: list[str] | None = None,
) -> list[dict]:
    """
    扫描目录，生成文件索引。

    Args:
        directory: 目标目录路径
        recursive: 是否递归子目录
        file_types: 限定文件类型列表，如 [".md", ".pdf"]

    Returns:
        文件元数据列表
    """
    file_list = []

    if not os.path.isdir(directory):
        print(f"Error: '{directory}' is not a valid directory", file=sys.stderr)
        return file_list

    for root, dirs, files in os.walk(directory):
        # 跳过隐藏目录
        dirs[:] = [d for d in dirs if not d.startswith(".")]

        for filename in files:
            filepath = os.path.join(root, filename)
            _, ext = os.path.splitext(filename)

            # 类型过滤
            if file_types and ext.lower() not in file_types:
                continue

            stat = os.stat(filepath)

            entry = {
                "path": filepath,
                "relative_path": os.path.relpath(filepath, directory),
                "name": filename,
                "ext": ext.lower(),
                "type": classify_file(ext),
                "size_bytes": stat.st_size,
                "size_kb": round(stat.st_size / 1024, 2),
                "mtime": datetime.fromtimestamp(stat.st_mtime).isoformat(),
            }
            file_list.append(entry)

        if not recursive:
            break

    return file_list


if __name__ == "__main__":
    target_dir = sys.argv[1] if len(sys.argv) > 1 else "."
    recursive = "--no-recursive" not in sys.argv
    result = index_directory(target_dir, recursive)
    print(json.dumps(result, ensure_ascii=False, indent=2))
