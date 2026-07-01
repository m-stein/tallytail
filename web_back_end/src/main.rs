mod error;

use crate::error::WebBackEndError;
use axum::{Json, Router, response::Html, routing::{get, get_service, post}};
use core_lib::call_macro_with_request_list;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

macro_rules! implement_requests {
    ($($request:ident($($arg_ty:ty)?) -> $ret_ty:ty;)*) => {

        fn router() -> Router {
            let static_service = get_service(ServeDir::new("web_front_end/dist"));

            Router::new()
                $(.route(concat!("/", stringify!($request)), post($request)))*
                .route("/", get(index))
                .fallback_service(static_service)
        }

        $(implement_requests!(@handler $request ($($arg_ty)?) -> $ret_ty);)*
    };
    (@handler $request:ident () -> $ret_ty:ty) => {
        async fn $request(
            Json(()): Json<()>,
        ) -> Result<Json<$ret_ty>, WebBackEndError> {
            Ok(Json(infra_lib::$request()?))
        }
    };
    (@handler $request:ident ($arg_ty:ty) -> $ret_ty:ty) => {
        async fn $request(
            Json(args): Json<$arg_ty>,
        ) -> Result<Json<$ret_ty>, WebBackEndError> {
            Ok(Json(infra_lib::$request(args)?))
        }
    };
}

async fn index() -> Html<String> {
    Html(std::fs::read_to_string("web_front_end/dist/index.html").unwrap())
}

call_macro_with_request_list!(implement_requests);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let router = router().layer(CorsLayer::permissive());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web back end runs on http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}
