use crate::models::VideoWallpaperState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use std::fs;

#[cfg(target_os = "windows")]
use crate::{desktop_injection, wmf_player};

#[cfg(target_os = "linux")]
use crate::video_wallpaper_linux;

#[cfg(target_os = "windows")]
use std::{thread, time::Duration};

lazy_static::lazy_static! {
    static ref VIDEO_WALLPAPER_STATE: Arc<Mutex<VideoWallpaperState>> = Arc::new(Mutex::new(VideoWallpaperState {
        is_active: false,
        video_path: None,
        video_url: None,
    }));
}

#[cfg(target_os = "windows")]
lazy_static::lazy_static! {
    static ref WMF_PLAYER: Mutex<Option<wmf_player::WmfPlayer>> = Mutex::new(None);
}

fn get_wallpaper_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("live_wallpapers");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create wallpaper directory: {}", e))?;
    Ok(dir)
}

fn get_state_file() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("live_wallpapers");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create wallpaper directory: {}", e))?;
    Ok(dir.join("wallpaper_state.json"))
}

fn save_wallpaper_state(state: &VideoWallpaperState) -> Result<(), String> {
    let state_file = get_state_file()?;
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("failed to serialize state: {}", e))?;
    fs::write(&state_file, json)
        .map_err(|e| format!("failed to write state file: {}", e))?;
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
        Ok(content) => {
            serde_json::from_str(&content).ok()
        }
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
        return Err("Wtf? Downloaded file is empty".to_string());
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

    std::fs::write(&file_path, bytes)
        .map_err(|e| format!("failed to save video file: {}", e))?;

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
        return Err(format!("Sadly... this is the Unsupported format: {}. The sites i scrape Use MP4 or MKV so this might be exceptional case", ext));
    }

    println!("Creating WMF video wallpaper for: {}", video_path);

    #[cfg(target_os = "windows")]
    {
        create_windows_wmf_wallpaper(video_path)?;
    }

    #[cfg(target_os = "linux")]
    {
        video_wallpaper_linux::create_linux_video_wallpaper(video_path)?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err("Video wallpapers not supported on this platform".into());
    }

    let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
    state.is_active = true;
    state.video_path = Some(video_path.to_string());
    state.video_url = Some(format!("file://{}", video_path));
    
    // Save state to disk for persistence
    let _ = save_wallpaper_state(&state);

    Ok(())
}

#[cfg(target_os = "windows")]
fn create_windows_wmf_wallpaper(video_path: &str) -> Result<(), String> {
    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("Failed to resolve video path: {}", e))?;
    let video_path_str = video_path_abs.display().to_string();

    // CRITICAL FIX: Check if player already exists and just replace the video
    let player_guard = WMF_PLAYER.lock().unwrap();
    
    if let Some(existing_player) = player_guard.as_ref() {
        println!("INFO- replcing existing video wallpaper with new video");
        println!("Stopping current playback...");
        
        // Stop current video
        if let Err(e) = existing_player.stop() {
            println!("[WARN] Failed to stop current video: {}", e);
        }
        
        thread::sleep(Duration::from_millis(200));
        
        println!("Loading new video: {}", video_path_str);
        existing_player.load_video(&video_path_str)?;
        
        thread::sleep(Duration::from_millis(500));
        
        println!("Starting playback of new video...");
        existing_player.play()?;
        
        println!("Video preloaded and set!!!");
        return Ok(());
    }
    
    drop(player_guard); // no lock before creating new player
    
    println!("wallpaper window: {}", video_path_str);
    
    let monitors = get_all_monitor_dimensions();
    let (desktop_x, desktop_y, desktop_width, desktop_height) = calculate_total_desktop_bounds(&monitors);

    println!("[INFO] Desktop bounds: {}x{} at ({}, {})", desktop_width, desktop_height, desktop_x, desktop_y);

    let player = wmf_player::WmfPlayer::new(desktop_width, desktop_height)
        .map_err(|e| format!("Failed to create WMF player: {}", e))?;

    println!("[INFO] Loading video: {}", video_path_str);
    player.load_video(&video_path_str)?;
    
    // Wait for video to load
    thread::sleep(Duration::from_millis(500));

    let hwnd = player.hwnd();
    println!("player window created: {:?}", hwnd);
    
    // Reduce thread priority to prevent GPU overload
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::{SetThreadPriority, GetCurrentThread, THREAD_PRIORITY_BELOW_NORMAL};
        unsafe {
            let current_thread = GetCurrentThread();
            let _ = SetThreadPriority(current_thread, THREAD_PRIORITY_BELOW_NORMAL);
        }
    }
    
    println!("[INFO] Injecting behind desktop...");
    desktop_injection::inject_behind_desktop(
        hwnd,
        desktop_x,
        desktop_y,
        desktop_width,
        desktop_height,
    )?;

    // Delay to ensure injection is complete
    thread::sleep(Duration::from_millis(300));

    println!("[INFO] Starting playback...");
    player.play()?;

    *WMF_PLAYER.lock().unwrap() = Some(player);

    println!("win created");
    Ok(())
}

#[cfg(target_os = "windows")]
fn stop_windows_wmf_wallpaper() -> Result<(), String> {
    println!("we are stopping the Windows WMF wallpaper");
    
    // Stop watchdog first
    desktop_injection::stop_watchdog();
    thread::sleep(Duration::from_millis(200));
    
    // Properly stop and destroy the player before dropping
    let mut player_guard = WMF_PLAYER.lock().unwrap();
    if let Some(player) = player_guard.take() {
        let hwnd = player.hwnd();
        
        println!("[INFO] Stopping playback...");
        let _ = player.stop();
        thread::sleep(Duration::from_millis(300));
        
        // unparent and hide window before destroying
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                SetParent, ShowWindow, SW_HIDE, IsWindow, DestroyWindow
            };
            
            if !hwnd.0.is_null() && IsWindow(Some(hwnd)).as_bool() {
                println!("Cleaning up window: {:?}", hwnd);     
                // Unparent the window from WorkerW (set parent to None)
                let _ = SetParent(hwnd, None);
                               // Hide the window immediately
                let _ = ShowWindow(hwnd, SW_HIDE);                
                    // Give time for unparenting to complete
                thread::sleep(Duration::from_millis(200));               
                // explicitly destroy the window
                let _ = DestroyWindow(hwnd);
                println!("[INFO] Window destroyed");
            }
        }
        
        // (drop impl will clean up media engine and resources)
        drop(player);
        println!("[INFO] Player dropped");
    } else {
        println!("[INFO] No active player to stop");
    }
    drop(player_guard);
    
    // extra time for all cleanup to complete (just in case)
    thread::sleep(Duration::from_millis(500));
    
    println!("WMF wallpaper cleanup complete!!");
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_all_monitor_dimensions() -> Vec<MonitorInfo> {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO};

    let mut monitors: Vec<MonitorInfo> = Vec::new();

    unsafe {
        extern "system" fn monitor_enum_proc(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            unsafe {
                let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

                let mut monitor_info = MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };

                if GetMonitorInfoW(hmonitor, &mut monitor_info) != BOOL(0) {
                    let rect = monitor_info.rcMonitor;
                    monitors.push(MonitorInfo {
                        x: rect.left,
                        y: rect.top,
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    });
                }
            }
            BOOL(1)
        }

        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }

    if monitors.is_empty() {
        monitors.push(MonitorInfo { x: 0, y: 0, width: 1920, height: 1080 });
    }

    monitors
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
struct MonitorInfo {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[cfg(target_os = "windows")]
fn calculate_total_desktop_bounds(monitors: &[MonitorInfo]) -> (i32, i32, i32, i32) {
    let min_x = monitors.iter().map(|m| m.x).min().unwrap_or(0);
    let min_y = monitors.iter().map(|m| m.y).min().unwrap_or(0);
    let max_x = monitors.iter().map(|m| m.x + m.width).max().unwrap_or(1920);
    let max_y = monitors.iter().map(|m| m.y + m.height).max().unwrap_or(1080);

    (min_x, min_y, max_x - min_x, max_y - min_y)
}

pub fn stop_video_wallpaper(_app: &AppHandle) -> Result<(), String> {
    println!("stopping video wallpaper...");

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

    println!("video wallpaper stopped!!");
    Ok(())
}

pub fn get_video_wallpaper_state() -> VideoWallpaperState {
    VIDEO_WALLPAPER_STATE.lock().unwrap().clone()
}

pub fn restore_wallpaper_on_startup(app: &AppHandle) -> Result<(), String> {
    if let Some(saved_state) = load_wallpaper_state() {
        if saved_state.is_active {
            if let Some(ref video_path) = saved_state.video_path {
                if std::path::Path::new(video_path).exists() {
                    println!("found saved wallpaper, restoring: {}", video_path);
                    
                    // Give Windows a moment to stabilize (app just launched)
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    match create_video_wallpaper_window(app, video_path) {
                        Ok(_) => {
                            println!("video wallpaper restored successfully!!");
                            
                            // we do verification for debugging purposes
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            let state = get_video_wallpaper_state();
                            if state.is_active {
                                println!("Wallpaper still active after 2 seconds lol");
                            } else {
                                println!("Wallpaper was deactivated! Something destroyed it.");
                            }
                            
                            Ok(())
                        }
                        Err(e) => {
                            println!("failed to restore wallpaper: {}", e);
                            let mut state = VIDEO_WALLPAPER_STATE.lock().unwrap();
                            state.is_active = false;
                            state.video_path = None;
                            state.video_url = None;
                            let _ = save_wallpaper_state(&state);
                            Ok(())
                        }
                    }
                } else {
                        println!("saved video file not found: {}", video_path);
                    // Clear invalid state
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
