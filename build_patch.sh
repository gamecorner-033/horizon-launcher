#!/bin/bash
set -euo pipefail

source /root/.cargo/env
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export CARGO_TARGET_DIR="/mnt/g/cargo_target_horizon"
export RUST_MIN_STACK=67108864

cd /mnt/g/horizon

echo "Building patched horizon-client for aarch64-unknown-linux-gnu..."
cargo build --release -p horizon-client --bin horizon-client --target aarch64-unknown-linux-gnu

echo "Copying built binary..."
cp "${CARGO_TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-client" /mnt/g/horizon-muos-stage/ROMS/PORTS/horizon/bin/horizon-client
cp "${CARGO_TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-client" /mnt/g/horizon/bins_arm64/horizon-client

ls -lh /mnt/g/horizon-muos-stage/ROMS/PORTS/horizon/bin/horizon-client
echo "BUILD_COMPLETE_SUCCESS"
