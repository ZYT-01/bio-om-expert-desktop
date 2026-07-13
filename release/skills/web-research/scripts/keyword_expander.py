"""
Keyword Expander · 关键词扩展器
基于研究主题生成多维度搜索关键词列表，覆盖中文/英文、学术/商业等搜索场景。

用途: web-research 技能 · 步骤1
输入: 研究主题 (str) + 商务目标 (str|None)
输出: list[dict{keyword, language, source_type, priority}]
"""

import json
import sys


def load_industry_terms(assets_dir: str = "../assets") -> dict:
    """加载生物科技行业术语库"""
    # TODO: 实际加载 industry_keywords.md 并解析为结构化术语表
    return {}


def expand_keywords(topic: str, business_goal: str | None = None, depth: str = "standard") -> list[dict]:
    """
    基于主题扩展搜索关键词。

    Args:
        topic: 研究主题，如 "AAV基因治疗商业化进展"
        business_goal: 商务目标，如 "评估行业竞争格局"
        depth: 搜索深度 basic/standard/deep

    Returns:
        关键词列表，每项含 keyword, language, source_type, priority
    """
    keywords = []

    # 阶段: 基础扩展
    keywords.append({"keyword": topic, "language": "zh", "source_type": "general", "priority": 1})

    # TODO: 实现完整的扩展逻辑
    # 1. 分词提取核心概念
    # 2. 查行业术语库补全同义词/上下位词
    # 3. 中英文互译扩展
    # 4. 按 source_type 分类: general/academic/industry/social

    return keywords


if __name__ == "__main__":
    topic = sys.argv[1] if len(sys.argv) > 1 else ""
    goal = sys.argv[2] if len(sys.argv) > 2 else None
    result = expand_keywords(topic, goal)
    print(json.dumps(result, ensure_ascii=False, indent=2))
