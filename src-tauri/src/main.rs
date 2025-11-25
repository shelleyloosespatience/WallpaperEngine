// prevents console window on windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod desktop_injection;
mod models;
mod scraper;
mod video_wallpaper;
#[cfg(target_os = "windows")]
mod wmf_player;

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

async fn download_image(url: &str) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .user_agent("WallpaperApp/1.0")
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
            "zerochan".to_string(),
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
            "zerochan" => {
                println!("[BACKEND:SCRAPE] zerochan - page: {}", page_num);
                scrape_zerochan(&query, limit, page_num).await
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
                error: Some(format!("Failed to download image: {}", e)),
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
            error: Some(format!("Failed to set wallpaper: {}", e)),
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
            error: Some(format!("Failed to get wallpaper: {}", e)),
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
            return Err(format!("Failed to get cache directory: {}", e));
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

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    match resolve_wallpaperflare_download(&detail_url, &client).await {
        Ok((high_res_url, _, _)) => {
            println!("ok: resolved to: {}", high_res_url);
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

// VIDEO WALLPAPER COMMANDS

#[tauri::command]
async fn resolve_motionbgs_video(detail_url: String) -> Result<ResolveHighResResponse, String> {
    println!("info: resolving MotionBGs video: {}", detail_url);

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
                error: Some(format!("Failed to download video: {}", e)),
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
            error: Some(format!("Failed to set video wallpaper: {}", e)),
        }),
    }
}

#[tauri::command]
async fn stop_video_wallpaper_command(app: tauri::AppHandle) -> Result<WallpaperResponse, String> {
    match stop_video_wallpaper(&app) {
        Ok(_) => Ok(WallpaperResponse {
            success: true,
            message: Some("Video wallpaper stopped".to_string()),
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Handle window close event to hide instead of quit
            let app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    println!(
                        "[main] Close button clicked - hiding window (wallpaper stays active)"
                    );

                    // Prevent default close behavior
                    api.prevent_close();

                    // Hide the window but DON'T stop wallpaper
                    if let Some(win) = app_handle.get_webview_window("main") {
                        let _ = win.hide();
                        println!("[main] Window hidden, wallpaper continues in background");
                    }
                }
            });

            // Setup system tray
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
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                println!("[main] Window shown from tray");
                            }
                        }
                        "quit" => {
                            println!("[main] Quit requested from tray");

                            // Stop video wallpaper before quitting
                            let _ = stop_video_wallpaper(&app_handle_for_tray);

                            // Give cleanup time to complete
                            std::thread::sleep(std::time::Duration::from_millis(500));

                            // Force exit
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
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
                // Delay to allow app to fully initialize
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
