/// settings management commands (will get worked on later when app is more stable)
use crate::models::*;
use crate::storage::get_settings_file;

#[tauri::command]
pub async fn get_settings() -> Result<SettingsResponse, String> {
    let settings_file = get_settings_file()?;

    if !settings_file.exists() {
        let default_settings = AppSettings {
            audio_enabled: false,
            live_wallpaper_enabled: true,
        };
        return Ok(SettingsResponse {
            success: true,
            settings: Some(default_settings),
            error: None,
        });
    }

    match std::fs::read_to_string(&settings_file) {
        Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
            Ok(settings) => Ok(SettingsResponse {
                success: true,
                settings: Some(settings),
                error: None,
            }),
            Err(e) => Ok(SettingsResponse {
                success: false,
                settings: None,
                error: Some(format!("failed to parse settings: {}", e)),
            }),
        },
        Err(e) => Ok(SettingsResponse {
            success: false,
            settings: None,
            error: Some(format!("failed to read settings: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<SettingsResponse, String> {
    let settings_file = get_settings_file()?;
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {}", e))?;

    std::fs::write(&settings_file, json).map_err(|e| format!("failed to write settings: {}", e))?;

    Ok(SettingsResponse {
        success: true,
        settings: Some(settings),
        error: None,
    })
}

