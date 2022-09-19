#! /bin/bash

set -e
cargo build --release
upx --best --lzma target/release/live_speed_timer
