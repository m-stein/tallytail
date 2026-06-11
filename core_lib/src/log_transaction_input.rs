use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    #[default]
    Buy,
    Sell,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LogTransactionInput {
    pub r#type: TransactionType,
    pub isin: String,
    pub quantity: String,
    pub stock_price: String,
    pub order_value: String,
}
