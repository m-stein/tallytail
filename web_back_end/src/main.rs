mod error;

use std::net::SocketAddr;

use axum::{
    Json, Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use core_lib::{
    AllocationRecord, GetAllocDiagramDataArgs, allocation_diagram_data::AllocationDiagramData,
    category::Category,
};

use crate::error::WebBackEndError;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let router = Router::new()
        .route("/get_latest_record", get(get_latest_record))
        .route("/get_alloc_diagram_data", post(get_alloc_diagram_data))
        .route("/get_categories", get(get_categories))
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

async fn get_categories() -> eyre::Result<Json<Vec<Category>>, WebBackEndError> {
    Ok(Json(infra_lib::get_categories()?))
}

async fn get_latest_record() -> eyre::Result<Json<Option<AllocationRecord>>, WebBackEndError> {
    Ok(Json(infra_lib::get_latest_record()?))
}
