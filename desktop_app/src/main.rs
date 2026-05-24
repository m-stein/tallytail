mod desktop_backend;

use core_lib::APP_NAME;
use eframe::{NativeOptions, run_native};

use egui::ViewportBuilder;
use ui_lib::eframe_app::EframeApp;

use crate::desktop_backend::DesktopBackend;

fn main() -> eyre::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_maximized(true)
            .with_title(APP_NAME),
        ..Default::default()
    };
    run_native(
        APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(EframeApp::new(cc, DesktopBackend)?))),
    )?;
    Ok(())
}
