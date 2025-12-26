/// store utilities for file paths and directories - for wallpapers, settings, etc
use std::path::PathBuf;

/// persistent app data directory (AppData on Windows, ~/.config on Linux)
pub fn get_app_data_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        use dirs::data_dir;
        let dir = data_dir()
            .ok_or_else(|| "Failed to get AppData directory".to_string())?
            .join("ColorWall");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        Ok(dir)
    }
    
    #[cfg(target_os = "linux")]
    {
        use dirs::config_dir;
        let dir = config_dir()
            .ok_or_else(|| "Failed to get config directory".to_string())?
            .join("ColorWall");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        Ok(dir)
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        // Fallback to temp for other platforms
        let dir = std::env::temp_dir().join("ColorWall");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        Ok(dir)
    }
}

/// cache directory for downloaded images (temp, can be cleared)
pub fn get_cache_dir() -> Result<PathBuf, String> {
    let cache_dir = std::env::temp_dir().join("wallpaper_cache");
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    Ok(cache_dir)
}

/// user wallpapers directory (temp, for user-uploaded files)
pub fn get_user_wallpapers_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("user_wallpapers");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// settings file path (persistent, in AppData)
pub fn get_settings_file() -> Result<PathBuf, String> {
    let dir = get_app_data_dir()?;
    Ok(dir.join("settings.json"))
}

