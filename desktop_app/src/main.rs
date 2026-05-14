use std::{sync::mpsc, thread};

use eyre::Result;

use infra_lib::read_users;
use ui_lib::{UserApp, UserResult, UnitResult};

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "RON egui Desktop POC",
        native_options,
        Box::new(|_cc| Ok(Box::new(UserApp::new(start_loading_users, start_adding_user)))),
    )?;

    Ok(())
}

fn start_adding_user(name: String) -> mpsc::Receiver<UnitResult> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = infra_lib::add_user(name);
        let _ = sender.send(result);
    });

    receiver
}

fn start_loading_users() -> mpsc::Receiver<UserResult> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = read_users();
        let _ = sender.send(result);
    });

    receiver
}