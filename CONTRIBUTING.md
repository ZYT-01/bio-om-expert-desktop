# Contributing to Bio-OM Expert

## Adding a new Skill

Skills are the building blocks of Bio-OM Expert. Each skill is a JSON manifest file that tells the app how to run a Claude Code invocation for a specific content operation.

### 3-step quick start

**Step 1: Scaffold a new skill**

```bash
./bin/bio-om-skill new "活动策划"
```

This creates a template manifest in `~/.bio-om-expert/skills/` with placeholder fields.

**Step 2: Edit the manifest**

Open the generated JSON file and fill in the fields:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | ✅ | Machine ID — kebab-case, e.g. `event-planning` |
| `display` | ✅ | Human-readable name shown in UI — Chinese recommended |
| `description` | ✅ | One-line description shown in the orchestration plan |
| `trigger_patterns` | ✅ | Keywords that trigger this skill. Matched case-insensitively against user input. At least one required. |
| `cli_invoke` | ✅ | Template for the Claude Code invocation. Use `{param_name}` for dynamic values. |
| `estimated_time` | ✅ | User-facing time estimate. Format: `"X-Y 分钟"` or `"X 秒"`. |

Example:
```json
{
  "$schema": "https://bio-om-expert.dev/skill-manifest.schema.json",
  "name": "event-planning",
  "display": "活动策划方案",
  "description": "基于主题生成完整的活动策划方案",
  "trigger_patterns": ["活动策划", "活动方案", "线下活动", "营销活动", "展会"],
  "cli_invoke": "/content-writing --topic \"{topic}\" --type event-planning",
  "estimated_time": "3-5 分钟"
}
```

**Step 3: Validate and test**

```bash
# Validate the manifest format
./bin/bio-om-skill validate ~/.bio-om-expert/skills/event-planning.json

# List all installed skills
./bin/bio-om-skill list

# Test with Claude CLI (optional, requires claude CLI)
./bin/bio-om-skill test event-planning
```

Launch Bio-OM Expert — your skill appears automatically. No rebuild required.

### Trigger pattern tips

- Use 5-10 specific keywords and common phrases your users would type
- Match **user intent**, not your internal naming
- Example: A "competitor analysis" skill should trigger on "竞品分析", "竞争对手", "行业对比", not "competitor-analysis"
- Broader patterns match more queries but may cause false positives. Start narrow, expand as you observe usage.

### Where skills are stored

- **User skills**: `~/.bio-om-expert/skills/` — Add your custom skills here
- **Built-in skills**: Embedded in the app — Fall back when no user skills exist

The app loads user skills first. If the skills directory is empty (first run), it seeds it with the built-in skills as templates.

### File naming

Skill manifest files must end with `.json`. The filename (without extension) should match the `name` field, but the app reads the `name` from inside the JSON, not from the filename. Use descriptive filenames — they help you and other contributors find skills in the directory.

## Manifest format reference

Full JSON Schema: [`skills-manifest/schema.json`](skills-manifest/schema.json)

Add `"$schema": "https://bio-om-expert.dev/skill-manifest.schema.json"` to your manifest for VS Code autocomplete and validation.

## Reporting issues

- **Bug reports**: [GitHub Issues](https://github.com/ZYT-01/bio-om-expert-desktop/issues/new?template=bug_report.md)
- **Feature requests**: [GitHub Issues](https://github.com/ZYT-01/bio-om-expert-desktop/issues/new?template=feature_request.md)

## Development setup

```bash
git clone https://github.com/ZYT-01/bio-om-expert-desktop.git
cd bio-om-expert-desktop
npm install
npm run tauri dev
```

Prerequisites: Rust toolchain, Node.js 18+, platform-specific Tauri dependencies. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
