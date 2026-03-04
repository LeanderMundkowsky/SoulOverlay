use async_trait::async_trait;

use super::dto::UserDto;
use crate::cache_store::Collection;
use crate::providers::{store_blob, BlobProvider, RefreshContext};
use crate::uex::types::UexUserProfile;
use crate::uex::UexClient;

pub struct UserProfileProvider;

#[async_trait]
impl BlobProvider for UserProfileProvider {
    fn collection(&self) -> Collection { Collection::UserProfile }
    fn requires_secret(&self) -> bool { true }

    async fn refresh(&self, ctx: &RefreshContext<'_>) -> Result<u32, String> {
        let secret = ctx.secret_key.ok_or("Secret key required for user profile")?;
        let profile = fetch_user_profile(ctx.client, ctx.api_key, secret).await?;
        let ttl = self.collection().ttl_for(ctx.settings);
        store_blob(ctx.cache, self.collection(), ttl, &profile, 1)
    }
}

/// Fetch the authenticated user's profile (for direct use by commands).
pub async fn fetch_user_profile(
    client: &UexClient,
    api_key: &str,
    secret_key: &str,
) -> Result<UexUserProfile, String> {
    let body = client
        .get_raw_with_secret("/user", &[], api_key, secret_key)
        .await?;

    let data = body
        .get("data")
        .ok_or_else(|| "No data field in UEX user response".to_string())?;

    let dto: UserDto = serde_json::from_value(data.clone())
        .map_err(|e| format!("Failed to parse user profile: {}", e))?;

    Ok(UexUserProfile::from(&dto))
}
