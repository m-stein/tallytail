use core_lib::{
    AllocationRecord, allocation_diagram_data::AllocationDiagramData, category::Category,
};
use std::sync::mpsc::Receiver;

pub type GetCategoriesRx = Receiver<eyre::Result<Vec<Category>>>;
pub type GetLatestRecordRx = Receiver<eyre::Result<Option<AllocationRecord>>>;
pub type GetAllocDiagramDataRx = Receiver<eyre::Result<AllocationDiagramData>>;

pub trait AppBackend {
    fn load_png_file(&self, path: &str) -> eyre::Result<Vec<u8>>;
    fn start_get_categories(&self) -> GetCategoriesRx;
    fn start_get_latest_record(&self) -> GetLatestRecordRx;
    fn start_get_alloc_diagram_data(&self, category_id: i64, days: i64) -> GetAllocDiagramDataRx;
}
