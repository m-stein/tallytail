use std::sync::mpsc::Receiver;

use eframe::egui;
use eyre::Result;
use core_lib::User;

pub type UserResult = Result<Vec<User>>;
pub type UnitResult = Result<()>;

#[derive(Default)]
pub struct UserApp {
    users: Vec<User>,
    error: Option<String>,
    loading: bool,
    receiver: Option<Receiver<UserResult>>,
    start_loading_users: Option<Box<dyn Fn() -> Receiver<UserResult>>>,
    
    new_user_name: String,
    add_receiver: Option<Receiver<UnitResult>>,
    start_adding_user: Option<Box<dyn Fn(String) -> Receiver<UnitResult>>>,
}

impl UserApp {
    pub fn new(
        start_loading_users: impl Fn() -> Receiver<UserResult> + 'static,
        start_adding_user: impl Fn(String) -> Receiver<UnitResult> + 'static,
    ) -> Self {
        Self {
            start_loading_users: Some(Box::new(start_loading_users)),
            start_adding_user: Some(Box::new(start_adding_user)),
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

    fn poll_add_receiver(&mut self) {
        if let Some(receiver) = &self.add_receiver {
            if let Ok(result) = receiver.try_recv() {
                if let Err(error) = result {
                    self.error = Some(error.to_string());
                }

                self.add_receiver = None;
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
        self.poll_add_receiver();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_user_name);

            if ui.button("Add user").clicked() {
                let name = self.new_user_name.trim().to_owned();

                if !name.is_empty() {
                    if let Some(start_adding_user) = &self.start_adding_user {
                        self.add_receiver = Some(start_adding_user(name));
                        self.new_user_name.clear();
                    }
                }
            }
        });

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