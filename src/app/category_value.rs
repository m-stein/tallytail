#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryValue {
    pub id: i64,
    pub name: String,
}