#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CategoryAssignment {
    pub value_id: i64,
    pub ratio: f64,
}