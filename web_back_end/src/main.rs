mod error;

use crate::error::WebBackEndError;
use axum::{Json, Router, routing::post};
use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_args::AddAssetArgs,
    allocation_diagram_data::AllocationDiagramData, call_macro_with_request_list,
    category::Category,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

macro_rules! implement_requests {
    ($($request:ident($($arg_ty:ty)?) -> $ret_ty:ty;)*) => {

        // Add one function for creating a router that has one "POST" route for each request
        fn router() -> Router {
            Router::new()$(.route(concat!("/", stringify!($request)), post($request)))*
        }
        // Add an async handler for each "POST" route. Each handler calls the
        // `infra_lib` function with the same name as the handled request
        $(implement_requests!(@handler $request ($($arg_ty)?) -> $ret_ty);)*
    };
    // Handler template for requests with no argument
    (@handler $request:ident () -> $ret_ty:ty) => {
        async fn $request(
            Json(()): Json<()>,
        ) -> Result<Json<$ret_ty>, WebBackEndError> {
            Ok(Json(infra_lib::$request()?))
        }
    };
    // Handler template for requests with one argument
    (@handler $request:ident ($arg_ty:ty) -> $ret_ty:ty) => {
        async fn $request(
            Json(args): Json<$arg_ty>,
        ) -> Result<Json<$ret_ty>, WebBackEndError> {
            Ok(Json(infra_lib::$request(args)?))
        }
    };
}

call_macro_with_request_list!(implement_requests);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let router = router().layer(CorsLayer::permissive());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web back end runs on http://{addr}");
    axum::serve(listener, router).await?;
    Ok(())
}
