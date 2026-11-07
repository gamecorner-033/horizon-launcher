@echo off
setlocal enabledelayedexpansion
title Horizon muOS - Minecraft Asset Downloader & Installer
color 0A

echo ================================================================================
echo           HORIZON 26.3 - MINECRAFT ASSET INSTALLER FOR muOS
echo ================================================================================
echo.
echo This 1-time installer downloads official Minecraft 26.3 assets directly from
echo Mojang / Microsoft official servers onto this SD card.
echo.
echo No proprietary or copyrighted assets are distributed with this port.
echo ================================================================================
echo.

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

set "RES_DIR=%SCRIPT_DIR%resources"
if not exist "%RES_DIR%" mkdir "%RES_DIR%"

echo [1/3] Downloading official Minecraft 26.3 client jar from Mojang CDN (41 MB)...
curl.exe -f -L -# "https://piston-data.mojang.com/v1/objects/e877b6a07acd633fb3bb475002175cec036e7b87/client.jar" -o "%SCRIPT_DIR%client_temp.jar"
if errorlevel 1 (
    echo.
    echo [ERROR] Failed to download client.jar! Please check your internet connection.
    pause
    exit /b 1
)

echo [2/3] Extracting textures, models, fonts, and data into resources folder...
tar.exe -xf "%SCRIPT_DIR%client_temp.jar" -C "%RES_DIR%" assets data version.json pack.mcmeta pack.png 2>nul
del /f /q "%SCRIPT_DIR%client_temp.jar"

echo [3/3] Downloading official asset index manifest...
curl.exe -f -s -L "https://piston-meta.mojang.com/v1/packages/abfaa525f923f807df8b4e4d29c1b5e3a104adbe/34.json" -o "%RES_DIR%\asset-index.json"

if exist "%SCRIPT_DIR%resourcepacks\8x8.zip" (
    echo [OPTIONAL] Applying 8x8 performance texture pack for RG34XX SP / RG35XX SP...
    tar.exe -xf "%SCRIPT_DIR%resourcepacks\8x8.zip" -C "%RES_DIR%" assets 2>nul
    if exist "%RES_DIR%\horizon-sprites.cache" del /f /q "%RES_DIR%\horizon-sprites.cache"
)

echo.
echo ================================================================================
echo SUCCESS! Minecraft 26.3 assets have been installed to your SD card.
echo.
echo You may now safely eject this SD card, insert it into your RG34XX / RG35XX,
echo and launch Horizon from the muOS Ports menu. Enjoy!
echo ================================================================================
echo.
pause
