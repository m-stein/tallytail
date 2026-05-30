use core_lib::{
    AllocationRecord, Asset, GetAllocDiagramDataArgs, add_asset_args::AddAssetArgs,
    allocation_diagram_data::AllocationDiagramData, category::Category,
};
use std::sync::mpsc::Receiver;

pub type GetCategoriesRx = Receiver<eyre::Result<Vec<Category>>>;
pub type GetAssetsRx = Receiver<eyre::Result<Vec<Asset>>>;
pub type GetLatestRecordRx = Receiver<eyre::Result<Option<AllocationRecord>>>;
pub type GetAllocDiagramDataRx = Receiver<eyre::Result<AllocationDiagramData>>;
pub type AddAssetRx = Receiver<eyre::Result<()>>;

pub trait AppBackend {
    fn load_png_file(&self, path: &str) -> eyre::Result<Vec<u8>>;
    fn start_get_categories(&self) -> GetCategoriesRx;
    fn start_get_assets(&self) -> GetAssetsRx;
    fn start_get_latest_record(&self) -> GetLatestRecordRx;
    fn start_get_alloc_diagram_data(&self, args: GetAllocDiagramDataArgs) -> GetAllocDiagramDataRx;
    fn start_add_asset(&self, args: AddAssetArgs) -> AddAssetRx;
}
