mod error;

use crate::error::WebBackEndError;
use axum::{Json, Router, routing::post};
use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_args::AddAssetArgs,
    allocation_diagram_data::AllocationDiagramData, category::Category,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let router = Router::new()
        .route("/get_latest_record", post(get_latest_record))
        .route("/get_alloc_diagram_data", post(get_alloc_diagram_data))
        .route("/get_categories", post(get_categories))
        .route("/get_assets", post(get_assets))
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
) -> Result<Json<AllocationDiagramData>, WebBackEndError> {
    Ok(Json(infra_lib::get_alloc_diagram_data(args)?))
}

async fn add_asset(Json(args): Json<AddAssetArgs>) -> Result<Json<()>, WebBackEndError> {
    infra_lib::add_asset(args)?;
    Ok(Json(()))
}

async fn get_categories(Json(()): Json<()>) -> Result<Json<Vec<Category>>, WebBackEndError> {
    Ok(Json(infra_lib::get_categories()?))
}

async fn get_assets(Json(()): Json<()>) -> Result<Json<Vec<Asset>>, WebBackEndError> {
    Ok(Json(infra_lib::get_assets()?))
}

async fn get_latest_record(
    Json(()): Json<()>,
) -> Result<Json<Option<AllocationRecord>>, WebBackEndError> {
    Ok(Json(infra_lib::get_latest_record()?))
}
