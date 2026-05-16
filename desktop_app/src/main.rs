use std::{sync::mpsc, thread};

use eyre::eyre;

use ui_lib::{
    EframeApp, GetAllocDiagramDataRx, GetCategoriesResult, GetLatestRecordRx, ListUsersResult,
    NoResult,
};

fn main() -> eyre::Result<()> {
    eframe::run_native(
        "Asset Allocation Tracker",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(EframeApp::new(
                start_get_alloc_diagram_data,
                start_get_latest_record,
                start_list_users,
                start_get_categories,
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

fn start_get_categories() -> mpsc::Receiver<GetCategoriesResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::get_categories();
        let _ = tx.send(result);
    });
    rx
}

fn start_get_latest_record() -> GetLatestRecordRx {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::get_latest_record();
        let _ = tx.send(result);
    });
    rx
}

fn start_get_alloc_diagram_data(category_id: i64, days: i64) -> GetAllocDiagramDataRx {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = infra_lib::get_alloc_diagram_data(category_id, days);
        let _ = tx.send(result);
    });
    rx
}
