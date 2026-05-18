@echo off
REM Button 1: fetch origin, fast-forward main, rebuild server + client.
REM Pass extra args through, e.g.:
REM   update-latest-main.bat -Release
REM   update-latest-main.bat -Force
REM   update-latest-main.bat -Help
setlocal
set "SCRIPT_DIR=%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%tools\dev-launcher\Update-LatestMain.ps1" %*
exit /b %ERRORLEVEL%
