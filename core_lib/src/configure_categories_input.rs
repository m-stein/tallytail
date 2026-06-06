use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CategoryValueInput {
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NewCategoryInput {
    pub name: String,
    pub new_value_inputs: Vec<CategoryValueInput>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdaptCategoryInput {
    pub new_value_inputs: Vec<CategoryValueInput>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigureCatgoriesInput {
    pub new_category_inputs: Vec<NewCategoryInput>,
    pub category_id_to_adapt_input: HashMap<i64, AdaptCategoryInput>,
}
