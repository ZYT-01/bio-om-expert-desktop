@echo off
REM Bio-OM Expert v1.3.1 — Windows 一键安装脚本
setlocal enabledelayedexpansion

echo.
echo ========================================
echo   Bio-OM Expert v1.3.1 安装向导 (Windows^)
echo ========================================
echo.

REM ── 平台检测 ──
echo [*] 平台: Windows
echo.

REM ── 1. Check Node.js ──
echo ─── 1/6 检查 Node.js ───
where node >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [X] Node.js 未安装
    echo.
    echo 请从 https://nodejs.org/ 下载并安装 LTS 版本
    echo 安装时勾选"Add to PATH"
    echo.
    pause
    exit /b 1
)
for /f "tokens=*" %%v in ('node --version') do echo [OK] Node.js %%v

REM ── 2. Check/Install Claude CLI ──
echo.
echo ─── 2/6 检查 Claude Code CLI ───
where claude >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [*] Claude Code CLI 未安装，正在安装...
    npm install -g @anthropic-ai/claude-code
    if %ERRORLEVEL% NEQ 0 (
        echo [X] Claude CLI 安装失败，请检查网络连接
        pause
        exit /b 1
    )
    echo [OK] Claude CLI 安装完成
) else (
    echo [OK] Claude CLI 已安装
)

REM ── 3. Check Python ──
echo.
echo ─── 3/6 检查 Python ───
set PYTHON=
where python3 >nul 2>&1
if %ERRORLEVEL% EQU 0 ( set "PYTHON=python3" )
if not defined PYTHON (
    where python >nul 2>&1
    if %ERRORLEVEL% EQU 0 ( set "PYTHON=python" )
)
if not defined PYTHON (
    echo [X] Python3 未安装
    echo.
    echo 请从 Microsoft Store 搜索"Python 3"安装
    echo 或访问 https://python.org/ 下载
    echo 安装时勾选"Add Python to PATH"
    echo.
    pause
    exit /b 1
)
%PYTHON% --version
echo [OK] Python 已安装

REM ── 4. Install python-docx ──
echo.
echo ─── 4/6 安装 python-docx ───
%PYTHON% -m pip install python-docx --quiet 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [OK] python-docx 已安装
) else (
    echo [!] python-docx 安装失败（Word 导出功能不可用）
)

REM ── 5. Install Skills（双目录） ──
set "APP_SUPPORT=%APPDATA%\com.bio-om.expert\skills"
set "CLAUDE_SKILLS=%USERPROFILE%\.claude\skills"
set "SCRIPT_DIR=%~dp0"

echo.
echo ─── 5/6 安装 Bio-OM Expert Skills ───

if not exist "%APP_SUPPORT%" mkdir "%APP_SUPPORT%"
if not exist "%CLAUDE_SKILLS%" mkdir "%CLAUDE_SKILLS%"

REM JSON manifests → AppData
set JSON_COUNT=0
if exist "%SCRIPT_DIR%skills-manifest\" (
    for %%f in ("%SCRIPT_DIR%skills-manifest\*.json") do (
        copy /Y "%%f" "%APP_SUPPORT%" >nul 2>&1
        echo   [JSON] %%~nxf -^> AppData
        set /a JSON_COUNT+=1
    )
)

REM SKILL.md files → ~/.claude/skills/
set MD_COUNT=0
if exist "%SCRIPT_DIR%skills\" (
    for %%f in ("%SCRIPT_DIR%skills\*.md") do (
        set "skill_name=%%~nf"
        if not exist "%CLAUDE_SKILLS%\!skill_name!" mkdir "%CLAUDE_SKILLS%\!skill_name!"
        copy /Y "%%f" "%CLAUDE_SKILLS%\!skill_name!\SKILL.md" >nul 2>&1
        echo   [MD]   !skill_name!/SKILL.md -^> ~/.claude/skills/
        set /a MD_COUNT+=1
    )
)

echo   JSON: %JSON_COUNT% 个 ^| SKILL.md: %MD_COUNT% 个

REM ── 6. Install CLAUDE.md ──
echo.
echo ─── 6/6 安装 CLAUDE.md 配置 ───
set "CLAUDE_DIR=%USERPROFILE%\.claude"
if not exist "%CLAUDE_DIR%" mkdir "%CLAUDE_DIR%"

if exist "%SCRIPT_DIR%CLAUDE.md" (
    copy /Y "%SCRIPT_DIR%CLAUDE.md" "%CLAUDE_DIR%\CLAUDE.md" >nul 2>&1
    echo [OK] CLAUDE.md -^> ~/.claude/CLAUDE.md
) else (
    echo [!] 未找到 CLAUDE.md
)

echo.
echo ========================================
echo   安装完成！
echo ========================================
echo.
echo 启动应用:
echo   - 双击 Bio-OM Expert.exe
echo   - 或从开始菜单启动 Bio-OM Expert
echo.
echo 前置检查清单:
echo   [ ] Node.js       — node --version
echo   [ ] Claude CLI    — claude --version
echo   [ ] API Key       — set ANTHROPIC_API_KEY=sk-...
echo   [ ] Skills JSON   — dir %%APPDATA%%\com.bio-om.expert\skills\
echo   [ ] Skills MD     — dir %%USERPROFILE%%\.claude\skills\
echo   [ ] CLAUDE.md     — type %%USERPROFILE%%\.claude\CLAUDE.md
echo   [ ] python-docx   — pip show python-docx
echo.
echo 提示: 将 API Key 设为永久环境变量:
echo   1. 打开"系统属性" → "环境变量"
echo   2. 新建用户变量: 变量名 ANTHROPIC_API_KEY, 值 sk-...
echo.
pause
