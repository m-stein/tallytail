use crate::app_backend::AppBackend;
use crate::percent_stacked_bar_chart::draw_percent_stacked_bar_chart;
use crate::png::load_png_texture_from_bytes;
use core_lib::{
    AddAssetArgs, AllocationDiagramData, AllocationPositionInput, AllocationRecord,
    AssetReferenceType, Category, CategoryAssignmentPc, CategoryValueInput,
    ConfigureCatgoriesInput, GetAllocDiagramDataArgs, LogTransactionInput, NewCategoryInput,
    call_macro_with_request_list,
};
use eframe::egui;
use egui::TextWrapMode;
use egui_extras::DatePickerButton;
use jiff::{Zoned, civil::Date};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use strum::IntoEnumIterator;

macro_rules! define_request_data {
    ($($request:ident($($arg_ty:ty)?) -> $ret_ty:ty;)*) => {
        paste::paste! {
            #[derive(Default)]
            struct RequestData {
                $([<$request _rx>]: Option<Receiver<eyre::Result<$ret_ty>>>,)*
            }
        }
    }
}

call_macro_with_request_list!(define_request_data);

macro_rules! implement_requests {

    // For each request, redirect to one of the @func arms depending on whether
    // the request has an argument or not
    ($($request:ident($($arg_ty:ty)?) -> $ret_ty:ty;)*) => {
        $(
            implement_requests!(@start_req_fn $request ($($arg_ty)?) -> $ret_ty);
            paste::paste! {
                fn [<poll_ $request _rx>](&mut self) -> Option<$ret_ty> {
                    let mut res: Option<$ret_ty> = None;
                    if let Some(rx) = &self.request_data.[<$request _rx>]
                        && let Ok(result) = rx.try_recv()
                    {
                        match result {
                            Ok(result) => {
                                self.message = None;
                                res = Some(result);
                            }
                            Err(error) => {
                                self.message = Some(error.to_string());
                            }
                        }
                        self.request_data.[<$request _rx>] = None;
                        self.decr_pending_req_cnt();
                    }
                    res
                }
            }
        )*
    };
    // Start-request function-template for requests without an argument
    (@start_req_fn $request:ident () -> $ret_ty:ty) => {
        paste::paste! {
            fn [<start_ $request>](&mut self) {
                self.message = None;
                self.request_data.[<$request _rx>] = Some(self.backend.[<start_ $request>]());
                self.incr_pending_req_cnt();
            }
        }
    };
    // Start-request function-template for requests with one argument
    (@start_req_fn $request:ident ($arg_ty:ty) -> $ret_ty:ty) => {
        paste::paste! {
            fn [<start_ $request>](&mut self, arg: $arg_ty) {
                self.message = None;
                self.request_data.[<$request _rx>] = Some(self.backend.[<start_ $request>](arg));
                self.incr_pending_req_cnt();
            }
        }
    };
}

#[derive(PartialEq)]
enum Page {
    AllocationDiagram,
    AddAsset,
    ConfigureCategories,
    AddAllocationRecord,
    LogTransaction,
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
    log_transaction_input: LogTransactionInput,
    cfg_catgs_input: ConfigureCatgoriesInput,
    request_data: RequestData,
    page: Page,
    squirrel_texture: Option<egui::TextureHandle>,
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
    const SQUIRREL_IMG_PATH: &str = "img/squirrel_68x68.png";

    pub fn new(backend: BACKEND) -> eyre::Result<Self> {
        let mut app = Self {
            backend,
            squirrel_texture: None,
            request_data: RequestData::default(),
            page: Page::AllocationDiagram,
            allocation_record_date: Zoned::now().date(),
            allocation_record_assets: Vec::new(),
            message: None,
            alloc_diagram_category_id: None,
            alloc_diagram_data: None,
            categories: Vec::new(),
            asset_name_by_id: HashMap::new(),
            cfg_catgs_input: ConfigureCatgoriesInput::default(),
            pending_req_cnt: 0,
            latest_record: None,
            add_asset_args: AddAssetArgs::default(),
            log_transaction_input: LogTransactionInput::default(),
        };
        app.start_load_png_data(Self::SQUIRREL_IMG_PATH.to_string());
        app.start_get_categories();
        app.start_get_latest_record();
        Ok(app)
    }

    call_macro_with_request_list!(implement_requests);

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
            if let Some(category_id) = self.alloc_diagram_category_id {
                self.start_get_alloc_diagram_data(GetAllocDiagramDataArgs {
                    category_id,
                    days: 5,
                });
            } else {
                self.alloc_diagram_data = None;
            }
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
        self.start_get_categories();
        self.message = None;
        Ok(())
    }

    fn init_alocation_diagram_page(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    fn reset_log_transaction_page(&mut self) {
        self.log_transaction_input = LogTransactionInput::default();
    }

    fn init_log_transaction_page(&mut self) -> eyre::Result<()> {
        self.reset_log_transaction_page();
        self.message = None;
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
            self.start_add_asset(self.add_asset_args.clone());
        }
    }

    fn show_log_transaction_page(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Log Transaction")
                .heading()
                .size(Self::H2_SIZE),
        );
        ui.add_space(Self::SPACE_2);

        ui.label("ISIN:");
        ui.text_edit_singleline(&mut self.log_transaction_input.isin);
        ui.add_space(Self::SPACE_2);

        ui.label("Quantity:");
        ui.text_edit_singleline(&mut self.log_transaction_input.quantity);
        ui.add_space(Self::SPACE_2);

        ui.label("Stock price:");
        ui.text_edit_singleline(&mut self.log_transaction_input.stock_price);
        ui.add_space(Self::SPACE_2);

        ui.label("Order value:");
        ui.text_edit_singleline(&mut self.log_transaction_input.order_value);
        ui.add_space(Self::SPACE_2);

        if ui.button("Save").clicked() {
            self.start_log_transaction(self.log_transaction_input.clone());
        }
    }

    fn show_configure_categories_page(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Configure Categories")
                .heading()
                .size(Self::H2_SIZE),
        );
        ui.add_space(Self::SPACE_2);
        if ui.button("Save").clicked() {
            self.start_configure_categories(self.cfg_catgs_input.clone());
        }
        ui.add_space(Self::SPACE_2);
        let mut focus_next_catg_input = false;
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                    egui::Button::new("+"),
                )
                .clicked()
            {
                self.cfg_catgs_input
                    .new_category_inputs
                    .push(NewCategoryInput::default());
                focus_next_catg_input = true;
            }
            ui.label("Categories:");
        });

        /* Show inputs for new categories */
        let mut del_catg_idx: Option<usize> = None;
        for (catg_idx, catg_input) in self
            .cfg_catgs_input
            .new_category_inputs
            .iter_mut()
            .enumerate()
            .rev()
        {
            ui.add_space(Self::SPACE_2);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("•");
                    let id = ui.make_persistent_id(("NewCatg", catg_idx));
                    let response = ui.add(egui::TextEdit::singleline(&mut catg_input.name).id(id));
                    if focus_next_catg_input {
                        response.request_focus();
                        focus_next_catg_input = false;
                    }
                    if ui
                        .add_sized(
                            [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                            egui::Button::new("-"),
                        )
                        .clicked()
                    {
                        del_catg_idx = Some(catg_idx);
                    }
                });
                ui.horizontal(|ui| {
                    ui.add_space(Self::SPACE_3);
                    ui.vertical(|ui| {
                        let mut focus_next_val_input = false;
                        ui.horizontal(|ui| {
                            if ui
                                .add_sized(
                                    [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                                    egui::Button::new("+"),
                                )
                                .clicked()
                            {
                                catg_input
                                    .new_value_inputs
                                    .push(CategoryValueInput::default());
                                focus_next_val_input = true;
                            }
                            ui.label("Values:");
                        });

                        /* Show inputs for new values */
                        let mut del_val_idx: Option<usize> = None;
                        for (val_idx, val_input) in
                            catg_input.new_value_inputs.iter_mut().enumerate().rev()
                        {
                            ui.horizontal(|ui| {
                                ui.label("•");
                                let id = ui.make_persistent_id(("NewCatgVal", catg_idx, val_idx));
                                let response =
                                    ui.add(egui::TextEdit::singleline(&mut val_input.name).id(id));
                                if focus_next_val_input {
                                    response.request_focus();
                                    focus_next_val_input = false;
                                }
                                if ui
                                    .add_sized(
                                        [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                                        egui::Button::new("-"),
                                    )
                                    .clicked()
                                {
                                    del_val_idx = Some(val_idx);
                                }
                            });
                        }
                        if let Some(idx) = del_val_idx {
                            catg_input.new_value_inputs.remove(idx);
                        }
                    });
                });
            });
        }
        if let Some(idx) = del_catg_idx {
            self.cfg_catgs_input.new_category_inputs.remove(idx);
        }

        /* Show existing categories */
        for catg in &self.categories {
            let catg_input = &mut self
                .cfg_catgs_input
                .category_id_to_adapt_input
                .entry(catg.id)
                .or_default();

            ui.add_space(Self::SPACE_2);
            ui.label(format!("• {}", catg.name));
            ui.horizontal(|ui| {
                ui.add_space(Self::SPACE_3);
                ui.vertical(|ui| {
                    let mut focus_next_val_input = false;
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                                egui::Button::new("+"),
                            )
                            .clicked()
                        {
                            catg_input
                                .new_value_inputs
                                .push(CategoryValueInput::default());
                            focus_next_val_input = true;
                        }
                        ui.label("Values:");
                    });

                    /* Show inputs for new values */
                    let mut del_val_idx: Option<usize> = None;
                    for (val_idx, val_input) in
                        catg_input.new_value_inputs.iter_mut().enumerate().rev()
                    {
                        ui.horizontal(|ui| {
                            ui.label("•");
                            let id = ui.make_persistent_id(("AdaptCatgVal", catg.id, val_idx));
                            let response =
                                ui.add(egui::TextEdit::singleline(&mut val_input.name).id(id));
                            if focus_next_val_input {
                                response.request_focus();
                                focus_next_val_input = false;
                            }
                            if ui
                                .add_sized(
                                    [Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE],
                                    egui::Button::new("-"),
                                )
                                .clicked()
                            {
                                del_val_idx = Some(val_idx);
                            }
                        });
                    }
                    if let Some(idx) = del_val_idx {
                        catg_input.new_value_inputs.remove(idx);
                    }

                    /* Show existing values */
                    for value in &catg.values {
                        ui.label(format!("• {}", value.name));
                    }
                });
            });
        }
    }

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

    fn poll_request_receivers(&mut self, ui: &mut egui::Ui) {
        if let Some(data) = self.poll_load_png_data_rx() {
            match load_png_texture_from_bytes(ui.ctx(), Self::SQUIRREL_IMG_PATH, data) {
                Ok(texture) => {
                    self.squirrel_texture = Some(texture);
                }
                Err(err) => {
                    self.squirrel_texture = None;
                    self.message = Some(err.to_string());
                }
            }
        }
        if let Some(categories) = self.poll_get_categories_rx() {
            self.categories = categories;
        }
        if let Some(assets) = self.poll_get_assets_rx() {
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
        }
        if let Some(record) = self.poll_get_latest_record_rx() {
            self.latest_record = record;
        }
        if let Some(data) = self.poll_get_alloc_diagram_data_rx() {
            self.alloc_diagram_data = Some(data);
        }
        if let Some((categories, err)) = self.poll_configure_categories_rx() {
            if let Some(err) = err {
                self.message = Some(format!("Partial save. First error: {}", err));
            } else {
                self.message = Some("All saved".into());
            }
            self.cfg_catgs_input = categories;
            self.start_get_categories();
        }
        self.poll_add_asset_rx();
        if self.poll_log_transaction_rx().is_some() {
            self.message = Some("Transaction logged".into());
            self.reset_log_transaction_page();
        }
    }

    fn show_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(Self::SPACE_2);
        ui.horizontal(|ui| {
            if let Some(texture) = &self.squirrel_texture {
                ui.image((texture.id(), egui::vec2(68.0, 68.0)));
            }
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
                self.show_page_button(
                    ui,
                    Page::LogTransaction,
                    "Log Transaction",
                    Self::init_log_transaction_page,
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
                        Page::LogTransaction => self.show_log_transaction_page(ui),
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
        self.poll_request_receivers(ui);
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
