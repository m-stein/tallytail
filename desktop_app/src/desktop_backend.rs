use core_lib::{GetAllocDiagramDataArgs, add_asset_args::AddAssetArgs};
use ui_lib::app_backend::AppBackend;

macro_rules! requests {
    ($( $req:ident($($arg:ident : $arg_ty:ty),*); )*) => {
        paste::paste! {
            $(fn [<start_ $req>](&self, $($arg: $arg_ty),*) -> ui_lib::app_backend::[<$req:camel Rx>] {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let result = infra_lib::$req($($arg),*);
                    let _ = tx.send(result);
                });
                rx
            })*
        }
    };
}

pub struct DesktopBackend;

impl AppBackend for DesktopBackend {
    fn load_png_file(&self, path: &str) -> eyre::Result<Vec<u8>> {
        Ok(std::fs::read(format!("../{path}"))?)
    }
    requests! {
        get_categories();
        get_assets();
        get_latest_record();
        get_alloc_diagram_data(args: GetAllocDiagramDataArgs);
        add_asset(args: AddAssetArgs);
    }
}
