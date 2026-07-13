"""
Prompt Enhancer · AI提示词增强器
优化AI绘图/视频生成提示词，添加画幅、风格、质量参数。

用途: content-writing 技能 · 步骤7
输入: 原始提示词 (str) + 风格参数 (dict|None)
输出: str 增强后的提示词（英文，含画幅和风格参数）
"""

import json
import sys

# 默认风格参数
DEFAULT_STYLE = {
    "aspect_ratio": "16:9",
    "quality": "high quality, photorealistic",
    "style": "clean scientific visualization, professional lighting",
    "negative": "text, watermark, logo, low quality, blurry, oversaturated",
    "resolution": "--ar 16:9 --q 2",
}

# 场景风格预设
SCENE_PRESETS = {
    "lab": {
        "style": "modern laboratory, clean room, blue-white lighting, scientific equipment",
        "negative": "clutter, dark, messy, casual",
    },
    "data": {
        "style": "data visualization, clean infographic, floating numbers, holographic display",
        "negative": "cluttered chart, low resolution, pixelated",
    },
    "abstract": {
        "style": "abstract scientific concept, molecular structures, glowing particles, dark background",
        "negative": "cartoon, childish, low detail",
    },
    "product": {
        "style": "product photography, clean white background, studio lighting, premium",
        "negative": "busy background, harsh shadows, low quality",
    },
}


def enhance(
    prompt: str,
    scene_type: str = "abstract",
    custom_style: dict | None = None,
    include_negative: bool = True,
) -> str:
    """
    增强AI提示词。

    Args:
        prompt: 原始画面描述（中文或英文）
        scene_type: 场景类型 (lab/data/abstract/product)
        custom_style: 自定义风格参数（覆盖预设）
        include_negative: 是否包含负面提示词

    Returns:
        增强后的完整提示词（英文）
    """
    style = SCENE_PRESETS.get(scene_type, DEFAULT_STYLE)

    # 合并自定义参数
    if custom_style:
        style = {**DEFAULT_STYLE, **style, **custom_style}

    # 构建增强提示词
    parts = [prompt.strip()]
    parts.append(style.get("style", DEFAULT_STYLE["style"]))
    parts.append(style.get("quality", DEFAULT_STYLE["quality"]))

    enhanced = ", ".join(p for p in parts if p)

    # 添加分辨率参数
    resolution = style.get("resolution", DEFAULT_STYLE["resolution"])
    enhanced += f" {resolution}"

    # 添加负面提示词
    if include_negative:
        negative = style.get("negative", DEFAULT_STYLE["negative"])
        enhanced += f" --no {negative}"

    return enhanced


def enhance_batch(prompts: list[dict]) -> list[dict]:
    """批量增强提示词"""
    enhanced = []
    for item in prompts:
        result = {
            "original": item.get("prompt", ""),
            "enhanced": enhance(
                prompt=item.get("prompt", ""),
                scene_type=item.get("scene_type", "abstract"),
                custom_style=item.get("custom_style"),
            ),
            "scene_type": item.get("scene_type", "abstract"),
        }
        enhanced.append(result)
    return enhanced


if __name__ == "__main__":
    input_data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else []

    if isinstance(input_data, list):
        result = enhance_batch(input_data)
    else:
        result = enhance(input_data.get("prompt", ""))

    print(json.dumps(result, ensure_ascii=False, indent=2))
