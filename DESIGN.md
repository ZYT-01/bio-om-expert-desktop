# Bio-OM Expert — Design System

## Colors

```css
:root {
  --bg: #0f1117;           /* Page background */
  --surface: #1a1b23;      /* Panel/header background */
  --border: #2a2b35;       /* Separators, input borders */
  --text: #c9cdd4;         /* Primary body text */
  --text-dim: #8b8f9b;     /* Secondary text (WCAG AA: 5.1:1 on bg) */
  --accent: #4f8fff;       /* Primary action color */
  --accent-bg: rgba(79, 143, 255, 0.12);
  --error: #f04f5f;        /* Errors, destructive actions */
  --success: #3ecf8e;      /* Success states, completion */
  --warning: #f0a04f;      /* Warnings, interrupt states */
}
```

**Semantic usage:**
- `accent`: primary buttons, active tabs, progress bar, focus rings
- `error`: error messages, cancel button, delete actions
- `success`: done states, idle badge, folder actions
- `warning`: revise button, interruption notices

**Dark-only in v1.** Light mode reserved for v2 with toggle.

## Typography

```css
--font-body: "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Noto Sans SC", sans-serif;
--font-mono: "SF Mono", "Fira Code", "Monaco", monospace;
```

**Scale:**

| Token | Size | Weight | Use |
|-------|------|--------|-----|
| Hero | 28px | 700 | Empty state brand name |
| Heading | 15px | 600 | App header, panel titles |
| Body | 14px | 400 | Log output, main content |
| Small | 13px | 400 | Sidebar topics, input placeholder |
| Caption | 11px | 400 | Timestamps, badges, meta |

Mono font used only for: source code preview, log timestamps.

## Spacing

| Token | Value | Use |
|-------|-------|-----|
| xs | 4-6px | Tight gaps (tab buttons, icon padding) |
| sm | 8-10px | Input gaps, header padding |
| md | 14-16px | Panel padding, section gaps |
| lg | 20-24px | Content area padding, empty state spacing |
| xl | 32-40px | Empty state vertical rhythm |

## Border Radius

| Level | Value | Use |
|-------|-------|-----|
| Small | 4px | Inline buttons, toggle tabs |
| Default | 8px | Inputs, action buttons, panels |
| Large | 12px | Status badges |

## Layout

- 3-panel desktop app (sidebar 280px + main flex + preview 420px)
- Sidebar width fixed (text overflow truncated with ellipsis)
- Preview panel toggleable (auto-opens on task completion)
- Header fixed height, content areas scroll independently
- Progress bar above main content, visible during execution only

## Motion

- Transitions: 150-200ms ease (button hover, tab switch)
- Progress pulse: 2s ease-in-out infinite
- All animations disabled when `prefers-reduced-motion: reduce`
- No entrance animations, no scroll-linked motion

## States

| State | Visual |
|-------|--------|
| Empty | Brand name + subtitle + 4 clickable example chips |
| Loading/Orchestrating | Progress bar at 5% + "正在识别意图…" |
| Running | Progress bar with skill name + output-line-count feedback |
| Error | Red left-border card + retry button |
| Cancelled | Yellow card with "任务已中止" + saved file summary |
| Done | Progress bar 100% + "完成" + preview panel opens |
| Timeout | 30s no output → warning in progress bar text |

## Typography Rules

- No monospace for body content (only for code/source preview)
- Headings closer to their content than to preceding sections
- Body text ≥ 14px, captions ≥ 11px
- Contrast ratio ≥ 4.5:1 on all text (achieved: text-dim 5.1:1 on bg)
