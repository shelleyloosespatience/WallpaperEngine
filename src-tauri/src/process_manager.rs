// Process Manager for wallpaper-player sidecar
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

lazy_static::lazy_static! {
    static ref PLAYER_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

/// Get the path to the wallpaper-player executable
/// Dev mode: Looks in target/debug/
/// Production: Looks in same directory as main exe (bundled by Tauri build)
fn get_player_binary_path(_app: &AppHandle) -> Result<std::path::PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        // DEV MODE: Player binary is in target/debug/ alongside main binary
        println!("[process_manager] Dev mode - looking for player in target/debug/");

        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {}", e))?;

        let exe_dir = exe_path.parent().ok_or("Failed to get exe directory")?;

        let player_path = exe_dir.join("wallpaper-player.exe");

        println!("[process_manager] Looking for player at: {:?}", player_path);

        if !player_path.exists() {
            return Err(format!(
                "Player binary not found at {:?}. Run 'cargo build --bin wallpaper-player' first!",
                player_path
            ));
        }

        Ok(player_path)
    }

    #[cfg(not(debug_assertions))]
    {
        // PRODUCTION MODE: Player is bundled in same directory as main exe
        println!("[process_manager] Production mode - looking for bundled player");

        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {}", e))?;

        let exe_dir = exe_path.parent().ok_or("Failed to get exe directory")?;

        let player_path = exe_dir.join("wallpaper-player.exe");

        if !player_path.exists() {
            return Err(format!(
                "Player binary not found. Installation may be corrupted. Expected at: {:?}",
                player_path
            ));
        }

        Ok(player_path)
    }
}

/// Spawn the wallpaper player process
pub fn spawn_player(
    app: &AppHandle,
    video_path: &str,
    width: i32,
    height: i32,
) -> Result<(), String> {
    // Stop any existing player first
    stop_player()?;

    let player_path = get_player_binary_path(app)?;

    println!("[process_manager] Spawning player: {:?}", player_path);
    println!(
        "[process_manager] Args: {} {}x{}",
        video_path, width, height
    );

    let child = Command::new(&player_path)
        .args(&[video_path, &width.to_string(), &height.to_string()])
        .spawn()
        .map_err(|e| {
            format!(
                "Failed to spawn player process: {}. Path: {:?}",
                e, player_path
            )
        })?;

    println!("[process_manager] Player spawned with PID: {}", child.id());

    *PLAYER_PROCESS.lock().unwrap() = Some(child);

    Ok(())
}

/// Stop the wallpaper player process
pub fn stop_player() -> Result<(), String> {
    let mut player_lock = PLAYER_PROCESS.lock().unwrap();

    if let Some(mut child) = player_lock.take() {
        println!(
            "[process_manager] Stopping player process (PID: {})",
            child.id()
        );

        // Kill the process
        let _ = child.kill();

        // Wait briefly for cleanup
        std::thread::sleep(std::time::Duration::from_millis(200));

        println!("[process_manager] Player stopped");
    }

    Ok(())
}

// Check if player is running for testing

// pub fn is_player_running() -> bool {
//     let mut player_lock = PLAYER_PROCESS.lock().unwrap();

//     if let Some(child) = player_lock.as_mut() {
//         // Check if process is still alive
//         match child.try_wait() {
//             Ok(Some(_status)) => {
//                 // Process exited
//                 *player_lock = None;
//                 false
//             }
//             Ok(None) => {
//                 // Process still running
//                 true
//             }
//             Err(_) => {
//                 // Error checking, assume dead
//                 *player_lock = None;
//                 false
//             }
//         }
//     } else {
//         false
//     }
// }
