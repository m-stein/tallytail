use crate::app::category_value::CategoryValue;

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub values: Vec<CategoryValue>,
}