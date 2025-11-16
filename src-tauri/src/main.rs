// prevents additional console window on windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use std::collections::HashSet;
use rand::seq::SliceRandom;
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WallpaperItem {
    id: String,
    source: String,
    title: Option<String>,
    image_url: String,
    thumbnail_url: Option<String>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    tags: Option<Vec<String>>,
    original: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    success: bool,
    items: Vec<WallpaperItem>,
    errors: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WallpaperResponse {
    success: bool,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheSizeResponse {
    success: bool,
    size_mb: String,
    file_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClearCacheResponse {
    success: bool,
    files_deleted: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResolveHighResResponse {
    success: bool,
    url: Option<String>,
    error: Option<String>,
}

async fn scrape_wallhaven(
    query: &str,
    page: u32,
    ai_art: bool,
    purity: &str,
    limit: usize,
) -> Result<Vec<WallpaperItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("WallpaperApp/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let ai_filter = if ai_art { "0" } else { "1" };
    let url = format!(
        "https://wallhaven.cc/search?q={}&page={}&purity={}&ai_art_filter={}",
        urlencoding::encode(query),
        page,
        purity,
        ai_filter
    );

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let thumb_selector = Selector::parse(".thumb-listing-page ul li .thumb").unwrap();
    let preview_selector = Selector::parse(".preview").unwrap();
    let thumb_info_selector = Selector::parse(".thumb-info .png span").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let mut items = Vec::new();

    for element in document.select(&thumb_selector).take(limit) {
        if let Some(preview) = element.select(&preview_selector).next() {
            if let Some(preview_url) = preview.value().attr("href") {
                if let Some(id) = preview_url.split('/').last() {
                    let is_png = element.select(&thumb_info_selector).next().is_some();
                    let ext = if is_png { ".png" } else { ".jpg" };
                    let short = &id[..2.min(id.len())];
                    let image_url = format!("https://w.wallhaven.cc/full/{}/wallhaven-{}{}", short, id, ext);

                    let thumbnail_url = element
                        .select(&img_selector)
                        .next()
                        .and_then(|img| {
                            img.value()
                                .attr("data-src")
                                .or_else(|| img.value().attr("src"))
                        })
                        .map(String::from);

                    items.push(WallpaperItem {
                        id: format!("wallhaven-{}", id),
                        source: "wallhaven".to_string(),
                        title: Some(id.to_string()),
                        image_url,
                        thumbnail_url,
                        media_type: Some("image".to_string()),
                        width: None,
                        height: None,
                        tags: None,
                        original: None,
                    });
                }
            }
        }
    }

    if items.is_empty() {
        return Err("Wallhaven returned no results".to_string());
    }

    Ok(items)
}

async fn scrape_zerochan(query: &str, limit: usize) -> Result<Vec<WallpaperItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://www.zerochan.net/{}", urlencoding::encode(query));
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let item_selector = Selector::parse("#wrapper #content ul li").unwrap();
    let link_selector = Selector::parse("p a").unwrap();
    let img_selector = Selector::parse("img").unwrap();
    let anchor_selector = Selector::parse("a").unwrap();

    let mut items = Vec::new();

    for (index, element) in document.select(&item_selector).enumerate() {
        if items.len() >= limit {
            break;
        }
        
        let img = element.select(&img_selector).next();
        if img.is_none() {
            continue;
        }
        
        let img_elem = img.unwrap();
        
        let thumb_src = img_elem
            .value()
            .attr("data-src")
            .or_else(|| img_elem.value().attr("src"))
            .or_else(|| img_elem.value().attr("data-original"))
            .unwrap_or("");
            
        if thumb_src.is_empty() {
            continue;
        }
        
        let link = element.select(&link_selector).next()
            .or_else(|| element.select(&anchor_selector).next());
            
        let image_link = if let Some(l) = link {
            l.value().attr("href").unwrap_or("")
        } else {
            ""
        };
        
        let title = img_elem.value().attr("alt").unwrap_or("Zerochan Wallpaper");
        
        let thumbnail_url = absolute_url(thumb_src, "https://www.zerochan.net");
        let image_url = if !image_link.is_empty() {
            absolute_url(image_link, "https://static.zerochan.net")
        } else {
            thumbnail_url.clone()
        };
        
        let id = if !image_link.is_empty() {
            image_link.split('/').last().unwrap_or(&index.to_string()).to_string()
        } else {
            index.to_string()
        };

        items.push(WallpaperItem {
            id: format!("zerochan-{}", id),
            source: "zerochan".to_string(),
            title: Some(title.to_string()),
            image_url,
            thumbnail_url: Some(thumbnail_url),
            media_type: Some("image".to_string()),
            width: None,
            height: None,
            tags: None,
            original: None,
        });
    }

    if items.is_empty() {
        return Err("Zerochan returned no results".to_string());
    }

    Ok(items)
}

async fn scrape_wallpapers_com(query: &str, limit: usize) -> Result<Vec<WallpaperItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://wallpapers.com/search/{}", urlencoding::encode(query));
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let item_selector = Selector::parse(".tab-content ul.kw-contents li").unwrap();
    let figure_selector = Selector::parse("figure").unwrap();
    let _anchor_selector = Selector::parse("a").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let mut items = Vec::new();

    for element in document.select(&item_selector).take(limit) {
        if let Some(figure) = element.select(&figure_selector).next() {
            let title = figure.value().attr("data-title").unwrap_or("Wallpapers.com");
            let key = figure.value().attr("data-key").unwrap_or("");

            if key.is_empty() {
                continue;
            }

            let thumb_src = element
                .select(&img_selector)
                .next()
                .and_then(|img| {
                    img.value()
                        .attr("data-src")
                        .or_else(|| img.value().attr("src"))
                })
                .unwrap_or("");

            let thumbnail_url = if !thumb_src.is_empty() {
                absolute_url(thumb_src, "https://wallpapers.com")
            } else {
                String::new()
            };

            items.push(WallpaperItem {
                id: format!("wallpapers-{}", key),
                source: "wallpapers".to_string(),
                title: Some(title.to_string()),
                image_url: thumbnail_url.clone(),
                thumbnail_url: Some(thumbnail_url),
                media_type: Some("image".to_string()),
                width: None,
                height: None,
                tags: None,
                original: None,
            });
        }
    }

    if items.is_empty() {
        return Err("Wallpapers.com returned no results".to_string());
    }

    Ok(items)
}

fn absolute_url(href: &str, base: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{}", href)
    } else if href.starts_with('/') {
        format!("{}{}", base.trim_end_matches('/'), href)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), href)
    }
}

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
    let extension = url.split('.').last()
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
        ]
    });
    let limit = limit_per_source.unwrap_or(10);
    let should_randomize = randomize.unwrap_or(true);
    let page_num = page.unwrap_or(1);
    let purity_val = purity.unwrap_or_else(|| "100".to_string());
    let ai_art_enabled = ai_art.unwrap_or(false);

    let mut all_items = Vec::new();
    let mut errors = Vec::new();

    for source in sources {
        let result = match source.as_str() {
            "wallhaven" => {
                scrape_wallhaven(&query, page_num, ai_art_enabled, &purity_val, limit).await
            }
            "zerochan" => scrape_zerochan(&query, limit).await,
            "moewalls" => scrape_moewalls(Some(&query), limit, false).await,
            "wallpapers" => scrape_wallpapers_com(&query, limit).await,
            "wallpaperflare" => scrape_wallpaperflare(&query, limit).await,
            _ => continue,
        };

        match result {
            Ok(items) => all_items.extend(items),
            Err(e) => errors.push(format!("{}: {}", source, e)),
        }
    }

    let mut seen = HashSet::new();
    all_items.retain(|item| seen.insert(item.id.clone()));

    if should_randomize {
        let mut rng = rand::thread_rng();
        all_items.shuffle(&mut rng);
    }

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
    match scrape_moewalls(query.as_deref(), 50, true).await {
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

fn pick_image_source(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    
    let first_segment = value.split(',').next().unwrap_or("").trim();
    first_segment
        .trim_start_matches("url(\"")
        .trim_start_matches("url('")
        .trim_start_matches("url(")
        .trim_end_matches("\")")
        .trim_end_matches("')")
        .trim_end_matches(")")
        .to_string()
}
// // unused, old code
// fn normalize_wallpaperflare_href(raw: &str) -> Option<String> {
//     if raw.is_empty() {
//         return None;
//     }
    
//     let normalized = absolute_url(raw, "https://www.wallpaperflare.com");
    
//     if let Ok(url) = url::Url::parse(&normalized) {
//         let path = url.path().to_lowercase();
        
//         if path.is_empty() 
//             || path == "/" 
//             || path.starts_with("/search") 
//             || path.starts_with("/tag") 
//             || path.starts_with("/page") {
//             return None;
//         }
        
//         if !path.contains("wallpaper") {
//             return None;
//         }
        
//         return Some(url.to_string());
//     }
    
//     None
// }

fn parse_resolution(text: &str) -> (Option<u32>, Option<u32>) {
    let re = Regex::new(r"(\d{3,5})\s*[x×]\s*(\d{3,5})").unwrap();
    
    if let Some(caps) = re.captures(text) {
        let width = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok());
        let height = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok());
        return (width, height);
    }
    
    (None, None)
}

async fn resolve_wallpaperflare_download(
    detail_url: &str,
    client: &reqwest::Client,
) -> Result<(String, Option<u32>, Option<u32>), String> {
    let absolute = absolute_url(detail_url, "https://www.wallpaperflare.com");
    let download_page_url = format!("{}/download", absolute.trim_end_matches('/'));
    
    // go to thier main page with search
    if let Ok(response) = client
        .get(&download_page_url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
        .header("Referer", &absolute)
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await
    {
        if let Ok(html) = response.text().await {
            let document = Html::parse_document(&html);
            
            // get high-res imag
            let show_img_selector = Selector::parse("#show_img").unwrap();
            let content_url_selector = Selector::parse("img[itemprop=\"contentUrl\"]").unwrap();
            
            let high_res_image = document
                .select(&show_img_selector)
                .next()
                .and_then(|el| el.value().attr("src"))
                .or_else(|| {
                    document
                        .select(&content_url_selector)
                        .next()
                        .and_then(|el| el.value().attr("src"))
                });
            
            if let Some(img_url) = high_res_image {
                // res
                let width_selector = Selector::parse("span[itemprop=\"width\"] span[itemprop=\"value\"]").unwrap();
                let height_selector = Selector::parse("span[itemprop=\"height\"] span[itemprop=\"value\"]").unwrap();
                
                let width = document
                    .select(&width_selector)
                    .next()
                    .and_then(|el| el.text().collect::<String>().parse::<u32>().ok());
                
                let height = document
                    .select(&height_selector)
                    .next()
                    .and_then(|el| el.text().collect::<String>().parse::<u32>().ok());
                
                let final_url = absolute_url(img_url, "https://www.wallpaperflare.com");
                return Ok((final_url, width, height));
            }
        }
    }
    
    // fallback
    match client.get(&absolute)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
        .header("Referer", "https://www.wallpaperflare.com/")
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await
    {
        Ok(response) => {
            let html = response.text().await.map_err(|e| e.to_string())?;
            let document = Html::parse_document(&html);
            
            let content_url_selector = Selector::parse("img[itemprop=\"contentUrl\"]").unwrap();
            let vimg_selector = Selector::parse("#vimg").unwrap();
            let og_image_selector = Selector::parse("meta[property=\"og:image\"]").unwrap();
            
            let detail_image = document
                .select(&content_url_selector)
                .next()
                .and_then(|el| el.value().attr("src"))
                .map(pick_image_source)
                .or_else(|| {
                    document
                        .select(&vimg_selector)
                        .next()
                        .and_then(|el| el.value().attr("src"))
                        .map(|s| pick_image_source(s))
                })
                .or_else(|| {
                    document
                        .select(&og_image_selector)
                        .next()
                        .and_then(|el| el.value().attr("content"))
                        .map(|s| pick_image_source(s))
                });
            
            if let Some(img_url) = detail_image {
                let meta_desc_selector = Selector::parse("meta[itemprop=\"description\"]").unwrap();
                let meta_description = document
                    .select(&meta_desc_selector)
                    .next()
                    .and_then(|el| el.value().attr("content"))
                    .unwrap_or("");
                
                let (width, height) = parse_resolution(meta_description);
                
                let final_url = absolute_url(&img_url, "https://www.wallpaperflare.com");
                return Ok((final_url, width, height));
            }
            
            Err("No image found on detail page".to_string())
        }
        Err(e) => Err(format!("Failed to fetch detail page: {}", e)),
    }
}

async fn scrape_wallpaperflare(query: &str, limit: usize) -> Result<Vec<WallpaperItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://www.wallpaperflare.com/search?wallpaper={}",
        urlencoding::encode(query)
    );

    let response = client
        .get(&url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
        .header("Referer", "https://www.wallpaperflare.com/")
        .header("Upgrade-Insecure-Requests", "1")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let html = response.text().await.map_err(|e| e.to_string())?;

    #[derive(Clone)]
    struct TempItem {
        id: String,
        title: String,
        thumbnail_url: String,
        detail_url: String,
    }

    let mut temp_items = Vec::new();
    let mut seen_ids = HashSet::new();

    {
        let document = Html::parse_document(&html);
        let link_selector = Selector::parse("a[href]").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        for link_element in document.select(&link_selector) {
            if temp_items.len() >= limit {
                break;
            }

            let href = link_element.value().attr("href").unwrap_or("");
            if href.is_empty() 
                || href.starts_with('#')
                || href.starts_with("/search")
                || href.starts_with("/tag")
                || href.starts_with("/page")
                || href == "/"
                || !href.contains("wallpaper") {
                continue;
            }

            let normalized_href = absolute_url(href, "https://www.wallpaperflare.com");
            if !normalized_href.to_lowercase().contains("wallpaper") {
                continue;
            }

            let media = link_element.select(&img_selector).next();
            if media.is_none() {
                continue;
            }

            let media_elem = media.unwrap();
            let thumb = media_elem
                .value()
                .attr("data-src")
                .or_else(|| media_elem.value().attr("data-original"))
                .or_else(|| media_elem.value().attr("data-srcset"))
                .or_else(|| media_elem.value().attr("srcset"))
                .or_else(|| media_elem.value().attr("src"))
                .map(|s| pick_image_source(s))
                .unwrap_or_default();

            if thumb.is_empty() {
                continue;
            }

            let id = href
                .trim_start_matches('/')
                .split('-')
                .last()
                .unwrap_or("")
                .to_string();

            if id.is_empty() || id.len() < 3 || seen_ids.contains(&id) {
                continue;
            }
            seen_ids.insert(id.clone());

            let thumbnail_url = absolute_url(&thumb, "https://www.wallpaperflare.com");
            let title = media_elem
                .value()
                .attr("alt")
                .or_else(|| media_elem.value().attr("title"))
                .unwrap_or("WallpaperFlare Wallpaper")
                .to_string();

            temp_items.push(TempItem {
                id: id.clone(),
                title,
                thumbnail_url: thumbnail_url.clone(),
                detail_url: normalized_href,
            });
        }
    }

    if temp_items.is_empty() {
        return Err("WallpaperFlare returned no results".to_string());
    }

    let items = temp_items.into_iter().map(|temp_item| WallpaperItem {
        id: format!("wallpaperflare-{}", temp_item.id),
        source: "wallpaperflare".to_string(),
        title: Some(temp_item.title),
        image_url: temp_item.thumbnail_url.clone(),
        thumbnail_url: Some(temp_item.thumbnail_url),
        media_type: Some("image".to_string()),
        width: None,
        height: None,
        tags: None,
        // Uh.. Store detail_url in original for lazy high-res fetch
        // (serde_json::Value would be better, but for now use Option<String>)
        // we may later want to change WallpaperItem.original to Option<String> or Option<HashMap<String, String>>
        original: Some(serde_json::json!({ "detailUrl": temp_item.detail_url })),
    }).collect();

    Ok(items)
}

async fn scrape_moewalls(
    query: Option<&str>,
    limit: usize,
    include_videos: bool,
) -> Result<Vec<WallpaperItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("WallpaperApp/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let url = if let Some(q) = query {
        format!("https://moewalls.com/?s={}", urlencoding::encode(q))
    } else {
        "https://moewalls.com/".to_string()
    };

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let item_selector = Selector::parse("#primary ul li").unwrap();
    let anchor_selector = Selector::parse("a").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let mut items = Vec::new();
    let video_regex = Regex::new(r"/(\d{4})/\d{2}/([a-z0-9-]+)-thumb").unwrap();

    for element in document.select(&item_selector).take(limit) {
        if let Some(anchor) = element.select(&anchor_selector).next() {
            let title = anchor
                .value()
                .attr("title")
                .unwrap_or("Moewalls Live2D")
                .to_string();
            
            let _url_ref = anchor.value().attr("href").unwrap_or("").to_string();

            if let Some(img) = element.select(&img_selector).next() {
                if let Some(thumbnail) = img.value().attr("src") {
                    let thumbnail_owned = thumbnail.to_string();
                    
                    let video_url = if let Some(caps) = video_regex.captures(&thumbnail_owned) {
                        Some(format!(
                            "https://static.moewalls.com/videos/preview/{}/{}-preview.mp4",
                            &caps[1], &caps[2]
                        ))
                    } else {
                        None
                    };

                    let high_res_image = thumbnail_owned
                        .replace("-thumb", "")
                        .replace("-poster", "");

                    let (media_type, image_url) = if include_videos && video_url.is_some() {
                        ("video", video_url.clone().unwrap())
                    } else {
                        ("image", high_res_image.clone())
                    };

                    let title_slug = title.replace(" ", "-").to_lowercase();
                    let id_slug = video_url
                        .as_ref()
                        .and_then(|v| v.split('/').last())
                        .and_then(|s| s.strip_suffix("-preview.mp4"))
                        .unwrap_or(&title_slug);

                    items.push(WallpaperItem {
                        id: format!("moewalls-{}", id_slug),
                        source: "moewalls".to_string(),
                        title: Some(title),
                        image_url,
                        thumbnail_url: Some(thumbnail_owned),
                        media_type: Some(media_type.to_string()),
                        width: None,
                        height: None,
                        tags: None,
                        original: None,
                    });
                }
            }
        }
    }

    if items.is_empty() {
        return Err("Moewalls returned no results".to_string());
    }

    Ok(items)
}

#[tauri::command]
async fn resolve_wallpaperflare_highres(detail_url: String) -> Result<ResolveHighResResponse, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    match resolve_wallpaperflare_download(&detail_url, &client).await {
        Ok((high_res_url, _, _)) => Ok(ResolveHighResResponse {
            success: true,
            url: Some(high_res_url),
            error: None,
        }),
        Err(e) => Ok(ResolveHighResResponse {
            success: false,
            url: None,
            error: Some(e),
        }),
    }
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
            resolve_wallpaperflare_highres
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}