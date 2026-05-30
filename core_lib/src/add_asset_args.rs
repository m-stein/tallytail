use crate::{AssetReference, AssetReferenceType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CategoryAssignment {
    pub value_id: Option<i64>,
    pub percentage: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddAssetArgs {
    pub name: String,
    pub reference: AssetReference,
    pub category_id_to_assignment: HashMap<i64, Vec<CategoryAssignment>>,
}

impl Default for AddAssetArgs {
    fn default() -> Self {
        Self {
            name: String::new(),
            reference: AssetReference {
                r#type: AssetReferenceType::Isin,
                value: String::new(),
            },
            category_id_to_assignment: HashMap::new(),
        }
    }
}
