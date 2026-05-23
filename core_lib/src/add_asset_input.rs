use std::collections::HashMap;

use crate::{AssetReference, AssetReferenceType};

#[derive(Default)]
pub struct CategoryAssignmentInput {
    pub value_id: Option<i64>,
    pub percentage: f64,
}

pub struct AddAssetInput {
    pub name: String,
    pub reference: AssetReference,
    pub catgy_id_to_assignm_inputs: HashMap<i64, Vec<CategoryAssignmentInput>>,
}

impl Default for AddAssetInput {
    fn default() -> Self {
        Self {
            name: String::new(),
            reference: AssetReference {
                r#type: AssetReferenceType::Isin,
                value: String::new(),
            },
            catgy_id_to_assignm_inputs: HashMap::new(),
        }
    }
}
