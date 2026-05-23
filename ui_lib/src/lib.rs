mod percent_stacked_bar_chart;
pub mod png;

use std::sync::mpsc::{self, Receiver};

use core_lib::{
    AllocationRecord, allocation_diagram_data::AllocationDiagramData, category::Category,
};
use eframe::egui;
use eframe::egui::{Context, TextureHandle};
use egui::TextWrapMode;
use eyre::Result;

use crate::percent_stacked_bar_chart::draw_percent_stacked_bar_chart;

pub type GetCategoriesResult = Result<Vec<Category>>;
pub type GetCategoriesRx = Receiver<GetCategoriesResult>;
pub type NoResult = Result<()>;
pub type GetLatestRecordRx = Receiver<Result<Option<AllocationRecord>>>;
pub type GetAllocDiagramDataRx = Receiver<Result<AllocationDiagramData>>;

#[derive(PartialEq)]
enum Page {
    AllocationDiagram,
    AddAsset,
    ConfigureCategories,
    AddAllocationRecord,
}

pub trait AppBackend {
    fn load_png_texture(&self, ctx: &Context, path: &str) -> eyre::Result<TextureHandle>;
    fn start_get_categories(&self) -> mpsc::Receiver<GetCategoriesResult>;
    fn start_get_latest_record(&self) -> GetLatestRecordRx;
    fn start_get_alloc_diagram_data(&self, category_id: i64, days: i64) -> GetAllocDiagramDataRx;
}

#[allow(unused)]
pub struct EframeApp<B: AppBackend> {
    backend: B,
    message: Option<String>,
    pending_req_cnt: usize,

    alloc_diagram_category_id: Option<i64>,
    alloc_diagram_data: Option<AllocationDiagramData>,
    latest_record: Option<AllocationRecord>,
    categories: Vec<Category>,

    get_latest_record_rx: Option<GetLatestRecordRx>,
    get_alloc_diagram_data_rx: Option<GetAllocDiagramDataRx>,
    get_categories_rx: Option<GetCategoriesRx>,

    page: Page,

    squirrel_texture: egui::TextureHandle,
}

impl<BACKEND: AppBackend> EframeApp<BACKEND> {
    const MAX_CONTENT_WIDTH: f32 = 700.;
    const SPACE_2: f32 = 12.0;
    const SPACE_3: f32 = 24.0;
    const H1_SIZE: f32 = 32.0;
    const H2_SIZE: f32 = 24.0;

    pub fn new(
        creat_ctx: &eframe::CreationContext<'_>,
        backend: BACKEND,
    ) -> eyre::Result<Self> {
        let squirrel_texture =
            backend.load_png_texture(&creat_ctx.egui_ctx, "img/squirrel_68x68.png")?;
        let mut app = Self {
            backend,
            squirrel_texture,
            page: Page::AllocationDiagram,
            message: None,
            alloc_diagram_category_id: None,
            alloc_diagram_data: None,
            categories: Vec::new(),
            pending_req_cnt: 0,
            latest_record: None,
            get_alloc_diagram_data_rx: None,
            get_categories_rx: None,
            get_latest_record_rx: None,
        };
        app.start_get_categories();
        app.start_get_latest_record();
        Ok(app)
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

    fn poll_get_categories_rx(&mut self) {
        if let Some(rx) = &self.get_categories_rx
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(categories) => {
                    self.categories = categories;
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

    fn start_get_categories(&mut self) {
        self.message = None;
        self.get_categories_rx = Some(self.backend.start_get_categories());
        self.incr_pending_req_cnt();
    }

    fn start_get_latest_record(&mut self) {
        self.message = None;
        self.get_latest_record_rx = Some(self.backend.start_get_latest_record());
        self.incr_pending_req_cnt();
    }

    fn start_get_alloc_diagram_data(&mut self) {
        if let Some(category_id) = self.alloc_diagram_category_id {
            self.message = None;
            self.get_alloc_diagram_data_rx = Some(self.backend.start_get_alloc_diagram_data(category_id, 5));
            self.incr_pending_req_cnt();
        } else {
            self.alloc_diagram_data = None;
        }
    }

    fn allocation_diagram_category_selected_text(&self) -> &str {
        match self.alloc_diagram_category_id {
            Some(category_id) => self
                .categories
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
                for category in &self.categories {
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

    fn show_page_button(
        &mut self,
        ui: &mut egui::Ui,
        page: Page,
        label: &str,
        init_page_fn: fn(&mut Self) -> eyre::Result<()>,
    ) {
        let response = ui.add_sized(
            [180.0, 20.0],
            egui::Button::selectable(self.page == page, label),
        );
        if response.clicked() {
            match init_page_fn(self) {
                Ok(_) => {
                    self.page = page;
                }
                Err(e) => {
                    self.message = Some(e.to_string());
                }
            }
        }
    }

    fn init_add_allocation_record_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn init_configure_categories_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn init_alocation_diagram_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn init_add_asset_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn show_add_asset_page(&mut self, _ui: &mut egui::Ui) {}

    fn show_configure_categories_page(&mut self, _ui: &mut egui::Ui) {}

    fn show_add_allocation_record_page(&mut self, _ui: &mut egui::Ui) {}

    fn show_content(&mut self, ui: &mut egui::Ui) {
        self.poll_get_categories_rx();
        self.poll_get_latest_record_rx();
        self.poll_get_alloc_diagram_data_rx();

        ui.add_space(Self::SPACE_2);
        ui.horizontal(|ui| {
            ui.image((self.squirrel_texture.id(), egui::vec2(68.0, 68.0)));
            ui.add_space(Self::SPACE_2);
            ui.label(
                egui::RichText::new("Asset Allocation Tracker")
                    .heading()
                    .size(Self::H1_SIZE),
            );
        });
        ui.add_space(Self::SPACE_3);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.show_page_button(
                    ui,
                    Page::AllocationDiagram,
                    "Allocation Diagram",
                    Self::init_alocation_diagram_page,
                );
                self.show_page_button(ui, Page::AddAsset, "Add Asset", Self::init_add_asset_page);
                self.show_page_button(
                    ui,
                    Page::ConfigureCategories,
                    "Configure Categories",
                    Self::init_configure_categories_page,
                );
                self.show_page_button(
                    ui,
                    Page::AddAllocationRecord,
                    "Add Allocation Record",
                    Self::init_add_allocation_record_page,
                );
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                if self.pending_req_cnt > 0 {
                    ui.label("Loading...");
                } else {
                    match self.page {
                        Page::AddAsset => self.show_add_asset_page(ui),
                        Page::AllocationDiagram => self.show_allocation_diagram_page(ui),
                        Page::ConfigureCategories => self.show_configure_categories_page(ui),
                        Page::AddAllocationRecord => self.show_add_allocation_record_page(ui),
                    }
                }
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Message").heading().size(Self::H2_SIZE));
                ui.add_space(Self::SPACE_2);
                if let Some(message) = &self.message {
                    ui.colored_label(egui::Color32::RED, message);
                }
            });
        });
    }
}

impl<B: AppBackend> eframe::App for EframeApp<B> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(Self::MAX_CONTENT_WIDTH);
                        self.show_content(ui);
                    });
                });
        });
    }
}
