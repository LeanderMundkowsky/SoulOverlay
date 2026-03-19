use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredItemDto {
    pub quantity: u32,
    pub items: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikeloTradeDto {
    pub id: String,
    pub mission_name: String,
    #[serde(deserialize_with = "deserialize_reward")]
    pub reward_name: Vec<String>,
    pub category: String,
    pub patch: String,
    pub reputation: String,
    pub required_items: Vec<RequiredItemDto>,
    #[serde(default)]
    pub description: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct RequiredItem {
    pub quantity: u32,
    pub item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WikeloTrade {
    pub id: String,
    pub mission_name: String,
    pub reward_names: Vec<String>,
    pub category: String,
    pub patch: String,
    pub reputation: String,
    pub required_items: Vec<RequiredItem>,
    pub description: String,
    pub active: bool,
}

impl From<WikeloTradeDto> for WikeloTrade {
    fn from(dto: WikeloTradeDto) -> Self {
        Self {
            id: dto.id,
            mission_name: dto.mission_name,
            reward_names: dto.reward_name,
            category: dto.category,
            patch: dto.patch,
            reputation: dto.reputation,
            required_items: dto
                .required_items
                .into_iter()
                .map(|r| RequiredItem { quantity: r.quantity, item: r.items })
                .collect(),
            description: dto.description,
            active: dto.active,
        }
    }
}

/// Handles both `"single reward"` and `["reward a", "reward b"]`.
fn deserialize_reward<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => Ok(vec![s]),
        Value::Array(arr) => arr
            .into_iter()
            .map(|x| match x {
                Value::String(s) => Ok(s),
                other => Ok(other.to_string()),
            })
            .collect(),
        other => Ok(vec![other.to_string()]),
    }
}
