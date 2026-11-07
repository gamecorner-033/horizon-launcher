# Horizon for muOS (Anbernic RG35XX SP & H700 Handhelds)

<p align="center">
  <b>A native, high-performance port of the Horizon Rust engine for Anbernic Allwinner H700 retro handhelds running muOS.</b>
</p>

<p align="center">
  <a href="#credits--acknowledgements"><b>Credits & Acknowledgements</b></a> ·
  <a href="#quick-start--installation"><b>Quick Start Guide</b></a> ·
  <a href="#features--handheld-optimizations"><b>Handheld Features</b></a> ·
  <a href="#controls"><b>Controls</b></a> ·
  <a href="CHANGELOG.md"><b>Changelog</b></a> ·
  <a href="#building-from-source"><b>Building from Source</b></a>
</p>

---

## Credits & Acknowledgements

This port is built upon the incredible work of the **Horizon Project**:

- **Upstream Project**: [Horizon - The game you know, rebuilt in Rust](https://github.com/horizon-rust/horizon)
- **Lead Author & Creator**: [Ludovic Marie (@ludovic111)](https://github.com/ludovic111) / [lu4ovic.com](https://lu4ovic.com)
- **Official Website**: [playhorizon.dev](https://playhorizon.dev)
- **Official Discord**: [discord.gg/playhorizon](https://discord.gg/playhorizon)
- **Original License**: [MIT License](LICENSE)

Special thanks to all upstream contributors who made a pure Rust, zero-Java reimplementation of Minecraft: Java Edition possible.

> [!IMPORTANT]
> **Legal Notice & Disclaimer**:
> This is an independent, unofficial community port. It is **NOT AN OFFICIAL MINECRAFT PRODUCT** and is **NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT**.
> Minecraft is a trademark of Mojang Synergies AB. This repository and its release archives contain **NO proprietary Minecraft code, jars, textures, sounds, or assets**. Users must supply their own game files using the provided 1-time asset installer.

---

## Supported Devices

- **Anbernic RG35XX SP** (Recommended)
- **Anbernic RG34XX SP**
- **Anbernic RG40XX H / RG40XX V**
- **Anbernic RG28XX**
- **Anbernic RG35XX Plus / RG35XX 2024**
- **Operating System**: [muOS](https://muos.dev/) (Banana, Beans, or newer)
- **Hardware**: Allwinner H700 SoC (4× ARM Cortex-A53 @ 1.5 GHz), Mali-G31 MP2 GPU, 1GB–2GB LPDDR4 RAM

---

## Quick Start & Installation

Setting up Horizon on muOS takes less than 3 minutes.

### Step 1: Download & Extract
1. Download the latest `horizon-rg35xxsp-muos.zip` from [Releases](https://github.com/gamecorner-033/horizon-muos/releases).
2. Insert your muOS SD card (SD1 or SD2) into your computer.
3. Extract the `ROMS` folder from the zip directly to the root of your SD card.
   Your SD card will have:
   ```text
   SD:/ROMS/PORTS/Horizon.sh
   SD:/ROMS/PORTS/horizon/
   ```

### Step 2: 1-Click Asset Installer (PC Setup)
Due to licensing and distribution rules, official Mojang assets cannot be redistributed. A convenient 1-time installer is included:

- **On Windows**:
  1. Open your SD card in File Explorer.
  2. Navigate to `ROMS/PORTS/horizon/`.
  3. Double-click `Install_Assets.bat`.
- **On macOS / Linux**:
  1. Open a terminal.
  2. Navigate to `ROMS/PORTS/horizon/`.
  3. Run `chmod +x Install_Assets.sh && ./Install_Assets.sh`.

The script automatically downloads the official Java Edition 26.3 client jar from Mojang's CDN and extracts clean vanilla textures, models, definitions, and sound tables into `resources/`. (The 8×8 performance texture pack is optionally included in `resourcepacks/` and disabled by default).

### Step 3: Play!
1. Safely eject the SD card and insert it back into your handheld device.
2. Turn on your handheld and open muOS.
3. Go to **Ports** (or *Explore Content* → `ROMS` → `PORTS`).
4. Select **Horizon.sh** to launch!

---

## Features & Handheld Optimizations

- **100% Native ARM64 (aarch64)**: Runs bare-metal on the 64-bit Linux kernel with zero Java runtime overhead and zero emulation.
- **Bedrock-Style Handheld Crafting UI**: Complete redesign of crafting grids tailored for 640×480 handheld screens, supporting D-Pad category filtering and 1-click recipe crafting without awkward mouse dragging.
- **Mali-G31 GPU Potato Preset**:
  - Optional lightweight 8×8 texture pack provided in `resourcepacks/` (off by default).
  - Directional chunk reachability culling to eliminate visual pop-in while maintaining steady 60 FPS in standard terrain.
  - Synchronous entity culling and fast mesh generation.
  - Native 640×480 framebuffer rendering with hardware-accelerated GLES / Panfrost.
- **Native Gamepad & evdev Input**: Full out-of-the-box controller support mapped directly to muOS kernel evdev input events.
- **Built-in Quality-of-Life Handheld Mods**:
  - **WAILA (What Am I Looking At)**: Instant block and entity HUD inspector.
  - **Veinminer**: Streamlined mining for handheld ergonomics.

---

## Controls

Horizon utilizes a Bedrock Edition layout configured for retro handheld ergonomics:

| Button | Action |
| --- | --- |
| **D-Pad** | Movement (Walk forward, backward, strafe) |
| **A (South)** | Jump / Menu Confirm |
| **B (East)** | Sneak / Cancel / Drop |
| **X (West)** | Inventory & Bedrock Crafting UI |
| **Y (North)** | Chat / Quick Action |
| **R1 (Right Bumper)** | Attack / Mine block |
| **L1 (Left Bumper)** | Use / Place block / Interact |
| **R2 (Right Trigger)** | Fast Break / Veinminer |
| **L2 (Left Trigger)** | Off-hand item use |
| **Start** | Pause Menu & Options |
| **Select** | Camera Perspective (1st / 3rd person) |

---

## Building from Source

To cross-compile the handheld ARM64 client binary on a Linux or WSL2 development machine:

```bash
# 1. Install prerequisites
sudo apt update
sudo apt install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libasound2-dev:arm64 libudev-dev:arm64

# 2. Add Rust target
rustup target add aarch64-unknown-linux-gnu

# 3. Clone and build
git clone https://github.com/gamecorner-033/horizon-muos.git
cd horizon-muos
cargo build --release --target aarch64-unknown-linux-gnu -p horizon-client
```

To create a clean release package ready for muOS:
```bash
./package_muos.sh
```

---

## License

This project is licensed under the [MIT License](LICENSE), matching upstream Horizon. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for included third-party libraries.
