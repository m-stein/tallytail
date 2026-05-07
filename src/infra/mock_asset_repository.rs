use crate::app::allocation_record::AllocationRecord;
use crate::app::allocation_record_input::AllocationRecordInput;
use crate::app::asset::Asset;
use crate::app::repository::AssetRepository;
use crate::app::category::Category;
use crate::app::category_assignment::CategoryAssignment;
use crate::app::category_value::CategoryValue;
use crate::app::error::Error;

#[derive(Debug, Default)]
pub struct MockAssetRepository {
    assets: Vec<Asset>,
    categories: Vec<Category>,
    allocation_records: Vec<AllocationRecord>,
    next_category_id: i64,
    next_category_value_id: i64,
}

impl MockAssetRepository {
    pub fn new() -> Self {
        Self {
            next_category_id: 1,
            next_category_value_id: 1,
            ..Default::default()
        }
    }
}

impl AssetRepository for MockAssetRepository {
    fn add_asset(
        &mut self,
        asset: &Asset,
        catgy_assignms: &Vec<CategoryAssignment>,
    ) -> Result<(), Error> {
        println!("Adding asset: {:?}", asset);

        for assignm in catgy_assignms {
            println!(
                "CategoryAssignment => value_id: {}, ratio: {}",
                assignm.value_id,
                assignm.ratio
            );
        }

        self.assets.push(asset.clone());

        Ok(())
    }

    fn add_category(&mut self, name: &str) -> Result<i64, Error> {
        let id = self.next_category_id;
        self.next_category_id += 1;

        self.categories.push(Category {
            id,
            name: name.to_string(),
            values: Vec::new(),
        });

        Ok(id)
    }

    fn get_assets(&self) -> Result<Vec<Asset>, Error> {
        Ok(self.assets.clone())
    }

    fn add_allocation_record(
        &mut self,
        record: &AllocationRecordInput,
    ) -> Result<(), Error> {
        println!("AllocationRecordInput => date: {}", record.date);

        for position in &record.positions {
            println!(
                "AllocationPositionInput => asset_id: {}, amount: {}",
                position.asset_id,
                position.amount
            );
        }

        Ok(())
    }

    fn get_categories_without_values(&self) -> Result<Vec<Category>, Error> {
        Ok(self
            .categories
            .iter()
            .map(|category| Category {
                id: category.id,
                name: category.name.clone(),
                values: Vec::new(),
            })
            .collect())
    }

    fn get_category_values(&self, category_id: i64) -> Result<Vec<CategoryValue>, Error> {
        Ok(self
            .categories
            .iter()
            .find(|category| category.id == category_id)
            .map(|category| category.values.clone())
            .unwrap_or_default())
    }

    fn add_category_value(
        &mut self,
        category_id: i64,
        value_name: &str,
    ) -> Result<(), Error> {
        let Some(category) = self
            .categories
            .iter_mut()
            .find(|category| category.id == category_id)
        else {
            return Err(Error::App(format!(
                "Category with id {category_id} not found"
            )));
        };

        let id = self.next_category_value_id;
        self.next_category_value_id += 1;

        category.values.push(CategoryValue {
            id,
            name: value_name.to_string(),
        });

        Ok(())
    }

    fn get_latest_allocation_records(
        &self,
        limit: usize,
    ) -> Result<Vec<AllocationRecord>, Error> {
        Ok(self
            .allocation_records
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }

    fn get_category_name_by_id(&self, category_id: i64) -> Result<String, Error> {
        self.categories
            .iter()
            .find(|category| category.id == category_id)
            .map(|category| category.name.clone())
            .ok_or_else(|| Error::App(format!(
                "Category with id {category_id} not found"
            )))
    }
}