use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::app::allocation_record::AllocationRecord;
use crate::app::allocation_record_input::AllocationRecordInput;
use crate::app::asset::Asset;
use crate::app::repository::AssetRepository;
use crate::app::category::Category;
use crate::app::category_assignment::CategoryAssignment;
use crate::app::category_value::CategoryValue;
use crate::app::error::Error;
use crate::infra::sqlite_asset_repository::SqliteAssetRepository;

type SharedRepository = Arc<Mutex<SqliteAssetRepository>>;

#[derive(Debug, Deserialize)]
struct AddAssetRequest {
    asset: Asset,
    category_assignments: Vec<CategoryAssignment>,
}

#[derive(Debug, Deserialize)]
struct AddCategoryRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct AddCategoryValueRequest {
    value_name: String,
}

#[derive(Debug, Deserialize)]
struct LatestRecordsQuery {
    limit: usize,
}

pub fn router(repository: SqliteAssetRepository) -> Router {
    let repository = Arc::new(Mutex::new(repository));

    Router::new()
        .route("/assets", get(get_assets).post(add_asset))
        .route("/categories", get(get_categories_without_values).post(add_category))
        .route(
            "/categories/:category_id/name",
            get(get_category_name_by_id),
        )
        .route(
            "/categories/:category_id/values",
            get(get_category_values).post(add_category_value),
        )
        .route(
            "/allocation-records",
            post(add_allocation_record),
        )
        .route(
            "/allocation-records/latest",
            get(get_latest_allocation_records),
        )
        .with_state(repository)
}

pub async fn serve(
    repository: SqliteAssetRepository,
    address: &str,
) -> Result<(), Error> {
    let listener = tokio::net::TcpListener::bind(address).await?;
    Ok(axum::serve(listener, router(repository)).await?)
}

async fn add_asset(
    State(repository): State<SharedRepository>,
    Json(request): Json<AddAssetRequest>,
) -> Result<StatusCode, Error> {
    let mut repository = lock_repository(&repository)?;

    repository.add_asset(
        &request.asset,
        &request.category_assignments,
    )?;

    Ok(StatusCode::CREATED)
}

async fn get_assets(
    State(repository): State<SharedRepository>,
) -> Result<Json<Vec<Asset>>, Error> {
    let repository = lock_repository(&repository)?;
    Ok(Json(repository.get_assets()?))
}

async fn add_category(
    State(repository): State<SharedRepository>,
    Json(request): Json<AddCategoryRequest>,
) -> Result<Json<i64>, Error> {
    let mut repository = lock_repository(&repository)?;
    let id = repository.add_category(&request.name)?;
    Ok(Json(id))
}

async fn get_categories_without_values(
    State(repository): State<SharedRepository>,
) -> Result<Json<Vec<Category>>, Error> {
    let repository = lock_repository(&repository)?;
    Ok(Json(repository.get_categories_without_values()?))
}

async fn get_category_values(
    State(repository): State<SharedRepository>,
    Path(category_id): Path<i64>,
) -> Result<Json<Vec<CategoryValue>>, Error> {
    let repository = lock_repository(&repository)?;
    Ok(Json(repository.get_category_values(category_id)?))
}

async fn add_category_value(
    State(repository): State<SharedRepository>,
    Path(category_id): Path<i64>,
    Json(request): Json<AddCategoryValueRequest>,
) -> Result<StatusCode, Error> {
    let mut repository = lock_repository(&repository)?;

    repository.add_category_value(
        category_id,
        &request.value_name,
    )?;

    Ok(StatusCode::CREATED)
}

async fn add_allocation_record(
    State(repository): State<SharedRepository>,
    Json(record): Json<AllocationRecordInput>,
) -> Result<StatusCode, Error> {
    let mut repository = lock_repository(&repository)?;
    repository.add_allocation_record(&record)?;
    Ok(StatusCode::CREATED)
}

async fn get_latest_allocation_records(
    State(repository): State<SharedRepository>,
    Query(query): Query<LatestRecordsQuery>,
) -> Result<Json<Vec<AllocationRecord>>, Error> {
    let repository = lock_repository(&repository)?;
    Ok(Json(repository.get_latest_allocation_records(query.limit)?))
}

async fn get_category_name_by_id(
    State(repository): State<SharedRepository>,
    Path(category_id): Path<i64>,
) -> Result<Json<String>, Error> {
    let repository = lock_repository(&repository)?;
    Ok(Json(repository.get_category_name_by_id(category_id)?))
}

fn lock_repository(
    repository: &SharedRepository,
) -> Result<std::sync::MutexGuard<'_, SqliteAssetRepository>, Error> {
    Ok(repository.lock()?)
}