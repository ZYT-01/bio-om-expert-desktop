#!/usr/bin/env python3
"""voiceover_generator.py — 中文口播稿生成器

将视频脚本中的旁白/配音文案转化为口语化口播稿，包含：
  - 语速与时长计算
  - 留白/停顿标记
  - 口语化改写
  - 节奏控制标记
  - 平台差异化处理
"""

import re
import math
from dataclasses import dataclass
from typing import Optional


@dataclass
class VoiceoverConfig:
    """口播稿生成配置"""
    platform: str = "long"  # short | long
    base_speed: int = 190  # 字/分钟（标准语速）
    padding_ratio: float = 0.15  # 留白余量
    max_chars_short: int = 100  # 短平台最大字数
    max_chars_long: int = 600  # 长平台最大字数


@dataclass
class VoiceoverResult:
    """口播稿生成结果"""
    platform: str
    speed: int
    total_chars: int
    pure_reading_time: float  # 纯读时长（分钟）
    padded_time: float  # 含余量时长（分钟）
    suggested_duration_min: float  # 建议成片最短时长（秒）
    suggested_duration_max: float  # 建议成片最长时长（秒）
    text: str


def get_platform_speed(platform: str) -> int:
    """根据平台类型返回推荐语速"""
    if platform == "short":
        return 225  # 快语速：200-250字/分钟，取中间值
    return 190  # 标准语速：180-200字/分钟


def count_chinese_chars(text: str) -> int:
    """统计中文字符数（排除标点、空格、标记符号）"""
    # 移除节奏标记
    cleaned = re.sub(r"\[.*?\]", "", text)
    cleaned = re.sub(r"（.*?）", "", cleaned)
    # 只统计中文字符和中文标点
    chinese_chars = re.findall(r"[一-鿿　-〿＀-￯]", cleaned)
    return len(chinese_chars)


def convert_to_spoken(script_text: str) -> str:
    """将书面语旁白转化为口语化口播稿"""
    # 基础口语化替换规则
    replacements = [
        # 书面语 → 口语
        ("此外", "还有一点"),
        ("因此", "所以说"),
        ("然而", "但是"),
        ("换言之", "也就是说"),
        ("综上所述", "总结一下"),
        ("进行", "做"),
        ("予以", "给"),
        ("具有", "有"),
        ("显著", "很明显"),
        ("呈现", "出现"),
        ("导致", "造成了"),
        ("采用", "用"),
        ("通过...方式", "用...方法"),
        ("从而", "这样就能"),
    ]

    result = script_text
    for written, spoken in replacements:
        result = result.replace(written, spoken)

    # 切分长句（超过40字的句子加逗号切分）
    sentences = re.split(r"[。！？；\n]", result)
    new_sentences = []
    for sent in sentences:
        if len(sent) > 40:
            # 在合适的位置（连词前后）插入逗号
            sent = re.sub(r"(但是|而且|然后|所以|因为|如果|那么)", r"，\1", sent)
            sent = re.sub(r"(了)([^，。！？\s])", r"\1，\2", sent)
        new_sentences.append(sent)
    result = "。".join(new_sentences)

    return result


def add_rhythm_marks(text: str, platform: str = "long") -> str:
    """为口播稿添加节奏控制标记"""
    lines = text.strip().split("\n")
    marked_lines = []

    total_lines = len(lines)
    for i, line in enumerate(lines):
        line = line.strip()
        if not line:
            marked_lines.append("")
            continue

        # 根据位置判断节奏
        if i < total_lines * 0.15:
            # 开场部分：正常语速
            marked_lines.append(f"[开场 正常]\n{line}")
        elif i < total_lines * 0.3:
            # 过渡部分：稍快
            marked_lines.append(f"[过渡 稍快]\n{line}")
        elif i < total_lines * 0.8:
            # 核心内容：根据重要性调整
            if any(kw in line for kw in ["关键", "核心", "最重要", "独家", "突破"]):
                marked_lines.append(f"[核心卖点 慢速+重音]\n{line}")
                marked_lines.append("（停顿）")
            else:
                marked_lines.append(f"[核心内容 正常]\n{line}")
        else:
            # 结尾：正常，可稍慢
            marked_lines.append(f"[结尾 正常]\n{line}")

    return "\n".join(marked_lines)


def generate_voiceover(
    script_text: str,
    platform: str = "long",
    config: Optional[VoiceoverConfig] = None,
) -> VoiceoverResult:
    """主导函数：基于视频脚本旁白生成中文口播稿"""
    if config is None:
        config = VoiceoverConfig(platform=platform)

    speed = get_platform_speed(platform)

    # 口语化改写
    spoken_text = convert_to_spoken(script_text)

    # 添加节奏标记
    marked_text = add_rhythm_marks(spoken_text, platform)

    # 统计字数并计算时长
    total_chars = count_chinese_chars(marked_text)
    pure_reading_time = total_chars / speed  # 分钟
    padded_time = pure_reading_time * (1 + config.padding_ratio)

    # 建议成片时长（秒）
    suggested_min = math.ceil(padded_time * 60)
    suggested_max = math.ceil(padded_time * 60 * 1.1)

    return VoiceoverResult(
        platform=platform,
        speed=speed,
        total_chars=total_chars,
        pure_reading_time=round(pure_reading_time, 2),
        padded_time=round(padded_time, 2),
        suggested_duration_min=suggested_min,
        suggested_duration_max=suggested_max,
        text=marked_text,
    )


def render_voiceover_markdown(result: VoiceoverResult, topic: str = "") -> str:
    """将口播稿渲染为 Markdown 输出"""
    plat_label = "短平台（抖音/快手，0-30秒）" if result.platform == "short" else "长平台（视频号/小红书，1-3分钟）"

    lines = [
        f"# {topic} · 中文口播稿" if topic else "# 中文口播稿",
        "",
        f"**平台类型**：{plat_label}",
        f"**目标语速**：{result.speed} 字/分钟",
        f"**总字数**：{result.total_chars} 字",
        f"**预计纯读时长**：{result.pure_reading_time} 分钟",
        f"**预留余量（+15%）**：{result.padded_time} 分钟",
        f"**建议成片时长**：{result.suggested_duration_min}-{result.suggested_duration_max} 秒",
        "",
        "---",
        "",
        result.text,
    ]
    return "\n".join(lines)


if __name__ == "__main__":
    # 示例
    sample_script = """
    你知道吗？AAV生产中，空壳率竟然能高达15%。
    这意味着，你辛辛苦苦生产的一批病毒，有近六分之一是废的。
    其实，问题主要出在三个环节：质粒比例、细胞培养、纯化工艺。
    其中，质粒配比是最容易被忽略的关键点。
    调整三种质粒的比例，空壳率可以从15%直降到5%以下。
    如果你也在做AAV纯化，不妨试试这个方法。
    """

    result = generate_voiceover(sample_script, platform="long")
    print(render_voiceover_markdown(result, "AAV空壳率优化"))
