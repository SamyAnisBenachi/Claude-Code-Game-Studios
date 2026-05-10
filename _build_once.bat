@echo off
call "C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=x64 -no_logo
echo === BEGIN CARGO BUILD ===
cargo build -p server -v
echo === END CARGO BUILD (exit=%ERRORLEVEL%) ===
