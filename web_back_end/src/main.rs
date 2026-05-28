mod error;

use crate::error::WebBackEndError;
use axum::{
    Json, Router,
    routing::{get, post},
};
use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_input::AddAssetInput,
    allocation_diagram_data::AllocationDiagramData, category::Category,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let router = Router::new()
        .route("/get_latest_record", get(get_latest_record))
        .route("/get_alloc_diagram_data", post(get_alloc_diagram_data))
        .route("/get_categories", get(get_categories))
        .route("/get_assets", get(get_assets))
        .route("/add_asset", post(add_asset))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web back end runs on http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn get_alloc_diagram_data(
    Json(args): Json<GetAllocDiagramDataArgs>,
) -> eyre::Result<Json<AllocationDiagramData>, WebBackEndError> {
    Ok(Json(infra_lib::get_alloc_diagram_data(
        args.catg_id,
        args.days,
    )?))
}

async fn add_asset(Json(input): Json<AddAssetInput>) -> eyre::Result<Json<()>, WebBackEndError> {
    Ok(Json(infra_lib::add_asset(input)?))
}

async fn get_categories() -> eyre::Result<Json<Vec<Category>>, WebBackEndError> {
    Ok(Json(infra_lib::get_categories()?))
}

async fn get_assets() -> eyre::Result<Json<Vec<Asset>>, WebBackEndError> {
    Ok(Json(infra_lib::get_assets()?))
}

async fn get_latest_record() -> eyre::Result<Json<Option<AllocationRecord>>, WebBackEndError> {
    Ok(Json(infra_lib::get_latest_record()?))
}
