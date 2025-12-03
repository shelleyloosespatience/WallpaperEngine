use crate::models::VideoWallpaperState;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

#[cfg(target_os = "linux")]
use wallpaperengine::video_wallpaper_linux;

lazy_static::lazy_static! {
    static ref VIDEO_WALLPAPER_STATE: Arc<Mutex<VideoWallpaperState>> = Arc::new(Mutex::new(VideoWallpaperState {
        is_active: false,
        video_path: None,
        video_url: None,
    }));
}

fn get_wallpaper_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("live_wallpapers");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create wallpaper directory: {}", e))?;
    Ok(dir)
}

fn get_state_file() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("live_wallpapers");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create wallpaper directory: {}", e))?;
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

pub fn create_video_wallpaper_window(_app: &AppHandle, video_path: &str) -> Result<(), String> {
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

        let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
        state.is_active = true;
        state.video_path = Some(video_path.to_string());
        state.video_url = Some(format!("file://{}", video_path));
        let _ = save_wallpaper_state(&state);
        drop(state);

        println!("[video_wallpaper] Wallpaper created successfully");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        video_wallpaper_linux::create_linux_video_wallpaper(video_path)?;

        let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
        state.is_active = true;
        state.video_path = Some(video_path.to_string());
        state.video_url = Some(format!("file://{}", video_path));
        let _ = save_wallpaper_state(&state);
        drop(state);

        return Ok(());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err("Video wallpapers not supported on this platform temproarily".into());
    }
}

#[cfg(target_os = "windows")]
fn create_windows_wmf_wallpaper(app: &AppHandle, video_path: &str) -> Result<(), String> {
    use crate::process_manager;

    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("Failed to resolve video path: {}", e))?;
    let video_path_str = video_path_abs.display().to_string();

    println!("[video_wallpaper] Setting up video wallpaper via separate process");

    // Get screen dimensions for wallpaper player
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

    // Spawn the player process (DWM-isolated!)
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
    state.video_path = None;
    state.video_url = None;
    let _ = save_wallpaper_state(&state);
    drop(state);

    Ok(())
}

pub fn get_video_wallpaper_state() -> VideoWallpaperState {
    VIDEO_WALLPAPER_STATE.lock().unwrap().clone()
}

pub fn restore_wallpaper_on_startup(app: &AppHandle) -> Result<(), String> {
    println!("[startup] Attempting to restore wallpaper");

    if let Some(saved_state) = load_wallpaper_state() {
        if saved_state.is_active {
            if let Some(ref video_path) = saved_state.video_path {
                if std::path::Path::new(video_path).exists() {
                    println!("[startup] Restoring wallpaper: {}", video_path);
                    std::thread::sleep(std::time::Duration::from_millis(800));

                    match create_video_wallpaper_window(app, video_path) {
                        Ok(_) => {
                            println!("[startup] Wallpaper restored");
                            Ok(())
                        }
                        Err(e) => {
                            println!("[startup] Failed to restore: {}", e);
                            let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                            state.is_active = false;
                            state.video_path = None;
                            state.video_url = None;
                            let _ = save_wallpaper_state(&state);
                            Err(e)
                        }
                    }
                } else {
                    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                    state.is_active = false;
                    state.video_path = None;
                    state.video_url = None;
                    let _ = save_wallpaper_state(&state);
                    Ok(())
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}
