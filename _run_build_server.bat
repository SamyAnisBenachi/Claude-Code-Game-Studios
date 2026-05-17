@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=x64 -no_logo
set CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
set CARGO_INCREMENTAL=0
cd /d D:\_DEV\Work\Claude-Code-Game-Studios
echo === BEGIN SERVER BUILD ===
cargo build -p server
echo === END SERVER BUILD (exit=%ERRORLEVEL%) ===
