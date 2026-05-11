use std::{fs, sync::mpsc, thread};

use ui_lib::{UserApp, UserResult};
use eyre::Result;
use core_lib::{Data, User};

const DATA_PATH: &str = "../data/data.ron";

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "RON egui Desktop POC",
        native_options,
        Box::new(|_cc| Ok(Box::new(UserApp::new(start_loading_users)))),
    )?;

    Ok(())
}

fn start_loading_users() -> mpsc::Receiver<UserResult> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = read_users();
        let _ = sender.send(result);
    });

    receiver
}

fn read_users() -> Result<Vec<User>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let data: Data = ron::from_str(&text)?;
    Ok(data.users)
}