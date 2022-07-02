@echo off
cargo build --release
IF %ERRORLEVEL% NEQ 0 (
    exit %ERRORLEVEL%
)
upx --best --lzma target/release/live_speed_timer.exe
