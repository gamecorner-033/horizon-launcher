#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RES_DIR="${SCRIPT_DIR}/resources"

echo "================================================================================"
echo "          HORIZON 26.3 - MINECRAFT ASSET INSTALLER FOR muOS"
echo "================================================================================"
echo ""
echo "This 1-time installer downloads official Minecraft 26.3 assets directly from"
echo "Mojang / Microsoft official servers onto this SD card."
echo ""
echo "No proprietary or copyrighted assets are distributed with this port."
echo "================================================================================"
echo ""

mkdir -p "${RES_DIR}"

CLIENT_JAR_URL="https://piston-data.mojang.com/v1/objects/e877b6a07acd633fb3bb475002175cec036e7b87/client.jar"
ASSET_INDEX_URL="https://piston-meta.mojang.com/v1/packages/abfaa525f923f807df8b4e4d29c1b5e3a104adbe/34.json"

echo "[1/3] Downloading official Minecraft 26.3 client jar from Mojang CDN (41 MB)..."
if command -v curl >/dev/null 2>&1; then
    curl -f -L --progress-bar "${CLIENT_JAR_URL}" -o "${SCRIPT_DIR}/client_temp.jar"
elif command -v wget >/dev/null 2>&1; then
    wget -q --show-progress "${CLIENT_JAR_URL}" -O "${SCRIPT_DIR}/client_temp.jar"
else
    echo "[ERROR] Neither curl nor wget found! Please install curl or wget."
    exit 1
fi

echo "[2/3] Extracting textures, models, fonts, and data into resources folder..."
if command -v unzip >/dev/null 2>&1; then
    unzip -q -o "${SCRIPT_DIR}/client_temp.jar" "assets/*" "data/*" "version.json" "pack.mcmeta" "pack.png" -d "${RES_DIR}" 2>/dev/null || true
elif command -v tar >/dev/null 2>&1; then
    tar -xf "${SCRIPT_DIR}/client_temp.jar" -C "${RES_DIR}" assets data version.json pack.mcmeta pack.png 2>/dev/null || true
fi
rm -f "${SCRIPT_DIR}/client_temp.jar"

echo "[3/3] Downloading official asset index manifest..."
if command -v curl >/dev/null 2>&1; then
    curl -f -s -L "${ASSET_INDEX_URL}" -o "${RES_DIR}/asset-index.json"
elif command -v wget >/dev/null 2>&1; then
    wget -q "${ASSET_INDEX_URL}" -O "${RES_DIR}/asset-index.json"
fi

# Ensure texture sprite cache is regenerated cleanly from official assets
rm -f "${RES_DIR}/horizon-sprites.cache"

echo ""
echo "================================================================================"
echo "SUCCESS! Minecraft 26.3 assets have been installed to your SD card."
echo ""
echo "You may now safely eject this SD card, insert it into your RG34XX / RG35XX,"
echo "and launch Horizon from the muOS Ports menu. Enjoy!"
echo "================================================================================"
