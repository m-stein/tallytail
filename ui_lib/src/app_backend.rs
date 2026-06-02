use core_lib::call_macro_with_request_list;
use std::sync::mpsc::Receiver;

macro_rules! define_app_backend {
    ($($request:ident($($arg_ty:ty)?) -> $ret_ty:ty;)*) => {
        paste::paste! {
            $(pub type [<$request:camel Rx>] = Receiver<eyre::Result<$ret_ty>>;)*

            pub trait AppBackend {
                $(define_app_backend!(@method $request ($($arg_ty)?) -> $ret_ty);)*
            }
        }
    };
    (@method $request:ident () -> $ret_ty:ty) => {
        paste::paste! {
            fn [<start_ $request>](&self) -> [<$request:camel Rx>];
        }
    };
    (@method $request:ident ($arg_ty:ty) -> $ret_ty:ty) => {
        paste::paste! {
            fn [<start_ $request>](&self, args: $arg_ty) -> [<$request:camel Rx>];
        }
    };
}

call_macro_with_request_list!(define_app_backend);
