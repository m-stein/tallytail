use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::AllocationRecord;

#[derive(Serialize, Deserialize)]
pub struct AllocationDiagramSegment {
    pub name: Option<String>,
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct AllocationDiagramBar {
    pub date: String,
    pub segments: Vec<AllocationDiagramSegment>,
}

#[derive(Serialize, Deserialize)]
pub struct AllocationDiagramData {
    pub title: String,
    pub bars: Vec<AllocationDiagramBar>,
}

impl AllocationDiagramData {
    pub fn new(records: Vec<AllocationRecord>, category_name: &str) -> AllocationDiagramData {
        AllocationDiagramData {
            title: category_name.to_string(),
            bars: records
                .into_iter()
                .map(|record| {
                    let mut value_amounts: HashMap<String, f64> = HashMap::new(); /* value name -> summed-up amount */
                    let mut total_amount: f64 = 0.;
                    let mut total_categorized_amount: f64 = 0.;

                    for position in record.positions {
                        total_amount += position.amount;
                        let Some(category) = position
                            .asset
                            .categories
                            .iter()
                            .find(|category| category.name == category_name)
                        else {
                            continue;
                        };

                        for value in &category.values {
                            let value_amount = position.amount * value.ratio;
                            *value_amounts.entry(value.name.clone()).or_insert(0.0) += value_amount;
                            total_categorized_amount += value_amount;
                        }
                    }

                    let mut value_distributions: Vec<AllocationDiagramSegment> = value_amounts
                        .into_iter()
                        .map(|(name, amount)| AllocationDiagramSegment {
                            name: Some(name),
                            amount,
                        })
                        .collect();

                    if total_amount > total_categorized_amount {
                        value_distributions.push(
                            AllocationDiagramSegment {
                                name: None,
                                amount: total_amount - total_categorized_amount });
                    }
                    value_distributions.sort_by(|a, b| {
                        b.amount
                            .partial_cmp(&a.amount)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| a.name.cmp(&b.name))
                    });

                    AllocationDiagramBar {
                        date: record.date,
                        segments: value_distributions,
                    }
                })
                .collect()
        }
    }
}
