use crate::app_backend::{
    AddAssetRx, AppBackend, GetAllocDiagramDataRx, GetAssetsRx, GetCategoriesRx, GetLatestRecordRx,
};
use crate::percent_stacked_bar_chart::draw_percent_stacked_bar_chart;
use crate::png::load_png_texture_from_bytes;
use core_lib::{
    AddAssetArgs, AllocationDiagramData, AllocationPositionInput, AllocationRecord,
    AssetReferenceType, Category, CategoryAssignmentPc, GetAllocDiagramDataArgs,
};
use eframe::egui;
use egui::TextWrapMode;
use egui_extras::DatePickerButton;
use jiff::{Zoned, civil::Date};
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(PartialEq)]
enum Page {
    AllocationDiagram,
    AddAsset,
    ConfigureCategories,
    AddAllocationRecord,
}

pub struct PositionItem {
    pub asset_id: i64,
    pub label: String,
    pub amount: String,
}

#[allow(unused)]
pub struct EframeApp<B: AppBackend> {
    backend: B,
    message: Option<String>,
    pending_req_cnt: usize,

    asset_name_by_id: HashMap<i64, String>,
    allocation_record_date: Date,
    allocation_record_assets: Vec<PositionItem>,
    alloc_diagram_category_id: Option<i64>,
    alloc_diagram_data: Option<AllocationDiagramData>,
    latest_record: Option<AllocationRecord>,
    categories: Vec<Category>,
    add_asset_args: AddAssetArgs,

    get_latest_record_rx: Option<GetLatestRecordRx>,
    get_alloc_diagram_data_rx: Option<GetAllocDiagramDataRx>,
    get_categories_rx: Option<GetCategoriesRx>,
    get_assets_rx: Option<GetAssetsRx>,
    add_asset_rx: Option<AddAssetRx>,

    page: Page,

    squirrel_texture: egui::TextureHandle,
}

impl<BACKEND: AppBackend> EframeApp<BACKEND> {
    const MAX_CONTENT_WIDTH: f32 = 700.;
    const H1_SIZE: f32 = 32.0;
    const H2_SIZE: f32 = 24.0;
    const SPACE_1: f32 = 8.0;
    const SPACE_2: f32 = 12.0;
    const SPACE_3: f32 = 24.0;
    const DEFAULT_INPUT_HEIGHT: f32 = 19.0;
    const SYM_BTN_SIZE: f32 = Self::DEFAULT_INPUT_HEIGHT;

    pub fn new(creat_ctx: &eframe::CreationContext<'_>, backend: BACKEND) -> eyre::Result<Self> {
        let squirrel_path = "img/squirrel_68x68.png";
        let squirrel_texture = load_png_texture_from_bytes(
            &creat_ctx.egui_ctx,
            squirrel_path,
            backend.load_png_file(squirrel_path)?,
        )?;
        let mut app = Self {
            backend,
            squirrel_texture,
            page: Page::AllocationDiagram,
            allocation_record_date: Zoned::now().date(),
            allocation_record_assets: Vec::new(),
            message: None,
            alloc_diagram_category_id: None,
            alloc_diagram_data: None,
            categories: Vec::new(),
            asset_name_by_id: HashMap::new(),
            pending_req_cnt: 0,
            latest_record: None,
            add_asset_args: AddAssetArgs::default(),
            get_alloc_diagram_data_rx: None,
            get_categories_rx: None,
            get_assets_rx: None,
            get_latest_record_rx: None,
            add_asset_rx: None,
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

    fn poll_get_assets_rx(&mut self) {
        if let Some(rx) = &self.get_assets_rx
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(assets) => {
                    self.asset_name_by_id.clear();
                    self.allocation_record_assets.clear();

                    for asset in assets {
                        self.asset_name_by_id.insert(asset.id, asset.name.clone());

                        self.allocation_record_assets.push(PositionItem {
                            asset_id: asset.id,
                            label: format!("{} ({})", asset.name, asset.reference.value),
                            amount: String::new(),
                        });
                    }
                    self.message = None;
                }
                Err(error) => {
                    self.message = Some(error.to_string());
                }
            }
            self.get_assets_rx = None;
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

    fn poll_add_asset_rx(&mut self) {
        if let Some(rx) = &self.add_asset_rx
            && let Ok(result) = rx.try_recv()
        {
            if let Err(error) = result {
                self.message = Some(error.to_string());
            }
            self.add_asset_rx = None;
            self.decr_pending_req_cnt();
        }
    }

    fn start_get_categories(&mut self) {
        self.message = None;
        self.get_categories_rx = Some(self.backend.start_get_categories());
        self.incr_pending_req_cnt();
    }

    fn start_get_assets(&mut self) {
        self.message = None;
        self.get_assets_rx = Some(self.backend.start_get_assets());
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
            self.get_alloc_diagram_data_rx = Some(self.backend.start_get_alloc_diagram_data(
                GetAllocDiagramDataArgs {
                    category_id,
                    days: 5,
                },
            ));
            self.incr_pending_req_cnt();
        } else {
            self.alloc_diagram_data = None;
        }
    }

    fn start_add_asset(&mut self) {
        self.add_asset_rx = Some(self.backend.start_add_asset(self.add_asset_args.clone()));
        self.reset_add_asset_page();
        self.incr_pending_req_cnt();
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

    fn reset_add_allocation_record_page(&mut self) {
        self.allocation_record_date = Zoned::now().date();
        for asset in &mut self.allocation_record_assets {
            asset.amount.clear();
        }
    }

    fn init_add_allocation_record_page(&mut self) -> eyre::Result<()> {
        self.reset_add_allocation_record_page();
        self.start_get_assets();
        self.message = None;
        Ok(())
    }

    fn init_configure_categories_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn init_alocation_diagram_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn reset_add_asset_page(&mut self) {
        self.add_asset_args = AddAssetArgs::default();
    }

    fn init_add_asset_page(&mut self) -> eyre::Result<()> {
        self.reset_add_asset_page();
        self.start_get_categories();
        self.message = None;
        Ok(())
    }

    fn show_add_asset_page(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Add Asset")
                .heading()
                .size(Self::H2_SIZE),
        );
        ui.add_space(Self::SPACE_2);

        ui.label("Name:");
        ui.text_edit_singleline(&mut self.add_asset_args.name);
        ui.add_space(Self::SPACE_2);

        ui.label("Reference type:");
        egui::ComboBox::from_id_salt("reference_type")
            .selected_text(self.add_asset_args.reference.r#type.to_string())
            .show_ui(ui, |ui| {
                for t in AssetReferenceType::iter() {
                    ui.selectable_value(
                        &mut self.add_asset_args.reference.r#type,
                        t,
                        t.to_string(),
                    );
                }
            });
        ui.add_space(Self::SPACE_2);

        ui.label("Reference value:");
        ui.text_edit_singleline(&mut self.add_asset_args.reference.value);
        ui.vertical(|ui| {
            for catgy in &mut self.categories {
                let assignments = self
                    .add_asset_args
                    .category_id_to_assignment
                    .entry(catgy.id)
                    .or_default();

                ui.add_space(Self::SPACE_2);
                ui.horizontal(|ui| {
                    if assignments.len() < catgy.values.len()
                        && ui
                            .add_sized(
                                [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                                egui::Button::new("+"),
                            )
                            .clicked()
                    {
                        assignments.push(CategoryAssignmentPc {
                            percentage: if assignments.is_empty() { 100. } else { 0. },
                            value_id: None,
                        });
                    }
                    ui.label(format!(" {}:", &catgy.name));
                });
                ui.add_space(Self::SPACE_1);

                let mut del_assignm_idx: Option<usize> = None;
                for assignm_idx in (0..assignments.len()).rev() {
                    let assignment = &mut assignments[assignm_idx];
                    let selected_text = assignment
                        .value_id
                        .and_then(|id| catgy.values.iter().find(|val| val.id == id))
                        .map(|val| val.name.clone())
                        .unwrap_or_else(|| "Select...".to_string());

                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [70.0, Self::DEFAULT_INPUT_HEIGHT],
                            egui::DragValue::new(&mut assignment.percentage)
                                .range(0.0..=100.0)
                                .speed(0.1)
                                .fixed_decimals(2)
                                .suffix("%"),
                        );
                        egui::ComboBox::from_id_salt(format!("{}:{}", catgy.id, assignm_idx))
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                for value in catgy.values.iter() {
                                    ui.selectable_value(
                                        &mut assignment.value_id,
                                        Some(value.id),
                                        &value.name,
                                    );
                                }
                            });
                        if ui
                            .add_sized(
                                [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                                egui::Button::new("-"),
                            )
                            .clicked()
                        {
                            del_assignm_idx = Some(assignm_idx);
                        }
                    });
                }
                if let Some(idx) = del_assignm_idx {
                    assignments.remove(idx);
                }
            }
        });
        ui.add_space(Self::SPACE_2);
        if ui.button("Save").clicked() {
            self.start_add_asset();
        }
    }

    fn show_configure_categories_page(&mut self, _ui: &mut egui::Ui) {}

    fn show_add_allocation_record_page(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Add Allocation Record")
                .heading()
                .size(Self::H2_SIZE),
        );
        ui.add_space(Self::SPACE_2);

        ui.label("Date:");
        ui.add(DatePickerButton::new(&mut self.allocation_record_date));

        ui.add_space(Self::SPACE_2);
        ui.label("Positions:");

        ui.vertical(|ui| {
            for asset in &mut self.allocation_record_assets {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut asset.amount).desired_width(80.0));
                    ui.label(&asset.label);
                });
            }
        });

        ui.add_space(Self::SPACE_2);
        if ui.button("Save").clicked() {
            let mut positions = Vec::new();
            let mut validation_error = None;

            for asset in &self.allocation_record_assets {
                let trimmed = asset.amount.trim();

                if trimmed.is_empty() {
                    continue;
                }

                let amount = match trimmed.parse::<f64>() {
                    Ok(value) => value,
                    Err(_) => {
                        validation_error =
                            Some(format!("Invalid amount for asset '{}'", asset.label));
                        break;
                    }
                };

                if amount <= 0. {
                    validation_error = Some(format!(
                        "Amount must be greater than 0 for asset '{}'",
                        asset.label
                    ));
                    break;
                }

                positions.push(AllocationPositionInput {
                    asset_id: asset.asset_id,
                    amount,
                });
            }

            if let Some(message) = validation_error {
                self.message = Some(message);
            } else {
                /*
                match self.asset_service.add_allocation_record(
                    self.allocation_record_date,
                    positions,
                ) {
                    Ok(()) => {
                        self.message = Some(format!(
                            "Allocation record '{}' was saved.",
                            self.allocation_record_date.to_string()
                        ));
                        self.reset_add_allocation_record_page();
                    }
                    Err(err) => {
                        self.message = Some(err.to_string());
                    }
                }
                */
            }
        }
    }

    fn show_content(&mut self, ui: &mut egui::Ui) {
        self.poll_get_categories_rx();
        self.poll_get_assets_rx();
        self.poll_get_latest_record_rx();
        self.poll_get_alloc_diagram_data_rx();
        self.poll_add_asset_rx();

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
