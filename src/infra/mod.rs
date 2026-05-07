#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite_asset_repository;

#[cfg(target_arch = "wasm32")]
pub mod mock_asset_repository;