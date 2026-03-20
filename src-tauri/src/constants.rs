/// Base URL of the SoulOverlay backend.
/// Set `BACKEND_URL` env var at build time to override (e.g. for local dev).
/// Default: https://overlay.soulreturns.com
pub const BACKEND_URL: &str = env!("BACKEND_URL");

/// Static app token sent as `X-Soul-App-Token` header to authenticate with the backend.
/// Set `SOUL_APP_TOKEN` env var at build time. Must match the backend's `SOUL_APP_TOKEN` env var.
pub const SOUL_APP_TOKEN: &str = env!("SOUL_APP_TOKEN");

/// Backend endpoint that returns the UEX API key.
pub const BACKEND_CONFIG_ENDPOINT: &str = "/api/config";
