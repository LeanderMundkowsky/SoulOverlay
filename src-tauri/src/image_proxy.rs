use log::{debug, warn};

const ALLOWED_HOSTS: [&str; 2] = ["cdn.uexcorp.space", "assets.uexcorp.space"];

/// Fetches an external image by proxying through reqwest.
/// The request path encodes the original URL: `/{host}/{rest_of_path}`
pub async fn fetch(request: tauri::http::Request<Vec<u8>>) -> tauri::http::Response<Vec<u8>> {
    let path = request.uri().path();

    // Path format: /cdn.uexcorp.space/vehicles/73/cover/hash.jpg
    let trimmed = path.strip_prefix('/').unwrap_or(path);
    let (host, rest) = match trimmed.split_once('/') {
        Some((h, r)) => (h, r),
        None => {
            warn!("[image_proxy] Invalid path: {}", path);
            return error_response(400);
        }
    };

    if !ALLOWED_HOSTS.contains(&host) {
        warn!("[image_proxy] Blocked host: {}", host);
        return error_response(403);
    }

    let target_url = format!("https://{}/{}", host, rest);
    debug!("[image_proxy] Fetching: {}", target_url);

    let client = reqwest::Client::new();
    match client.get(&target_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("image/jpeg")
                .to_string();

            match resp.bytes().await {
                Ok(bytes) => tauri::http::Response::builder()
                    .status(200)
                    .header("content-type", content_type)
                    .header("cache-control", "public, max-age=86400")
                    .body(bytes.to_vec())
                    .unwrap(),
                Err(e) => {
                    warn!("[image_proxy] Body read failed: {}", e);
                    error_response(502)
                }
            }
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            warn!("[image_proxy] Upstream {} for {}", status, target_url);
            error_response(status)
        }
        Err(e) => {
            warn!("[image_proxy] Fetch error: {}", e);
            error_response(502)
        }
    }
}

fn error_response(status: u16) -> tauri::http::Response<Vec<u8>> {
    tauri::http::Response::builder()
        .status(status)
        .body(Vec::new())
        .unwrap()
}
