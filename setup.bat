@echo off
REM Bio-OM Expert Windows 一键安装脚本
setlocal

echo ========================================
echo   Bio-OM Expert 安装向导 (Windows)
echo ========================================
echo.

REM 1. Check Node.js
where node >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [X] Node.js 未安装，请从 https://nodejs.org/ 下载安装
    pause
    exit /b 1
)
echo [OK] Node.js 已安装

REM 2. Install Claude Code CLI
where claude >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo 正在安装 Claude Code CLI...
    npm install -g @anthropic-ai/claude-code
)
echo [OK] Claude CLI 已安装

REM 3. Check Python3
where python3 >nul 2>&1
if %ERRORLEVEL% NEQ 0 ( where python >nul 2>&1 )
if %ERRORLEVEL% NEQ 0 (
    echo [X] Python3 未安装，请从 https://python.org/ 下载安装
    pause
    exit /b 1
)
echo [OK] Python3 已安装

REM 4. Install python-docx
pip install python-docx --quiet 2>nul

REM 5. Install Skills
set SKILLS_DIR=%USERPROFILE%\.claude\skills
set SCRIPT_DIR=%~dp0

echo.
echo 正在安装 Skills...

for %%s in (content-writing web-research url-research local-research report-generator) do (
    if exist "%SCRIPT_DIR%skills\%%s" (
        xcopy /E /I /Y "%SCRIPT_DIR%skills\%%s" "%SKILLS_DIR%\%%s" >nul 2>&1
        echo   [OK] %%s
    ) else (
        echo   [!] %%s (源目录不存在，跳过)
    )
)

echo.
echo ========================================
echo   安装完成！
echo ========================================
echo.
pause
