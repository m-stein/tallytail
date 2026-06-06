use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct CategoryValueInput {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct NewCategoryInput {
    pub name: String,
    pub new_value_inputs: Vec<CategoryValueInput>,
}

#[derive(Debug, Clone, Default)]
pub struct AdaptCategoryInput {
    pub new_value_inputs: Vec<CategoryValueInput>,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigureCatgoriesInput {
    pub new_category_inputs: Vec<NewCategoryInput>,
    pub category_id_to_adapt_input: HashMap<i64, AdaptCategoryInput>,
}
