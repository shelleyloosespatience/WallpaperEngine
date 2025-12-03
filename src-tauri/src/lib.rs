pub mod models;
pub mod scraper;

#[cfg(target_os = "linux")]
pub mod video_wallpaper_linux;

pub use models::*;
