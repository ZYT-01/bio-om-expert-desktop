# Bio-OM Expert — Claude Code Configuration

## File Naming Conventions (CRITICAL)

When outputting files for Bio-OM Expert, **ALWAYS use English filenames**.
NEVER output Chinese filenames (the dashboard depends on English keywords for
category detection).

### Output file reference

| Content | Filename |
|---------|----------|
| Research report | `research_report.md` |
| Article outline | `article_outline.md` |
| Article draft | `article_draft.md` |
| Headline options | `article_headlines.md` |
| Engagement design | `article_interaction.md` |
| Image suggestions (MD) | `image_suggestions.md` |
| Image suggestions (JSON) | `配图建议.json` (only exception) |
| Video script | `video_script.md` |
| Voiceover script | `voiceover.md` |
| Scene prompts | `script_scenes.md` |
| Video engagement | `article_engage.md` |

### Rules

1. Output all files to the directory specified in the prompt
2. Do NOT create Chinese-named copies of any file
3. The single exception is `配图建议.json` — the dashboard needs both `.md` and `.json`
4. Use UTF-8 encoding for all files
5. Do not output `研究报告.md` — use `research_report.md` instead
