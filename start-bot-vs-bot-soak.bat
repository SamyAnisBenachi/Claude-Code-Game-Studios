@echo off
REM PROMPT 1603: launch a headless bot-vs-bot soak run (default 5 min).
REM Pass extra args through, e.g.:
REM   start-bot-vs-bot-soak.bat -DurationSeconds 60
REM   start-bot-vs-bot-soak.bat -Port 5050 -Release
REM   start-bot-vs-bot-soak.bat -Help
setlocal
set "SCRIPT_DIR=%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%tools\dev-launcher\Start-BotVsBotSoak.ps1" %*
exit /b %ERRORLEVEL%
