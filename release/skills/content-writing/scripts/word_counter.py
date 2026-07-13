"""
Word Counter · 字数统计与校验器
实时统计推文各部分字数，确保在目标范围内。

用途: content-writing 技能 · 步骤3
输入: 推文文本 (str) + 目标字数范围 (tuple[int,int])
输出: dict{total, sections, within_range, suggestions}
"""

import re
import sys
import json


def count_chinese(text: str) -> int:
    """统计中文字符数（含标点）"""
    chinese_chars = re.findall(r"[一-鿿　-〿＀-￯]", text)
    return len(chinese_chars)


def count_words(text: str) -> int:
    """
    统计"等效字数"。
    中文：1个字符 = 1字
    英文：按空格分词
    """
    chinese = count_chinese(text)
    # 去除中文字符后统计英文单词
    non_chinese = re.sub(r"[一-鿿　-〿＀-￯]", " ", text)
    english_words = len([w for w in non_chinese.split() if w.strip()])
    return chinese + english_words


def count_by_sections(text: str, section_delimiter: str = "##") -> dict:
    """按章节分别统计字数"""
    sections = {}
    parts = text.split(f"\n{section_delimiter} ")

    for part in parts:
        if not part.strip():
            continue
        lines = part.strip().split("\n")
        section_name = lines[0].strip() if lines else "未命名"
        section_text = "\n".join(lines[1:]) if len(lines) > 1 else ""
        sections[section_name] = {
            "word_count": count_words(section_text),
            "char_count": len(section_text),
        }

    return sections


def validate(
    text: str,
    target_min: int = 800,
    target_max: int = 1500,
    section_budgets: dict | None = None,
) -> dict:
    """
    校验字数并给出调整建议。

    Returns:
        {
            "total_words": int,
            "within_range": bool,
            "by_section": dict,
            "suggestions": list[str],
        }
    """
    total = count_words(text)
    within_range = target_min <= total <= target_max
    suggestions = []

    if total < target_min:
        suggestions.append(f"当前 {total} 字，距目标下限 {target_min} 字还差 {target_min - total} 字。建议扩展技术解析或应用场景部分。")
    elif total > target_max:
        suggestions.append(f"当前 {total} 字，超出目标上限 {target_max} 字 {total - target_max} 字。建议精简问题引入或合并相似要点。")

    by_section = count_by_sections(text)

    return {
        "total_words": total,
        "total_chars": len(text),
        "chinese_chars": count_chinese(text),
        "within_range": within_range,
        "by_section": by_section,
        "suggestions": suggestions,
    }


if __name__ == "__main__":
    text = sys.argv[1] if len(sys.argv) > 1 else ""
    target_min = int(sys.argv[2]) if len(sys.argv) > 2 else 800
    target_max = int(sys.argv[3]) if len(sys.argv) > 3 else 1500
    result = validate(text, target_min, target_max)
    print(json.dumps(result, ensure_ascii=False, indent=2))
