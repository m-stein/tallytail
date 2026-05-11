use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub users: Vec<User>,
}