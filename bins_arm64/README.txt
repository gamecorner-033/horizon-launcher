================================================================================
HORIZON 26.3 - Anbernic RG35XX SP / H700 Handheld Port for muOS
================================================================================

CREDITS & ACKNOWLEDGEMENTS:
  - Original Project: Horizon (Minecraft Java Edition rewritten in Rust)
    Website: https://playhorizon.dev
    GitHub:  https://github.com/horizon-rust/horizon
    Lead Author: Ludovic Marie (@ludovic111, https://lu4ovic.com)
    Licensed under the MIT License.
  - Handheld muOS Port: gamecorner-033
    GitHub: https://github.com/gamecorner-033/horizon-muos

DISCLAIMER:
  NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG
  OR MICROSOFT. Minecraft is a trademark of Mojang Synergies AB. This software
  contains NO Minecraft proprietary code or assets.

TARGET HARDWARE:
  - Anbernic RG35XX SP / RG34XX SP / RG40XX H / RG40XX V / RG28XX / RG35XX Plus
  - SoC: Allwinner H700 (Quad-core ARM Cortex-A53 @ 1.5 GHz)
  - GPU: Mali-G31 MP2
  - OS:  muOS (Banana, Beans, or newer)

--------------------------------------------------------------------------------
INSTALLATION & 1-TIME ASSET SETUP:
--------------------------------------------------------------------------------
1. Connect your muOS SD card (SD1 or SD2) to your PC.
2. Copy the 'ROMS' folder from this zip directly to the root of your SD card.
   Ensure the directory structure matches:
     SD:/ROMS/PORTS/Horizon.sh
     SD:/ROMS/PORTS/horizon/
3. Run the legal 1-time asset installer on your PC:
     - On Windows: Double-click 'Install_Assets.bat' inside SD:/ROMS/PORTS/horizon/
     - On macOS / Linux: Run './Install_Assets.sh' inside SD:/ROMS/PORTS/horizon/
   This downloads the official 26.3 client jar directly from Mojang's CDN,
   and extracts clean vanilla textures, fonts, data, and sounds.
   (The 8x8 texture pack is provided in resourcepacks/ as an optional choice
   and is disabled by default).
4. Safely eject the SD card and insert it back into your handheld console.
5. In muOS, navigate to "Ports" (or Explore Content -> ROMS -> PORTS) and select
   "Horizon.sh" to launch!

--------------------------------------------------------------------------------
CONTROLS (Bedrock / Handheld Layout):
--------------------------------------------------------------------------------
  - D-Pad:          Move forward / backward / strafe left / strafe right
  - Look Mode:      Press Select or face button combinations to look / aim
  - A (South):      Jump / Confirm menu
  - B (East):       Sneak / Cancel / Drop
  - X (West):       Open Handheld Bedrock Crafting UI / Inventory
  - Y (North):      Chat / Quick Action
  - R1 / RB:        Attack / Mine block
  - L1 / LB:        Use / Place block / Interact
  - R2 / RT:        Fast break / Veinminer trigger
  - L2 / LT:        Off-hand item use
  - Start:          Pause menu / Game options
  - Select:         Third-person perspective toggle / Look stick toggle

--------------------------------------------------------------------------------
PORT OPTIMIZATIONS INCLUDED:
--------------------------------------------------------------------------------
  - 100% Native ARM64 (aarch64) Linux execution - zero emulation overhead.
  - Tailored Bedrock-style 640x480 Handheld Crafting UI.
  - Optional 8x8 texture pack included in resourcepacks/ (off by default).
  - Aggressive chunk culling and memory tuning for 1GB-2GB RAM handhelds.
  - Pre-integrated WAILA inspector and Veinminer rust mods.

TROUBLESHOOTING:
  - If the game does not start, inspect 'ROMS/PORTS/horizon/launch.log' or
    'ROMS/PORTS/horizon/logs/latest.log' on your SD card.
================================================================================
