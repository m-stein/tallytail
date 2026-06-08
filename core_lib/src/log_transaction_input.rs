use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LogTransactionInput {
    pub isin: String,
    pub quantity: String,
    pub price: String,
}
