pub mod models;
pub mod scraper;

#[cfg(target_os = "windows")]
pub mod wmf_player;

#[cfg(target_os = "linux")]
pub mod video_wallpaper_linux;

pub use models::*;