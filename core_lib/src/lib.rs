use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub users: Vec<User>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize, EnumString, Display,
)]
pub enum AssetReferenceType {
    Iban,
    Isin,
    Ticker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReference {
    pub r#type: AssetReferenceType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationCategoryValue {
    pub name: String,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationAssetCategory {
    pub name: String,
    pub values: Vec<AllocationCategoryValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationAsset {
    pub name: String,
    pub reference: AssetReference,
    pub categories: Vec<AllocationAssetCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPosition {
    pub asset: AllocationAsset,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    pub date: String,
    pub positions: Vec<AllocationPosition>,
}
