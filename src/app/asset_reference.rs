use serde::{Deserialize, Serialize};

use crate::app::{asset_reference_type::AssetReferenceType};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReference {
    pub r#type: AssetReferenceType,
    pub value: String,
}