@echo off
REM Bio-OM Expert v1.3.0 — Windows 一键安装脚本
setlocal enabledelayedexpansion

echo.
echo ========================================
echo   Bio-OM Expert v1.3.0 安装向导 (Windows^)
echo ========================================
echo.

REM ── 平台检测 ──
echo [*] 平台: Windows
echo.

REM ── 1. Check Node.js ──
echo ─── 1/5 检查 Node.js ───
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
echo ─── 2/5 检查 Claude Code CLI ───
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
echo ─── 3/5 检查 Python ───
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
echo ─── 4/5 安装 python-docx ───
%PYTHON% -m pip install python-docx --quiet 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [OK] python-docx 已安装
) else (
    echo [!] python-docx 安装失败（Word 导出功能不可用）
)

REM ── 5. Install Skills ──
set "SKILLS_DIR=%USERPROFILE%\.claude\skills"
set "SCRIPT_DIR=%~dp0"

echo.
echo ─── 5/5 安装 Bio-OM Expert Skills ───

if not exist "%SKILLS_DIR%" mkdir "%SKILLS_DIR%"

set COUNT=0
for %%f in ("%SCRIPT_DIR%Skills\*.json") do (
    copy /Y "%%f" "%SKILLS_DIR%" >nul 2>&1
    echo   [OK] %%~nxf
    set /a COUNT+=1
)

if %COUNT% EQU 0 (
    echo   [!] 未找到 Skill 文件，请确认 Skills\ 目录存在
) else (
    echo   已安装 %COUNT% 个 Skills
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
echo   [ ] Skills        — dir %%USERPROFILE%%\.claude\skills\
echo   [ ] python-docx   — pip show python-docx
echo.
echo 提示: 将 API Key 设为永久环境变量:
echo   1. 打开"系统属性" → "环境变量"
echo   2. 新建用户变量: 变量名 ANTHROPIC_API_KEY, 值 sk-...
echo.
pause
