/// search and scraping related commands for the triple load
use crate::models::SearchResponse;
use crate::scraper::*;
use rand::seq::SliceRandom;
use std::collections::HashSet;

#[tauri::command]
pub async fn search_wallpapers(
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
pub async fn fetch_live2d(query: Option<String>) -> Result<SearchResponse, String> {
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
pub async fn resolve_wallpaperflare_highres(
    detail_url: String,
) -> Result<crate::models::ResolveHighResResponse, String> {
    println!("info: resolving high-res for: {}", detail_url);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x84) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    match resolve_wallpaperflare_download(&detail_url, &client).await {
        Ok((high_res_url, _, _)) => {
            println!("OK resolved to: {}", high_res_url);
            Ok(crate::models::ResolveHighResResponse {
                success: true,
                url: Some(high_res_url),
                url4k: None,
                error: None,
            })
        }
        Err(e) => {
            println!("error: failed to resolve: {}", e);
            Ok(crate::models::ResolveHighResResponse {
                success: false,
                url: None,
                url4k: None,
                error: Some(e),
            })
        }
    }
}

#[tauri::command]
pub async fn resolve_motionbgs_video(detail_url: String) -> Result<crate::models::ResolveHighResResponse, String> {
    println!("info: RESOLVING motionBg video: {}", detail_url);

    match scrape_motionbgs_detail(&detail_url).await {
        Ok((video_url, video_url_4k)) => {
            println!("ok: found video url: {}", video_url);
            Ok(crate::models::ResolveHighResResponse {
                success: true,
                url: Some(video_url),
                url4k: video_url_4k,
                error: None,
            })
        }
        Err(e) => {
            println!("error: failed to resolve: {}", e);
            Ok(crate::models::ResolveHighResResponse {
                success: false,
                url: None,
                url4k: None,
                error: Some(e),
            })
        }
    }
}

