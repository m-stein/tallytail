use std::{sync::mpsc, thread};

use eyre::eyre;

use infra_lib::list_users;
use ui_lib::{EframeApp, ListUsersResult, NoResult};

fn main() -> eyre::Result<()> {
    eframe::run_native(
        "Asset Allocation Tracker",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(EframeApp::new(
                snd_req_list_users,
                snd_req_add_user,
            )))
        }),
    )
    .map_err(|e| eyre!(e.to_string()))?;
    Ok(())
}

fn snd_req_add_user(name: String) -> mpsc::Receiver<NoResult> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::add_user(name);
        let _ = sender.send(result);
    });
    receiver
}

fn snd_req_list_users() -> mpsc::Receiver<ListUsersResult> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = list_users();
        let _ = sender.send(result);
    });
    receiver
}
