use std::{
    fs,
    path::{Path, PathBuf},
};

use core_lib::{
    AllocationRecord, allocation_diagram_data::AllocationDiagramData, category::Category,
};
use eyre::eyre;

pub fn get_alloc_diagram_data(category_id: i64, days: i64) -> eyre::Result<AllocationDiagramData> {
    let records = get_latest_records(days as usize)?;
    let category_name = get_category_name_by_id(category_id)?;
    Ok(AllocationDiagramData::new(records, &category_name))
}

pub fn get_categories() -> eyre::Result<Vec<Category>> {
    let connection = rusqlite::Connection::open("../data/assets.sdb")?;
    Ok(connection
        .prepare(
            "
            SELECT id, name
            FROM asset_categories
            ORDER BY name ASC",
        )
        .and_then(|mut stmt| {
            stmt.query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    values: Vec::new(),
                })
            })?
            .collect()
        })?)
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
