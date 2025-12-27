use crate::models::VideoWallpaperState;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
// DETECTS PLATFORMS AND USES APPROPRIATE WALLPAPER MODULE
#[cfg(target_os = "linux")]
use wallpaperengine::video_wallpaper_linux;

lazy_static::lazy_static! {
    static ref VIDEO_WALLPAPER_STATE: Arc<Mutex<VideoWallpaperState>> = Arc::new(Mutex::new(VideoWallpaperState {
        is_active: false,
        video_path: None,
        video_url: None,
        original_url: None,
        set_at: None,
    }));
}

// storage module for wallpaper state file location
use crate::storage::get_app_data_dir;

/// wallpaper cache directory (temp for downloaded videos, can be cleared)
fn get_wallpaper_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("live_wallpapers");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create wallpaper directory: {}", e))?;
    Ok(dir)
}

/// persistent state file location (in AppData, survives cache clears)
fn get_state_file() -> Result<PathBuf, String> {
    let dir = get_app_data_dir()?;
    Ok(dir.join("wallpaper_state.json"))
}

fn save_wallpaper_state(state: &VideoWallpaperState) -> Result<(), String> {
    let state_file = get_state_file()?;
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("failed to serialize state: {}", e))?;

    fs::write(&state_file, json).map_err(|e| format!("failed to write state file: {}", e))?;
    Ok(())
}

fn load_wallpaper_state() -> Option<VideoWallpaperState> {
    let state_file = match get_state_file() {
        Ok(f) => f,
        Err(_) => return None,
    };

    if !state_file.exists() {
        return None;
    }

    match fs::read_to_string(&state_file) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(state) => Some(state),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub async fn download_video(url: &str) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .user_agent("WallpaperApp/1.0")
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("failed to download video: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    println!("downloading {} bytes...", total_size);

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read video data: {}", e))?;

    if bytes.is_empty() {
        return Err("Downloaded file is empty".to_string());
    }

    let wallpaper_dir = get_wallpaper_dir()?;
    let extension = if url.contains(".mkv") { "mkv" } else { "mp4" };
    let file_name = format!(
        "wallpaper_{}.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        extension
    );
    let file_path = wallpaper_dir.join(file_name);

    std::fs::write(&file_path, bytes).map_err(|e| format!("failed to save video file: {}", e))?;

    println!("downloaded to: {:?}", file_path);
    Ok(file_path)
}

/// create video wallpaper window (internal, doesn't save original_url)
fn create_video_wallpaper_window_internal(_app: &AppHandle, video_path: &str) -> Result<(), String> {
    if !std::path::Path::new(video_path).exists() {
        return Err(format!("Video file not found: {}", video_path));
    }

    let ext = std::path::Path::new(video_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if !matches!(ext, "mp4" | "mkv") {
        return Err(format!("Unsupported format: {}. Use MP4 or MKV", ext));
    }

    println!("[video_wallpaper] Setting video wallpaper: {}", video_path);

    #[cfg(target_os = "windows")]
    {
        create_windows_wmf_wallpaper(_app, video_path)?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        video_wallpaper_linux::create_linux_video_wallpaper(video_path)?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err("Video wallpapers not supported on this platform temproarily".into());
    }
}

/// create video wallpaper and save state with original URL
pub fn create_video_wallpaper_window(_app: &AppHandle, video_path: &str, original_url: Option<String>) -> Result<(), String> {
    create_video_wallpaper_window_internal(_app, video_path)?;

    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
    state.is_active = true;
    state.video_path = Some(video_path.to_string());
    state.video_url = Some(format!("file://{}", video_path));
    state.original_url = original_url;
    state.set_at = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    );
    let _ = save_wallpaper_state(&state);
    drop(state);

    println!("[video_wallpaper] Wallpaper created and state saved successfully");
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_windows_wmf_wallpaper(app: &AppHandle, video_path: &str) -> Result<(), String> {
    use crate::process_manager;

    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("Failed to resolve video path: {}", e))?;
    let video_path_str = video_path_abs.display().to_string();

    println!("[video_wallpaper] Setting up video wallpaper via separate process");

    // screen dimensions for wallpaper player
    let (width, height) = unsafe {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::RECT;
        use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowRect};

        let progman = FindWindowW(
            PCWSTR(windows::core::w!("Progman").as_ptr()),
            PCWSTR(windows::core::w!("Program Manager").as_ptr()),
        )
        .map_err(|e| format!("FindWindowW failed: {}", e))?;

        let mut progman_rect = RECT::default();
        GetWindowRect(progman, &mut progman_rect)
            .map_err(|e| format!("GetWindowRect failed: {}", e))?;

        let w = progman_rect.right - progman_rect.left;
        let h = progman_rect.bottom - progman_rect.top;
        println!("[video_wallpaper] Screen dimensions: {}x{}", w, h);
        (w, h)
    };

    // Spawn the player process (DWM-isolated yewwe)
    process_manager::spawn_player(app, &video_path_str, width, height)?;

    println!("[video_wallpaper] Wallpaper player spawned successfully");
    Ok(())
}

#[cfg(target_os = "windows")]
fn stop_windows_wmf_wallpaper() -> Result<(), String> {
    use crate::process_manager;

    println!("[video_wallpaper] Stopping wallpaper player process");
    process_manager::stop_player()?;
    Ok(())
}

pub fn stop_video_wallpaper(_app: &AppHandle) -> Result<(), String> {
    println!("[video_wallpaper] Stopping video wallpaper");

    #[cfg(target_os = "windows")]
    {
        stop_windows_wmf_wallpaper()?;
    }

    #[cfg(target_os = "linux")]
    {
        video_wallpaper_linux::stop_linux_video_wallpaper()?;
    }

    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
    state.is_active = false;
    // keep original_url and set_at for restoration tracking, only clear current paths
    state.video_path = None;
    state.video_url = None;
    let _ = save_wallpaper_state(&state);
    drop(state);

    Ok(())
}

pub fn get_video_wallpaper_state() -> VideoWallpaperState {
    VIDEO_WALLPAPER_STATE.lock().unwrap().clone()
}

/// periodically save state to prevent data loss (call this from a background task)
pub fn periodic_state_save() {
    let state = VIDEO_WALLPAPER_STATE.lock().unwrap();
    if state.is_active {
        let _ = save_wallpaper_state(&state);
    }
}

pub fn restore_wallpaper_on_startup(app: &AppHandle) -> Result<(), String> {
    println!("[startup] Attempting to restore wallpaper");

    let saved_state = match load_wallpaper_state() {
        Some(state) => state,
        None => {
            println!("[startup] No saved wallpaper state found");
            return Ok(());
        }
    };

    if !saved_state.is_active {
        println!("[startup] Saved state indicates wallpaper is not active");
        return Ok(());
    }

    // try to restore from saved video path first
    if let Some(ref video_path) = saved_state.video_path {
        if std::path::Path::new(video_path).exists() {
            println!("[startup] Found video file at saved path: {}", video_path);
            std::thread::sleep(std::time::Duration::from_millis(800));

            match create_video_wallpaper_window_internal(app, video_path) {
                Ok(_) => {
                    // restore full state including original_url
                    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                    *state = saved_state.clone();
                    state.is_active = true;
                    let _ = save_wallpaper_state(&state);
                    drop(state);
                    println!("[startup] Wallpaper restored from saved path");
                    return Ok(());
                }
                Err(e) => {
                    println!("[startup] Failed to restore from saved path: {}", e);
                }
            }
        } else {
            println!("[startup] Video file not found at saved path: {}", video_path);
        }
    }

    // if saved path doesn't work, try to re-download from original URL
    if let Some(ref original_url) = saved_state.original_url {
        println!("[startup] Attempting to re-download from original URL: {}", original_url);
        
        // tokio runtime for async download
        let app_clone = app.clone();
        let url_clone = original_url.clone();
        
        // spawn async task for re-download
        tauri::async_runtime::spawn(async move {
            match download_video(&url_clone).await {
                Ok(new_video_path) => {
                    println!("[startup] Re-downloaded video to: {:?}", new_video_path);
                    std::thread::sleep(std::time::Duration::from_millis(800));
                    
                    match create_video_wallpaper_window_internal(&app_clone, &new_video_path.to_string_lossy()) {
                        Ok(_) => {
                            // update state with new path
                            let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                            state.is_active = true;
                            state.video_path = Some(new_video_path.to_string_lossy().to_string());
                            state.video_url = Some(format!("file://{}", new_video_path.to_string_lossy()));
                            // keep original_url and set_at
                            let _ = save_wallpaper_state(&state);
                            drop(state);
                            println!("[startup] Wallpaper restored from re-download");
                        }
                        Err(e) => {
                            eprintln!("[startup] Failed to set re-downloaded wallpaper: {}", e);
                            let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                            state.is_active = false;
                            state.video_path = None;
                            state.video_url = None;
                            let _ = save_wallpaper_state(&state);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[startup] Failed to re-download video: {}", e);
                    // clear state if re-download fails
                    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                    state.is_active = false;
                    state.video_path = None;
                    state.video_url = None;
                    let _ = save_wallpaper_state(&state);
                }
            }
        });
        
        // Return OK immediately, restoration happens in background
        return Ok(());
    }

    // if we get here, no valid path or URL to restore from, so clear state
    println!("[startup] No valid video path or original URL to restore from");
    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
    state.is_active = false;
    state.video_path = None;
    state.video_url = None;
    let _ = save_wallpaper_state(&state);
    Ok(())
}
