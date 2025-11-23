use crate::models::*;
use scraper::{Html, Selector};
use regex::Regex;
use std::collections::HashSet;

// url normalization
pub fn absolute_url(href: &str, base: &str) -> String {
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

// srcset/cdn parsing
pub fn pick_image_source(value: &str) -> String {
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

// regex parse for WxH
pub fn parse_resolution(text: &str) -> (Option<u32>, Option<u32>) {
    let re = Regex::new(r"(\d{3,5})\s*[x×]\s*(\d{3,5})").unwrap();
    
    if let Some(caps) = re.captures(text) {
        let width = caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok());
        let height = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok());
        return (width, height);
    }
    
    (None, None)
}

// wallhaven main scraper
pub async fn scrape_wallhaven(
    query: &str,
    page: u32,
    ai_art: bool,
    purity: &str,
    limit: usize,
) -> Result<Vec<WallpaperItem>, String> {
    // http client setup
    let client = reqwest::Client::builder()
        .user_agent("WallpaperApp/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    // ai filter logic
    let ai_filter = if ai_art { "0" } else { "1" };
    let url = format!(
        "https://wallhaven.cc/search?q={}&page={}&purity={}&ai_art_filter={}",
        urlencoding::encode(query),
        page,
        purity,
        ai_filter
    );

    // fetch + parse
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let thumb_selector = Selector::parse(".thumb-listing-page ul li .thumb").unwrap();
    let preview_selector = Selector::parse(".preview").unwrap();
    let thumb_info_selector = Selector::parse(".thumb-info .png span").unwrap();
    let img_selector = Selector::parse("img").unwrap();

    let mut items = Vec::new();

    for element in document.select(&thumb_selector).take(limit) {
        // preview url extraction
        if let Some(preview) = element.select(&preview_selector).next() {
            if let Some(preview_url) = preview.value().attr("href") {
                if let Some(id) = preview_url.split('/').last() {
                    // png/jpg logic
                    let is_png = element.select(&thumb_info_selector).next().is_some();
                    let ext = if is_png { ".png" } else { ".jpg" };
                    let short = &id[..2.min(id.len())];
                    let image_url = format!("https://w.wallhaven.cc/full/{}/wallhaven-{}{}", short, id, ext);

                    // thumb fallback
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
                        detail_url: None,
                        original: None,
                    });
                }
            }
        }
    }

    if items.is_empty() {
        return Err("wallhaven returned no results".to_string());
    }

    Ok(items)
}

// zerochan main scraper
pub async fn scrape_zerochan(query: &str, limit: usize) -> Result<Vec<WallpaperItem>, String> {
    // http client setup
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
            detail_url: None,
            original: None,
        });
    }

    if items.is_empty() {
        return Err("zerochan returned no results".to_string());
    }

    Ok(items)
}

// wallpapers.com main scraper
pub async fn scrape_wallpapers_com(query: &str, limit: usize) -> Result<Vec<WallpaperItem>, String> {
    // http client setup
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
                detail_url: None,
                original: None,
            });
        }
    }

    if items.is_empty() {
        return Err("wallpapers.com returned no results".to_string());
    }

    Ok(items)
}

// wallpaperflare download resolver
pub async fn resolve_wallpaperflare_download(
    detail_url: &str,
    client: &reqwest::Client,
) -> Result<(String, Option<u32>, Option<u32>), String> {
    // try download page first
    let absolute = absolute_url(detail_url, "https://www.wallpaperflare.com");
    let download_page_url = format!("{}/download", absolute.trim_end_matches('/'));
    
    println!("debug: resolving high-res from: {}", download_page_url); // resolve-path
    
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
            
            let show_img_selector = Selector::parse("#show_img").unwrap();
            let content_url_selector = Selector::parse("img[itemprop=\"contentUrl\"]").unwrap();
            
            // try both selectors
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
                println!("ok: found high-res image: {}", final_url); // found-image
                return Ok((final_url, width, height));
            }
        }
    }
    
    println!("debug: download page failed, trying detail page: {}", absolute); // fallback-detail
    
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
            
            // try all fallback selectors
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
                println!("ok: found image from detail page: {}", final_url); // detail-ok
                return Ok((final_url, width, height));
            }
            
            Err("no image found on detail page".to_string())
        }
        Err(e) => Err(format!("failed to fetch detail page: {}", e)),
    }
}

// wallpaperflare main scraper
pub async fn scrape_wallpaperflare(query: &str, limit: usize) -> Result<Vec<WallpaperItem>, String> {
    // http client setup
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
        return Err("wallpaperflare returned no results".to_string());
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
        detail_url: Some(temp_item.detail_url),
        original: None,
    }).collect();

    Ok(items)
}

// moewalls main scraper
pub async fn scrape_moewalls(
    query: Option<&str>,
    limit: usize,
    include_videos: bool,
) -> Result<Vec<WallpaperItem>, String> {
    // http client setup
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
                        detail_url: None,
                        original: None,
                    });
                }
            }
        }
    }

    if items.is_empty() {
        return Err("moewalls returned no results".to_string());
    }

    Ok(items)
}

// slugify for motionbgs
fn motionbgs_tag_slug(query: &str) -> String {
    let sanitized = query
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>();

    sanitized
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// motionbgs main scraper
pub async fn scrape_motionbgs(query: &str, limit: usize, page: u32) -> Result<Vec<WallpaperItem>, String> {
    // http client for motionbgs (client-note)
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    // slug logic
    let slug = {
        let slugged = motionbgs_tag_slug(query);
        if slugged.is_empty() {
            "featured".to_string()
        } else {
            slugged
        }
    };

    let page_index = page.max(1);
    let url = if page_index <= 1 {
        format!("https://motionbgs.com/tag:{}/", slug)
    } else {
        format!("https://motionbgs.com/tag:{}/{}/", slug, page_index)
    };

    println!("info: fetching motionbgs: {}", url); // fetch-url
    
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    let document = Html::parse_document(&html);
    let tmb_selector = Selector::parse("div.tmb a[href]").unwrap();
    let img_selector = Selector::parse("img").unwrap();
    let title_selector = Selector::parse("span.ttl").unwrap();
    let format_selector = Selector::parse("span.frm").unwrap();

    let mut items = Vec::new();

    for element in document.select(&tmb_selector) {
        let detail_url = element.value().attr("href").unwrap_or("");
        
        if detail_url.is_empty() || detail_url.starts_with("http") {
            continue;
        }

        let img = element.select(&img_selector).next();
        if img.is_none() {
            continue;
        }

        let img_elem = img.unwrap();
        let thumbnail = img_elem.value().attr("src").unwrap_or("");
        
        if thumbnail.is_empty() {
            continue;
        }

        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "MotionBGs Live Wallpaper".to_string());

        let format = element
            .select(&format_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "".to_string());

        // id from url
        let id = detail_url
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();

        let thumbnail_url = absolute_url(thumbnail, "https://motionbgs.com");
        let full_detail_url = absolute_url(detail_url, "https://motionbgs.com");

        // parse res from format string
        let (width, height) = if format.contains("4K") {
            (Some(3840), Some(2160))
        } else if format.contains("1080p") || format.contains("FHD") {
            (Some(1920), Some(1080))
        } else {
            (None, None)
        };

        items.push(WallpaperItem {
            id: format!("motionbgs-{}", id),
            source: "motionbgs".to_string(),
            title: Some(title),
            image_url: thumbnail_url.clone(),
            thumbnail_url: Some(thumbnail_url),
            media_type: Some("video".to_string()),
            width,
            height,
            tags: None,
            detail_url: Some(full_detail_url),
            original: None,
        });

        if items.len() >= limit {
            break;
        }
    }

    if items.is_empty() {
        return Err("motionbgs returned no results".to_string());
    }

    Ok(items)
}

// motionbgs detail extractor
pub async fn scrape_motionbgs_detail(detail_url: &str) -> Result<(String, Option<String>), String> {
    // http client for detail parsing (client-note)
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    println!("info: fetching motionbgs detail: {}", detail_url); // fetch-detail

    let response = client.get(detail_url).send().await.map_err(|e| e.to_string())?;
    let html = response.text().await.map_err(|e| e.to_string())?;

    // json-ld block extraction
    let json_ld_start = html.find(r#"<script type=application/ld+json>"#)
        .ok_or_else(|| "json-ld script not found".to_string())?;

    let preview_url = (|| {
        let json_start = json_ld_start + r#"<script type=application/ld+json>"#.len();
        html[json_start..]
            .find("</script>")
            .and_then(|json_end| {
                let json_str = &html[json_start..json_start + json_end];
                serde_json::from_str::<serde_json::Value>(json_str)
                    .ok()
                    .and_then(|json_value| {
                        json_value["contentUrl"]
                            .as_str()
                            .map(|url| absolute_url(url, "https://motionbgs.com"))
                    })
            })
    })()
    .ok_or_else(|| "preview video url not found in json-ld".to_string())?;

    println!("[info] found preview video url: {}", preview_url);

    // 4k download link extraction
    let document = Html::parse_document(&html);
    let download_selector = Selector::parse("div.download a[href*='/dl/4k/']").unwrap();
    
    let download_4k_url = document
        .select(&download_selector)
        .next()
        .and_then(|link| link.value().attr("href"))
        .map(|href| absolute_url(href, "https://motionbgs.com"));

    if let Some(ref url) = download_4k_url {
        println!("[success] found 4k download url: {}", url);
    } else {
        println!("[warn] 4k download link not found, using preview url");
    }

    Ok((preview_url, download_4k_url))
}
