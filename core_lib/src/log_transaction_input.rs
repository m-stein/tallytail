use jiff::{Zoned, civil::Date};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum Currency {
    #[strum(serialize = "EUR")]
    Eur,
    #[strum(serialize = "USD")]
    Usd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogTransactionInput {
    pub r#type: TransactionType,
    pub currency: Currency,
    pub isin: String,
    pub quantity: String,
    pub share_price: String,
    pub order_value: String,
    pub date: Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSellTransactionInput {
    pub currency: Currency,
    pub isin: String,
    pub portfolio_item_id_to_quantity: HashMap<i64, String>,
    pub share_price: String,
    pub order_value: String,
    pub date: Date,
}

impl Default for LogSellTransactionInput {
    fn default() -> Self {
        Self {
            currency: Currency::Eur,
            isin: String::new(),
            portfolio_item_id_to_quantity: HashMap::new(),
            share_price: String::new(),
            order_value: String::new(),
            date: Zoned::now().date(),
        }
    }
}

impl Default for LogTransactionInput {
    fn default() -> Self {
        Self {
            r#type: TransactionType::Buy,
            currency: Currency::Eur,
            date: Zoned::now().date(),
            isin: String::new(),
            quantity: String::new(),
            share_price: String::new(),
            order_value: String::new(),
        }
    }
}
