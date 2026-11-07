# Changelog

All notable changes to the **Horizon for muOS** port (Anbernic RG34XX SP / RG35XX SP / H700) will be documented in this file.

The project follows an incremental versioning format (`1.01`, `1.02`, `1.03`...), working toward milestone **v2.0** (stable 30+ FPS at 3+ chunks render distance).

---

## [v1.01] - 2026-11-07

### Added
- **Scaling Filter (`filterMode`)**: Added support for choosing between `Linear` (smooth bilinear blit) and `Nearest` (sharp retro pixel-art) upscaling filters.
- **In-Game Filter Option**: Added an interactive "Scaling Filter" toggle in `Video Settings -> Image` and serialized in `horizon-graphics.txt`.
- **Extended Render Scale Range**: Expanded the resolution scale slider from 50%..=200% down to **25%..=200%** (35 distinct 5% steps).

### Changed
- **50% Potato Preset by Default**:
  - Pre-configured `resolutionScale: 50` and `filterMode: nearest` in the Potato performance preset and `horizon-graphics.txt`.
  - On 480p screens (RG34XX SP / RG35XX SP), 50% scale renders the 3D world at 240p vertical—providing a pixel-perfect 2× vertical integer upscale that relieves **75% of GPU fragment shading workload**.
  - All 2D UI elements (hearts, hotbar, text, WAILA, Bedrock crafting grid, menus) remain rendered at native 480p.
- **Clean Vanilla 16×16 Textures**:
  - `Install_Assets.bat` and `Install_Assets.sh` no longer bake/extract the 8×8 texture pack over the base game resources.
  - Base game now runs pure 16×16 vanilla assets directly from Mojang CDN to avoid texture atlas fragmentation and GPU cache misses. The 8×8 pack remains in `resourcepacks/` as an optional choice.
  - Sprite atlas cache is purged automatically upon re-running the installer.
- **Release Documentation**: Added explicit device support for the RG34XX SP alongside the RG35XX SP.

---

## [v1.00] - 2026-11-07

### Added
- **Initial Handheld Port Release**: Native 64-bit ARM Linux (`aarch64-unknown-linux-gnu`) execution for Allwinner H700 handhelds on muOS.
- **1-Click Legal Asset Setup**: Bundled `Install_Assets.bat` (Windows) and `Install_Assets.sh` (macOS/Linux) in `ROMS/PORTS/horizon/` to fetch official assets legally from Mojang's CDN.
- **Frustum & Occlusion Culling**: Eliminated camera pop-in by widening culling frustum FOV and keeping the immediate 3×3 chunks around the player unconditionally visible.
- **Bedrock-Style Handheld Crafting UI**: Tailored full-screen crafting layout for 480p handheld displays with smooth D-pad category navigation.
- **Native Gamepad Controls**: Pre-configured `muOS-Keys` kernel evdev mapping via gilrs.
- **Integrated Handheld Mods**: Built-in WAILA HUD inspector and Veinminer.
- **Upstream Sync**: Rebased cleanly on top of latest upstream Horizon codebase.
- **Author Credits**: Full attribution to Ludovic Marie and the Horizon Rust project under the MIT License.
