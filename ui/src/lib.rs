use std::sync::mpsc::Receiver;

use eframe::egui;
use eyre::Result;
use shared::User;

pub type UserResult = Result<Vec<User>>;

#[derive(Default)]
pub struct UserApp {
    users: Vec<User>,
    error: Option<String>,
    loading: bool,
    receiver: Option<Receiver<UserResult>>,
    start_loading_users: Option<Box<dyn Fn() -> Receiver<UserResult>>>,
}

impl UserApp {
    pub fn new(start_loading_users: impl Fn() -> Receiver<UserResult> + 'static) -> Self {
        Self {
            start_loading_users: Some(Box::new(start_loading_users)),
            ..Default::default()
        }
    }

    fn poll_receiver(&mut self) {
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
    }

    fn start_loading(&mut self) {
        if let Some(start_loading_users) = &self.start_loading_users {
            self.loading = true;
            self.error = None;
            self.receiver = Some(start_loading_users());
        }
    }
}

impl eframe::App for UserApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_receiver();

        if ui.button("List users").clicked() {
            self.start_loading();
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