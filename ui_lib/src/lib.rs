use std::sync::mpsc::Receiver;

use core_lib::{AllocationRecord, User};
use eframe::egui;
use eyre::Result;

pub type ListUsersResult = Result<Vec<User>>;
pub type NoResult = Result<()>;
pub type GetLatestRecordRx = Receiver<Result<Option<AllocationRecord>>>;

#[derive(Default)]
pub struct EframeApp {
    message: Option<String>,
    loading: bool,

    latest_record: Option<AllocationRecord>,
    listed_users: Vec<User>,
    new_user_name: String,

    list_users_rx: Option<Receiver<ListUsersResult>>,
    start_list_users: Option<Box<dyn Fn() -> Receiver<ListUsersResult>>>,

    add_user_rx: Option<Receiver<NoResult>>,
    start_add_user: Option<Box<dyn Fn(String) -> Receiver<NoResult>>>,

    get_latest_record_rx: Option<GetLatestRecordRx>,
    start_get_latest_record: Option<Box<dyn Fn() -> GetLatestRecordRx>>,
}

impl EframeApp {
    const SPACE_2: f32 = 12.0;
    const H2_SIZE: f32 = 24.0;

    pub fn new(
        start_get_latest_record: impl Fn() -> GetLatestRecordRx + 'static,
        start_list_users: impl Fn() -> Receiver<ListUsersResult> + 'static,
        start_add_user: impl Fn(String) -> Receiver<NoResult> + 'static,
    ) -> Self {
        Self {
            start_list_users: Some(Box::new(start_list_users)),
            start_add_user: Some(Box::new(start_add_user)),
            start_get_latest_record: Some(Box::new(start_get_latest_record)),
            ..Default::default()
        }
    }

    fn poll_list_user_rx(&mut self) {
        if let Some(receiver) = &self.list_users_rx
            && let Ok(result) = receiver.try_recv()
        {
            self.loading = false;
            match result {
                Ok(users) => {
                    self.listed_users = users;
                    self.message = None;
                }
                Err(error) => {
                    self.message = Some(error.to_string());
                }
            }
            self.list_users_rx = None;
        }
    }

    fn poll_get_latest_record_rx(&mut self) {
        if let Some(rx) = &self.get_latest_record_rx
            && let Ok(result) = rx.try_recv()
        {
            self.loading = false;
            match result {
                Ok(latest_record) => {
                    self.latest_record = latest_record;
                    self.message = None;
                }
                Err(error) => {
                    self.message = Some(error.to_string());
                }
            }
            self.get_latest_record_rx = None;
        }
    }

    fn poll_add_user_rx(&mut self) {
        if let Some(receiver) = &self.add_user_rx
            && let Ok(result) = receiver.try_recv()
        {
            self.loading = false;
            if let Err(error) = result {
                self.message = Some(error.to_string());
            }

            self.add_user_rx = None;
        }
    }

    fn start_list_users(&mut self) {
        if let Some(snd_req_list_users) = &self.start_list_users {
            self.loading = true;
            self.message = None;
            self.list_users_rx = Some(snd_req_list_users());
        }
    }

    fn start_add_user(&mut self) {
        let name = self.new_user_name.trim().to_owned();
        if !name.is_empty()
            && let Some(snd_req_add_user) = &self.start_add_user
        {
            self.loading = true;
            self.add_user_rx = Some(snd_req_add_user(name));
            self.new_user_name.clear();
        }
    }

    fn start_get_latest_record(&mut self) {
        if let Some(tx) = &self.start_get_latest_record {
            self.loading = true;
            self.message = None;
            self.get_latest_record_rx = Some(tx());
        }
    }

    fn show_allocation_diagram_page(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Allocation Diagram")
                .heading()
                .size(Self::H2_SIZE),
        );
        ui.add_space(Self::SPACE_2);

        // FIXME: Show category selection

        // FIXME: Consider showing diagram for specific category

        if let Some(record) = &self.latest_record {
            let total: f64 = record.positions.iter().map(|p| p.amount).sum();

            if total <= 0. {
                ui.label("The latest allocation record contains no positive positions.");
                return;
            }

            ui.label(format!("Record from {}:", record.date));
            ui.add_space(10.0);

            for position in &record.positions {
                let percentage = (position.amount / total) * 100.0;
                let fraction = position.amount as f32 / total as f32;

                ui.label(format!(
                    "{} - {} ({:.1}%)",
                    position.asset.name, position.amount, percentage
                ));

                ui.add(
                    egui::ProgressBar::new(fraction)
                        .desired_width(320.0)
                        .text(format!("{:.1}%", percentage)),
                );

                ui.add_space(6.0);
            }
        }
    }
}

impl eframe::App for EframeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_list_user_rx();
        self.poll_add_user_rx();
        self.poll_get_latest_record_rx();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_user_name);
            if ui.button("Add user").clicked() {
                self.start_add_user();
            }
        });
        if ui.button("List users").clicked() {
            self.start_list_users();
        }
        if ui.button("Show allocation").clicked() {
            self.start_get_latest_record();
        }
        if self.loading {
            ui.label("Loading...");
        } else {
            if let Some(error) = &self.message {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                for user in &self.listed_users {
                    ui.label(&user.name);
                }
                self.show_allocation_diagram_page(ui);
            }
        }
    }
}
