/// Wallpaper management commands (static images, video wallpapers, user wallpapers)
use crate::models::*;
use crate::storage::*;
use crate::video_wallpaper::*;
use tauri::AppHandle;

/// Download image from URL to cache
async fn download_image(url: &str) -> Result<std::path::PathBuf, String> {
    let client = reqwest::Client::builder()
        .user_agent("LaxentaInc/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let cache_dir = get_cache_dir()?;
    let extension = url
        .split('.')
        .last()
        .and_then(|ext| ext.split('?').next())
        .unwrap_or("jpg");

    let file_name = format!(
        "wallpaper_{}.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        extension
    );
    let file_path = cache_dir.join(file_name);

    std::fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    Ok(file_path)
}

#[tauri::command]
pub async fn set_wallpaper(image_url: String) -> Result<WallpaperResponse, String> {
    let file_path = match download_image(&image_url).await {
        Ok(path) => path,
        Err(e) => {
            return Ok(WallpaperResponse {
                success: false,
                message: None,
                error: Some(format!("failed to download image: {}", e)),
            });
        }
    };

    match wallpaper::set_from_path(&file_path.to_string_lossy()) {
        Ok(_) => Ok(WallpaperResponse {
            success: true,
            message: Some("Wallpaper set successfully".to_string()),
            error: None,
        }),
        Err(e) => Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some(format!("failed to set wallpaper: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn get_current_wallpaper() -> Result<WallpaperResponse, String> {
    match wallpaper::get() {
        Ok(path) => Ok(WallpaperResponse {
            success: true,
            message: Some(path),
            error: None,
        }),
        Err(e) => Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some(format!("failed to get wallpaper: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn get_cache_size() -> Result<CacheSizeResponse, String> {
    let cache_dir = match get_cache_dir() {
        Ok(dir) => dir,
        Err(_) => {
            return Ok(CacheSizeResponse {
                success: true,
                size_mb: "0".to_string(),
                file_count: 0,
            });
        }
    };

    let mut total_size: u64 = 0;
    let mut file_count = 0;

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
    }

    let size_mb = format!("{:.2}", total_size as f64 / 1_048_576.0);

    Ok(CacheSizeResponse {
        success: true,
        size_mb,
        file_count,
    })
}

#[tauri::command]
pub async fn clear_cache() -> Result<ClearCacheResponse, String> {
    let cache_dir = match get_cache_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("NOT OK failed to get cache directory: {}", e));
        }
    };

    let mut files_deleted = 0;

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if std::fs::remove_file(entry.path()).is_ok() {
                        files_deleted += 1;
                    }
                }
            }
        }
    }

    Ok(ClearCacheResponse {
        success: true,
        files_deleted,
    })
}

#[tauri::command]
pub async fn set_video_wallpaper(
    app: AppHandle,
    video_url: String,
) -> Result<WallpaperResponse, String> {
    println!("[main] setting video wallpaper: {}", video_url);

    let video_path = match download_video(&video_url).await {
        Ok(path) => path,
        Err(e) => {
            return Ok(WallpaperResponse {
                success: false,
                message: None,
                error: Some(format!("failed to download video: {}", e)),
            });
        }
    };

    // Save original URL for restoration capability
    match create_video_wallpaper_window(&app, &video_path.to_string_lossy(), Some(video_url.clone())) {
        Ok(_) => Ok(WallpaperResponse {
            success: true,
            message: Some("video wallpaper set successfully".to_string()),
            error: None,
        }),
        Err(e) => Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some(format!("failed to set video wallpaper: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn set_video_wallpaper_from_file(
    app: AppHandle,
    file_path: String,
) -> Result<WallpaperResponse, String> {
    println!("[main] setting video wallpaper from local file: {}", file_path);

    if !std::path::Path::new(&file_path).exists() {
        return Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some(format!("File not found: {}", file_path)),
        });
    }

    // For local files, use file:// URL format and no original_url (it's already local)
    match create_video_wallpaper_window(&app, &file_path, None) {
        Ok(_) => Ok(WallpaperResponse {
            success: true,
            message: Some("video wallpaper set successfully".to_string()),
            error: None,
        }),
        Err(e) => Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some(format!("failed to set video wallpaper: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn stop_video_wallpaper_command(app: AppHandle) -> Result<WallpaperResponse, String> {
    match stop_video_wallpaper(&app) {
        Ok(_) => Ok(WallpaperResponse {
            success: true,
            message: Some("video wallpaper stopped".to_string()),
            error: None,
        }),
        Err(e) => Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some(e),
        }),
    }
}

#[tauri::command]
pub fn get_video_wallpaper_status() -> VideoWallpaperState {
    get_video_wallpaper_state()
}

#[tauri::command]
pub async fn list_user_wallpapers() -> Result<UserWallpapersResponse, String> {
    let wallpapers_dir = get_user_wallpapers_dir()?;
    let mut wallpapers = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&wallpapers_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

                    if matches!(extension, "mp4" | "mkv" | "jpg" | "jpeg" | "png" | "gif") {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        let media_type = if matches!(extension, "mp4" | "mkv") {
                            "video"
                        } else {
                            "image"
                        };

                        let added_at = metadata
                            .created()
                            .or_else(|_| metadata.modified())
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        wallpapers.push(UserWallpaper {
                            id: format!("{:x}", md5::compute(&name)),
                            name,
                            path: path.to_string_lossy().to_string(),
                            media_type: media_type.to_string(),
                            thumbnail: None,
                            added_at,
                        });
                    }
                }
            }
        }
    }

    wallpapers.sort_by(|a, b| b.added_at.cmp(&a.added_at));

    Ok(UserWallpapersResponse {
        success: true,
        wallpapers,
    })
}

#[tauri::command]
pub async fn upload_user_wallpaper(source_path: String) -> Result<WallpaperResponse, String> {
    let source = std::path::Path::new(&source_path);

    if !source.exists() {
        return Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some("Source file does not exist".to_string()),
        });
    }

    let dest_dir = get_user_wallpapers_dir()?;
    let file_name = source
        .file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();

    let dest_path = dest_dir.join(&file_name);

    std::fs::copy(source, &dest_path).map_err(|e| format!("failed to copy file: {}", e))?;

    Ok(WallpaperResponse {
        success: true,
        message: Some(dest_path.to_string_lossy().to_string()),
        error: None,
    })
}

#[tauri::command]
pub async fn delete_user_wallpaper(wallpaper_path: String) -> Result<WallpaperResponse, String> {
    let path = std::path::Path::new(&wallpaper_path);

    if !path.exists() {
        return Ok(WallpaperResponse {
            success: false,
            message: None,
            error: Some("File does not exist".to_string()),
        });
    }

    std::fs::remove_file(path).map_err(|e| format!("failed to delete file: {}", e))?;

    Ok(WallpaperResponse {
        success: true,
        message: Some("File deleted successfully".to_string()),
        error: None,
    })
}

#[tauri::command]
pub async fn get_wallpaper_storage_path() -> Result<PathResponse, String> {
    match get_user_wallpapers_dir() {
        Ok(path) => Ok(PathResponse {
            success: true,
            path: Some(path.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(PathResponse {
            success: false,
            path: None,
            error: Some(e),
        }),
    }
}

