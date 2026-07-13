#!/bin/bash
# Bio-OM Expert 一键安装脚本
# 适用于 macOS / Linux

set -e

echo "========================================"
echo "  Bio-OM Expert 安装向导"
echo "========================================"
echo ""

# 1. 检查 Node.js
if ! command -v node &>/dev/null; then
    echo "❌ 未检测到 Node.js，请先安装: https://nodejs.org/"
    exit 1
fi
echo "✅ Node.js $(node --version)"

# 2. 安装 Claude Code CLI
if ! command -v claude &>/dev/null; then
    echo "📦 正在安装 Claude Code CLI..."
    npm install -g @anthropic-ai/claude-code
fi
echo "✅ Claude CLI $(claude --version 2>&1 | head -1)"

# 3. 检查 Python3
if ! command -v python3 &>/dev/null; then
    echo "❌ 未检测到 Python3，请先安装"
    exit 1
fi
echo "✅ Python3 $(python3 --version)"

# 4. 安装 python-docx（生成 Word 文档需要）
python3 -m pip install python-docx --quiet 2>/dev/null || true

# 5. 安装 Skills
SKILLS_DIR="$HOME/.claude/skills"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo ""
echo "📦 安装 Bio-OM Expert Skills..."

install_skill() {
    local name=$1
    local target="$SKILLS_DIR/$name"
    local source="$SCRIPT_DIR/skills/$name"
    mkdir -p "$target"
    if [ -d "$source" ]; then
        cp -r "$source"/* "$target/" 2>/dev/null
        echo "  ✅ $name"
    else
        echo "  ⚠️ $name (源目录不存在，跳过)"
    fi
}

install_skill "content-writing"
install_skill "web-research"
install_skill "url-research"
install_skill "local-research"
install_skill "report-generator"

echo ""
echo "========================================"
echo "  ✅ 安装完成！"
echo "========================================"
echo ""
echo "启动方式:"
echo "  macOS:  双击 Bio-OM Expert.app"
echo "  Windows: 双击 Bio-OM Expert.msi 安装后启动"
echo ""
echo "前置检查清单:"
echo "  □ Node.js 已安装"
echo "  □ Claude Code CLI 已安装"
echo "  □ Anthropic API Key 已配置"
echo "  □ Skills 已安装"
echo ""
