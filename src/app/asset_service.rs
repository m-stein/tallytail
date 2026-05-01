use std::collections::HashMap;

use jiff::civil::Date;

use crate::app::allocation_record_input::{AllocationPositionInput, AllocationRecordInput};
use crate::app::allocation_record::AllocationRecord;
use crate::app::asset::Asset;
use crate::app::asset_input::AssetInput;
use crate::app::configure_categories_input::{AdaptCategoryInput, ConfigureCatgoriesInput, NewCategoryInput};
use crate::app::error::Error;
use crate::app::repository::AssetRepository;
use crate::app::asset_reference::AssetReference;
use crate::app::category::Category;
use crate::app::category_value::CategoryValue;
use crate::app::category_assignment::CategoryAssignment;
use crate::app::category_assignment_input::CategoryAssignmentInput;
use crate::app::named_distribution::{DatedDistribution, NamedDistribution};

pub struct AssetService {
    repository: Box<dyn AssetRepository>,
}

impl AssetService {
    pub fn new(repository: Box<dyn AssetRepository>) -> Self {
        Self { repository }
    }

    pub fn configure_categories(&mut self, input: ConfigureCatgoriesInput) -> (ConfigureCatgoriesInput, Option<Error>) {
        let mut remaining = ConfigureCatgoriesInput::default();
        let mut first_error: Option<Error> = None;

        // Neue Kategorien + deren neue Values
        for new_category in input.new_category_inputs {
            let category_name = new_category.name.trim();

            if category_name.is_empty() {
                remaining.new_category_inputs.push(new_category);
                continue;
            }

            match self.repository.add_category(category_name) {
                Ok(category_id) => {
                    let mut remaining_values = Vec::new();

                    for value_input in new_category.new_value_inputs {
                        let value_name = value_input.name.trim();

                        if value_name.is_empty() {
                            remaining_values.push(value_input);
                            continue;
                        }

                        if let Err(err) = self.repository.add_category_value(category_id, value_name) {
                            if first_error.is_none() {
                                first_error = Some(err);
                            }
                            remaining_values.push(value_input);
                        }
                    }

                    if !remaining_values.is_empty() {
                        remaining.new_category_inputs.push(NewCategoryInput {
                            name: new_category.name,
                            new_value_inputs: remaining_values,
                        });
                    }
                }
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }

                    remaining.new_category_inputs.push(new_category);
                }
            }
        }

        // Bestehende Kategorien erweitern
        for (category_id, adapt_input) in input.category_id_to_adapt_input {
            let mut remaining_values = Vec::new();

            for value_input in adapt_input.new_value_inputs {
                let value_name = value_input.name.trim();

                if value_name.is_empty() {
                    remaining_values.push(value_input);
                    continue;
                }

                if let Err(err) = self.repository.add_category_value(category_id, value_name) {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    remaining_values.push(value_input);
                }
            }

            if !remaining_values.is_empty() {
                remaining.category_id_to_adapt_input.insert(
                    category_id,
                    AdaptCategoryInput { new_value_inputs: remaining_values },
                );
            }
        }

        (remaining, first_error)
    }

    pub fn calc_distribution_for_category(
        &self,
        records: Vec<AllocationRecord>,
        category_name: &str,
    ) -> Vec<DatedDistribution> {
        records
            .into_iter()
            .map(|record| {
                let mut amounts: HashMap<String, f64> = HashMap::new();

                for position in record.positions {
                    let Some(category) = position
                        .asset
                        .categories
                        .iter()
                        .find(|category| category.name == category_name)
                    else {
                        continue;
                    };

                    for value in &category.values {
                        *amounts.entry(value.name.clone()).or_insert(0.0) +=
                            position.amount as f64 * value.ratio;
                    }
                }

                let mut values: Vec<NamedDistribution> = amounts
                    .into_iter()
                    .map(|(name, amount)| NamedDistribution {
                        name,
                        amount: amount,
                    })
                    .collect();

                values.sort_by(|a, b| {
                    b.amount
                        .partial_cmp(&a.amount)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.name.cmp(&b.name))
                });

                DatedDistribution {
                    date: record.date,
                    values,
                }
            })
            .collect()
    }

    pub fn get_distribution_for_category(
        &self,
        category_id: i64,
        days: i64,
    ) -> Result<Vec<DatedDistribution>, Error> {

        let records = self.repository.get_latest_allocation_records(days as usize)?;
        let category_name = self.repository.get_category_name_by_id(category_id)?;
        Ok(self.calc_distribution_for_category(records, &category_name))
    }

    pub fn add_asset(
        &mut self,
        asset_input: &AssetInput,
        catgy_id_to_assignm_inputs: &HashMap<i64, Vec<CategoryAssignmentInput>>,
    ) -> Result<(), Error> {
        let name = asset_input.name.trim();
        if name.is_empty() {
            return Err(Error::App(
                "Asset name must not be empty".into(),
            ));
        }
        let reference = AssetReference::new(
            asset_input.reference_type, asset_input.reference_value.clone()
        )?;
        let asset = Asset {
            id: 0,
            name: name.to_string(),
            reference,
        };
        let mut catgy_assignms: Vec<CategoryAssignment> = Vec::new();
        for (_, assignm_inputs) in catgy_id_to_assignm_inputs.iter() {
            for assignm_input in assignm_inputs {
                if let Some(id) = assignm_input.value_id {
                    catgy_assignms.push(CategoryAssignment { value_id: id, ratio: assignm_input.percentage / 100. })
                } else {
                    return Err(Error::App("Unset catgory value".into()));
                };
            }
        }
        self.repository.add_asset(&asset, &catgy_assignms)
    }
    
    pub fn list_assets(&self) -> Result<Vec<Asset>, Error> {
        self.repository.get_assets()
    }

    pub fn add_allocation_record(
        &mut self,
        date: Date,
        positions: Vec<AllocationPositionInput>,
    ) -> Result<(), Error> {
        let record = AllocationRecordInput::new(date, positions)?;
        self.repository.add_allocation_record(&record)
    }

    pub fn get_latest_allocation_record(
        &self,
    ) -> Result<Option<AllocationRecord>, Error> {
        Ok(self.repository.get_latest_allocation_records(1)?.pop())
    }

    pub fn get_categories(&self) -> Result<Vec<Category>, Error> {
        let mut catgs = self.repository.get_categories_without_values()?;
        for catg in catgs.iter_mut() {
            catg.values = self.repository.get_category_values(catg.id)?;
        }
        Ok(catgs)
    }
    
    pub fn get_category_values(&self, category_id: i64) -> Result<Vec<CategoryValue>, Error> {
        self.repository.get_category_values(category_id)
    }
}