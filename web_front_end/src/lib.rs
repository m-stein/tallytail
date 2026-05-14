use std::sync::mpsc;

use core_lib::{AllocationRecord, User};
use eyre::Result;
use serde::Serialize;
use ui_lib::{EframeApp, GetLatestRecordRx, ListUsersResult, NoResult};
use wasm_bindgen::JsCast;

#[derive(Serialize)]
struct AddUserRequest {
    name: String,
}

const SERVER_URL: &str = "http://127.0.0.1:3000/users";
const GET_LATEST_RECORD_URL: &str = "http://127.0.0.1:3000/get_latest_record";

async fn fetch_users() -> Result<Vec<User>> {
    Ok(reqwest::get(SERVER_URL).await?.json::<Vec<User>>().await?)
}

async fn get_latest_record() -> Result<Option<AllocationRecord>> {
    Ok(reqwest::get(GET_LATEST_RECORD_URL)
        .await?
        .json::<Option<AllocationRecord>>()
        .await?)
}

fn start_list_users() -> mpsc::Receiver<ListUsersResult> {
    let (sender, receiver) = mpsc::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_users().await;
        let _ = sender.send(result);
    });
    receiver
}

fn start_get_latest_record() -> GetLatestRecordRx {
    let (tx, rx) = mpsc::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let result = get_latest_record().await;
        let _ = tx.send(result);
    });
    rx
}

async fn add_user(name: String) -> Result<()> {
    reqwest::Client::new()
        .post(SERVER_URL)
        .json(&AddUserRequest { name })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

fn start_add_user(name: String) -> mpsc::Receiver<NoResult> {
    let (sender, receiver) = mpsc::channel();

    wasm_bindgen_futures::spawn_local(async move {
        let result = add_user(name).await;
        let _ = sender.send(result);
    });

    receiver
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("the_canvas_id"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("Canvas not found"))?;

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|_cc| {
                Ok(Box::new(EframeApp::new(
                    start_get_latest_record,
                    start_list_users,
                    start_add_user,
                )))
            }),
        )
        .await
}
