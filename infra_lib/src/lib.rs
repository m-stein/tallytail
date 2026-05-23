use std::{
    fs,
    path::{Path, PathBuf},
};

use core_lib::{
    AllocationRecord, allocation_diagram_data::AllocationDiagramData, category::Category,
    category_value::CategoryValue,
};
use eyre::eyre;

pub fn get_alloc_diagram_data(category_id: i64, days: i64) -> eyre::Result<AllocationDiagramData> {
    let records = get_latest_records(days as usize)?;
    let category_name = get_category_name_by_id(category_id)?;
    Ok(AllocationDiagramData::new(records, &category_name))
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
