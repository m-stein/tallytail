use std::collections::HashMap;

use eframe::egui;
use egui_extras::DatePickerButton;
use jiff::civil::Date;
use jiff::Zoned;
use strum::IntoEnumIterator;

use crate::app::allocation_record::AllocationRecord;
use crate::app::asset_input::AssetInput;
use crate::app::asset_service::AssetService;
use crate::app::allocation_record_input::AllocationPositionInput;
use crate::app::category::Category;
use crate::app::category_value::CategoryValue;
use crate::app::configure_categories_input::{CategoryValueInput, ConfigureCatgoriesInput, NewCategoryInput};
use crate::app::category_assignment_input::CategoryAssignmentInput;
use crate::app::error::Error;
use crate::app::named_distribution::DatedDistribution;
use crate::app::asset_reference_type::AssetReferenceType;
use crate::ui::desktop_app::distribution_history::draw_distribution_history;

pub struct PositionItem {
    pub id: i64,
    pub label: String,
    pub amount_input: String,
}

#[derive(PartialEq)]
enum Page {
    AllocationDiagram,
    AddAsset,
    ConfigureCategories,
    AddAllocationRecord,
}

pub struct DesktopApp {
    asset_service: AssetService,

    allocation_record_date: Date,
    allocation_record_assets: Vec<PositionItem>,

    latest_allocation_record: Option<AllocationRecord>,
    asset_name_by_id: HashMap<i64, String>,

    cfg_catgs_input: ConfigureCatgoriesInput,

    existing_catgs: Vec<Category>,
    existing_catg_values: HashMap<i64, Vec<CategoryValue>>,

    alloc_diagram_category_id: Option<i64>,
    alloc_diagram_data: Option<Vec<DatedDistribution>>,

    add_asset_asset_input: AssetInput,
    add_asset_catgy_id_to_assignm_input_cnt: HashMap<i64, i64>,
    add_asset_catgy_id_to_assignm_inputs: HashMap<i64, Vec<CategoryAssignmentInput>>,

    message: Option<String>,

    page: Page,
}

impl DesktopApp {
    const H1_SIZE: f32 = 32.0;
    const H2_SIZE: f32 = 24.0;
    const SPACE_1: f32 = 8.0;
    const SPACE_2: f32 = 12.0;
    const SPACE_3: f32 = 24.0;
    const DEFAULT_INPUT_HEIGHT: f32 = 19.0;
    const SYM_BTN_SIZE: f32 = DesktopApp::DEFAULT_INPUT_HEIGHT;

    pub fn new(asset_service: AssetService) -> Self {
        let mut app = Self {
            asset_service,

            allocation_record_date: Zoned::now().date(),
            allocation_record_assets: Vec::new(),

            latest_allocation_record: None,
            asset_name_by_id: HashMap::new(),

            cfg_catgs_input: ConfigureCatgoriesInput::default(),

            existing_catgs: Vec::new(),
            existing_catg_values: HashMap::new(),

            add_asset_asset_input: AssetInput::default(),
            add_asset_catgy_id_to_assignm_input_cnt: HashMap::new(),

            alloc_diagram_category_id: None,
            alloc_diagram_data: None,

            add_asset_catgy_id_to_assignm_inputs: HashMap::new(),

            message: None,

            page: Page::AllocationDiagram,
        };
        if let Err(e) = app.init_alocation_diagram_page() {
            app.message = Some(e.to_string());
        }
        app.reload_latest_allocation_record();
        app.reload_asset_list_for_allocation_record();
        app
    }

    fn reload_latest_allocation_record(&mut self) {
        match self.asset_service.get_latest_allocation_record() {
            Ok(record) => {
                self.latest_allocation_record = record;
                self.message = None;
            }
            Err(err) => {
                self.latest_allocation_record = None;
                self.message = Some(err.to_string());
            }
        }
    }

    fn reload_existing_catgs_and_catg_values(&mut self) -> Result<(), Error> {
        self.existing_catgs = self.asset_service.get_categories()?;
        for catgy in &mut self.existing_catgs {
            self.existing_catg_values.insert(
                catgy.id,
                self.asset_service.get_category_values(catgy.id)?);
        }
        Ok(())
    }

    fn allocation_diagram_category_selected_text(&self) -> &str {
        match self.alloc_diagram_category_id {
            Some(category_id) => self.existing_catgs
                .iter()
                .find(|category| category.id == category_id)
                .map(|category| category.name.as_str())
                .unwrap_or("Position"),
            None => "Position",
        }
    }

    fn reset_add_asset_page(&mut self) {
        self.add_asset_asset_input = AssetInput::default();
        self.add_asset_catgy_id_to_assignm_input_cnt.clear();
    }

    fn reload_asset_list_for_allocation_record(&mut self) {
        match self.asset_service.list_assets() {
            Ok(assets) => {
                self.asset_name_by_id.clear();
                self.allocation_record_assets.clear();

                for asset in assets {
                    self.asset_name_by_id.insert(asset.id, asset.name.clone());

                    self.allocation_record_assets.push(PositionItem {
                        id: asset.id,
                        label: format!("{} ({})", asset.name, asset.reference.value),
                        amount_input: String::new(),
                    });
                }
            }
            Err(err) => {
                self.message = Some(err.to_string());
            }
        }
    }

    fn reset_add_allocation_record_page(&mut self) {
        self.allocation_record_date = Zoned::now().date();
        for asset in &mut self.allocation_record_assets {
            asset.amount_input.clear();
        }
    }

    fn reference_type_label(reference_type: AssetReferenceType) -> &'static str {
        match reference_type {
            AssetReferenceType::Iban => "IBAN",
            AssetReferenceType::Isin => "ISIN",
            AssetReferenceType::Ticker => "Ticker",
        }
    }

    fn init_add_allocation_record_page(&mut self) -> Result<(), Error> {
        self.reload_asset_list_for_allocation_record();
        self.reset_add_allocation_record_page();
        self.message = None;
        Ok(())
    }

    fn show_add_allocation_record_page(&mut self, ui: &mut egui::Ui) {

        ui.label(egui::RichText::new("Add Allocation Record").heading().size(Self::H2_SIZE));
        ui.add_space(Self::SPACE_2);

        ui.label("Date:");
        ui.add(DatePickerButton::new(&mut self.allocation_record_date));

        ui.add_space(Self::SPACE_2);
        ui.label("Positions:");

        ui.vertical(|ui| {
            for asset in &mut self.allocation_record_assets {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut asset.amount_input)
                            .desired_width(80.0),
                    );
                    ui.label(&asset.label);
                });
            }
        });

        ui.add_space(Self::SPACE_2);
        if ui.button("Save").clicked() {
            let mut positions = Vec::new();
            let mut validation_error = None;

            for asset in &self.allocation_record_assets {
                let trimmed = asset.amount_input.trim();

                if trimmed.is_empty() {
                    continue;
                }

                let amount = match trimmed.parse::<f64>() {
                    Ok(value) => value,
                    Err(_) => {
                        validation_error = Some(format!(
                            "Invalid amount for asset '{}'",
                            asset.label
                        ));
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
                    asset_id: asset.id,
                    amount,
                });
            }

            if let Some(message) = validation_error {
                self.message = Some(message);
            } else {
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
            }
        }
    }

    fn reload_alloc_diagram_data(&mut self) {
        if let Some(category_id) = self.alloc_diagram_category_id {
            match self.asset_service.get_distribution_for_category(category_id, 5) {
                Ok(data) => {
                    self.alloc_diagram_data = Some(data)
                }
                Err(err) => {
                    self.alloc_diagram_data = None;
                    self.message = Some(err.to_string());
                }
            }
        } else {
            self.alloc_diagram_data = None;
        }
    }

    fn init_alocation_diagram_page(&mut self) -> Result<(), Error> {
        self.reload_existing_catgs_and_catg_values()?;
        self.reload_alloc_diagram_data();
        self.reload_latest_allocation_record();
        Ok(())
    }

    fn show_allocation_diagram_page(&mut self, ui: &mut egui::Ui) {

        ui.label(egui::RichText::new("Allocation Diagram").heading().size(Self::H2_SIZE));
        ui.add_space(Self::SPACE_2);

        ui.label("Category:");

        let prev_category_id = self.alloc_diagram_category_id;
        egui::ComboBox::from_id_salt("allocation_diagram_category")
            .selected_text(self.allocation_diagram_category_selected_text())
            .show_ui(ui, |ui| {
                for category in &self.existing_catgs {
                    ui.selectable_value(
                        &mut self.alloc_diagram_category_id,
                        Some(category.id),
                        &category.name,
                    );
                }
                ui.selectable_value(
                    &mut self.alloc_diagram_category_id,
                    None,
                    "Position",
                );
            });
        ui.add_space(Self::SPACE_2);

        if prev_category_id != self.alloc_diagram_category_id {
            self.reload_alloc_diagram_data();
            self.reload_latest_allocation_record();
        }
        if let Some(distr_history) = self.alloc_diagram_data.as_ref() {
            draw_distribution_history(ui, self.allocation_diagram_category_selected_text(), distr_history);
        } else if let Some(record) = &self.latest_allocation_record {
            let total: f64 = record.positions.iter().map(|p| p.amount).sum();

            if total <= 0. {
                ui.label("The latest allocation record contains no positive positions.");
                return;
            }

            ui.label(format!(
                "Record from {}:",
                record.date
            ));
            ui.add_space(10.0);

            for position in &record.positions {

                let percentage = (position.amount as f64 / total as f64) * 100.0;
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
        &mut self, ui: &mut egui::Ui, page: Page, label: &str,
        init_page_fn: fn(&mut Self) -> Result<(), Error>
    ) {
        let response = ui.add_sized(
            [180.0, 20.0],
            egui::Button::selectable(self.page == page, label),
        );
        if response.clicked() {
            match init_page_fn(self) {
                Ok(_) => { self.page = page; }
                Err(e) => { self.message = Some(e.to_string()); }
            }
        }
    }

    fn init_configure_categories_page(&mut self) -> Result<(), Error> {
        self.reload_existing_catgs_and_catg_values()?;
        self.message = None;
        Ok(())
    }

    fn save_configured_categories(&mut self) {
        let (new_cfg_catgs_input, err) = self.asset_service.configure_categories(self.cfg_catgs_input.clone());
        if let Some(err) = err {
            self.message = Some(format!("Partial save. First error: {}", err));
        } else {
            self.message = Some("All saved".into());
        }
        self.cfg_catgs_input = new_cfg_catgs_input;
    }

    fn show_configure_categories_page(&mut self, ui: &mut egui::Ui) {

        ui.label(egui::RichText::new("Configure Categories").heading().size(Self::H2_SIZE));
        ui.add_space(Self::SPACE_2);
        if ui.button("Save").clicked() {
            self.save_configured_categories();
            if let Err(e) = self.reload_existing_catgs_and_catg_values() {
                self.message = Some(e.to_string());
            }
        }
        ui.add_space(Self::SPACE_2);
        let mut focus_next_catg_input = false;
        ui.horizontal(|ui| {
            if ui
                .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("+"))
                .clicked()
            {
                self.cfg_catgs_input.new_category_inputs.push(NewCategoryInput::default());
                focus_next_catg_input = true;
            }
            ui.label("Categories:");
        });
        
        /* Show inputs for new categories */
        let mut del_catg_idx: Option<usize> = None;
        for (catg_idx, catg_input) in self.cfg_catgs_input.new_category_inputs.iter_mut().enumerate().rev() {
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
                        .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("-"))
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
                                .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("+"))
                                .clicked()
                            {
                                catg_input.new_value_inputs.push(CategoryValueInput::default());
                                focus_next_val_input = true;
                            }
                            ui.label("Values:");
                        });

                        /* Show inputs for new values */
                        let mut del_val_idx: Option<usize> = None;
                        for (val_idx, val_input) in catg_input.new_value_inputs.iter_mut().enumerate().rev() {
                            ui.horizontal(|ui| {
                                ui.label("•");
                                let id = ui.make_persistent_id(("NewCatgVal", catg_idx, val_idx));
                                let response = ui.add(egui::TextEdit::singleline(&mut val_input.name).id(id));
                                if focus_next_val_input {
                                    response.request_focus();
                                    focus_next_val_input = false;
                                }
                                if ui
                                    .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("-"))
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
        for catg in &self.existing_catgs {

            let catg_input = &mut self.cfg_catgs_input.category_id_to_adapt_input
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
                            .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("+"))
                            .clicked()
                        {
                            catg_input.new_value_inputs.push(CategoryValueInput::default());
                            focus_next_val_input = true;
                        }
                        ui.label("Values:");
                    });

                    /* Show inputs for new values */
                    let mut del_val_idx: Option<usize> = None;
                    for (val_idx, val_input) in catg_input.new_value_inputs.iter_mut().enumerate().rev() {
                        ui.horizontal(|ui| {
                            ui.label("•");
                            let id = ui.make_persistent_id(("AdaptCatgVal", catg.id, val_idx));
                            let response = ui.add(egui::TextEdit::singleline(&mut val_input.name).id(id));
                            if focus_next_val_input {
                                response.request_focus();
                                focus_next_val_input = false;
                            }
                            if ui
                                .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("-"))
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

    fn init_add_asset_page(&mut self) -> Result<(), Error> {
        self.reset_add_asset_page();
        self.reload_existing_catgs_and_catg_values()?;
        self.add_asset_catgy_id_to_assignm_inputs.clear();
        self.message = None;
        Ok(())
    }

    fn save_new_asset(&mut self) {
        match self.asset_service.add_asset(
            &self.add_asset_asset_input,
            &self.add_asset_catgy_id_to_assignm_inputs
        ) {
            Ok(()) => {
                self.message = Some(format!("Asset '{}' was saved", self.add_asset_asset_input.name.trim()));
                self.reset_add_asset_page();
            }
            Err(err) => {
                self.message = Some(err.to_string());
            }
        }
    }

    fn show_add_asset_page(&mut self, ui: &mut egui::Ui) {

        ui.label(egui::RichText::new("Add Asset").heading().size(Self::H2_SIZE));
        ui.add_space(Self::SPACE_2);

        ui.label("Name:");
        ui.text_edit_singleline(&mut self.add_asset_asset_input.name);
        ui.add_space(Self::SPACE_2);

        ui.label("Reference type:");
        egui::ComboBox::from_id_salt("reference_type")
            .selected_text(Self::reference_type_label(self.add_asset_asset_input.reference_type))
            .show_ui(ui, |ui| {
                for t in AssetReferenceType::iter() {
                    ui.selectable_value(
                        &mut self.add_asset_asset_input.reference_type, t, Self::reference_type_label(t));
                }
            });
        ui.add_space(Self::SPACE_2);

        ui.label("Reference value:");
        ui.text_edit_singleline(&mut self.add_asset_asset_input.reference_value);
        ui.vertical(|ui| {
            for catgy in &mut self.existing_catgs {

                let selectable_vals = self.existing_catg_values
                    .entry(catgy.id)
                    .or_default();

                let assignm_inputs =
                    self.add_asset_catgy_id_to_assignm_inputs
                        .entry(catgy.id)
                        .or_default();

                let assignm_input_cnt =
                    *self.add_asset_catgy_id_to_assignm_input_cnt
                        .entry(catgy.id)
                        .or_insert(1) as usize;

                ui.add_space(Self::SPACE_2);
                ui.horizontal(|ui| {
                    if assignm_input_cnt < selectable_vals.len() {
                        if ui
                            .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("+"))
                            .clicked()
                        {
                            self.add_asset_catgy_id_to_assignm_input_cnt
                                .entry(catgy.id)
                                .and_modify(|cnt| *cnt = assignm_input_cnt as i64 + 1)
                                .or_insert(2);
                        }
                    }
                    ui.label(format!(" {}:", &catgy.name));
                });
                ui.add_space(Self::SPACE_1);


                assignm_inputs.resize_with(
                    assignm_input_cnt, || {
                        CategoryAssignmentInput {
                            value_id: None,
                            percentage:
                                if assignm_input_cnt == 1 { 100. }
                                else { 0. } }
                    });

                for input_idx in (0..assignm_input_cnt).rev() {
                    
                    let assignm_input = &mut assignm_inputs[input_idx];
                    let selected_text = assignm_input.value_id
                        .and_then(|id| selectable_vals.iter().find(|val| val.id == id))
                        .map(|val| val.name.clone())
                        .unwrap_or_else(|| "Select...".to_string());

                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [70.0, Self::DEFAULT_INPUT_HEIGHT],
                            egui::DragValue::new(&mut assignm_input.percentage)
                                .range(0.0..=100.0)
                                .speed(0.1)
                                .fixed_decimals(2)
                                .suffix("%"),
                        );
                        egui::ComboBox::from_id_salt(format!("{}:{}", catgy.id, input_idx))
                            .selected_text(selected_text)
                            .show_ui(ui, |ui| {
                                for value in selectable_vals.iter() {
                                    ui.selectable_value(
                                        &mut assignm_input.value_id,
                                        Some(value.id),
                                        &value.name,
                                    );
                                }
                            });
                        if input_idx == assignm_input_cnt - 1 {
                            if assignm_input_cnt > 1 {
                                if ui
                                    .add_sized([Self::SYM_BTN_SIZE, Self::SYM_BTN_SIZE], egui::Button::new("-"))
                                    .clicked()
                                {
                                    self.add_asset_catgy_id_to_assignm_input_cnt
                                        .entry(catgy.id)
                                        .and_modify(|cnt| *cnt = assignm_input_cnt as i64 - 1);
                                }
                            }
                        }
                    });
                }
            }
        });
        ui.add_space(Self::SPACE_2);
        if ui.button("Save").clicked() {
            self.save_new_asset()
        }
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(egui::RichText::new("Asset Allocation Tracker").heading().size(Self::H1_SIZE));
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.show_page_button(ui, Page::AllocationDiagram, "Allocation Diagram", Self::init_alocation_diagram_page);
                    self.show_page_button(ui, Page::AddAsset, "Add Asset", Self::init_add_asset_page);
                    self.show_page_button(ui, Page::ConfigureCategories, "Configure Categories", Self::init_configure_categories_page);
                    self.show_page_button(ui, Page::AddAllocationRecord, "Add Allocation Record", Self::init_add_allocation_record_page);
                });
                ui.add_space(20.0);
                ui.vertical(|ui| {
                    match self.page {
                        Page::AddAsset => self.show_add_asset_page(ui),
                        Page::AllocationDiagram => self.show_allocation_diagram_page(ui),
                        Page::ConfigureCategories => self.show_configure_categories_page(ui),
                        Page::AddAllocationRecord => self.show_add_allocation_record_page(ui),
                    }
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("Message").heading().size(Self::H2_SIZE));
                    ui.add_space(Self::SPACE_2);
                    if let Some(message) = &self.message {
                        ui.label(message);
                    }
                });
            });
        });
    }
}