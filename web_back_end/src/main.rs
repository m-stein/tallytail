use std::net::SocketAddr;

use axum::{
    Json, Router,
    routing::{get, post},
};
use eyre::Result;
use tower_http::cors::CorsLayer;

use core_lib::{
    AddUserArgs, AllocationRecord, GetAllocDiagramDataArgs, User,
    allocation_diagram_data::AllocationDiagramData, category::Category,
};

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .route("/users", get(get_users).post(add_user))
        .route("/get_latest_record", get(get_latest_record))
        .route("/get_alloc_diagram_data", post(get_alloc_diagram_data))
        .route("/list_categories", get(list_categories))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web back end läuft auf http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn get_alloc_diagram_data(
    Json(args): Json<GetAllocDiagramDataArgs>,
) -> Result<Json<AllocationDiagramData>, AppError> {
    let data = infra_lib::get_alloc_diagram_data(args.catg_id, args.days)?;
    Ok(Json(data))
}

async fn get_users() -> Result<Json<Vec<User>>, AppError> {
    let users = infra_lib::list_users()?;
    Ok(Json(users))
}

async fn list_categories() -> Result<Json<Vec<Category>>, AppError> {
    Ok(Json(infra_lib::list_categories()?))
}

async fn get_latest_record() -> Result<Json<Option<AllocationRecord>>, AppError> {
    let res = infra_lib::get_latest_record()?;
    Ok(Json(res))
}

async fn add_user(Json(args): Json<AddUserArgs>) -> Result<(), AppError> {
    infra_lib::add_user(args.name)?;
    Ok(())
}

struct AppError(eyre::Report);

impl<E> From<E> for AppError
where
    E: Into<eyre::Report>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Web back end error: {}", self.0),
        )
            .into_response()
    }
}
