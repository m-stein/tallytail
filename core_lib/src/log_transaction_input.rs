use jiff::{Zoned, civil::Date};
use serde::{Deserialize, Serialize};
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
