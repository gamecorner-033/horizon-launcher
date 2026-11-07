#!/bin/bash
set -euo pipefail

STAGE_DIR="/mnt/g/horizon-muos-stage"
ROMS_DIR="${STAGE_DIR}/ROMS/PORTS"
GAME_DIR="${ROMS_DIR}/horizon"
BIN_DIR="${GAME_DIR}/bin"
LIBS_DIR="${GAME_DIR}/libs"
MODS_DIR="${GAME_DIR}/horizon-mods"
RES_SRC="/mnt/c/Users/Brookes/AppData/Roaming/horizon-launcher/versions/nightly-2026-10-31-d91cab6/.local/horizon-client/26.3"
CONFIG_SRC="/mnt/g/horizon/bins_arm64"
BUILD_DIR="/mnt/g/cargo_target_horizon/aarch64-unknown-linux-gnu/release"
EXPORT_DIR="/mnt/g/horizon-export"

echo "=== Staging muOS Port Layout in ${STAGE_DIR} ==="
mkdir -p "${ROMS_DIR}"
mkdir -p "${BIN_DIR}"
mkdir -p "${LIBS_DIR}"
mkdir -p "${GAME_DIR}/resources"
mkdir -p "${GAME_DIR}/resourcepacks"
mkdir -p "${MODS_DIR}"
mkdir -p "${GAME_DIR}/saves"
mkdir -p "${GAME_DIR}/logs"

echo "=== Copying Binaries ==="
cp "${BUILD_DIR}/horizon-client" "${BIN_DIR}/"
cp "${BUILD_DIR}/horizon-ctl" "${BIN_DIR}/" 2>/dev/null || cp "${CONFIG_SRC}/horizon-ctl" "${BIN_DIR}/"
cp "${BUILD_DIR}/horizon-prepare" "${BIN_DIR}/"
chmod +x "${BIN_DIR}"/*

echo "=== Copying Bundled Fallback Shared Libraries ==="
cp -a /usr/lib/aarch64-linux-gnu/libasound.so* "${LIBS_DIR}/"
cp -a /usr/lib/aarch64-linux-gnu/libudev.so* "${LIBS_DIR}/"

echo "=== Copying Pre-prepared Minecraft 26.3 Resources ==="
if [ ! -f "${GAME_DIR}/resources/asset-index.json" ]; then
    cp -r "${RES_SRC}"/* "${GAME_DIR}/resources/"
else
    echo "Resources already staged, skipping full copy."
fi

echo "=== Installing 8x8 (8-bit) Texture Pack ==="
cp "/mnt/g/horizon/bins_arm64/8x8.zip" "${GAME_DIR}/resourcepacks/8x8.zip"
unzip -n -q "/mnt/g/horizon/bins_arm64/8x8.zip" "assets/*" -d "${GAME_DIR}/resources/" 2>/dev/null || true
# Invalidate old 16x16 pre-baked sprite cache so 8x8 textures are used natively
rm -f "${GAME_DIR}/resources/horizon-sprites.cache"

echo "=== Copying Potato Preset Configs ==="
cp "${CONFIG_SRC}/options.txt" "${GAME_DIR}/options.txt"
cp "${CONFIG_SRC}/horizon-graphics.txt" "${GAME_DIR}/horizon-graphics.txt"
cp "${CONFIG_SRC}/horizon-mods.txt" "${GAME_DIR}/horizon-mods.txt"

echo "=== Copying Sandboxed Mods ==="
if [ -d "${CONFIG_SRC}/horizon-mods" ]; then
    cp -r "${CONFIG_SRC}/horizon-mods"/* "${MODS_DIR}/"
fi

echo "=== Installing Horizon.sh Launcher Script ==="
cp "${CONFIG_SRC}/Horizon.sh" "${ROMS_DIR}/Horizon.sh"
chmod +x "${ROMS_DIR}/Horizon.sh"

echo "=== Writing Controller Config ==="
cat << 'EOF' > "${GAME_DIR}/horizon-controller.txt"
# Horizon Controller Layout for RG35XX SP / Handhelds
# Bedrock Edition default mapping
deadzone.left:0.15
deadzone.right:0.15
lookSensitivity:100
lookCurve:bedrock
invertY:false
glyphStyle:xbox
EOF

echo "=== Writing Port README ==="
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

echo "=== Packaging muOS Archive ==="
mkdir -p "${EXPORT_DIR}"

cd "${STAGE_DIR}"
echo "Creating ZIP archive at ${EXPORT_DIR}/horizon-rg35xxsp-muos.zip..."
rm -f "${EXPORT_DIR}/horizon-rg35xxsp-muos.zip"
zip -r -q "${EXPORT_DIR}/horizon-rg35xxsp-muos.zip" ROMS/

echo "=== Done! Package details: ==="
ls -lh "${EXPORT_DIR}/horizon-rg35xxsp-muos.zip"
