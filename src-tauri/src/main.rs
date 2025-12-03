// prevents console window on windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod scraper;
mod video_wallpaper;

// Process manager for wallpaper-player sidecar
mod process_manager;

use models::*;
use scraper::*;
use tauri::{Manager, WindowEvent};
use video_wallpaper::*;

use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::path::PathBuf;

fn get_cache_dir() -> Result<PathBuf, String> {
    let cache_dir = std::env::temp_dir().join("wallpaper_cache");
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    Ok(cache_dir)
}

fn get_user_wallpapers_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("user_wallpapers");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn get_settings_file() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("wallpaper_app");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

async fn download_image(url: &str) -> Result<PathBuf, String> {
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

// TAURI COMMANDS
// debug derive
#[tauri::command]
async fn search_wallpapers(
    query: String,
    sources: Option<Vec<String>>,
    limit_per_source: Option<usize>,
    randomize: Option<bool>,
    page: Option<u32>,
    purity: Option<String>,
    ai_art: Option<bool>,
) -> Result<SearchResponse, String> {
    let sources = sources.unwrap_or_else(|| {
        vec![
            "wallhaven".to_string(),
            "moewalls".to_string(),
            "wallpapers".to_string(),
            "wallpaperflare".to_string(),
            "motionbgs".to_string(),
        ]
    });
    let limit = limit_per_source.unwrap_or(10);
    let should_randomize = randomize.unwrap_or(true);
    let page_num = page.unwrap_or(1);
    let purity_val = purity.unwrap_or_else(|| "100".to_string());
    let ai_art_enabled = ai_art.unwrap_or(false);

    println!(
        "[BACKEND:SEARCH] Starting search - query: '{}', page: {}, limit: {}, sources: {}",
        query,
        page_num,
        limit,
        sources.join(",")
    );

    let mut all_items = Vec::new();
    let mut errors = Vec::new();

    for source in sources {
        println!("[BACKEND:SEARCH] Scraping source: {}", source);

        let result = match source.as_str() {
            "wallhaven" => {
                println!("[BACKEND:SCRAPE] wallhaven - page: {}", page_num);
                scrape_wallhaven(&query, page_num, ai_art_enabled, &purity_val, limit).await
            }
            "moewalls" => {
                println!("[BACKEND:SCRAPE] moewalls - page: {}", page_num);
                scrape_moewalls(Some(&query), limit, false, page_num).await
            }
            "wallpapers" => {
                println!("[BACKEND:SCRAPE] wallpapers.com - page: {}", page_num);
                scrape_wallpapers_com(&query, limit, page_num).await
            }
            "wallpaperflare" => {
                println!("[BACKEND:SCRAPE] wallpaperflare - page: {}", page_num);
                scrape_wallpaperflare(&query, limit, page_num).await
            }
            "motionbgs" => {
                println!("[BACKEND:SCRAPE] motionbgs - page: {}", page_num);
                scrape_motionbgs(&query, limit, page_num).await
            }
            _ => continue,
        };

        match result {
            Ok(items) => {
                let count = items.len();
                println!("[BACKEND:SCRAPE] {}: Got {} items", source, count);
                all_items.extend(items);
            }
            Err(e) => {
                println!("[BACKEND:SCRAPE] {}: ERROR - {}", source, e);
                errors.push(format!("{}: {}", source, e));
            }
        }
    }

    println!(
        "[BACKEND:SEARCH] Total items before dedup: {}",
        all_items.len()
    );

    let mut seen = HashSet::new();
    all_items.retain(|item| seen.insert(item.id.clone()));

    println!(
        "[BACKEND:SEARCH] Total items after dedup: {}",
        all_items.len()
    );

    if should_randomize {
        let mut rng = rand::thread_rng();
        all_items.shuffle(&mut rng);
        println!("[BACKEND:SEARCH] Shuffled results");
    }

    println!(
        "[BACKEND:SEARCH] Returning {} items with {} errors",
        all_items.len(),
        errors.len()
    );

    Ok(SearchResponse {
        success: !all_items.is_empty(),
        items: all_items,
        errors: if errors.is_empty() {
            None
        } else {
            Some(errors)
        },
    })
}

#[tauri::command]
async fn fetch_live2d(query: Option<String>) -> Result<SearchResponse, String> {
    match scrape_moewalls(query.as_deref(), 50, true, 1).await {
        Ok(items) => Ok(SearchResponse {
            success: true,
            items,
            errors: None,
        }),
        Err(e) => Ok(SearchResponse {
            success: false,
            items: Vec::new(),
            errors: Some(vec![e]),
        }),
    }
}

#[tauri::command]
async fn set_wallpaper(image_url: String) -> Result<WallpaperResponse, String> {
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
async fn get_current_wallpaper() -> Result<WallpaperResponse, String> {
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
async fn get_cache_size() -> Result<CacheSizeResponse, String> {
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
async fn clear_cache() -> Result<ClearCacheResponse, String> {
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
async fn resolve_wallpaperflare_highres(
    detail_url: String,
) -> Result<ResolveHighResResponse, String> {
    println!("info: resolving high-res for: {}", detail_url);
    // x84 or x64 doesn't really matter
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x84) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    match resolve_wallpaperflare_download(&detail_url, &client).await {
        Ok((high_res_url, _, _)) => {
            println!("OK resolved to: {}", high_res_url);
            Ok(ResolveHighResResponse {
                success: true,
                url: Some(high_res_url),
                url4k: None,
                error: None,
            })
        }
        Err(e) => {
            println!("error: failed to resolve: {}", e);
            Ok(ResolveHighResResponse {
                success: false,
                url: None,
                url4k: None,
                error: Some(e),
            })
        }
    }
}

#[tauri::command]
async fn resolve_motionbgs_video(detail_url: String) -> Result<ResolveHighResResponse, String> {
    println!("info: RESOLVING motionBg video: {}", detail_url);

    match scrape_motionbgs_detail(&detail_url).await {
        Ok((video_url, video_url_4k)) => {
            println!("ok: found video url: {}", video_url);
            Ok(ResolveHighResResponse {
                success: true,
                url: Some(video_url),
                url4k: video_url_4k,
                error: None,
            })
        }
        Err(e) => {
            println!("error: failed to resolve: {}", e);
            Ok(ResolveHighResResponse {
                success: false,
                url: None,
                url4k: None,
                error: Some(e),
            })
        }
    }
}

#[tauri::command]
async fn set_video_wallpaper(
    app: tauri::AppHandle,
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

    match create_video_wallpaper_window(&app, &video_path.to_string_lossy()) {
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
async fn stop_video_wallpaper_command(app: tauri::AppHandle) -> Result<WallpaperResponse, String> {
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
fn get_video_wallpaper_status() -> VideoWallpaperState {
    get_video_wallpaper_state()
}

// usr wallpaper Management cmds
#[tauri::command]
async fn list_user_wallpapers() -> Result<UserWallpapersResponse, String> {
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
async fn upload_user_wallpaper(source_path: String) -> Result<WallpaperResponse, String> {
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
async fn delete_user_wallpaper(wallpaper_path: String) -> Result<WallpaperResponse, String> {
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

// Management cmds
#[tauri::command]
async fn get_settings() -> Result<SettingsResponse, String> {
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
async fn save_settings(settings: AppSettings) -> Result<SettingsResponse, String> {
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

#[tauri::command]
async fn get_wallpaper_storage_path() -> Result<PathResponse, String> {
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            search_wallpapers,
            fetch_live2d,
            set_wallpaper,
            get_current_wallpaper,
            get_cache_size,
            clear_cache,
            resolve_wallpaperflare_highres,
            resolve_motionbgs_video,
            set_video_wallpaper,
            stop_video_wallpaper_command,
            get_video_wallpaper_status,
            list_user_wallpapers,
            upload_user_wallpaper,
            delete_user_wallpaper,
            get_settings,
            save_settings,
            get_wallpaper_storage_path,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // this will continue running in background :)
            let _app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { .. } = event {
                    println!(
                        "[main] Close button clicked - UI will close, wallpaper continues in background"
                    );
                }
            });

            // systray
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, TrayIconBuilder};
            let show_item =
                MenuItem::with_id(app, "show", "Show Window", true, None::<&str>).unwrap();
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show_item, &quit_item]).unwrap();

            let app_handle_for_tray = app.handle().clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        // Check if window exists, if not recreate it
                        if app.get_webview_window("main").is_none() {
                            println!("[main] Window doesn't exist, recreating...");

                            use tauri::{WebviewUrl, WebviewWindowBuilder};
                            let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                                .title("ColorWall - Wallpaper Engine")
                                .inner_size(1000.0, 900.0)
                                .resizable(true)
                                .decorations(false)
                                .build();
                        }

                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            println!("[main] Window shown from tray");
                        }
                    }
                    "quit" => {
                        println!("[main] Quit requested from tray");

                        let _ = stop_video_wallpaper(&app_handle_for_tray);

                        std::thread::sleep(std::time::Duration::from_millis(500));

                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        // if not recreate it (also uh we disable the decorations explicitly cz it gets double title bars)
                        if tray.app_handle().get_webview_window("main").is_none() {
                            println!("[main] Saddddly Window doesn't exist, recreating from tray click...");

                            use tauri::{WebviewUrl, WebviewWindowBuilder};
                            let _ = WebviewWindowBuilder::new(
                                tray.app_handle(),
                                "main",
                                WebviewUrl::default(),
                            )
                            .title("ColorWall - Wallpaper Engine")
                            .inner_size(1000.0, 900.0)
                            .resizable(true)
                            .decorations(false)
                            .build();
                        }

                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            println!("[main] Window shown from tray icon click");
                        }
                    }
                })
                .build(app)
                .unwrap();

            // Restore wallpaper on startup in background task
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                println!("[startup] attempting wallpaper restoration...");
                match restore_wallpaper_on_startup(&app_handle) {
                    Ok(_) => println!("[startup] restoration completed"),
                    Err(e) => eprintln!("[startup] error: failed to restore wallpaper: {}", e),
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
