# {选题名称} · 分镜提示词

**镜头总数**：{total_shots}个
**画幅比例**：16:9
**统一风格**：{style_description}

---

{for each shot}

## 镜头 {N}：{shot_name}（{duration}秒）

| 要素 | 内容 |
|------|------|
| **时段** | {start} - {end} |
| **画面描述** | {中文：构图、元素、动作、色彩、光线} |
| **Camera** | {镜头运动：static/pan/tilt/dolly/zoom} |
| **AI Prompt (EN)** | {完整英文提示词，含画幅和风格参数} |
| **Negative Prompt** | {负面提示词} |
| **字幕内容** | {叠加的中文文案} |
| **参考图** | {如有参考图链接或路径} |

---

{end for}

## 全局参数

```json
{
  "aspect_ratio": "16:9",
  "fps": 24,
  "style_strength": 0.7,
  "motion_scale": 0.5,
  "seed": "随机",
  "negative_global": "text, watermark, logo, low quality, blurry"
}
```
