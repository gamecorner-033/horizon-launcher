@echo off
echo ======================================================================
echo Launching Horizon 26.3 muOS Port (ARM64) on PC via WSL2 / QEMU
echo ======================================================================
echo Window Size: 640x480 (RG35XX SP Native)
echo Preset     : Potato Performance Preset (5 RD, 3 Sim, 2 Threads)
echo Mods       : Potato Optimizer + Adventurer's Backpack
echo ======================================================================

wsl -u root bash -c "qemu-aarch64-static -L /usr/aarch64-linux-gnu -E LD_LIBRARY_PATH=/root/horizon-muos-stage/ROMS/PORTS/horizon/libs:/usr/lib/aarch64-linux-gnu -E WAYLAND_DISPLAY= -E DISPLAY=:0 -E LIBGL_ALWAYS_SOFTWARE=1 /root/horizon-muos-stage/ROMS/PORTS/horizon/bin/horizon-client --resources /root/horizon-muos-stage/ROMS/PORTS/horizon/resources --game-dir /root/horizon-muos-stage/ROMS/PORTS/horizon --width 640 --height 480"

pause
