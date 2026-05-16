use crate::AllocationCategoryValue;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub values: Vec<AllocationCategoryValue>,
}
