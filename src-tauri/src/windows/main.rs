// Wallpaper Player Binary - Separate Process
// This runs independently from the Tauri UI to avoid DWM composition conflicts
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

mod desktop_injection;
mod os_version;
mod wmf_player;

#[cfg(target_os = "windows")]
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: wallpaper-player <video_path> <width> <height>");
        eprintln!("Example: wallpaper-player C:\\video.mp4 1920 1080");
        std::process::exit(1);
    }

    let video_path = &args[1];
    let width: i32 = args[2].parse().expect("Invalid width");
    let height: i32 = args[3].parse().expect("Invalid height");

    println!("[player] Starting wallpaper player");
    println!("[player] Video: {}", video_path);
    println!("[player] Resolution: {}x{}", width, height);
    unsafe {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            eprintln!("[player] Failed to initialize COM: {:?}", hr);
            std::process::exit(1);
        }
    }

    let player = match wmf_player::WmfPlayer::new(width, height) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[player] Failed to create WMF player: {}", e);
            std::process::exit(1);
        }
    };

    println!("[player] WMF player created");

    if let Err(e) = player.load_video(video_path) {
        eprintln!("[player] Failed to load video: {}", e);
        std::process::exit(1);
    }

    println!("[player] Video loaded");

    let hwnd = player.hwnd();

    if let Err(e) = desktop_injection::inject_behind_desktop(hwnd, 0, 0, width, height) {
        eprintln!("[player] Failed to inject behind desktop: {}", e);
        std::process::exit(1);
    }

    println!("[player] Injected behind desktop");

    if let Err(e) = player.play() {
        eprintln!("[player] Failed to start playback: {}", e);
        std::process::exit(1);
    }

    println!("[player] Playback started");
    println!("[player] Player running - DWM-isolated message loop");

    // Simple, fast Win32 message loop (NO Tauri/WebView overhead!)
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    println!("[player] Player exiting");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    panic!("wallpaper-player is Windows-only, we are working towards linux support!");
}