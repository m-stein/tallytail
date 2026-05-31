use eyre::eyre;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::mpsc;

use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_args::AddAssetArgs,
    allocation_diagram_data::AllocationDiagramData, category::Category,
};
use ui_lib::app_backend::{
    AddAssetRx, AppBackend, GetAllocDiagramDataRx, GetAssetsRx, GetCategoriesRx, GetLatestRecordRx,
};

pub struct WebBackend;

impl WebBackend {
    const SERVER_URL: &str = "http://127.0.0.1:3000";

    fn request_url(request: &str) -> String {
        format!("{}/{}", Self::SERVER_URL, request)
    }

    async fn post<Args, Ret>(request: &str, args: Args) -> eyre::Result<Ret>
    where
        Args: Serialize,
        Ret: DeserializeOwned,
    {
        Ok(reqwest::Client::new()
            .post(Self::request_url(request))
            .json(&args)
            .send()
            .await?
            .error_for_status()?
            .json::<Ret>()
            .await?)
    }

    fn start_request<Args, Ret>(
        request: &'static str,
        args: Args,
    ) -> mpsc::Receiver<eyre::Result<Ret>>
    where
        Args: Serialize + 'static,
        Ret: DeserializeOwned + 'static,
    {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::post::<Args, Ret>(request, args).await;
            let _ = tx.send(result);
        });

        rx
    }
}

impl AppBackend for WebBackend {
    fn load_png_file(&self, path: &str) -> eyre::Result<Vec<u8>> {
        let bytes: &[u8] = match path {
            "img/squirrel_68x68.png" => {
                include_bytes!("../../img/squirrel_68x68.png")
            }
            _ => return Err(eyre!("unknown embedded asset path: {path}")),
        };

        Ok(bytes.into())
    }

    fn start_get_categories(&self) -> GetCategoriesRx {
        Self::start_request::<(), Vec<Category>>("get_categories", ())
    }

    fn start_get_assets(&self) -> GetAssetsRx {
        Self::start_request::<(), Vec<Asset>>("get_assets", ())
    }

    fn start_get_latest_record(&self) -> GetLatestRecordRx {
        Self::start_request::<(), Option<AllocationRecord>>("get_latest_record", ())
    }

    fn start_get_alloc_diagram_data(&self, args: GetAllocDiagramDataArgs) -> GetAllocDiagramDataRx {
        Self::start_request::<GetAllocDiagramDataArgs, AllocationDiagramData>(
            "get_alloc_diagram_data",
            args,
        )
    }

    fn start_add_asset(&self, args: AddAssetArgs) -> AddAssetRx {
        Self::start_request::<AddAssetArgs, ()>("add_asset", args)
    }
}
