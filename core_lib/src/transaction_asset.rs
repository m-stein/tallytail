use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTransactionAssetsInput {
    pub isins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAsset {
    pub id: i64,
    pub isin: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub quote_type: Option<String>,
    pub updated_at_date: Option<String>,
    pub updated_at_time: Option<String>,
}
