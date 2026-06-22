use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItem {
    pub id: i64,
    pub buy_transaction_id: i64,
    pub buy_date: String,
    pub isin: String,
    pub initial_quantity: String,
    pub remaining_quantity: String,
    pub share_price: String,
    pub order_value: String,
    pub currency: String,
}
