@echo off
REM Button 2: start one server + two native clients for manual testing.
REM Pass extra args through, e.g.:
REM   start-two-clients.bat -Port 5050
REM   start-two-clients.bat -Release
REM   start-two-clients.bat -StrictPort
REM   start-two-clients.bat -Help
setlocal
set "SCRIPT_DIR=%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%tools\dev-launcher\Start-TwoClients.ps1" %*
exit /b %ERRORLEVEL%
