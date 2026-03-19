pub mod dto;
pub mod provider;

pub use dto::WikeloTrade;
pub use provider::{fetch_all_trades, WikiloTradesProvider};
