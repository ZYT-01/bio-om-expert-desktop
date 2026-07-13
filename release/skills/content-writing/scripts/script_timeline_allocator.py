"""
Script Timeline Allocator · 视频脚本时长分配器
根据视频总时长和各段落权重，计算旁白字数、镜头数和时间节点。

用途: content-writing 技能 · 步骤6
输入: 总时长(秒) + 段落权重配置
输出: dict 各段落的时间分配和字数预算
"""

import json
import sys

# 默认四段式权重
DEFAULT_WEIGHTS = {
    "开场痛点": 0.18,
    "技术原理": 0.37,
    "应用案例": 0.32,
    "价值总结": 0.13,
}

# 中文旁白语速：约 3-5 字/秒（技术类取 3.5）
WORDS_PER_SECOND = 3.5

# 视频镜头切换频率参考（秒/镜头）
SECONDS_PER_SHOT = {
    "开场痛点": 3.0,   # 快节奏，每3秒切换
    "技术原理": 5.0,   # 需要时间展示原理
    "应用案例": 4.0,   # 中速
    "价值总结": 3.5,   # 偏快
}


def allocate(
    total_duration: int = 180,
    weights: dict | None = None,
    words_per_second: float = WORDS_PER_SECOND,
) -> dict:
    """
    计算各段落的时间、字数、镜头分配。

    Args:
        total_duration: 视频总时长（秒），默认180秒（3分钟）
        weights: 各段落权重
        words_per_second: 旁白语速（字/秒）

    Returns:
        {
            "total_duration": int,
            "total_words_budget": int,
            "total_shots": int,
            "segments": [{
                "name": "开场痛点",
                "duration": int,
                "start_time": "0:00",
                "end_time": "0:32",
                "word_budget": int,
                "shot_count": int,
                "narration_pace": str,
            }, ...]
        }
    """
    weights = weights or DEFAULT_WEIGHTS
    total_words = int(total_duration * words_per_second)

    segments = []
    current_time = 0.0
    total_shots = 0

    for name, weight in weights.items():
        duration = round(total_duration * weight)
        word_budget = int(duration * words_per_second)
        shots_per_sec = SECONDS_PER_SHOT.get(name, 4.0)
        shot_count = max(1, round(duration / shots_per_sec))
        total_shots += shot_count

        start_m, start_s = divmod(int(current_time), 60)
        end_time = current_time + duration
        end_m, end_s = divmod(int(end_time), 60)

        segments.append({
            "name": name,
            "duration": duration,
            "start_time": f"{start_m}:{start_s:02d}",
            "end_time": f"{end_m}:{end_s:02d}",
            "word_budget": word_budget,
            "shot_count": shot_count,
            "narration_pace": f"约{words_per_second}字/秒",
            "weight": weight,
        })

        current_time = end_time

    return {
        "total_duration": total_duration,
        "total_words_budget": total_words,
        "total_shots": total_shots,
        "words_per_second": words_per_second,
        "segments": segments,
    }


if __name__ == "__main__":
    duration = int(sys.argv[1]) if len(sys.argv) > 1 else 180
    result = allocate(duration)
    print(json.dumps(result, ensure_ascii=False, indent=2))
