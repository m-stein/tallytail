use std::fs;

use eyre::Result;
use core_lib::{Data, User};

const DATA_PATH: &str = "../data/data.ron";

pub fn read_users() -> Result<Vec<User>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let data: Data = ron::from_str(&text)?;
    Ok(data.users)
}

pub fn add_user(name: String) -> Result<()> {
    let text = fs::read_to_string(DATA_PATH)?;
    let mut data: Data = ron::from_str(&text)?;

    data.users.push(User { name });

    let text = ron::ser::to_string_pretty(
        &data,
        ron::ser::PrettyConfig::default(),
    )?;

    fs::write(DATA_PATH, text)?;

    Ok(())
}