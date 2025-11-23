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
    pub url4k: Option<String>, // 4K download URL for MotionBGs
    pub error: Option<String>,
}

// ✅ Video wallpaper state
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VideoWallpaperState {
    pub is_active: bool,
    pub video_path: Option<String>,
    pub video_url: Option<String>,
}