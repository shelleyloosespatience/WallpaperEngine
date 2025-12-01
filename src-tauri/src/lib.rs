pub mod models;
pub mod scraper;
pub mod os_version;
pub mod desktop_injection;

#[cfg(target_os = "windows")]
pub mod wmf_player;

#[cfg(target_os = "linux")]
pub mod video_wallpaper_linux;

pub use models::*;
pub use desktop_injection::*;