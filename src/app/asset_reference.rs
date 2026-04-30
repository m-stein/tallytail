use serde::{Deserialize, Serialize};

use crate::app::{asset_reference_type::AssetReferenceType, error::Error};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReference {
    pub r#type: AssetReferenceType,
    pub value: String,
}

impl AssetReference {
    pub fn new(reference_type: AssetReferenceType, value: String) -> Result<Self, Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err("Reference value must not be empty".to_string())?
        }
        Ok(Self {
            r#type: reference_type,
            value: trimmed.to_string(),
        })
    }
}