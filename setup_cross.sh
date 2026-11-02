#!/bin/bash
set -euo pipefail

mkdir -p /root/.cargo
cat << 'CONFIG_EOF' > /root/.cargo/config.toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
CONFIG_EOF

echo "Cargo config:"
cat /root/.cargo/config.toml

echo "Testing aarch64 pkg-config:"
PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig pkg-config --cflags --libs alsa libudev

echo "Cross-compilation environment ready!"
