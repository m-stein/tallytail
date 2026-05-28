use eyre::eyre;
use std::sync::mpsc;

use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_input::AddAssetInput,
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

    async fn get_categories() -> eyre::Result<Vec<Category>> {
        Ok(reqwest::get(Self::request_url("get_categories"))
            .await?
            .json::<Vec<Category>>()
            .await?)
    }

    async fn get_assets() -> eyre::Result<Vec<Asset>> {
        Ok(reqwest::get(Self::request_url("get_assets"))
            .await?
            .json::<Vec<Asset>>()
            .await?)
    }

    async fn get_latest_record() -> eyre::Result<Option<AllocationRecord>> {
        Ok(reqwest::get(Self::request_url("get_latest_record"))
            .await?
            .json::<Option<AllocationRecord>>()
            .await?)
    }

    async fn get_alloc_diagram_data(
        catg_id: i64,
        days: i64,
    ) -> eyre::Result<AllocationDiagramData> {
        Ok(reqwest::Client::new()
            .post(Self::request_url("get_alloc_diagram_data"))
            .json(&GetAllocDiagramDataArgs { catg_id, days })
            .send()
            .await?
            .error_for_status()?
            .json::<AllocationDiagramData>()
            .await?)
    }

    async fn add_asset(input: AddAssetInput) -> eyre::Result<()> {
        reqwest::Client::new()
            .post(Self::request_url("add_asset"))
            .json(&input)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
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
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let res = Self::get_categories().await;
            let _ = tx.send(res);
        });
        rx
    }

    fn start_get_assets(&self) -> GetAssetsRx {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let res = Self::get_assets().await;
            let _ = tx.send(res);
        });
        rx
    }

    fn start_get_latest_record(&self) -> GetLatestRecordRx {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::get_latest_record().await;
            let _ = tx.send(result);
        });
        rx
    }

    fn start_get_alloc_diagram_data(&self, catg_id: i64, days: i64) -> GetAllocDiagramDataRx {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::get_alloc_diagram_data(catg_id, days).await;
            let _ = tx.send(result);
        });
        rx
    }

    fn start_add_asset(&self, input: AddAssetInput) -> AddAssetRx {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::add_asset(input).await;
            let _ = tx.send(result);
        });
        rx
    }
}
