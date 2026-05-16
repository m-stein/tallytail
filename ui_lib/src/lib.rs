mod percent_stacked_bar_chart;

use std::sync::mpsc::Receiver;

use core_lib::{
    AllocationRecord, User, allocation_diagram_data::AllocationDiagramData, category::Category,
};
use eframe::egui;
use eyre::Result;

use crate::percent_stacked_bar_chart::draw_percent_stacked_bar_chart;

pub type ListUsersResult = Result<Vec<User>>;
pub type GetCategoriesResult = Result<Vec<Category>>;
pub type GetCategoriesRx = Receiver<GetCategoriesResult>;
pub type NoResult = Result<()>;
pub type GetLatestRecordRx = Receiver<Result<Option<AllocationRecord>>>;
pub type GetAllocDiagramDataRx = Receiver<Result<AllocationDiagramData>>;

#[derive(Default)]
pub struct EframeApp {
    message: Option<String>,
    pending_req_cnt: usize,

    alloc_diagram_category_id: Option<i64>,
    alloc_diagram_data: Option<AllocationDiagramData>,
    latest_record: Option<AllocationRecord>,
    listed_users: Vec<User>,
    listed_categories: Vec<Category>,
    new_user_name: String,

    list_users_rx: Option<Receiver<ListUsersResult>>,
    start_list_users_fn: Option<Box<dyn Fn() -> Receiver<ListUsersResult>>>,

    add_user_rx: Option<Receiver<NoResult>>,
    start_add_user_fn: Option<Box<dyn Fn(String) -> Receiver<NoResult>>>,

    get_latest_record_rx: Option<GetLatestRecordRx>,
    start_get_latest_record_fn: Option<Box<dyn Fn() -> GetLatestRecordRx>>,

    get_alloc_diagram_data_rx: Option<GetAllocDiagramDataRx>,
    start_get_alloc_diagram_data_fn: Option<Box<dyn Fn(i64, i64) -> GetAllocDiagramDataRx>>,

    get_categories_rx: Option<GetCategoriesRx>,
    start_get_categories_fn: Option<Box<dyn Fn() -> GetCategoriesRx>>,
}

impl EframeApp {
    const SPACE_2: f32 = 12.0;
    const H2_SIZE: f32 = 24.0;

    pub fn new(
        start_get_alloc_diagram_data: impl Fn(i64, i64) -> GetAllocDiagramDataRx + 'static,
        start_get_latest_record: impl Fn() -> GetLatestRecordRx + 'static,
        start_list_users: impl Fn() -> Receiver<ListUsersResult> + 'static,
        start_get_categories: impl Fn() -> Receiver<GetCategoriesResult> + 'static,
        start_add_user: impl Fn(String) -> Receiver<NoResult> + 'static,
    ) -> Self {
        let mut app = Self {
            start_list_users_fn: Some(Box::new(start_list_users)),
            start_get_categories_fn: Some(Box::new(start_get_categories)),
            start_add_user_fn: Some(Box::new(start_add_user)),
            start_get_latest_record_fn: Some(Box::new(start_get_latest_record)),
            start_get_alloc_diagram_data_fn: Some(Box::new(start_get_alloc_diagram_data)),
            ..Default::default()
        };
        app.start_get_categories();
        app.start_get_latest_record();
        app
    }

    fn decr_pending_req_cnt(&mut self) {
        if self.pending_req_cnt > 0 {
            self.pending_req_cnt -= 1;
        } else {
            self.message = Some("Failed to decrease pending request counter".to_string());
        }
    }

    fn incr_pending_req_cnt(&mut self) {
        self.pending_req_cnt += 1;
    }

    fn poll_list_users_rx(&mut self) {
        if let Some(rx) = &self.list_users_rx
            && let Ok(result) = rx.try_recv()
        {
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
            self.decr_pending_req_cnt();
        }
    }

    fn poll_get_categories_rx(&mut self) {
        if let Some(rx) = &self.get_categories_rx
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(categories) => {
                    self.listed_categories = categories;
                    self.message = None;
                }
                Err(error) => {
                    self.message = Some(error.to_string());
                }
            }
            self.get_categories_rx = None;
            self.decr_pending_req_cnt();
        }
    }

    fn poll_get_latest_record_rx(&mut self) {
        if let Some(rx) = &self.get_latest_record_rx
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(latest_record) => {
                    self.latest_record = latest_record;
                    self.message = None;
                }
                Err(error) => {
                    self.latest_record = None;
                    self.message = Some(error.to_string());
                }
            }
            self.get_latest_record_rx = None;
            self.decr_pending_req_cnt();
        }
    }

    fn poll_get_alloc_diagram_data_rx(&mut self) {
        if let Some(rx) = &self.get_alloc_diagram_data_rx
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(alloc_diagram_data) => {
                    self.alloc_diagram_data = Some(alloc_diagram_data);
                    self.message = None;
                }
                Err(error) => {
                    self.alloc_diagram_data = None;
                    self.message = Some(error.to_string());
                }
            }
            self.get_latest_record_rx = None;
            self.decr_pending_req_cnt();
        }
    }

    fn poll_add_user_rx(&mut self) {
        if let Some(rx) = &self.add_user_rx
            && let Ok(result) = rx.try_recv()
        {
            if let Err(error) = result {
                self.message = Some(error.to_string());
            }
            self.add_user_rx = None;
            self.decr_pending_req_cnt();
        }
    }

    fn start_list_users(&mut self) {
        if let Some(start_fn) = &self.start_list_users_fn {
            self.message = None;
            self.list_users_rx = Some(start_fn());
            self.incr_pending_req_cnt();
        }
    }

    fn start_get_categories(&mut self) {
        if let Some(start_fn) = &self.start_get_categories_fn {
            self.message = None;
            self.get_categories_rx = Some(start_fn());
            self.incr_pending_req_cnt();
        }
    }

    fn start_add_user(&mut self) {
        let name = self.new_user_name.trim().to_owned();
        if !name.is_empty()
            && let Some(start_fn) = &self.start_add_user_fn
        {
            self.add_user_rx = Some(start_fn(name));
            self.new_user_name.clear();
            self.incr_pending_req_cnt();
        }
    }

    fn start_get_latest_record(&mut self) {
        if let Some(start_fn) = &self.start_get_latest_record_fn {
            self.message = None;
            self.get_latest_record_rx = Some(start_fn());
            self.incr_pending_req_cnt();
        }
    }

    fn start_get_alloc_diagram_data(&mut self) {
        if let Some(category_id) = self.alloc_diagram_category_id
            && let Some(start_fn) = &self.start_get_alloc_diagram_data_fn
        {
            self.message = None;
            self.get_alloc_diagram_data_rx = Some(start_fn(category_id, 5));
            self.incr_pending_req_cnt();
        } else {
            self.alloc_diagram_data = None;
        }
    }

    fn allocation_diagram_category_selected_text(&self) -> &str {
        match self.alloc_diagram_category_id {
            Some(category_id) => self
                .listed_categories
                .iter()
                .find(|category| category.id == category_id)
                .map(|category| category.name.as_str())
                .unwrap_or("Position"),
            None => "Position",
        }
    }

    fn show_allocation_diagram_page(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Allocation Diagram")
                .heading()
                .size(Self::H2_SIZE),
        );
        ui.add_space(Self::SPACE_2);

        let prev_category_id = self.alloc_diagram_category_id;
        egui::ComboBox::from_id_salt("allocation_diagram_category")
            .selected_text(self.allocation_diagram_category_selected_text())
            .show_ui(ui, |ui| {
                for category in &self.listed_categories {
                    ui.selectable_value(
                        &mut self.alloc_diagram_category_id,
                        Some(category.id),
                        &category.name,
                    );
                }
                ui.selectable_value(&mut self.alloc_diagram_category_id, None, "Position");
            });
        ui.add_space(Self::SPACE_2);

        if prev_category_id != self.alloc_diagram_category_id {
            self.start_get_alloc_diagram_data();
            self.start_get_latest_record();
        }
        if let Some(data) = self.alloc_diagram_data.as_ref() {
            draw_percent_stacked_bar_chart(ui, data);
        } else if let Some(record) = &self.latest_record {
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
        self.poll_list_users_rx();
        self.poll_get_categories_rx();
        self.poll_add_user_rx();
        self.poll_get_latest_record_rx();
        self.poll_get_alloc_diagram_data_rx();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_user_name);
            if ui.button("Add user").clicked() {
                self.start_add_user();
            }
        });
        if ui.button("List users").clicked() {
            self.start_list_users();
        }
        if self.pending_req_cnt > 0 {
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
