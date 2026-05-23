use std::sync::mpsc;

use core_lib::{
    AllocationRecord, GetAllocDiagramDataArgs, allocation_diagram_data::AllocationDiagramData,
    category::Category,
};
use eyre::eyre;
use ui_lib::{
    AppBackend, EframeApp, GetAllocDiagramDataRx, GetCategoriesResult, GetLatestRecordRx,
};
use wasm_bindgen::JsCast;

const GET_LATEST_RECORD_URL: &str = "http://127.0.0.1:3000/get_latest_record";
const GET_CATEGORIES_URL: &str = "http://127.0.0.1:3000/get_categories";
const GET_ALLOC_DIAGRAM_DATA_URL: &str = "http://127.0.0.1:3000/get_alloc_diagram_data";

async fn get_categories() -> eyre::Result<Vec<Category>> {
    println!("1");
    Ok(reqwest::get(GET_CATEGORIES_URL)
        .await?
        .json::<Vec<Category>>()
        .await?)
}

async fn get_latest_record() -> eyre::Result<Option<AllocationRecord>> {
    Ok(reqwest::get(GET_LATEST_RECORD_URL)
        .await?
        .json::<Option<AllocationRecord>>()
        .await?)
}

async fn get_alloc_diagram_data(catg_id: i64, days: i64) -> eyre::Result<AllocationDiagramData> {
    let client = reqwest::Client::new();
    let response = client
        .post(GET_ALLOC_DIAGRAM_DATA_URL)
        .json(&GetAllocDiagramDataArgs { catg_id, days })
        .send()
        .await?
        .error_for_status()?;
    Ok(response.json::<AllocationDiagramData>().await?)
}

pub struct WebBackend;

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

    fn start_get_categories(&self) -> mpsc::Receiver<GetCategoriesResult> {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let res = get_categories().await;
            let _ = tx.send(res);
        });
        rx
    }

    fn start_get_latest_record(&self) -> GetLatestRecordRx {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = get_latest_record().await;
            let _ = tx.send(result);
        });
        rx
    }

    fn start_get_alloc_diagram_data(&self, catg_id: i64, days: i64) -> GetAllocDiagramDataRx {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = get_alloc_diagram_data(catg_id, days).await;
            let _ = tx.send(result);
        });
        rx
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn start() -> eyre::Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("the_canvas_id"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("Canvas not found"))?;

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|cc| Ok(Box::new(EframeApp::new(cc, WebBackend)?))),
        )
        .await
}
