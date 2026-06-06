use core_lib::{
    AdaptCategoryInput, AllocationRecord, Asset, AssetReference, AssetReferenceType,
    CategoryAssignment, ConfigureCatgoriesInput, GetAllocDiagramDataArgs, NewCategoryInput,
    add_asset_args::AddAssetArgs, allocation_diagram_data::AllocationDiagramData,
    category::Category, category_value::CategoryValue,
};
use eyre::eyre;
use rusqlite::{params, types::FromSqlError};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub fn get_alloc_diagram_data(
    args: GetAllocDiagramDataArgs,
) -> eyre::Result<AllocationDiagramData> {
    let records = get_latest_records(args.days as usize)?;
    let category_name = get_category_name_by_id(args.category_id)?;
    Ok(AllocationDiagramData::new(records, &category_name))
}

pub fn load_png_data(path: String) -> eyre::Result<Vec<u8>> {
    Ok(std::fs::read(format!("../{path}"))?)
}

pub fn add_asset(args: AddAssetArgs) -> eyre::Result<()> {
    let name = args.name.trim();
    if name.is_empty() {
        return Err(eyre!("Asset name must not be empty"));
    }
    if args.reference.value.is_empty() {
        return Err(eyre!("Reference value must not be empty"));
    }
    let asset = Asset {
        id: 0,
        name: name.to_string(),
        reference: args.reference.clone(),
    };
    let mut catgy_assignms: Vec<CategoryAssignment> = Vec::new();
    for (_, assignments) in args.category_id_to_assignment.iter() {
        let mut percentage = 0.;
        let mut seen_value_ids = HashSet::new();
        for assignment in assignments {
            if assignment.percentage == 0. {
                return Err(eyre!("Category value has percentage of 0%"));
            }
            if let Some(id) = assignment.value_id {
                if !seen_value_ids.insert(id) {
                    return Err(eyre!("Duplicate category values"));
                }
                percentage += assignment.percentage;
                catgy_assignms.push(CategoryAssignment {
                    value_id: id,
                    ratio: assignment.percentage / 100.,
                });
            } else {
                return Err(eyre!("Category value unset"));
            }
        }
        if percentage > 100. {
            return Err(eyre!("Percentages for a category add up to more than 100%"));
        }
    }
    add_asset_raw(&asset, &catgy_assignms)
}

fn add_asset_raw(asset: &Asset, catgy_assignms: &[CategoryAssignment]) -> eyre::Result<()> {
    let mut connection = rusqlite::Connection::open("../data/assets.sdb")?;
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO assets (name, reference_type, reference_value) VALUES (?1, ?2, ?3)",
        params![
            asset.name,
            asset.reference.r#type.to_string(),
            asset.reference.value
        ],
    )?;
    let asset_id = tx.last_insert_rowid();
    for assignm in catgy_assignms.iter() {
        tx.execute(
            "
            INSERT INTO asset_category_value_assignments
            (asset_id, asset_category_value_id, ratio)
            VALUES (?1, ?2, ?3)",
            params![asset_id, assignm.value_id, assignm.ratio],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn get_assets() -> eyre::Result<Vec<Asset>> {
    let connection = rusqlite::Connection::open("../data/assets.sdb")?;
    let mut stmt = connection.prepare(
        "SELECT id, name, reference_type, reference_value
                FROM assets
                ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let reference_type_str: String = row.get(2)?;
        let reference_type: AssetReferenceType = reference_type_str.parse().map_err(|_| {
            FromSqlError::Other(
                format!("Invalid AssetReferenceType: '{reference_type_str}'").into(),
            )
        })?;
        Ok(Asset {
            id: row.get(0)?,
            name: row.get(1)?,
            reference: AssetReference {
                r#type: reference_type,
                value: row.get(3)?,
            },
        })
    })?;
    let mut assets = Vec::new();
    for row in rows {
        assets.push(row?);
    }
    Ok(assets)
}

pub fn get_categories() -> eyre::Result<Vec<Category>> {
    let connection = rusqlite::Connection::open("../data/assets.sdb")?;

    let mut stmt = connection.prepare(
        "
        SELECT id, name
        FROM asset_categories
        ORDER BY name ASC
        ",
    )?;

    let category_rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut categories = Vec::new();

    for category_row in category_rows {
        let (category_id, category_name) = category_row?;

        let mut value_stmt = connection.prepare(
            "
            SELECT id, name
            FROM asset_category_values
            WHERE asset_category_id = ?
            ORDER BY name ASC
            ",
        )?;

        let values = value_stmt
            .query_map([category_id], |row| {
                Ok(CategoryValue {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        categories.push(Category {
            id: category_id,
            name: category_name,
            values,
        });
    }

    Ok(categories)
}

pub fn get_latest_record() -> eyre::Result<Option<AllocationRecord>> {
    Ok(get_latest_records(1)?.pop())
}

fn get_latest_record_paths(dir: &Path, limit: usize) -> eyre::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Err(eyre!(format!("Directory does not exist: {:?}", dir)));
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().is_some_and(|ext| ext == "ron")
                && path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| jiff::civil::Date::strptime("%Y-%m-%d", stem).is_ok())
        })
        .collect();

    paths.sort_by(|a, b| {
        let a = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let b = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        b.cmp(a) // newest first
    });

    paths.truncate(limit);
    Ok(paths)
}

fn get_latest_records(limit: usize) -> eyre::Result<Vec<AllocationRecord>> {
    get_latest_record_paths(Path::new("../data/allocation_records"), limit)?
        .into_iter()
        .map(|path| Ok(ron::from_str(&fs::read_to_string(path)?)?))
        .collect()
}

fn get_category_name_by_id(category_id: i64) -> eyre::Result<String> {
    let connection = rusqlite::Connection::open("../data/assets.sdb")?;
    Ok(connection.query_row(
        "SELECT name FROM asset_categories WHERE id = ?1",
        rusqlite::params![category_id],
        |row| row.get(0),
    )?)
}

fn add_category_value(category_id: i64, value_name: &str) -> eyre::Result<()> {
    let connection = rusqlite::Connection::open("../data/assets.sdb")?;
    connection.execute(
        "INSERT INTO asset_category_values (asset_category_id, name)
        VALUES (?1, ?2)",
        rusqlite::params![category_id, value_name],
    )?;
    Ok(())
}

fn add_category(name: &str) -> eyre::Result<i64> {
    let connection = rusqlite::Connection::open("../data/assets.sdb")?;
    connection.execute(
        "INSERT INTO asset_categories (name) VALUES (?1)",
        params![name],
    )?;
    Ok(connection.last_insert_rowid())
}

pub fn configure_categories(
    input: ConfigureCatgoriesInput,
) -> eyre::Result<(ConfigureCatgoriesInput, Option<String>)> {
    let mut remaining = ConfigureCatgoriesInput::default();
    let mut first_error: Option<String> = None;

    // Neue Kategorien + deren neue Values
    for new_category in input.new_category_inputs {
        let category_name = new_category.name.trim();

        if category_name.is_empty() {
            remaining.new_category_inputs.push(new_category);
            continue;
        }

        match add_category(category_name) {
            Ok(category_id) => {
                let mut remaining_values = Vec::new();

                for value_input in new_category.new_value_inputs {
                    let value_name = value_input.name.trim();

                    if value_name.is_empty() {
                        remaining_values.push(value_input);
                        continue;
                    }

                    if let Err(err) = add_category_value(category_id, value_name) {
                        if first_error.is_none() {
                            first_error = Some(err.to_string());
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
                    first_error = Some(err.to_string());
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

            if let Err(err) = add_category_value(category_id, value_name) {
                if first_error.is_none() {
                    first_error = Some(err.to_string());
                }
                remaining_values.push(value_input);
            }
        }

        if !remaining_values.is_empty() {
            remaining.category_id_to_adapt_input.insert(
                category_id,
                AdaptCategoryInput {
                    new_value_inputs: remaining_values,
                },
            );
        }
    }

    Ok((remaining, first_error))
}
