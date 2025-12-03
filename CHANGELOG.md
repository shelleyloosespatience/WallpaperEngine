# 🎯 CHANGELOG - DWM Crash Fix v2.2.1

## MAJOR bug Fixed: Desktop Crashes due to single composite process

**Status:** **FIXED**
This took me 4 days of research and trying diffrent player until i realized dwm.exe tries to 
literally composite our whole app with itself! so we now use two seperate binaries
since there was no rust projects for reference, this took an painful amount of debugging
i had to sit for hours even after my whole day of studies to fix this major bug
### The Problem 
When dragging files/icons on desktop while video wallpaper was running:
- Windows 10: Intermittent crashes
- Windows 11 24H2: Severe crashes + desktop corruption
- Explorer.exe would hang waiting for wallpaper window to respond
- DWM (Desktop Window Manager) would restart desktop to recover

**Root cause:** Cross-process synchronization deadlock between Explorer.exe and wallpaper window in Tauri's message loop.

---

## The Solution: Separate Process Architecture

### What Changed

**Before (Single Process - BROKEN):**
```
wallpaperengine.exe
  ├─ Tauri + React UI
  └─ WMF Player (in-process)
       └─ DWM tries to composite with UI → CRASH!
```

**After (Dual Process - FINALLY FIXED):**
```
wallpaperengine.exe         wallpaper-player.exe
  └─ Tauri UI only            ├─ WMF player
                              ├─ Desktop injection  
       │                      └─ Fast Win32 message loop
       └─ spawns ──────────►      (DWM-isolated!)
```

---

## Technical Changes

### New Files Created
- [src/player/main.rs](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/src/player/main.rs) - Standalone player binary entry point
- [src/player/wmf_player.rs](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/src/player/wmf_player.rs) - WMF player (copied for player binary)
- [src/player/desktop_injection.rs](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/src/player/desktop_injection.rs) - Desktop injection (copied for player binary)
- `src/player/os_version.rs` - OS detection (copied for player binary)
- [src/process_manager.rs](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/src/process_manager.rs) - Process spawning/lifecycle management

### Modified Files
- [Cargo.toml](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/Cargo.toml) - Added dual binary configuration + `default-run`
- [tauri.conf.json](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/tauri.conf.json) - Removed `externalBin` (dev mode compatibility)
- [src/main.rs](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/src/main.rs) - Removed WMF/desktop_injection, added process_manager
- [src/video_wallpaper.rs](file:///c:/Users/MY-PC/Documents/WallpaperEngine/src-tauri/src/video_wallpaper.rs) - Replaced in-process creation with process spawning

---
**Testing:** Confirmed working on Windows 10 Build 19045  

---

## Version Info

**Version:** 2.2.1  
**Date:** December 3, 2025  
**Fix Type:** Architecture refactor (breaking internal change)  
**User Impact:** No breaking changes for end users  
**Installer:** Still one `.msi` file  

---

## Conclusion

**THE DWM CRASH IS FIXED!**

After days of testing window styles, player libraries (WMF → GStreamer → MPV), and desktop injection techniques, the solution was **architectural:** separate the wallpaper player into its own process to eliminate DWM composition conflicts with Tauri's WebView.
Back to studying now bbye kys