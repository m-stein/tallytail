use std::fs;

use core_lib::{Data, User};

const DATA_PATH: &str = "../data/data.ron";

pub fn list_users() -> eyre::Result<Vec<User>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let data: Data = ron::from_str(&text)?;
    Ok(data.users)
}

pub fn add_user(name: String) -> eyre::Result<()> {
    let text = fs::read_to_string(DATA_PATH)?;
    let mut data: Data = ron::from_str(&text)?;

    data.users.push(User { name });
    let text = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())?;
    fs::write(DATA_PATH, text)?;
    Ok(())
}
