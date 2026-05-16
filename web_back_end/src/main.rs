mod error;

use std::net::SocketAddr;

use axum::{
    Json, Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use core_lib::{
    AddUserArgs, AllocationRecord, GetAllocDiagramDataArgs, User,
    allocation_diagram_data::AllocationDiagramData, category::Category,
};

use crate::error::WebBackEndError;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let router = Router::new()
        .route("/users", get(get_users).post(add_user))
        .route("/get_latest_record", get(get_latest_record))
        .route("/get_alloc_diagram_data", post(get_alloc_diagram_data))
        .route("/get_categories", get(get_categories))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web back end läuft auf http://{addr}");
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

async fn get_users() -> eyre::Result<Json<Vec<User>>, WebBackEndError> {
    Ok(Json(infra_lib::list_users()?))
}

async fn get_categories() -> eyre::Result<Json<Vec<Category>>, WebBackEndError> {
    Ok(Json(infra_lib::get_categories()?))
}

async fn get_latest_record() -> eyre::Result<Json<Option<AllocationRecord>>, WebBackEndError> {
    Ok(Json(infra_lib::get_latest_record()?))
}

async fn add_user(Json(args): Json<AddUserArgs>) -> eyre::Result<Json<()>, WebBackEndError> {
    Ok(Json(infra_lib::add_user(args.name)?))
}
