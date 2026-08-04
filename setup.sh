#!/bin/bash
# Bio-OM Expert v1.3.2 — 一键安装脚本
# 适用于 macOS / Linux

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Bio-OM Expert v1.3.2 安装向导${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# ── 平台检测 ──
case "$(uname -s)" in
    Darwin*)  PLATFORM="macOS" ;;
    Linux*)   PLATFORM="Linux" ;;
    *)        PLATFORM="Unknown" ;;
esac
echo -e "🖥  检测到平台: ${GREEN}${PLATFORM}${NC}"
echo ""

# ── 1. 检查 Node.js ──
echo "━━━ 1/7 检查 Node.js ━━━"
if ! command -v node &>/dev/null; then
    echo -e "${RED}❌ 未检测到 Node.js${NC}"
    echo ""
    echo "请先安装 Node.js (≥18.x):"
    echo "  macOS: brew install node"
    echo "  或: https://nodejs.org/ 下载 LTS 版本"
    echo ""
    exit 1
fi
NODE_VER=$(node --version)
echo -e "✅ Node.js ${GREEN}${NODE_VER}${NC}"

# ── 2. 检查/安装 Claude Code CLI ──
echo ""
echo "━━━ 2/7 检查 Claude Code CLI ━━━"
if ! command -v claude &>/dev/null; then
    echo -e "${YELLOW}📦 Claude Code CLI 未安装，正在安装...${NC}"
    npm install -g @anthropic-ai/claude-code
    echo -e "${GREEN}✅ Claude Code CLI 安装完成${NC}"
else
    CLAUDE_VER=$(claude --version 2>&1 | head -1)
    echo -e "✅ Claude CLI ${GREEN}${CLAUDE_VER}${NC}"
fi

# ── 3. 检查 Python3 ──
echo ""
echo "━━━ 3/7 检查 Python3 ━━━"
PYTHON=""
if command -v python3 &>/dev/null; then
    PYTHON="python3"
elif command -v python &>/dev/null; then
    PYTHON="python"
fi
if [ -z "$PYTHON" ]; then
    echo -e "${RED}❌ 未检测到 Python3${NC}"
    echo "macOS: brew install python3"
    echo "Linux: sudo apt install python3"
    exit 1
fi
PY_VER=$($PYTHON --version)
echo -e "✅ ${GREEN}${PY_VER}${NC}"

# ── 4. 安装 python-docx ──
echo ""
echo "━━━ 4/7 安装 python-docx ━━━"
$PYTHON -m pip install python-docx --quiet 2>/dev/null && \
    echo -e "✅ python-docx 已安装" || \
    echo -e "${YELLOW}⚠️  python-docx 安装失败（Word 导出功能不可用）${NC}"

# ── 5. 安装 Skills（双目录） ──
echo ""
echo "━━━ 5/7 安装 Bio-OM Expert Skills ━━━"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

APP_SUPPORT="$HOME/Library/Application Support/com.bio-om.expert/skills"
CLAUDE_SKILLS="$HOME/.claude/skills"
mkdir -p "$APP_SUPPORT" "$CLAUDE_SKILLS"

# JSON manifests → Application Support
JSON_COUNT=0
if [ -d "$SCRIPT_DIR/skills-manifest" ]; then
    for json_file in "$SCRIPT_DIR/skills-manifest"/*.json; do
        if [ -f "$json_file" ]; then
            skill_name=$(basename "$json_file")
            cp "$json_file" "$APP_SUPPORT/$skill_name"
            echo -e "  [JSON] ${GREEN}${skill_name}${NC} → Application Support"
            JSON_COUNT=$((JSON_COUNT + 1))
        fi
    done
fi

# SKILL.md files → ~/.claude/skills/
MD_COUNT=0
if [ -d "$SCRIPT_DIR/skills" ]; then
    for md_file in "$SCRIPT_DIR/skills"/*.md; do
        if [ -f "$md_file" ]; then
            skill_name=$(basename "$md_file" .md)
            mkdir -p "$CLAUDE_SKILLS/$skill_name"
            cp "$md_file" "$CLAUDE_SKILLS/$skill_name/SKILL.md"
            echo -e "  [MD]   ${GREEN}${skill_name}/SKILL.md${NC} → ~/.claude/skills/"
            MD_COUNT=$((MD_COUNT + 1))
        fi
    done
fi

echo -e "  📦 JSON: ${GREEN}${JSON_COUNT}${NC} 个 | SKILL.md: ${GREEN}${MD_COUNT}${NC} 个"

# ── 6. 安装应用 ──
echo ""
echo "━━━ 6/7 安装应用 ━━━"
if [ "$PLATFORM" = "macOS" ]; then
    DMG=$(ls "$SCRIPT_DIR"/*.dmg 2>/dev/null | head -1)
    if [ -n "$DMG" ]; then
        echo "  挂载 DMG..."
        VOLUME=$(hdiutil attach "$DMG" -nobrowse 2>&1 | grep /Volumes/ | awk '{print $NF}')
        if [ -d "$VOLUME" ]; then
            APP=$(ls -d "$VOLUME"/*.app 2>/dev/null | head -1)
            if [ -n "$APP" ]; then
                cp -R "$APP" /Applications/
                echo "  已复制到 /Applications"
            fi
            hdiutil detach "$VOLUME" -quiet
        fi
        # 清除隔离属性并重新签名
        xattr -cr "/Applications/Bio-OM Expert.app" 2>/dev/null
        codesign --force --deep --sign - "/Applications/Bio-OM Expert.app" 2>/dev/null
        echo -e "  ${GREEN}✅ 隔离标记已清除，代码签名已更新${NC}"
    else
        echo -e "  ${YELLOW}⚠️  未找到 DMG，请手动拖入 /Applications${NC}"
    fi
else
    echo -e "  ${YELLOW}⚠️  macOS 专属步骤，请手动安装应用${NC}"
fi

# ── 7. 安装 CLAUDE.md ──
echo ""
echo "━━━ 7/7 安装 CLAUDE.md 配置 ━━━"
if [ -f "$SCRIPT_DIR/CLAUDE.md" ]; then
    mkdir -p "$HOME/.claude"
    cp "$SCRIPT_DIR/CLAUDE.md" "$HOME/.claude/CLAUDE.md"
    echo -e "✅ CLAUDE.md → ~/.claude/CLAUDE.md"
else
    echo -e "  ${YELLOW}⚠️  未找到 CLAUDE.md${NC}"
fi

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}  ✅ 安装完成！${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo "🚀 启动应用:"
echo "  macOS:   双击 /Applications/Bio-OM Expert.app"
echo "  Windows: 双击 Bio-OM Expert.exe（或从开始菜单启动）"
echo ""
echo "📋 前置检查清单:"
echo "  □  Node.js 已安装     → node --version"
echo "  □  Claude CLI 已安装    → claude --version"
echo "  □  Anthropic API Key    → echo \$ANTHROPIC_API_KEY"
echo "  □  Skills (JSON)        → ls ~/Library/Application\\ Support/com.bio-om.expert/skills/"
echo "  □  Skills (MD)          → ls ~/.claude/skills/"
echo "  □  CLAUDE.md            → cat ~/.claude/CLAUDE.md"
echo "  □  python-docx 已安装   → pip3 show python-docx"
echo ""
echo "💡 提示: 将 API Key 写入 ~/.zshrc 永久生效:"
echo "   echo 'export ANTHROPIC_API_KEY=\"sk-...\"' >> ~/.zshrc"
echo "   source ~/.zshrc"
echo ""
