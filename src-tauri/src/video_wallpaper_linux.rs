//auto generated filler file for now
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref MPV_PROCESS: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
}

pub fn create_linux_video_wallpaper(video_path: &str) -> Result<(), String> {
    // Stop any existing wallpaper first
    stop_linux_video_wallpaper()?;
    
    println!("[INFO] Creating Linux video wallpaper for: {}", video_path);
    
    // Check if mpv is available
    let mpv_check = Command::new("which")
        .arg("mpv")
        .output();
    
    if mpv_check.is_err() || !mpv_check.unwrap().status.success() {
        return Err("guess, mpv is not installed. Please install mpv: sudo apt install mpv (Ubuntu/Debian) or sudo pacman -S mpv (Arch)".to_string());
    }
    
    // Get screen dimensions
    let (width, height) = get_screen_dimensions()?;
    
    // Use mpv with xwinwrap to create video wallpaper
    // xwinwrap wraps a window behind the desktop
    let xwinwrap_check = Command::new("which")
        .arg("xwinwrap")
        .output();
    
    if xwinwrap_check.is_err() || !xwinwrap_check.unwrap().status.success() {
        // Fallback: try using mpv directly with --wid option
        // This requires finding the desktop window
        return create_mpv_wallpaper_direct(video_path, width, height);
    }
    
    // Use xwinwrap + mpv
    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("Failed to resolve video path: {}", e))?;
    
    let mut child = Command::new("xwinwrap")
        .arg("-g")
        .arg(format!("{}x{}+0+0", width, height))
        .arg("-ni")
        .arg("-nf")
        .arg("-ov")
        .arg("--")
        .arg("mpv")
        .arg("--wid")
        .arg("$WID")
        .arg("--loop")
        .arg("--no-audio")
        .arg("--no-osd")
        .arg("--no-input-default-bindings")
        .arg("--hwdec=auto")
        .arg(&video_path_abs)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start xwinwrap: {}", e))?;
    
    // Wait a bit to check if process started successfully
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    match child.try_wait() {
        Ok(Some(status)) => {
            if !status.success() {
                return Err("xwinwrap process exited early".to_string());
            }
        }
        Ok(None) => {
            // Process is still running, good
        }
        Err(e) => {
            return Err(format!("Failed to check xwinwrap process: {}", e));
        }
    }
    
    *MPV_PROCESS.lock().unwrap() = Some(child);
    
    println!("[SUCCESS] Linux video wallpaper created and playing");
    Ok(())
}

fn create_mpv_wallpaper_direct(video_path: &str, width: i32, height: i32) -> Result<(), String> {
    // Alternative method: use mpv with desktop window
    // This is a fallback if xwinwrap is not available
    println!("[INFO] Using direct mpv method (xwinwrap not found)");
    
    let video_path_abs = std::fs::canonicalize(video_path)
        .map_err(|e| format!("Failed to resolve video path: {}", e))?;
    
    // Try to find desktop window (this is desktop-specific)
    // For GNOME/KDE, we might need different approaches
    let desktop_window = find_desktop_window()?;
    
    let mut child = Command::new("mpv")
        .arg("--wid")
        .arg(format!("{}", desktop_window))
        .arg("--loop")
        .arg("--no-audio")
        .arg("--no-osd")
        .arg("--no-input-default-bindings")
        .arg("--hwdec=auto")
        .arg("--geometry")
        .arg(format!("{}x{}+0+0", width, height))
        .arg(&video_path_abs)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start mpv: {}", e))?;
    
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    match child.try_wait() {
        Ok(Some(status)) => {
            if !status.success() {
                return Err("mpv process exited early".to_string());
            }
        }
        Ok(None) => {
            // Process is still running, good
        }
        Err(e) => {
            return Err(format!("Failed to check mpv process: {}", e));
        }
    }
    
    *MPV_PROCESS.lock().unwrap() = Some(child);
    
    println!("[SUCCESS] Linux video wallpaper created (direct method)");
    Ok(())
}

fn find_desktop_window() -> Result<u64, String> {
    // Try to find desktop window using xdotool or xprop
    // This is a simplified version - may need adjustment for different DEs
    let output = Command::new("xdotool")
        .arg("search")
        .arg("--onlyvisible")
        .arg("--class")
        .arg("Desktop")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let window_id = String::from_utf8_lossy(&output.stdout)
                .trim()
                .lines()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| "Failed to parse desktop window ID".to_string())?;
            return Ok(window_id);
        }
    }
    
    // Fallback: try finding nautilus-desktop or similar
    let output = Command::new("xdotool")
        .arg("search")
        .arg("--onlyvisible")
        .arg("--name")
        .arg("Desktop")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let window_id = String::from_utf8_lossy(&output.stdout)
                .trim()
                .lines()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| "Failed to parse desktop window ID".to_string())?;
            return Ok(window_id);
        }
    }
    
    Err("Could not find desktop window. Please install xdotool: sudo apt install xdotool".to_string())
}

fn get_screen_dimensions() -> Result<(i32, i32), String> {
    // Try xrandr first
    let output = Command::new("xrandr")
        .arg("--current")
        .output()
        .map_err(|e| format!("Failed to run xrandr: {}", e))?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // Parse xrandr output to find primary screen resolution
        for line in output_str.lines() {
            if line.contains(" connected") && line.contains(" primary") {
                // Example: "eDP-1 connected primary 1920x1080+0+0"
                if let Some(res_part) = line.split_whitespace().find(|s| s.contains('x')) {
                    if let Some((w, h)) = res_part.split_once('x') {
                        if let (Ok(width), Ok(height)) = (w.parse::<i32>(), h.parse::<i32>()) {
                            return Ok((width, height));
                        }
                    }
                }
            }
        }
        
        // Fallback: find any connected display
        for line in output_str.lines() {
            if line.contains(" connected") {
                if let Some(res_part) = line.split_whitespace().find(|s| s.contains('x')) {
                    if let Some((w, h)) = res_part.split_once('x') {
                        if let (Ok(width), Ok(height)) = (w.parse::<i32>(), h.parse::<i32>()) {
                            return Ok((width, height));
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to default resolution
    Ok((1920, 1080))
}

pub fn stop_linux_video_wallpaper() -> Result<(), String> {
    println!("[INFO] Stopping Linux video wallpaper...");
    
    let mut process_guard = MPV_PROCESS.lock().unwrap();
    if let Some(mut child) = process_guard.take() {
        // Try graceful shutdown first
        let _ = child.kill();
        let _ = child.wait();
    }
    drop(process_guard);
    
    // Also try to kill any remaining mpv/xwinwrap processes
    let _ = Command::new("pkill")
        .arg("-f")
        .arg("xwinwrap.*mpv")
        .output();
    
    let _ = Command::new("pkill")
        .arg("-f")
        .arg("mpv.*--wid")
        .output();
    
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    println!("[SUCCESS] Linux video wallpaper stopped");
    Ok(())
}

