
#[cfg(not(target_arch = "wasm32"))]
use asset_allocation_tracker::backend::server;
#[cfg(not(target_arch = "wasm32"))]
use asset_allocation_tracker::infra::sqlite_asset_repository::SqliteAssetRepository;

#[cfg(not(target_arch = "wasm32"))]
const DB_PATH: &str = "./data/assets.sdb";

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let repository = SqliteAssetRepository::new(DB_PATH)
        .expect("Failed to initialize SQLite repository");

    server::serve(repository, "127.0.0.1:3000")
        .await
        .expect("Server failed");
}

#[cfg(target_arch = "wasm32")]
fn main() {}