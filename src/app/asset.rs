use crate::app::asset_reference::AssetReference;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub reference: AssetReference,
}