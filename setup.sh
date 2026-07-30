#!/bin/bash
# Bio-OM Expert v1.3.0 — 一键安装脚本
# 适用于 macOS / Linux

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Bio-OM Expert v1.3.0 安装向导${NC}"
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
echo "━━━ 1/5 检查 Node.js ━━━"
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
echo "━━━ 2/5 检查 Claude Code CLI ━━━"
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
echo "━━━ 3/5 检查 Python3 ━━━"
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
echo "━━━ 4/5 安装 python-docx ━━━"
$PYTHON -m pip install python-docx --quiet 2>/dev/null && \
    echo -e "✅ python-docx 已安装" || \
    echo -e "${YELLOW}⚠️  python-docx 安装失败（Word 导出功能不可用）${NC}"

# ── 5. 安装 Skills ──
echo ""
echo "━━━ 5/5 安装 Bio-OM Expert Skills ━━━"
SKILLS_DIR="$HOME/.claude/skills"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

mkdir -p "$SKILLS_DIR"

SKILL_COUNT=0
for json_file in "$SCRIPT_DIR/Skills"/*.json; do
    if [ -f "$json_file" ]; then
        skill_name=$(basename "$json_file")
        cp "$json_file" "$SKILLS_DIR/$skill_name"
        echo -e "  ✅ ${GREEN}${skill_name}${NC}"
        SKILL_COUNT=$((SKILL_COUNT + 1))
    fi
done

if [ "$SKILL_COUNT" -eq 0 ]; then
    echo -e "  ${YELLOW}⚠️  未找到 Skill 文件，请确认 Skills/ 目录存在${NC}"
else
    echo -e "  📦 已安装 ${GREEN}${SKILL_COUNT}${NC} 个 Skills"
fi

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}  ✅ 安装完成！${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo "🚀 启动应用:"
echo "  macOS:   双击 Bio-OM Expert.app"
echo "  Windows: 双击 Bio-OM Expert.exe（或从开始菜单启动）"
echo ""
echo "📋 前置检查清单:"
echo "  □  Node.js 已安装     → node --version"
echo "  □  Claude CLI 已安装    → claude --version"
echo "  □  Anthropic API Key    → echo \$ANTHROPIC_API_KEY"
echo "  □  Skills 已安装        → ls ~/.claude/skills/"
echo "  □  python-docx 已安装   → pip3 show python-docx"
echo ""
echo "💡 提示: 将 API Key 写入 ~/.zshrc 永久生效:"
echo "   echo 'export ANTHROPIC_API_KEY=\"sk-...\"' >> ~/.zshrc"
echo "   source ~/.zshrc"
echo ""
