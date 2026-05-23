use std::{sync::mpsc, thread};

use eframe::{
    NativeOptions,
    egui::{Context, TextureHandle},
    run_native,
};

use egui::ViewportBuilder;
use ui_lib::{
    AppBackend, EframeApp, GetAllocDiagramDataRx, GetCategoriesResult, GetLatestRecordRx,
    png::load_png_texture_from_bytes,
};

pub struct DesktopBackend;

impl AppBackend for DesktopBackend {
    fn load_png_texture(&self, ctx: &Context, path: &str) -> eyre::Result<TextureHandle> {
        let path = format!("../{path}");
        load_png_texture_from_bytes(ctx, &path, &std::fs::read(&path)?)
    }

    fn start_get_categories(&self) -> mpsc::Receiver<GetCategoriesResult> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = infra_lib::get_categories();
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
}

fn main() -> eyre::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_maximized(true)
            .with_title("Asset Allocation Tracker"),
        ..Default::default()
    };
    run_native(
        "Asset Allocation Tracker",
        options,
        Box::new(|cc| {
            Ok(Box::new(EframeApp::new(cc, DesktopBackend)?))
        }),
    )?;
    Ok(())
}
