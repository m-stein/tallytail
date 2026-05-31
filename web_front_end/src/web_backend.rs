use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_args::AddAssetArgs,
    allocation_diagram_data::AllocationDiagramData, call_macro_with_request_list,
    category::Category,
};
use eyre::eyre;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::mpsc;
use ui_lib::app_backend::AppBackend;

macro_rules! implement_requests {

    // For each request, redirect to one of the @handler arms depending on whether
    // the request has an argument or not
    ($($request:ident($($arg_ty:ty)?) -> $ret_ty:ty;)*) => {
        $(implement_requests!(@func $request ($($arg_ty)?) -> $ret_ty);)*
    };
    (@func $request:ident () -> $ret_ty:ty) => {
        paste::paste! {
            fn [<start_ $request>](&self) -> ui_lib::app_backend::[<$request:camel Rx>] {
                Self::start_request::<(), $ret_ty>(stringify!($request), ())
            }
        }
    };
    (@func $request:ident ($arg_ty:ty) -> $ret_ty:ty) => {
        paste::paste! {
            fn [<start_ $request>](&self, args: $arg_ty) -> ui_lib::app_backend::[<$request:camel Rx>] {
                Self::start_request::<$arg_ty, $ret_ty>(stringify!($request), args)
            }
        }
    };
}

pub struct WebBackend;

impl WebBackend {
    const SERVER_URL: &str = "http://127.0.0.1:3000";

    fn request_url(request: &str) -> String {
        format!("{}/{}", Self::SERVER_URL, request)
    }

    async fn post<Args, Ret>(request: &str, args: Args) -> eyre::Result<Ret>
    where
        Args: Serialize,
        Ret: DeserializeOwned,
    {
        Ok(reqwest::Client::new()
            .post(Self::request_url(request))
            .json(&args)
            .send()
            .await?
            .error_for_status()?
            .json::<Ret>()
            .await?)
    }

    fn start_request<Args, Ret>(
        request: &'static str,
        args: Args,
    ) -> mpsc::Receiver<eyre::Result<Ret>>
    where
        Args: Serialize + 'static,
        Ret: DeserializeOwned + 'static,
    {
        let (tx, rx) = mpsc::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::post::<Args, Ret>(request, args).await;
            let _ = tx.send(result);
        });

        rx
    }
}

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
    call_macro_with_request_list!(implement_requests);
}
