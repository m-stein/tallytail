use std::sync::mpsc::Receiver;

use core_lib::User;
use eframe::egui;
use eyre::Result;

pub type ListUsersResult = Result<Vec<User>>;
pub type NoResult = Result<()>;

#[derive(Default)]
pub struct EframeApp {
    error: Option<String>,
    loading: bool,

    listed_users: Vec<User>,
    list_user_recvr: Option<Receiver<ListUsersResult>>,
    snd_req_list_users: Option<Box<dyn Fn() -> Receiver<ListUsersResult>>>,

    add_user_name: String,
    add_user_recvr: Option<Receiver<NoResult>>,
    snd_req_add_user: Option<Box<dyn Fn(String) -> Receiver<NoResult>>>,
}

impl EframeApp {
    pub fn new(
        snd_req_list_users: impl Fn() -> Receiver<ListUsersResult> + 'static,
        snd_req_add_user: impl Fn(String) -> Receiver<NoResult> + 'static,
    ) -> Self {
        Self {
            snd_req_list_users: Some(Box::new(snd_req_list_users)),
            snd_req_add_user: Some(Box::new(snd_req_add_user)),
            ..Default::default()
        }
    }

    fn poll_list_user_recvr(&mut self) {
        if let Some(receiver) = &self.list_user_recvr
            && let Ok(result) = receiver.try_recv()
        {
            self.loading = false;

            match result {
                Ok(users) => {
                    self.listed_users = users;
                    self.error = None;
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                }
            }

            self.list_user_recvr = None;
        }
    }

    fn poll_add_user_recvr(&mut self) {
        if let Some(receiver) = &self.add_user_recvr
            && let Ok(result) = receiver.try_recv()
        {
            self.loading = false;
            if let Err(error) = result {
                self.error = Some(error.to_string());
            }

            self.add_user_recvr = None;
        }
    }

    fn start_list_users(&mut self) {
        if let Some(snd_req_list_users) = &self.snd_req_list_users {
            self.loading = true;
            self.error = None;
            self.list_user_recvr = Some(snd_req_list_users());
        }
    }

    fn start_add_user(&mut self) {
        let name = self.add_user_name.trim().to_owned();
        if !name.is_empty()
            && let Some(snd_req_add_user) = &self.snd_req_add_user
        {
            self.loading = true;
            self.add_user_recvr = Some(snd_req_add_user(name));
            self.add_user_name.clear();
        }
    }
}

impl eframe::App for EframeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_list_user_recvr();
        self.poll_add_user_recvr();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.add_user_name);
            if ui.button("Add user").clicked() {
                self.start_add_user();
            }
        });
        if ui.button("List users").clicked() {
            self.start_list_users();
        }
        if self.loading {
            ui.label("Loading...");
        }
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::RED, error);
        }
        ui.separator();
        for user in &self.listed_users {
            ui.label(&user.name);
        }
    }
}
