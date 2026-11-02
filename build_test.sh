#!/bin/bash
set -euo pipefail

source /root/.cargo/env
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export CARGO_TARGET_DIR=/tmp/horizon-target-aarch64

cd /mnt/g/horizon

echo "Building horizon-ctl for aarch64-unknown-linux-gnu (test run)..."
cargo build --release -p horizon-client --bin horizon-ctl --target aarch64-unknown-linux-gnu

echo "Test binary built! Checking file details:"
file /tmp/horizon-target-aarch64/aarch64-unknown-linux-gnu/release/horizon-ctl
