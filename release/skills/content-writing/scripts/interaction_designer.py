#!/usr/bin/env python3
"""interaction_designer.py — 互动设计脚本生成器

为图文和视频内容生成四种互动行为（点赞/收藏/转发/评论）的触发点设计。
支持两种模式：
  - article: 图文互动设计（基于正文内容）
  - video:   视频互动设计（基于视频脚本和口播稿）
"""

import json
from dataclasses import dataclass, field
from typing import Optional

INTERACTION_TYPES = ["like", "bookmark", "share", "comment"]


@dataclass
class InteractionTrigger:
    """单个互动触发点"""
    type: str
    label: str
    position: str
    copy: str
    psychological_benefit: str
    presentation: str = ""


@dataclass
class InteractionDesign:
    """互动设计脚本"""
    topic: str
    platform: str
    mode: str  # article | video
    triggers: list = field(default_factory=list)


def load_trigger_library() -> dict:
    """从素材库加载互动触发话术模板"""
    return {
        "like": {
            "心理收益": "认同感、情绪共鸣",
            "article_templates": [
                "如果你也遇到过{痛点}，点个赞让我知道",
                "觉得这篇文章讲清楚了{技术点}？点个赞支持一下",
            ],
            "video_templates": [
                "觉得有用的话，点个赞支持下吧",
                "如果你也遇到过这种情况，双击屏幕",
            ],
        },
        "bookmark": {
            "心理收益": "知识留存、未来使用",
            "article_templates": [
                "建议收藏：{主题}完整版",
                "收藏备用，下次做{场景}时直接对照",
            ],
            "video_templates": [
                "收藏起来，做实验时直接对照",
                "这个流程建议截图保存",
            ],
        },
        "share": {
            "心理收益": "社交货币、利他性",
            "article_templates": [
                "转发给正在做{工作}的同事，帮他少走弯路",
                "行业趋势洞察，值得分享给团队",
            ],
            "video_templates": [
                "分享给团队里负责{岗位}的同事，一起优化",
                "转给你身边的{行业}人，别让他们踩坑",
            ],
        },
        "comment": {
            "心理收益": "参与感、被重视",
            "article_templates": [
                "你在{场景}中遇到过什么问题？评论区聊聊",
                "你同意这个观点吗？欢迎分享你的看法",
            ],
            "video_templates": [
                "你在{场景}中遇到过什么坑？评论区告诉大家",
                "关于{主题}你还有什么疑问？评论区问我",
            ],
        },
    }


def generate_article_interaction(topic: str, keywords: dict = None) -> InteractionDesign:
    """生成图文互动设计脚本"""
    if keywords is None:
        keywords = {}

    design = InteractionDesign(topic=topic, platform="article", mode="article")

    trigger_configs = [
        ("like", "点赞", "文末", "认同感、情绪共鸣"),
        ("bookmark", "收藏", "总结/干货段落后", "知识留存、未来使用"),
        ("share", "转发", "核心价值段后", "社交货币、利他性"),
        ("comment", "评论", "技术解析段落后", "参与感、被重视"),
    ]

    for trig_type, label, default_pos, benefit in trigger_configs:
        trigger = InteractionTrigger(
            type=trig_type,
            label=label,
            position=default_pos,
            copy=f"[根据{topic}内容定制话术]",
            psychological_benefit=benefit,
        )
        design.triggers.append(trigger)

    return design


def generate_video_interaction(
    topic: str,
    platform: str = "long",
    duration: int = 180,
    keywords: dict = None,
) -> InteractionDesign:
    """生成视频互动设计脚本"""
    if keywords is None:
        keywords = {}

    design = InteractionDesign(topic=topic, platform=platform, mode="video")

    # 根据视频时长计算关键时间点
    if platform == "short":
        intro_pos = "00:03"
        mid_pos = "00:10"
        end_pos = f"00:{min(25, duration)}"
    else:
        intro_pos = "00:15"
        mid_pos = f"01:{min(30, int(duration * 0.4))}"
        end_pos = f"0{int(duration / 60)}:{min(50, int(duration * 0.85))}"

    trigger_configs = [
        ("like", "点赞引导", end_pos, "口播 + 字幕", "认同感"),
        ("bookmark", "收藏引导", end_pos, "口播 + 字幕", "知识留存"),
        ("share", "转发引导", mid_pos, "口播 + 画面提示", "社交货币"),
        ("comment", "评论引导", intro_pos, "口播 + 字幕", "参与感"),
    ]

    for trig_type, label, time_pos, presentation, benefit in trigger_configs:
        trigger = InteractionTrigger(
            type=trig_type,
            label=label,
            position=time_pos,
            copy=f"[根据{topic}视频内容定制话术]",
            psychological_benefit=benefit,
            presentation=presentation,
        )
        design.triggers.append(trigger)

    return design


def render_markdown(design: InteractionDesign) -> str:
    """将互动设计脚本渲染为 Markdown"""
    if design.mode == "article":
        lines = [
            f"# {design.topic} · 图文互动设计脚本",
            "",
        ]
        for t in design.triggers:
            lines.append(f"## {t.label}触发")
            lines.append(f"- **触发位置**：{t.position}")
            lines.append(f"- **触发文案**：{t.copy}")
            lines.append(f"- **心理收益**：{t.psychological_benefit}")
            lines.append("")
    else:
        lines = [
            f"# {design.topic} · 视频互动设计脚本",
            f"**平台**：{design.platform}",
            "",
        ]
        for t in design.triggers:
            lines.append(f"## {t.label}")
            lines.append(f"- **时间点**：{t.position}")
            lines.append(f"- **呈现方式**：{t.presentation}")
            lines.append(f"- **话术**：{t.copy}")
            lines.append("")

    return "\n".join(lines)


if __name__ == "__main__":
    # 示例：生成图文互动设计
    article_design = generate_article_interaction("AAV空壳率优化", {})
    print(render_markdown(article_design))
    print("\n---\n")
    # 示例：生成视频互动设计
    video_design = generate_video_interaction("AAV空壳率优化", "long", 180, {})
    print(render_markdown(video_design))
