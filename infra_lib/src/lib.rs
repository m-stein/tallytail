use std::fs;

use eyre::Result;
use core_lib::{Data, User};

const DATA_PATH: &str = "../data/data.ron";

pub fn read_users() -> Result<Vec<User>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let data: Data = ron::from_str(&text)?;
    Ok(data.users)
}