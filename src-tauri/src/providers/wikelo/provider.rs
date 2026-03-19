use async_trait::async_trait;
use log::{info, warn};
use regex::Regex;

use super::dto::{WikeloTrade, WikeloTradeDto};
use crate::cache_store::Collection;
use crate::providers::{store_blob, BlobProvider, RefreshContext};

const BASE_URL: &str = "https://wikelotrades.com";

pub struct WikiloTradesProvider;

#[async_trait]
impl BlobProvider for WikiloTradesProvider {
    fn collection(&self) -> Collection {
        Collection::WikiloTrades
    }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let all_trades = fetch_all_trades(ctx.client.client()).await?;
        let ttl = self.collection().ttl_for(ctx.settings);
        let count = all_trades.len() as u32;
        store_blob(ctx.cache, self.collection(), ttl, &all_trades, count)
    }
}

/// Fetches the manifest, then each enabled patch file, combines and deduplicates all trades.
pub async fn fetch_all_trades(
    client: &reqwest::Client,
) -> Result<Vec<WikeloTrade>, String> {
    let manifest_url = format!("{}/scripts/trades/manifest.js", BASE_URL);
    let manifest_js = client
        .get(&manifest_url)
        .header("User-Agent", concat!("SoulOverlay/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Wikelo manifest: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read Wikelo manifest: {}", e))?;

    let patches = parse_patch_manifest(&manifest_js)?;
    info!("Wikelo: found {} patch files in manifest", patches.len());

    // Fetch all enabled patch files, newest-first so we dedup by id keeping the latest.
    let mut seen_ids = std::collections::HashSet::new();
    let mut all_trades: Vec<WikeloTrade> = Vec::new();

    for patch_src in patches.iter().rev() {
        let url = format!("{}/{}", BASE_URL, patch_src.trim_start_matches('/'));
        match fetch_patch_file(client, &url).await {
            Ok(trades) => {
                info!("Wikelo: loaded {} trades from {}", trades.len(), url);
                for trade in trades {
                    if seen_ids.insert(trade.id.clone()) {
                        all_trades.push(trade);
                    }
                }
            }
            Err(e) => warn!("Wikelo: failed to load {}: {}", url, e),
        }
    }

    // Restore chronological order (oldest patch first -> newest last).
    all_trades.reverse();

    info!("Wikelo: total {} unique trades loaded", all_trades.len());
    Ok(all_trades)
}

/// Parse `window.tradePatches = [{patch,src,enabled}...]` and return enabled src paths.
fn parse_patch_manifest(js: &str) -> Result<Vec<String>, String> {
    let json = js_to_json(js)?;
    let patches: Vec<serde_json::Value> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;
    Ok(patches
        .into_iter()
        .filter(|p| p.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
        .filter_map(|p| p.get("src").and_then(|v| v.as_str()).map(String::from))
        .collect())
}

/// Fetch a single patch .js file and return parsed trades.
async fn fetch_patch_file(
    client: &reqwest::Client,
    url: &str,
) -> Result<Vec<WikeloTrade>, String> {
    let js = client
        .get(url)
        .header("User-Agent", concat!("SoulOverlay/", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?
        .text()
        .await
        .map_err(|e| format!("read failed: {}", e))?;

    let json = js_to_json(&js)?;
    let dtos: Vec<WikeloTradeDto> = serde_json::from_str(&json)
        .map_err(|e| format!("JSON parse failed: {}", e))?;
    Ok(dtos.into_iter().map(WikeloTrade::from).collect())
}

/// Convert JS object-notation to valid JSON:
///  1. Extract the array content between the first `[` and last `]`.
///  2. Quote unquoted identifier keys -- every object key follows `{` or `,` (handles
///     both minified single-line and pretty-printed JS).
///  3. Remove trailing commas before `}` or `]`.
fn js_to_json(js: &str) -> Result<String, String> {
    let start = js.find('[').ok_or("no array found in JS")?;
    let end = js.rfind(']').ok_or("no closing ] found in JS")?;
    if end < start {
        return Err("malformed JS array".to_string());
    }
    let array = &js[start..=end];

    // Quote unquoted keys. Every object key immediately follows `{` or `,`
    // (with optional whitespace/newlines). This handles both minified and
    // pretty-printed JS without mis-matching colon-containing string values.
    let key_re = Regex::new(r"([{,])(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*:")
        .map_err(|e| e.to_string())?;
    let quoted = key_re.replace_all(array, |caps: &regex::Captures| {
        format!("{}{}\"{}\":", &caps[1], &caps[2], &caps[3])
    });

    // Remove trailing commas before `}` or `]`.
    let comma_re = Regex::new(r",(\s*[}\]])").map_err(|e| e.to_string())?;
    let clean = comma_re.replace_all(&quoted, "$1");

    Ok(clean.into_owned())
}