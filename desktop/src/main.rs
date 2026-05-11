use std::fs;

use eframe::egui;
use eyre::Result;
use shared::{Data, User};

const DATA_PATH: &str = "../data/data.ron";

#[derive(Default)]
struct DesktopApp {
    users: Vec<User>,
    error: Option<String>,
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.button("List users").clicked() {
            match read_users() {
                Ok(users) => {
                    self.users = users;
                    self.error = None;
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                }
            }
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

fn read_users() -> Result<Vec<User>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let data: Data = ron::from_str(&text)?;
    Ok(data.users)
}

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "RON egui Desktop POC",
        native_options,
        Box::new(|_cc| Ok(Box::new(DesktopApp::default()))),
    )?;

    Ok(())
}