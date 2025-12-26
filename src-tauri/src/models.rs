use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperItem {
    pub id: String,
    pub source: String,
    pub title: Option<String>,
    pub image_url: String,
    pub thumbnail_url: Option<String>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tags: Option<Vec<String>>,
    pub detail_url: Option<String>,
    pub original: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub success: bool,
    pub items: Vec<WallpaperItem>,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheSizeResponse {
    pub success: bool,
    pub size_mb: String,
    pub file_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResponse {
    pub success: bool,
    pub files_deleted: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveHighResResponse {
    pub success: bool,
    pub url: Option<String>,
    pub url4k: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoWallpaperState {
    pub is_active: bool,
    pub video_path: Option<String>,
    pub video_url: Option<String>,
    /// Original URL from which the video was downloaded (for re-download if file is missing)
    pub original_url: Option<String>,
    /// Timestamp when wallpaper was set (for restoration tracking)
    pub set_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserWallpaper {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub thumbnail: Option<String>,
    pub added_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWallpapersResponse {
    pub success: bool,
    pub wallpapers: Vec<UserWallpaper>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub audio_enabled: bool,
    pub live_wallpaper_enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsResponse {
    pub success: bool,
    pub settings: Option<AppSettings>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathResponse {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}
