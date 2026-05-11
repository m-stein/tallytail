use std::sync::mpsc::{self, Receiver};

use wasm_bindgen::JsCast;
use eframe::egui;
use eyre::Result;
use shared::User;

const SERVER_URL: &str = "http://127.0.0.1:3000/users";

#[derive(Default)]
pub struct ClientApp {
    users: Vec<User>,
    error: Option<String>,
    loading: bool,
    receiver: Option<Receiver<Result<Vec<User>>>>,
}

impl eframe::App for ClientApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(receiver) = &self.receiver {
            if let Ok(result) = receiver.try_recv() {
                self.loading = false;

                match result {
                    Ok(users) => {
                        self.users = users;
                        self.error = None;
                    }
                    Err(error) => {
                        self.error = Some(error.to_string());
                    }
                }

                self.receiver = None;
            }
        }
        if ui.button("List users").clicked() {
            self.loading = true;
            self.error = None;

            let (sender, receiver) = mpsc::channel();
            self.receiver = Some(receiver);

            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_users().await;
                let _ = sender.send(result);
            });
        }

        if self.loading {
            ui.label("Loading...");
        }

        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.separator();

        for user in &self.users {
            ui.label(&user.name);
        }
    }
}

async fn fetch_users() -> Result<Vec<User>> {
    let users = reqwest::get(SERVER_URL).await?.json::<Vec<User>>().await?;
    Ok(users)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("the_canvas_id"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("Canvas not found"))?;

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|_cc| Ok(Box::new(ClientApp::default()))),
        )
        .await
}