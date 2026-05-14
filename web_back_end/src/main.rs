use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use eyre::Result;
use tower_http::cors::CorsLayer;

use core_lib::User;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AddUserRequest {
    name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/users", get(get_users).post(add_user))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Web back end läuft auf http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_users() -> Result<Json<Vec<User>>, AppError> {
    let users = infra_lib::read_users()?;
    Ok(Json(users))
}

async fn add_user(Json(request): Json<AddUserRequest>) -> Result<(), AppError> {
    infra_lib::add_user(request.name)?;
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