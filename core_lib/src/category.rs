use crate::category_value::CategoryValue;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub values: Vec<CategoryValue>,
}
