use std::{sync::mpsc, thread};

use eyre::eyre;

use ui_lib::{EframeApp, GetLatestRecordRx, ListUsersResult, NoResult};

fn main() -> eyre::Result<()> {
    eframe::run_native(
        "Asset Allocation Tracker",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(EframeApp::new(
                start_get_latest_record,
                start_list_users,
                start_add_user,
            )))
        }),
    )
    .map_err(|e| eyre!(e.to_string()))?;
    Ok(())
}

fn start_add_user(name: String) -> mpsc::Receiver<NoResult> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::add_user(name);
        let _ = sender.send(result);
    });
    receiver
}

fn start_list_users() -> mpsc::Receiver<ListUsersResult> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::list_users();
        let _ = sender.send(result);
    });
    receiver
}

fn start_get_latest_record() -> GetLatestRecordRx {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::get_latest_record();
        let _ = tx.send(result);
    });
    rx
}
