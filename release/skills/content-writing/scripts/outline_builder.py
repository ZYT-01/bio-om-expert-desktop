"""
Outline Builder · 大纲构建辅助器
基于研究报告核心要素，自动构建推文大纲骨架。

用途: content-writing 技能 · 步骤2
输入: 核心问题 + 技术要点 + 应用场景 + 价值结论
输出: dict 四段式大纲结构
"""

import json
import sys


def build_outline(
    core_problem: str,
    tech_points: list[str],
    application_scenarios: list[dict],
    value_conclusion: str,
    target_audience: str = "生物医药企业研发/QC人员",
) -> dict:
    """
    构建四段式推文大纲。

    Returns:
        {
            "title": str,
            "target_audience": str,
            "sections": [
                {"name": "问题引入", "word_budget": int, "content_plan": [...]},
                {"name": "技术解析", "word_budget": int, "content_plan": [...]},
                {"name": "应用场景", "word_budget": int, "content_plan": [...]},
                {"name": "价值总结", "word_budget": int, "content_plan": [...]},
            ]
        }
    """
    outline = {
        "title": "",
        "target_audience": target_audience,
        "total_word_budget": 1200,
        "sections": [
            {
                "name": "问题引入",
                "word_budget": 180,
                "content_plan": [
                    f"场景描述：{core_problem}",
                    "行业数据佐证（引用报告）",
                    "抛出核心问题",
                ],
            },
            {
                "name": "技术解析",
                "word_budget": 420,
                "content_plan": [
                    f"核心概念类比解释：{tech_points[0] if tech_points else ''}",
                ]
                + [
                    f"关键要点 {i+1}：{p}"
                    for i, p in enumerate(tech_points[:3])
                ]
                + ["配图建议：标注需要示意图的位置"],
            },
            {
                "name": "应用场景",
                "word_budget": 350,
                "content_plan": [
                    f"场景：{s.get('name', '')} — {s.get('description', '')}"
                    for s in application_scenarios[:2]
                ]
                + ["效果预期（引用报告数据）", "差异化价值对比"],
            },
            {
                "name": "价值总结",
                "word_budget": 180,
                "content_plan": [
                    value_conclusion,
                    "行动指引（客户下一步）",
                ],
            },
        ],
    }

    return outline


if __name__ == "__main__":
    input_data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
    result = build_outline(
        core_problem=input_data.get("core_problem", ""),
        tech_points=input_data.get("tech_points", []),
        application_scenarios=input_data.get("scenarios", []),
        value_conclusion=input_data.get("value_conclusion", ""),
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))
