#!/bin/bash
# ==============================================================================
# Horizon 26.3 - Native Client for muOS (Anbernic RG35XX SP / RG34XX SP)
# ==============================================================================

# 1. Discover PortMaster control environment
XDG_DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}

if [ -d "/opt/system/Tools/PortMaster/" ]; then
  controlfolder="/opt/system/Tools/PortMaster"
elif [ -d "/opt/tools/PortMaster/" ]; then
  controlfolder="/opt/tools/PortMaster"
elif [ -d "$XDG_DATA_HOME/PortMaster/" ]; then
  controlfolder="$XDG_DATA_HOME/PortMaster"
elif [ -d "/roms/ports/PortMaster" ]; then
  controlfolder="/roms/ports/PortMaster"
elif [ -d "/mnt/sdcard/ROMS/ports/PortMaster" ]; then
  controlfolder="/mnt/sdcard/ROMS/ports/PortMaster"
else
  controlfolder="/roms/ports/PortMaster"
fi

if [ -f "$controlfolder/control.txt" ]; then
  source "$controlfolder/control.txt"
  [ -f "${controlfolder}/device_info.txt" ] && source "${controlfolder}/device_info.txt"
  [ -f "${controlfolder}/mod_${CFW_NAME}.txt" ] && source "${controlfolder}/mod_${CFW_NAME}.txt"
  get_controls
fi

PORT_DIR="$(cd "$(dirname "$0")" && pwd)"
GAME_DIR="${PORT_DIR}/horizon"
BIN_DIR="${GAME_DIR}/bin"
LIBS_DIR="${GAME_DIR}/libs"
LOG_FILE="${GAME_DIR}/launch.log"

cd "${GAME_DIR}"
exec > "${LOG_FILE}" 2>&1

# Auto-detect native framebuffer geometry for RG34XX SP (720x480) vs RG35XX SP (640x480)
if [ -z "${DISPLAY_WIDTH:-}" ] || [ -z "${DISPLAY_HEIGHT:-}" ]; then
    if [ -r /sys/class/graphics/fb0/modes ]; then
        FB_MODE=$(head -n 1 /sys/class/graphics/fb0/modes | sed -E 's/^[^:]*:([0-9]+)x([0-9]+).*/\1 \2/')
        read -r FB_W FB_H <<< "$FB_MODE"
    fi
    if [ -z "${FB_W:-}" ] && [ -r /sys/class/graphics/fb0/virtual_size ]; then
        IFS=',' read -r FB_W FB_H < /sys/class/graphics/fb0/virtual_size
        if [ "${FB_W:-0}" -eq 720 ] && [ "${FB_H:-0}" -ge 480 ]; then
            FB_H=480
        elif [ "${FB_W:-0}" -eq 640 ] && [ "${FB_H:-0}" -ge 480 ]; then
            FB_H=480
        fi
    fi
    DISPLAY_WIDTH="${DISPLAY_WIDTH:-${FB_W:-720}}"
    DISPLAY_HEIGHT="${DISPLAY_HEIGHT:-${FB_H:-480}}"
fi

# Lock Allwinner H700 Cortex-A53 cores to performance governor during gameplay
for gov in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    [ -w "$gov" ] && echo performance > "$gov" 2>/dev/null || true
done

echo "========================================================"
echo "Starting Horizon 26.3 on muOS"
echo "Launch time    : $(date)"
echo "Game directory : ${GAME_DIR}"
echo "CFW Name       : ${CFW_NAME:-muOS}"
echo "Device         : ${DEVICE_NAME:-RG34XX-SP}"
echo "Display        : ${DISPLAY_WIDTH:-720}x${DISPLAY_HEIGHT:-480}"
echo "========================================================"

chmod +x "${BIN_DIR}/horizon-client" 2>/dev/null || true
chmod +x "${BIN_DIR}/horizon-ctl" 2>/dev/null || true
chmod +x "${BIN_DIR}/horizon-prepare" 2>/dev/null || true
chmod +x "${GAME_DIR}/Install_Assets.sh" 2>/dev/null || true

# Pre-flight: verify that Minecraft 26.3 assets were installed
if [ ! -f "${GAME_DIR}/resources/asset-index.json" ] || [ ! -d "${GAME_DIR}/resources/assets" ]; then
    echo "========================================================"
    echo "ERROR: Minecraft 26.3 assets not found in ${GAME_DIR}/resources!"
    echo "Please plug your SD card into a PC and run:"
    echo "  'Install_Assets.bat' (Windows) or 'Install_Assets.sh' (Mac/Linux)"
    echo "inside ROMS/PORTS/horizon/ to download official game assets."
    echo "========================================================"
    exit 1
fi

# Clean up any legacy SD card swap file (SD card swap causes extreme kswapd I/O thrashing and system lockup)
swapoff /mnt/sdcard/horizon.swap 2>/dev/null || true
rm -f /mnt/sdcard/horizon.swap 2>/dev/null || true

# Memory Protection: Keep swappiness at 0 and free up caches before engine boot
sync
echo 0 > /proc/sys/vm/swappiness 2>/dev/null || true
echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || true
echo 32768 > /proc/sys/vm/min_free_kbytes 2>/dev/null || true

export MIMALLOC_PURGE_DELAY=0
export MIMALLOC_ARENA_RESERVE=0
export WESTON_DRM_PAGEFLIP_TIMEOUT=1
export WESTON_REPAINT_WINDOW=16

# Export library paths for both direct and wrapped launch
export LD_LIBRARY_PATH="${LIBS_DIR}:${LD_LIBRARY_PATH:-/usr/lib:/lib}"
export WRAPPED_LIBRARY_PATH="${LIBS_DIR}"

# Explicitly define system EGL/SDL paths for Crusty wrapper
export CRUSTY_LIBEGL="/usr/lib64/libEGL.so"
export CRUSTY_LIBSDL="/usr/lib64/libSDL2.so"

# CRUSTY_SHOW_CURSOR must be 0: Crusty cursor conflicts with Weston compositor
export CRUSTY_SHOW_CURSOR=0

# Unset SDL_GAMECONTROLLERCONFIG so gilrs uses native evdev driver codes via muos_gilrs_button
unset SDL_GAMECONTROLLERCONFIG

# Graphics & Performance environment for Mali-G31 MP2
export WGPU_BACKEND=gl
export WGPU_POWER_PREF=low
export HORIZON_GPU_CULL=all
export HORIZON_NO_DISCORD=1
export HORIZON_NO_AUDIO=1

# Only set Panfrost driver overrides if on a mainline kernel distribution (e.g. ROCKNIX)
if [ "${CFW_NAME}" = "ROCKNIX" ]; then
    export MESA_LOADER_DRIVER_OVERRIDE=panfrost
    export PAN_MESA_DEBUG=gl3
fi

WIDTH="${DISPLAY_WIDTH:-720}"
HEIGHT="${DISPLAY_HEIGHT:-480}"

# 2. Locate and mount Weston compositor runtime
weston_runtime="weston_pkg_0.2"
weston_dir=/tmp/weston
weston_squashfs=""

if [ -f "${LIBS_DIR}/${weston_runtime}.squashfs" ]; then
    weston_squashfs="${LIBS_DIR}/${weston_runtime}.squashfs"
elif [ -f "$controlfolder/libs/${weston_runtime}.squashfs" ]; then
    weston_squashfs="$controlfolder/libs/${weston_runtime}.squashfs"
fi

if [ -n "${weston_squashfs}" ]; then
    echo "Found Weston runtime at: ${weston_squashfs}"
    $ESUDO mkdir -p "${weston_dir}"
    if [[ "$PM_CAN_MOUNT" != "N" ]]; then
        $ESUDO umount "${weston_dir}" 2>/dev/null || true
    fi
    $ESUDO mount -o loop "${weston_squashfs}" "${weston_dir}"
fi

# 3. Launch with Weston Compositor or Direct Fallback
if [ -f "${weston_dir}/westonwrap.sh" ]; then
    echo "Launching Horizon inside Weston Compositor..."
    chmod +x "${weston_dir}/westonwrap.sh"

    mkdir -p /tmp/weston_runtime
    chmod 700 /tmp/weston_runtime 2>/dev/null || true

    # PortMaster gptokeyb is bypassed by default to let Horizon use native evdev/gilrs controls
    if [ "${HORIZON_USE_GPTOKEYB:-0}" = "1" ] && [ -f "${GAME_DIR}/horizon.gptk" ]; then
        if [ -n "$GPTOKEYB" ]; then
            echo "Starting gptokeyb with horizon.gptk..."
            $GPTOKEYB "horizon-client" -c "${GAME_DIR}/horizon.gptk" &
        elif [ -n "$GPTOKEYB2" ]; then
            echo "Starting gptokeyb2 with horizon.gptk..."
            $GPTOKEYB2 "horizon-client" -c "${GAME_DIR}/horizon.gptk" &
        fi
    else
        echo "Using native controller subsystem (Focus menu navigation & Bedrock controls)"
    fi

    GFX_MODE="${HORIZON_GFX_MODE:-crusty_x11egl}"
    WESTON_BACKEND="${HORIZON_WESTON_BACKEND:-headless}"
    WESTON_RENDERER="${HORIZON_WESTON_RENDERER:-noop}"

    if [ "${HORIZON_FORCE_WAYLAND}" = "1" ]; then
        echo "Launching in native Wayland mode..."
        WL_OVERRIDE=""
    else
        echo "Launching in Xwayland mode (hardware accelerated X11 EGL via ${WESTON_BACKEND}/${WESTON_RENDERER}/${GFX_MODE})..."
        WL_OVERRIDE="WAYLAND_DISPLAY="
    fi

    export HORIZON_SUBMIT_THREAD="${HORIZON_SUBMIT_THREAD:-0}"
    export CRUSTY_SHOW_CURSOR=0

    $ESUDO env WRAPPED_LIBRARY_PATH="${LIBS_DIR}" CRUSTY_SHOW_CURSOR=0 HORIZON_SUBMIT_THREAD="${HORIZON_SUBMIT_THREAD}" \
        "${weston_dir}/westonwrap.sh" "${WESTON_BACKEND}" "${WESTON_RENDERER}" kiosk "${GFX_MODE}" \
        env ${WL_OVERRIDE} XDG_SESSION_TYPE="x11" HORIZON_SUBMIT_THREAD="${HORIZON_SUBMIT_THREAD}" \
        "${BIN_DIR}/horizon-client" \
            --game-dir "${GAME_DIR}" \
            --home "${GAME_DIR}" \
            --resources "${GAME_DIR}/resources" \
            --width "${WIDTH}" \
            --height "${HEIGHT}" \
            --fullscreen

    STATUS=$?
    echo "Horizon exited with code: ${STATUS}"

    echo "Cleaning up Weston and controls..."
    $ESUDO kill -9 $(pidof gptokeyb) 2>/dev/null || true
    $ESUDO kill -9 $(pidof gptokeyb2) 2>/dev/null || true
    $ESUDO "${weston_dir}/westonwrap.sh" cleanup
    if [[ "$PM_CAN_MOUNT" != "N" ]]; then
        $ESUDO umount "${weston_dir}" 2>/dev/null || true
    fi
else
    echo "Weston not found, attempting direct launch..."
    if [ "${HORIZON_USE_GPTOKEYB:-0}" = "1" ] && [ -n "$GPTOKEYB" ] && [ -f "${GAME_DIR}/horizon.gptk" ]; then
        $GPTOKEYB "horizon-client" -c "${GAME_DIR}/horizon.gptk" &
    fi
    if [ -z "${DISPLAY:-}" ] && [ -e /tmp/.X11-unix/X0 ]; then
        export DISPLAY=:0
    fi
    "${BIN_DIR}/horizon-client" \
        --game-dir "${GAME_DIR}" \
        --home "${GAME_DIR}" \
        --resources "${GAME_DIR}/resources" \
        --width "${WIDTH}" \
        --height "${HEIGHT}" \
        --fullscreen
    STATUS=$?
    echo "Horizon exited with code: ${STATUS}"
    $ESUDO kill -9 $(pidof gptokeyb) 2>/dev/null || true
    $ESUDO kill -9 $(pidof gptokeyb2) 2>/dev/null || true
fi

[ -n "$(type -t pm_finish)" ] && pm_finish
printf "\033c" > /dev/tty0 2>/dev/null || true
exit ${STATUS}
