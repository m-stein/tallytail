use core_lib::add_asset_input::AddAssetInput;
use std::{sync::mpsc, thread};
use ui_lib::app_backend::{
    AddAssetRx, AppBackend, GetAllocDiagramDataRx, GetAssetsRx, GetCategoriesRx, GetLatestRecordRx,
};

pub struct DesktopBackend;

impl AppBackend for DesktopBackend {
    fn load_png_file(&self, path: &str) -> eyre::Result<Vec<u8>> {
        Ok(std::fs::read(format!("../{path}"))?)
    }

    fn start_get_categories(&self) -> GetCategoriesRx {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = infra_lib::get_categories();
            let _ = tx.send(result);
        });
        rx
    }

    fn start_get_assets(&self) -> GetAssetsRx {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = infra_lib::get_assets();
            let _ = tx.send(result);
        });
        rx
    }

    fn start_get_latest_record(&self) -> GetLatestRecordRx {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = infra_lib::get_latest_record();
            let _ = tx.send(result);
        });
        rx
    }

    fn start_get_alloc_diagram_data(&self, category_id: i64, days: i64) -> GetAllocDiagramDataRx {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = infra_lib::get_alloc_diagram_data(category_id, days);
            let _ = tx.send(result);
        });
        rx
    }

    fn start_add_asset(&self, input: &AddAssetInput) -> AddAssetRx {
        let input = input.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = infra_lib::add_asset(&input);
            let _ = tx.send(result);
        });
        rx
    }
}
