// Wallpaper Player Binary - Separate Process
// This runs independently from the Tauri UI to avoid DWM composition conflicts
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

mod desktop_injection;
mod mpv_player;
mod os_version;
mod wmf_player;

#[cfg(target_os = "windows")]
fn main() {
    let args: Vec<String> = env::args().collect();

    // Usage: wallpaper-player <video_path> <width> <height> [backend] [mpv_path]
    // backend defaults to "wmf" if not provided
    if args.len() < 4 {
        eprintln!("Usage: wallpaper-player <video_path> <width> <height> [backend] [mpv_path]");
        std::process::exit(1);
    }

    let video_path = &args[1];
    let width: i32 = args[2].parse().expect("Invalid width");
    let height: i32 = args[3].parse().expect("Invalid height");
    let backend = if args.len() > 4 { &args[4] } else { "wmf" };
    let mpv_path = if args.len() > 5 {
        Some(args[5].clone())
    } else {
        None
    };

    println!("[player] Starting wallpaper player");
    println!("[player] Video: {}", video_path);
    println!("[player] Resolution: {}x{}", width, height);
    println!("[player] Backend: {}", backend);
    if let Some(ref path) = mpv_path {
        println!("[player] Custom MPV Path: {}", path);
    }

    unsafe {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            eprintln!("[player] Failed to initialize COM: {:?}", hr);
            // non-fatal for mpv? but good practice
        }
    }

    enum PlayerBackend {
        Wmf(wmf_player::WmfPlayer),
        Mpv(mpv_player::MpvPlayer),
    }

    let player_instance = if backend == "mpv" {
        let initial_window = unsafe { mpv_player::create_mpv_window(width, height) }
            .expect("Failed to create MPV host window");

        // Pass mpv_path to constructor
        let mpv = mpv_player::MpvPlayer::new(initial_window, video_path, width, height, mpv_path)
            .expect("Failed to start MPV");

        let _ = mpv.play();
        PlayerBackend::Mpv(mpv)
    } else {
        let wmf = wmf_player::WmfPlayer::new(width, height).expect("Failed to create WMF");
        wmf.load_video(video_path).expect("Failed to load video");
        wmf.play().expect("Failed to play");
        PlayerBackend::Wmf(wmf)
    };

    let hwnd = match &player_instance {
        PlayerBackend::Wmf(p) => p.hwnd(),
        PlayerBackend::Mpv(p) => p.hwnd(),
    };

    if let Err(e) = desktop_injection::inject_behind_desktop(hwnd, 0, 0, width, height) {
        eprintln!("[player] Failed to inject behind desktop: {}", e);
        std::process::exit(1);
    }

    println!("[player] Injected behind desktop");
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
