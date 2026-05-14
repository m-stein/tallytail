use std::sync::mpsc;

use ui_lib::{UserApp, UserResult, UnitResult};
use eyre::Result;
use core_lib::User;
use wasm_bindgen::JsCast;
use serde::Serialize;

#[derive(Serialize)]
struct AddUserRequest {
    name: String,
}

const SERVER_URL: &str = "http://127.0.0.1:3000/users";

async fn fetch_users() -> Result<Vec<User>> {
    let users = reqwest::get(SERVER_URL).await?.json::<Vec<User>>().await?;
    Ok(users)
}

fn start_loading_users() -> mpsc::Receiver<UserResult> {
    let (sender, receiver) = mpsc::channel();

    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_users().await;
        let _ = sender.send(result);
    });

    receiver
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

fn start_adding_user(name: String) -> mpsc::Receiver<UnitResult> {
    let (sender, receiver) = mpsc::channel();

    wasm_bindgen_futures::spawn_local(async move {
        let result = add_user(name).await;
        let _ = sender.send(result);
    });

    receiver
}

#[cfg(target_arch = "wasm32")]
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
            Box::new(|_cc| Ok(Box::new(UserApp::new(start_loading_users, start_adding_user)))),
        )
        .await
}