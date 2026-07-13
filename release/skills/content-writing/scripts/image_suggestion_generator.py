"""
Image Suggestion Generator · 配图节点识别与提示词生成器
扫描推文正文，自动识别需要配图的关键信息节点，生成画面描述和AI绘图提示词。

用途: content-writing 技能 · 步骤4
输入: 推文正文 (str) + 配图风格配置 (dict)
输出: list[dict] 配图建议清单
"""

import json
import sys
import re
import os

# 配图类型识别规则
IMAGE_TYPE_RULES = {
    "原理示意图": {
        "keywords": ["机制", "原理", "结构", "组成", "形成", "产生", "作用", "信号", "通路", "循环", "过程"],
        "patterns": [r"(如何|怎么).*(工作|运作|起作用)", r".*的(机制|原理|结构|组成)"],
    },
    "流程步骤图": {
        "keywords": ["首先", "然后", "最后", "步骤", "流程", "第一步", "第二步", "第三步", "路径", "工序"],
        "patterns": [r"第[一二三四五六七八九十\d]+步", r"分为.*(步|阶段|环节)"],
    },
    "数据对比图": {
        "keywords": ["从.*到", "对比", "相比", "提升", "下降", "降低", "增至", "减少", "超过", "高达", "%", "倍", "亿", "万"],
        "patterns": [r"\d+(\.\d+)?%", r"从\d+.*到\d+", r"[\d.]+\s*倍"],
    },
    "概念模型图": {
        "keywords": ["包括", "包含", "由.*组成", "分为.*类", "三种", "四种", "体系", "框架", "架构", "维度", "层次"],
        "patterns": [r".*(由|包括|包含).*(组成|构成)", r".*分为.*[一二三四五六七八九十\d]+.*(类|种|层|部分)"],
    },
    "场景应用图": {
        "keywords": ["操作", "使用", "应用", "场景", "实验室", "设备", "工程师", "技术人员", "实施", "部署"],
        "patterns": [r".*在(实验室|产线|现场|车间|平台).*", r"(实验员|工程师|操作员|技术人员).*"],
    },
}


def load_style_config(assets_dir: str = "../assets") -> dict:
    """加载配图风格配置"""
    try:
        with open(f"{assets_dir}/image_style_guide.json", "r", encoding="utf-8") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return {
            "brand_color": {"primary": "#0071e3", "secondary": "#1a1a2e"},
            "style_keywords": ["scientific illustration", "clean laboratory aesthetic", "high resolution", "4k"],
            "default_aspect_ratio": "16:9",
        }


def identify_image_nodes(text: str) -> list[dict]:
    """
    扫描正文，识别配图节点。

    Returns:
        [{paragraph_index, match_text, image_type, confidence}]
    """
    nodes = []
    paragraphs = text.split("\n")

    for i, para in enumerate(paragraphs):
        para = para.strip()
        if not para or len(para) < 30:
            continue

        # 跳过纯标题行
        if para.startswith("#"):
            continue

        for img_type, rules in IMAGE_TYPE_RULES.items():
            # 关键词匹配
            keyword_hits = sum(1 for kw in rules["keywords"] if kw in para)
            # 正则匹配
            pattern_hits = sum(
                1 for pat in rules["patterns"] if re.search(pat, para)
            )

            if keyword_hits >= 2 or pattern_hits >= 1:
                nodes.append({
                    "paragraph_index": i,
                    "match_text": para[:200],
                    "image_type": img_type,
                    "confidence": min(0.9, 0.4 + keyword_hits * 0.15 + pattern_hits * 0.25),
                })

    # 去重：同一段落只保留置信度最高的类型，且相邻段落不重复同类
    return _deduplicate_nodes(nodes)


def _deduplicate_nodes(nodes: list[dict]) -> list[dict]:
    """去重：同段保留最高置信度，相邻段同类去重"""
    # 按段落分组，取最高置信度类型
    seen = {}
    for node in nodes:
        idx = node["paragraph_index"]
        if idx not in seen or node["confidence"] > seen[idx]["confidence"]:
            seen[idx] = node

    # 按段落索引排序
    result = sorted(seen.values(), key=lambda x: x["paragraph_index"])

    # 过滤相邻同类（保留第一个）
    filtered = []
    for node in result:
        if filtered and node["image_type"] == filtered[-1]["image_type"]:
            continue
        filtered.append(node)

    return filtered


def generate_visual_description(node: dict, style_config: dict) -> str:
    """为配图节点生成画面描述"""
    category = node.get("image_type", "原理示意图")
    category_style = style_config.get("category_specific", {}).get(category, {})

    descriptions = {
        "原理示意图": f"科学示意图，{category_style.get('style', '3D剖面视图')}，深蓝渐变背景，柔和科技光线，主体突出，标注清晰",
        "流程步骤图": f"流程图，{category_style.get('style', '带图标步骤卡片，箭头连接')}，干净设计，深色背景，步骤编号标注",
        "数据对比图": f"数据可视化图，{category_style.get('style', '简洁柱状图/对比表')}，关键数据荧光高亮，深蓝背景最小化网格线",
        "概念模型图": f"等距概念模型，{category_style.get('style', '标注组件，专业期刊风格')}，分层结构清晰，品牌配色",
        "场景应用图": f"实际场景图，{category_style.get('style', '现代实验室实拍风格')}，柔和光线，人物操作自然",
    }

    return descriptions.get(category, descriptions["原理示意图"])


def generate_ai_prompt(node: dict, style_config: dict) -> str:
    """生成英文AI绘图提示词"""
    category = node.get("image_type", "原理示意图")
    style_kw = ", ".join(style_config.get("style_keywords", []))
    ratio = style_config.get("default_aspect_ratio", "16:9")
    category_style = style_config.get("category_specific", {}).get(category, {})

    prompts = {
        "原理示意图": f"Scientific illustration, {category_style.get('style', 'cross-section view')}, deep blue gradient background, soft technological lighting, clean laboratory aesthetic, high resolution, {ratio}, 4k, {style_kw}",
        "流程步骤图": f"Flowchart infographic, {category_style.get('style', 'step cards with icons connected by arrows')}, dark background, clean design, {ratio}, 4k, {style_kw}",
        "数据对比图": f"Data visualization, {category_style.get('style', 'clean bar chart')}, key data highlighted, dark blue background with minimal grid, {ratio}, 4k, {style_kw}",
        "概念模型图": f"Conceptual model, {category_style.get('style', 'isometric view with labeled components')}, layered structure, brand colors #0071e3 #1a1a2e, {ratio}, 4k, {style_kw}",
        "场景应用图": f"Realistic laboratory scene, {category_style.get('style', 'researcher operating equipment')}, candid natural lighting, professional, {ratio}, 4k",
    }

    return prompts.get(category, prompts["原理示意图"])


def generate_suggestions(text: str, topic: str = "未命名选题") -> list[dict]:
    """
    主入口：扫描正文并生成配图建议。

    Returns:
        [{id, type, placement, context, visual_description, ai_prompt}]
    """
    style_config = load_style_config()
    nodes = identify_image_nodes(text)

    suggestions = []
    for i, node in enumerate(nodes):
        img_id = f"IMG-{i+1:03d}"
        suggestions.append({
            "id": img_id,
            "type": node["image_type"],
            "placement": f"段落 {node['paragraph_index']+1}",
            "context": node["match_text"],
            "visual_description": generate_visual_description(node, style_config),
            "ai_prompt": generate_ai_prompt(node, style_config),
        })

    return suggestions


if __name__ == "__main__":
    input_text = sys.argv[1] if len(sys.argv) > 1 else ""
    topic = sys.argv[2] if len(sys.argv) > 2 else "未命名"
    if os.path.isfile(input_text):
        with open(input_text, "r", encoding="utf-8") as f:
            input_text = f.read()
    result = generate_suggestions(input_text, topic)
    print(json.dumps(result, ensure_ascii=False, indent=2))
