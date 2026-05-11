use std::{fs, net::SocketAddr};

use axum::{Json, Router, routing::get};
use eyre::Result;
use shared::{Data, User};
use tower_http::cors::CorsLayer;

const DATA_PATH: &str = "../data/data.ron";

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new()
        .route("/users", get(get_users))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Server läuft auf http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_users() -> Result<Json<Vec<User>>, AppError> {
    let users = read_users()?;
    Ok(Json(users))
}

fn read_users() -> Result<Vec<User>> {
    let text = fs::read_to_string(DATA_PATH)?;
    let data: Data = ron::from_str(&text)?;
    Ok(data.users)
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
            format!("Server error: {}", self.0),
        )
            .into_response()
    }
}