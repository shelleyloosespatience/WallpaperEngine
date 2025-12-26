# Codebase Structure & Architecture

This document explains the structure of the ColorWall wallpaper engine codebase, how modules relate to each other, and the overall architecture.

## 📁 Directory Structure

```
src-tauri/src/
├── main.rs                    # Application entry point & Tauri setup
├── lib.rs                     # Library exports (if needed)
├── models.rs                  # Data structures & API response types
├── storage.rs                 # File path utilities (AppData, cache, etc.)
├── scraper.rs                 # Web scraping logic for wallpaper sources
├── video_wallpaper.rs         # Video wallpaper state management & restoration
├── process_manager.rs         # Manages wallpaper-player subprocess
│
├── commands/                  # Tauri command handlers (organized by feature)
│   ├── mod.rs                 # Module exports
│   ├── search.rs              # Search/scraping commands
│   ├── wallpaper.rs           # Wallpaper management commands
│   └── settings.rs            # Settings commands
│
├── player/                    # Separate wallpaper player process
│   ├── main.rs                # Player process entry point
│   ├── wmf_player.rs          # Windows Media Foundation video player
│   ├── desktop_injection.rs   # Desktop window injection (Windows 10/11)
│   └── os_version.rs          # Windows version detection
│
└── linux/                     # Linux-specific implementations
    ├── mpv_player.rs
    └── video_wallpaper_linux.rs
```

## 🏗️ Architecture Overview

### High-Level Flow

```
User Action (Frontend)
    ↓
Tauri Command (commands/*.rs)
    ↓
Business Logic (video_wallpaper.rs, scraper.rs)
    ↓
System Integration (process_manager.rs, player/*.rs)
    ↓
Windows Desktop (desktop_injection.rs)
```

## 📦 Module Breakdown

### 1. **main.rs** - Application Entry Point
**Purpose:** Tauri app initialization and lifecycle management

**Responsibilities:**
- Initialize Tauri plugins
- Register all command handlers
- Set up system tray
- Start background tasks (restoration, periodic saves)
- Handle single-instance behavior

**Key Components:**
- `main()` - Entry point
- `.setup()` - App initialization
- System tray setup
- Background task spawning

**Dependencies:**
- `commands::*` - All command handlers
- `video_wallpaper::*` - Restoration logic

---

### 2. **models.rs** - Data Structures
**Purpose:** Define all data types used across the application

**Key Types:**
- `VideoWallpaperState` - Current wallpaper state (path, URL, active status)
- `WallpaperItem` - Search result item
- `SearchResponse` - Search API response
- `WallpaperResponse` - Wallpaper operation response
- `AppSettings` - User settings
- `UserWallpaper` - User-uploaded wallpaper metadata

**Used By:**
- All command handlers
- State management
- API responses

---

### 3. **storage.rs** - File Path Management
**Purpose:** Centralized file path utilities

**Functions:**
- `get_app_data_dir()` - Persistent AppData location (survives cache clears)
  - Windows: `%AppData%\ColorWall`
  - Linux: `~/.config/ColorWall`
- `get_cache_dir()` - Temporary cache directory
- `get_user_wallpapers_dir()` - User-uploaded wallpapers
- `get_settings_file()` - Settings file path

**Why Separate:**
- Single source of truth for paths
- Easy to change storage locations
- Platform-specific handling in one place

**Used By:**
- `video_wallpaper.rs` - State file location
- `commands/settings.rs` - Settings file
- `commands/wallpaper.rs` - User wallpapers directory

---

### 4. **commands/** - Command Handlers
**Purpose:** Tauri command handlers organized by feature

#### **commands/search.rs** - Search & Scraping
**Commands:**
- `search_wallpapers()` - Multi-source wallpaper search
- `fetch_live2d()` - Live2D wallpapers
- `resolve_wallpaperflare_highres()` - Get high-res URLs
- `resolve_motionbgs_video()` - Get video URLs

**Flow:**
```
User searches → search_wallpapers()
    ↓
Calls scraper functions (scraper.rs)
    ↓
Aggregates results from multiple sources
    ↓
Returns SearchResponse
```

**Dependencies:**
- `scraper::*` - Web scraping functions
- `models::*` - Response types

---

#### **commands/wallpaper.rs** - Wallpaper Management
**Commands:**
- `set_wallpaper()` - Set static image wallpaper
- `get_current_wallpaper()` - Get current wallpaper path
- `set_video_wallpaper()` - Set video wallpaper from URL
- `set_video_wallpaper_from_file()` - Set video wallpaper from local file
- `stop_video_wallpaper_command()` - Stop video wallpaper
- `get_video_wallpaper_status()` - Get wallpaper state
- `get_cache_size()` - Get cache size
- `clear_cache()` - Clear cache
- `list_user_wallpapers()` - List user wallpapers
- `upload_user_wallpaper()` - Upload wallpaper
- `delete_user_wallpaper()` - Delete wallpaper
- `get_wallpaper_storage_path()` - Get storage path

**Flow (Video Wallpaper):**
```
set_video_wallpaper()
    ↓
download_video() (video_wallpaper.rs)
    ↓
create_video_wallpaper_window() (video_wallpaper.rs)
    ↓
create_windows_wmf_wallpaper() (video_wallpaper.rs)
    ↓
spawn_player() (process_manager.rs)
    ↓
wallpaper-player.exe process starts
    ↓
Desktop injection (player/desktop_injection.rs)
```

**Dependencies:**
- `video_wallpaper::*` - Video wallpaper logic
- `storage::*` - File paths
- `process_manager::*` - Process management

---

#### **commands/settings.rs** - Settings Management
**Commands:**
- `get_settings()` - Load settings
- `save_settings()` - Save settings

**Flow:**
```
get_settings()
    ↓
Read from storage::get_settings_file()
    ↓
Parse JSON → AppSettings
    ↓
Return SettingsResponse
```

**Dependencies:**
- `storage::*` - Settings file path
- `models::*` - Settings types

---

### 5. **video_wallpaper.rs** - Video Wallpaper Core Logic
**Purpose:** Video wallpaper state management and restoration

**Key Functions:**
- `create_video_wallpaper_window()` - Create and inject wallpaper
- `download_video()` - Download video from URL
- `restore_wallpaper_on_startup()` - Restore wallpaper on app start
- `stop_video_wallpaper()` - Stop wallpaper
- `periodic_state_save()` - Periodic state saving
- `get_video_wallpaper_state()` - Get current state

**State Management:**
- Uses `lazy_static` for global state
- Saves to persistent storage (AppData, not temp)
- Includes `original_url` for re-download capability

**Restoration Flow:**
```
App starts
    ↓
restore_wallpaper_on_startup() (called from main.rs)
    ↓
Load state from AppData/wallpaper_state.json
    ↓
Check if video file exists
    ├─ Yes → Restore directly
    └─ No → Re-download from original_url
```

**Dependencies:**
- `storage::*` - State file location
- `process_manager::*` - Spawn player process
- `models::*` - State types

---

### 6. **process_manager.rs** - Process Management
**Purpose:** Manage the separate wallpaper-player process

**Why Separate Process:**
- DWM (Desktop Window Manager) isolation
- Prevents composition issues
- Better performance

**Functions:**
- `spawn_player()` - Spawn wallpaper-player.exe
- `stop_player()` - Kill player process

**Flow:**
```
spawn_player(video_path, width, height)
    ↓
Launch wallpaper-player.exe with args
    ↓
Player process handles video playback
    ↓
Desktop injection happens in player process
```

**Dependencies:**
- None (standalone process manager)

---

### 7. **player/** - Wallpaper Player Process
**Purpose:** Separate process for video playback and desktop injection

#### **player/main.rs** - Player Entry Point
**Flow:**
```
wallpaper-player.exe starts
    ↓
Parse args (video_path, width, height)
    ↓
Create WMF player (wmf_player.rs)
    ↓
Load video
    ↓
Inject behind desktop (desktop_injection.rs)
    ↓
Start playback
    ↓
Run message loop
```

#### **player/wmf_player.rs** - Video Player
**Purpose:** Windows Media Foundation video playback

**Key Components:**
- `WmfPlayer` - Main player struct
- `create_player_window()` - Create window
- `create_optimized_media_engine()` - Set up Media Foundation

**Dependencies:**
- `os_version::*` - Windows version detection

#### **player/desktop_injection.rs** - Desktop Injection
**Purpose:** Inject video window behind desktop icons

**Windows 10 Flow:**
```
Find Progman window
    ↓
Send 0x052C message (spawn WorkerW)
    ↓
Find WorkerW window
    ↓
Parent our window to WorkerW
    ↓
Position at (0, 0) with full screen size
```

**Windows 11 Flow:**
```
Find Progman window
    ↓
Send 0x052C message (raise desktop)
    ↓
Find ShellDLL_DefView (desktop icons)
    ↓
Find WorkerW (wallpaper)
    ↓
Parent our window to Progman
    ↓
Z-order: DefView (top) → Our Window → WorkerW (bottom)
    ↓
Position at (0, 0) with Progman size
```

**Key Functions:**
- `inject_behind_desktop()` - Main injection function
- `inject_windows_11()` - Windows 11 injection
- `inject_legacy_workerw()` - Windows 10 injection
- `start_watchdog()` - Monitor and fix z-order

**Dependencies:**
- `os_version::*` - Windows version detection

#### **player/os_version.rs** - OS Detection
**Purpose:** Detect Windows version for proper injection method

**Detection:**
- Build 22000+ = Windows 11 (all versions)
- Build < 22000 = Windows 10

**Used By:**
- `desktop_injection.rs` - Choose injection method
- `wmf_player.rs` - Window style selection

---

### 8. **scraper.rs** - Web Scraping
**Purpose:** Scrape wallpapers from various sources

**Sources:**
- Wallhaven
- Moewalls
- Wallpapers.com
- Wallpaperflare
- MotionBGs

**Functions:**
- `scrape_wallhaven()` - Wallhaven scraper
- `scrape_moewalls()` - Moewalls scraper
- `scrape_wallpapers_com()` - Wallpapers.com scraper
- `scrape_wallpaperflare()` - Wallpaperflare scraper
- `scrape_motionbgs()` - MotionBGs scraper
- `resolve_wallpaperflare_download()` - Get download URL
- `scrape_motionbgs_detail()` - Get video URL

**Used By:**
- `commands/search.rs` - Search commands

---

## 🔄 Data Flow Examples

### Example 1: Setting Video Wallpaper

```
Frontend: User clicks "Set Video Wallpaper"
    ↓
Tauri: set_video_wallpaper(video_url)
    ↓
commands/wallpaper.rs: set_video_wallpaper()
    ↓
video_wallpaper.rs: download_video(url)
    ↓
Download to temp/live_wallpapers/
    ↓
video_wallpaper.rs: create_video_wallpaper_window(path, original_url)
    ↓
video_wallpaper.rs: create_windows_wmf_wallpaper()
    ↓
process_manager.rs: spawn_player(video_path, width, height)
    ↓
Launch wallpaper-player.exe
    ↓
player/main.rs: Create WMF player
    ↓
player/desktop_injection.rs: inject_behind_desktop()
    ↓
Desktop injection (Windows 10/11 specific)
    ↓
Video plays behind desktop icons
    ↓
Save state to AppData/wallpaper_state.json
```

### Example 2: App Startup & Restoration

```
App launches
    ↓
main.rs: setup() runs
    ↓
Spawn background task: restore_wallpaper_on_startup()
    ↓
video_wallpaper.rs: restore_wallpaper_on_startup()
    ↓
Load state from AppData/wallpaper_state.json
    ↓
Check if video file exists
    ├─ Exists → Restore directly
    └─ Missing → Re-download from original_url
    ↓
Create wallpaper window
    ↓
Spawn player process
    ↓
Desktop injection
    ↓
Wallpaper restored
```

### Example 3: Search Wallpapers

```
Frontend: User searches "anime"
    ↓
Tauri: search_wallpapers("anime")
    ↓
commands/search.rs: search_wallpapers()
    ↓
For each source:
    scraper.rs: scrape_*()
    ↓
Parse HTML/JSON
    ↓
Extract wallpaper data
    ↓
Aggregate results
    ↓
Deduplicate by ID
    ↓
Randomize (optional)
    ↓
Return SearchResponse
```

## 🔗 Key Relationships

### State Management
- **Global State:** `video_wallpaper.rs` uses `lazy_static` for `VIDEO_WALLPAPER_STATE`
- **Persistence:** State saved to `AppData/ColorWall/wallpaper_state.json`
- **Restoration:** Automatic on app startup

### Process Architecture
- **Main Process:** Tauri app (UI, commands, state management)
- **Player Process:** `wallpaper-player.exe` (video playback, desktop injection)
- **Communication:** File-based (video path passed as argument)

### Windows Version Handling
- **Detection:** `player/os_version.rs` detects Windows version
- **Injection:** `player/desktop_injection.rs` uses appropriate method
- **Window Style:** `player/wmf_player.rs` sets correct window style

### Storage Strategy
- **Persistent:** AppData (state, settings) - survives cache clears
- **Temporary:** Temp directory (downloaded videos, cache) - can be cleared
- **User Files:** Temp/user_wallpapers (user uploads)

## 🎯 Design Principles

1. **Separation of Concerns:** Each module has a single responsibility
2. **Modularity:** Commands organized by feature in `commands/`
3. **Persistence:** Critical data in AppData, not temp
4. **Process Isolation:** Player in separate process for DWM isolation
5. **Platform Awareness:** Windows version detection for proper injection
6. **State Recovery:** Automatic restoration with re-download fallback

## 📝 Notes

- **Single Instance:** Enforced via `tauri-plugin-single-instance` - relaunch focuses existing window
- **Background Tasks:** Restoration and periodic saves run in background
- **Error Handling:** Graceful degradation (missing files → re-download)
- **Performance:** Separate player process prevents UI blocking

---

## Quick Reference

| Module | Purpose | Key Functions |
|--------|---------|---------------|
| `main.rs` | App setup | `main()`, `.setup()` |
| `commands/search.rs` | Search | `search_wallpapers()`, `fetch_live2d()` |
| `commands/wallpaper.rs` | Wallpapers | `set_video_wallpaper()`, `set_wallpaper()` |
| `commands/settings.rs` | Settings | `get_settings()`, `save_settings()` |
| `video_wallpaper.rs` | Video logic | `create_video_wallpaper_window()`, `restore_wallpaper_on_startup()` |
| `process_manager.rs` | Process mgmt | `spawn_player()`, `stop_player()` |
| `player/desktop_injection.rs` | Desktop injection | `inject_behind_desktop()`, `inject_windows_11()` |
| `player/wmf_player.rs` | Video playback | `WmfPlayer::new()`, `WmfPlayer::play()` |
| `storage.rs` | File paths | `get_app_data_dir()`, `get_cache_dir()` |
| `scraper.rs` | Web scraping | `scrape_wallhaven()`, `scrape_moewalls()` |

