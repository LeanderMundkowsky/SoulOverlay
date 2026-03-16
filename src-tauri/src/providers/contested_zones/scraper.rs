use log::debug;
use scraper::{Html, Selector};

use super::types::{CzMap, CzShip};

const BASE_URL: &str = "https://contestedzonetimers.com";

/// Fetch the cycle start epoch from cfg.dat.
pub async fn fetch_cycle_start(client: &reqwest::Client) -> Result<u32, String> {
    let url = format!("{}/lib/cfg.dat", BASE_URL);
    debug!("[contested_zones] Fetching cycle start from {}", url);

    let text = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch cfg.dat: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read cfg.dat body: {}", e))?;

    text.trim()
        .parse::<u32>()
        .map_err(|e| format!("Failed to parse cfg.dat epoch '{}': {}", text.trim(), e))
}

/// Scrape ship data from the /ships HTML page.
pub fn scrape_ships(html: &str) -> Vec<CzShip> {
    let doc = Html::parse_document(html);
    let item_sel = Selector::parse(".ship-item").unwrap();
    let title_sel = Selector::parse(".ship-title").unwrap();
    let type_sel = Selector::parse(".ship-type").unwrap();
    let credit_sel = Selector::parse(".image-credit").unwrap();
    let img_sel = Selector::parse(".image-wrapper img").unwrap();
    let link_sel = Selector::parse(".ship-type a").unwrap();

    let mut ships = Vec::new();

    for item in doc.select(&item_sel) {
        let name = item
            .select(&title_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Extract ship type from the text node before the first link
        let ship_type = item
            .select(&type_sel)
            .next()
            .map(|e| {
                e.text()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches('–')
                    .trim_end_matches('\u{2013}')
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        let credit = item
            .select(&credit_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let image_url = item
            .select(&img_sel)
            .next()
            .and_then(|e| e.value().attr("src"))
            .map(|src| {
                if src.starts_with('/') {
                    format!("{}{}", BASE_URL, src)
                } else {
                    src.to_string()
                }
            })
            .unwrap_or_default();

        // Extract wiki and pledge links
        let mut wiki_url = None;
        let mut pledge_url = None;
        for link in item.select(&link_sel) {
            if let Some(href) = link.value().attr("href") {
                if href.contains("starcitizen.tools") {
                    wiki_url = Some(href.to_string());
                } else if href.contains("robertsspaceindustries.com") {
                    pledge_url = Some(href.to_string());
                }
            }
        }

        if !name.is_empty() && !image_url.is_empty() {
            ships.push(CzShip {
                name,
                ship_type,
                image_url,
                wiki_url,
                pledge_url,
                credit,
            });
        }
    }

    debug!("[contested_zones] Scraped {} ships", ships.len());
    ships
}

/// Scrape map data from the /contested-zone-maps HTML page.
pub fn scrape_maps(html: &str) -> Vec<CzMap> {
    let doc = Html::parse_document(html);
    let img_sel = Selector::parse("img").unwrap();

    let mut maps = Vec::new();

    for img in doc.select(&img_sel) {
        let src = img.value().attr("src").unwrap_or("");
        let alt = img.value().attr("alt").unwrap_or("");

        // Only include images from the /maps/ directory
        if !src.contains("/maps/") {
            continue;
        }

        let image_url = if src.starts_with('/') {
            format!("{}{}", BASE_URL, src)
        } else {
            src.to_string()
        };

        let name = if !alt.is_empty() {
            alt.to_string()
        } else {
            // Derive name from filename
            src.rsplit('/')
                .next()
                .unwrap_or("Unknown")
                .trim_end_matches(".webp")
                .trim_end_matches(".png")
                .trim_end_matches(".jpg")
                .to_string()
        };

        maps.push(CzMap { name, image_url });
    }

    debug!("[contested_zones] Scraped {} maps", maps.len());
    maps
}

/// Fetch and scrape ships from the website.
pub async fn fetch_ships(client: &reqwest::Client) -> Result<Vec<CzShip>, String> {
    let url = format!("{}/ships", BASE_URL);
    debug!("[contested_zones] Fetching ships from {}", url);

    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch ships page: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read ships page body: {}", e))?;

    Ok(scrape_ships(&html))
}

/// Fetch and scrape maps from the website.
pub async fn fetch_maps(client: &reqwest::Client) -> Result<Vec<CzMap>, String> {
    let url = format!("{}/contested-zone-maps", BASE_URL);
    debug!("[contested_zones] Fetching maps from {}", url);

    let html = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch maps page: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read maps page body: {}", e))?;

    Ok(scrape_maps(&html))
}
