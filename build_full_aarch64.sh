#!/bin/bash
set -euo pipefail

source /root/.cargo/env
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export CARGO_TARGET_DIR=/mnt/g/cargo_target_horizon

cd /mnt/g/horizon

echo "=== Building horizon-client for aarch64 ==="
cargo build --release -p horizon-client --bin horizon-client --target aarch64-unknown-linux-gnu

echo "=== Building horizon-prepare for aarch64 ==="
cargo build --release -p horizon-prepare --bin horizon-prepare --target aarch64-unknown-linux-gnu

echo "=== Build finished! Checking binaries ==="
ls -lh /mnt/g/cargo_target_horizon/aarch64-unknown-linux-gnu/release/horizon-client
ls -lh /mnt/g/cargo_target_horizon/aarch64-unknown-linux-gnu/release/horizon-prepare
ls -lh /mnt/g/cargo_target_horizon/aarch64-unknown-linux-gnu/release/horizon-ctl

file /mnt/g/cargo_target_horizon/aarch64-unknown-linux-gnu/release/horizon-client
file /mnt/g/cargo_target_horizon/aarch64-unknown-linux-gnu/release/horizon-prepare
