"""
Headline Generator · 标题批量生成器
基于选题内容和研究报告，批量生成多种类型的推文标题。

用途: content-writing 技能 · 步骤4
输入: 选题名称 + 核心关键词 + 目标受众
输出: list[str] 20个候选标题（覆盖5种类型）
"""

import json
import sys

# 标题公式模板
HEADLINE_FORMULAS = {
    "数字型": [
        "{number}个关键数据，揭开{topic}的真相",
        "{topic}：{number}个你必须知道的事实",
        "关于{topic}，这{number}点90%的人都理解错了",
    ],
    "问题型": [
        "为什么你的{topic}总是出问题？",
        "{topic}到底有多重要？",
        "你真的了解{topic}吗？",
    ],
    "对比型": [
        "传统方案 vs 新方案：{topic}的颠覆性突破",
        "{topic}的前后对比：改变有多大？",
        "同样是{topic}，为什么有人做得好，有人做不好？",
    ],
    "价值型": [
        "{topic}如何帮你节省{benefit}？",
        "掌握{topic}，提升{benefit}的秘诀",
        "用{topic}，实现{benefit}的路径",
    ],
    "悬念型": [
        "行业大佬都在悄悄关注的{topic}",
        "{topic}正在改变行业，而你却还不知道",
        "下一个爆发点：{topic}为什么被看好？",
    ],
}


def load_formulas(assets_dir: str = "../assets") -> dict:
    """加载标题公式模板库"""
    try:
        with open(f"{assets_dir}/headline_formulas.json", "r", encoding="utf-8") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return HEADLINE_FORMULAS


def generate_headlines(
    topic: str,
    keywords: list[str] | None = None,
    target_audience: str = "生物医药企业研发/QC人员",
    count: int = 20,
) -> list[dict]:
    """
    基于选题生成多类型标题。

    Args:
        topic: 选题名称
        keywords: 核心关键词列表
        target_audience: 目标受众
        count: 生成数量（默认20）

    Returns:
        标题列表，每项含 type, headline, score
    """
    formulas = load_formulas()
    headlines = []

    numbers = [3, 5, 7, 10]
    benefits = ["效率30%", "成本50%", "成功率", "竞争力"]

    for formula_type, templates in formulas.items():
        for template in templates:
            headline = template.format(
                topic=topic,
                number=numbers[len(headlines) % len(numbers)],
                benefit=benefits[len(headlines) % len(benefits)],
            )
            headlines.append({
                "type": formula_type,
                "headline": headline,
                "score": 0,  # TODO: NLP 评分
            })

    return headlines[:count]


if __name__ == "__main__":
    topic = sys.argv[1] if len(sys.argv) > 1 else ""
    keywords = sys.argv[2].split(",") if len(sys.argv) > 2 else []
    result = generate_headlines(topic, keywords)
    print(json.dumps(result, ensure_ascii=False, indent=2))
