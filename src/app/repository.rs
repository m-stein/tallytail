use crate::app::allocation_record::AllocationRecord;
use crate::app::error::Error;

use crate::app::allocation_record_input::AllocationRecordInput;
use crate::app::asset::Asset;
use crate::app::category::Category;
use crate::app::category_value::CategoryValue;
use crate::app::category_assignment::CategoryAssignment;

pub trait AssetRepository {
    fn add_asset(&mut self, asset: &Asset, catgy_assignms: &Vec<CategoryAssignment>) -> Result<(), Error>;
    fn add_category(&mut self, name: &str) -> Result<i64, Error>;
    fn get_assets(&self) -> Result<Vec<Asset>, Error>;
    fn add_allocation_record(
        &mut self,
        record: &AllocationRecordInput,
    ) -> Result<(), Error>;
    fn get_categories_without_values(&self) -> Result<Vec<Category>, Error>;
    fn get_category_values(&self, category_id: i64) -> Result<Vec<CategoryValue>, Error>;
    fn add_category_value(&mut self, category_id: i64, value_name: &str) -> Result<(), Error>;
    fn get_latest_allocation_records(
        &self,
        limit: usize,
    ) -> Result<Vec<AllocationRecord>, Error>;
    fn get_category_name_by_id(&self, category_id: i64) -> Result<String, Error>;
}