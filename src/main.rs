#![deny(warnings)]

use asset_allocation_tracker::app::asset_service::AssetService;
use asset_allocation_tracker::ui::desktop_app::desktop_app::DesktopApp;

#[cfg(not(target_arch = "wasm32"))]
use asset_allocation_tracker::infra::sqlite_asset_repository::SqliteAssetRepository;

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
const DB_PATH: &str = "./data/assets.sdb";

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let repository = SqliteAssetRepository::new(DB_PATH)
        .unwrap_or_else(|err| panic!("Failed to initialize SQLite repository: {err}"));

    let service = AssetService::new(Box::new(repository));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_title("Asset Allocation Tracker"),
        ..Default::default()
    };

    eframe::run_native(
        "Asset Allocation Tracker",
        options,
        Box::new(move |creat_ctx| Ok(Box::new(DesktopApp::new(creat_ctx, service)))),
    )
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use asset_allocation_tracker::infra::mock_asset_repository::MockAssetRepository;

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("No window");
        let document = window.document().expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        let options = eframe::WebOptions::default();

        eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(|cc| {
                    let repository = MockAssetRepository::new();
                    let service = AssetService::new(Box::new(repository));

                    Ok(Box::new(DesktopApp::new(cc, service)))
                }),
            )
            .await
            .expect("Failed to start eframe");
    });
}