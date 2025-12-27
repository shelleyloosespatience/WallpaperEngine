// prevents console window on windows in release builds
// main tauri entry point for loading everything together
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod scraper;
mod video_wallpaper;
mod storage;
mod commands;

// Process manager for wallpaper-player sidecar
mod process_manager;

use tauri::{Manager, WindowEvent};
use video_wallpaper::*;
use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_single_instance::init(|app, _argv, _cwd| {
                // When a second instance is launched, focus the existing window
                println!("[main] Second instance detected, focusing existing window");
                
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                    println!("[main] Existing window focused");
                } else {
                    // Window doesn't exist, recreate it
                    println!("[main] Window doesn't exist, recreating...");
                    use tauri::{WebviewUrl, WebviewWindowBuilder};
                    let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                        .title("ColorWall - Wallpaper Engine")
                        .inner_size(1000.0, 900.0)
                        .resizable(true)
                        .decorations(false)
                        .build();
                    
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })
        )
        .invoke_handler(tauri::generate_handler![
            // Search commands
            search_wallpapers,
            fetch_live2d,
            resolve_wallpaperflare_highres,
            resolve_motionbgs_video,
            // Wallpaper commands
            set_wallpaper,
            get_current_wallpaper,
            get_cache_size,
            clear_cache,
            set_video_wallpaper,
            set_video_wallpaper_from_file,
            stop_video_wallpaper_command,
            get_video_wallpaper_status,
            list_user_wallpapers,
            upload_user_wallpaper,
            delete_user_wallpaper,
            get_wallpaper_storage_path,
            download_wallpaper,
            // Settings commands
            get_settings,
            save_settings,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Window close event handler - wallpaper continues in background
            let _app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { .. } = event {
                    println!(
                        "[main] Close button clicked - UI will close, wallpaper continues in background"
                    );
                }
            });

            // System tray setup
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
                        // Recreate window if it doesn't exist
                        if tray.app_handle().get_webview_window("main").is_none() {
                            println!("[main] Window doesn't exist, recreating from tray click...");

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

            // Periodic state saving to prevent data loss (every 30 seconds)
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    periodic_state_save();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
