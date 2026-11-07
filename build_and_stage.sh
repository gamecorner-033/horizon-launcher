#!/bin/bash
set -euo pipefail

source /root/.cargo/env
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export CARGO_TARGET_DIR="/mnt/g/cargo_target_horizon"

TARGET_DIR="/mnt/g/cargo_target_horizon"
SAVED_BIN="/mnt/g/horizon/bins_arm64"
ROOT_BIN="/mnt/g/horizon/bins_arm64"

mkdir -p "${TARGET_DIR}"
mkdir -p "${SAVED_BIN}"
mkdir -p "${ROOT_BIN}"

cd /mnt/g/horizon

echo "=========================================================="
echo "Step 1: Building horizon-client & horizon-ctl for aarch64"
echo "=========================================================="
cargo build --release -p horizon-client --bin horizon-client --bin horizon-ctl --target aarch64-unknown-linux-gnu

echo "Copying horizon-client and horizon-ctl to safety..."
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-client" "${ROOT_BIN}/"
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-client" "${SAVED_BIN}/"
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-ctl" "${ROOT_BIN}/"
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-ctl" "${SAVED_BIN}/"

echo "=========================================================="
echo "Step 2: Building horizon-prepare for aarch64"
echo "=========================================================="
cargo build --release -p horizon-prepare --bin horizon-prepare --target aarch64-unknown-linux-gnu

echo "Copying horizon-prepare to safety..."
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-prepare" "${ROOT_BIN}/"
cp "${TARGET_DIR}/aarch64-unknown-linux-gnu/release/horizon-prepare" "${SAVED_BIN}/"

echo "=========================================================="
echo "Step 3: Staging muOS Package"
echo "=========================================================="
STAGE_DIR="/mnt/g/horizon-muos-stage"
ROMS_DIR="${STAGE_DIR}/ROMS/PORTS"
GAME_DIR="${ROMS_DIR}/horizon"
BIN_DIR="${GAME_DIR}/bin"
LIBS_DIR="${GAME_DIR}/libs"
MODS_DIR="${GAME_DIR}/horizon-mods"
RES_SRC="/mnt/c/Users/Brookes/AppData/Roaming/horizon-launcher/versions/nightly-2026-10-31-d91cab6/.local/horizon-client/26.3"
CONFIG_SRC="/mnt/g/horizon/bins_arm64"
EXPORT_DIR="/mnt/g/horizon-export"

rm -rf "${STAGE_DIR}"
mkdir -p "${ROMS_DIR}"
mkdir -p "${BIN_DIR}"
mkdir -p "${LIBS_DIR}"
mkdir -p "${GAME_DIR}/resources"
mkdir -p "${GAME_DIR}/resourcepacks"
mkdir -p "${MODS_DIR}"
mkdir -p "${GAME_DIR}/saves"
mkdir -p "${GAME_DIR}/logs"

cp "${ROOT_BIN}"/* "${BIN_DIR}/"
chmod +x "${BIN_DIR}"/*

echo "Copying fallback shared libraries..."
cp -a /usr/lib/aarch64-linux-gnu/libasound.so* "${LIBS_DIR}/"
cp -a /usr/lib/aarch64-linux-gnu/libudev.so* "${LIBS_DIR}/"

echo "Copying Minecraft 26.3 prepared resources..."
cp -r "${RES_SRC}"/* "${GAME_DIR}/resources/"

echo "Installing 8x8 (8-bit) texture pack into resourcepacks/ and baking into base resources..."
cp "/mnt/g/horizon/bins_arm64/8x8.zip" "${GAME_DIR}/resourcepacks/8x8.zip"
unzip -o -q "/mnt/g/horizon/bins_arm64/8x8.zip" "assets/*" -d "${GAME_DIR}/resources/" || true

echo "Copying Potato Preset configs..."
cp "${CONFIG_SRC}/options.txt" "${GAME_DIR}/options.txt"
cp "${CONFIG_SRC}/horizon-graphics.txt" "${GAME_DIR}/horizon-graphics.txt"
cp "${CONFIG_SRC}/horizon-mods.txt" "${GAME_DIR}/horizon-mods.txt"

echo "Copying Sandboxed Mods..."
cp -r "${CONFIG_SRC}/horizon-mods"/* "${MODS_DIR}/"

echo "Writing Horizon.sh launcher..."
cat << 'EOF' > "${ROMS_DIR}/Horizon.sh"
#!/bin/bash
# ==============================================================================
# Horizon 26.3 - Native Client for muOS (Anbernic RG35XX SP / H700 Handhelds)
# ==============================================================================

PORT_DIR="$(cd "$(dirname "$0")" && pwd)"
GAME_DIR="${PORT_DIR}/horizon"
BIN_DIR="${GAME_DIR}/bin"
LIBS_DIR="${GAME_DIR}/libs"
LOG_FILE="${GAME_DIR}/launch.log"

cd "${GAME_DIR}"

# Redirect stdout and stderr to launch.log for diagnostics
exec > "${LOG_FILE}" 2>&1

echo "========================================================"
echo "Starting Horizon 26.3 on muOS"
echo "Launch time    : $(date)"
echo "Game directory : ${GAME_DIR}"
echo "========================================================"

# Make sure binaries are executable
chmod +x "${BIN_DIR}/horizon-client" 2>/dev/null || true
chmod +x "${BIN_DIR}/horizon-ctl" 2>/dev/null || true
chmod +x "${BIN_DIR}/horizon-prepare" 2>/dev/null || true

# Shared library path including bundled fallbacks
export LD_LIBRARY_PATH="${LIBS_DIR}:${LD_LIBRARY_PATH:-/usr/lib:/lib}"

# Display detection (X11 / Wayland)
if [ -z "${DISPLAY:-}" ] && [ -e /tmp/.X11-unix/X0 ]; then
    export DISPLAY=:0
fi

# Performance & Graphics Environment for Mali-G31 (Panfrost / GLES / Vulkan)
export MESA_LOADER_DRIVER_OVERRIDE=panfrost
export PAN_MESA_DEBUG=gl3
export WGPU_BACKEND=vulkan,gl
export WGPU_POWER_PREF=low
export HORIZON_GPU_CULL=all
export HORIZON_NO_DISCORD=1

# Launch Horizon with Potato preset on native 640x480 screen
"${BIN_DIR}/horizon-client" \
    --game-dir "${GAME_DIR}" \
    --home "${GAME_DIR}" \
    --resources "${GAME_DIR}/resources" \
    --width 640 \
    --height 480 \
    --fullscreen

STATUS=$?
echo "Horizon exited with code: ${STATUS}"
exit ${STATUS}
EOF
chmod +x "${ROMS_DIR}/Horizon.sh"

echo "Writing controller mapping..."
cat << 'EOF' > "${GAME_DIR}/horizon-controller.txt"
deadzone.left:0.15
deadzone.right:0.15
lookSensitivity:100
lookCurve:bedrock
invertY:false
glyphStyle:xbox
EOF

echo "Writing port documentation..."
cat << 'EOF' > "${GAME_DIR}/README.txt"
================================================================================
HORIZON 26.3 - Anbernic RG35XX SP / Handheld Port (muOS)
================================================================================

Target Device: Anbernic RG35XX SP (Allwinner H700, Mali-G31 MP2, 2 GB RAM)
Operating System: muOS (Banana, Beans, or newer)

INSTALLATION:
  1. Insert your muOS SD card into your PC.
  2. Copy the 'ROMS' folder from this package into the root of your SD card (SD1 or SD2).
     - Result should be: SD:/ROMS/PORTS/Horizon.sh and SD:/ROMS/PORTS/horizon/...
  3. Reinsert SD card into RG35XX SP.
  4. In muOS menu, go to "Ports" or "Explore Content -> ROMS -> PORTS".
  5. Select "Horizon.sh" to launch!

FEATURES INCLUDED:
  - 100% Native ARM64 (aarch64) Linux execution - zero emulation overhead!
  - Fully pre-extracted Minecraft Java Edition 26.3 assets (offline ready).
  - Pre-tuned "Potato Performance Preset":
      * 640x480 native resolution (matched to the SP screen)
      * Render Distance: 5 chunks, Simulation Distance: 3 chunks
      * Entity culling & cave culling enabled
      * Battery saver mode enabled (2 worker threads for quad-core A53)
      * Fast graphics, no smooth lighting, mipmaps disabled
  - Pre-installed Sandboxed Mods:
      * Potato Performance Optimizer (UI mod for tuning settings on low-end hardware)
      * Adventurer's Backpack (wearable bundle & placed chest storage)
  - Native evdev gamepad controls via gilrs (Bedrock Edition layout).

CONTROLS:
  - D-Pad / Left Stick: Movement
  - Right Stick (or Face Buttons): Look / Camera
  - A (South): Jump
  - B (East): Sneak / Drop
  - X (West): Inventory / Crafting
  - Y (North): Chat / Quick action
  - R1 / RB: Attack / Mine
  - L1 / LB: Use / Place Block
  - Start: Pause Menu / Settings
  - Select: Perspective / Third Person

TROUBLESHOOTING:
  - If the game does not start, inspect 'ROMS/PORTS/horizon/launch.log' or
    'ROMS/PORTS/horizon/logs/latest.log' on your SD card.
================================================================================
EOF

echo "=========================================================="
echo "Step 4: Packaging ZIP and folder export"
echo "=========================================================="
mkdir -p "${EXPORT_DIR}/horizon-muos-port"
cp -r "${STAGE_DIR}/ROMS" "${EXPORT_DIR}/horizon-muos-port/"

cd "${STAGE_DIR}"
echo "Zipping port package..."
zip -r -q "${EXPORT_DIR}/horizon-rg35xxsp-muos.zip" ROMS/

echo "=========================================================="
echo "SUCCESS! Export complete:"
echo "=========================================================="
ls -lh "${EXPORT_DIR}/horizon-rg35xxsp-muos.zip"
ls -lh "${EXPORT_DIR}/horizon-muos-port/ROMS/PORTS/Horizon.sh"
ls -lh "${SAVED_BIN}"/*
