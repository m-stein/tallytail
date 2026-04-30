use std::fs;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use ron::ser::PrettyConfig;
use rusqlite::{Connection, params};

use crate::app::allocation_record::{AllocationPosition, AllocationRecord, AllocationAssetCategory, AllocationCategoryValue, AllocationAsset};
use crate::app::asset_reference_type::AssetReferenceType;
use crate::app::category::Category;
use crate::app::error::Error;
use crate::app::repository::AssetRepository;
use crate::app::allocation_record_input::AllocationRecordInput;
use crate::app::asset::Asset;
use crate::app::asset_reference::AssetReference;
use crate::app::category_value::CategoryValue;
use crate::app::category_assignment::CategoryAssignment;

pub struct SqliteAssetRepository {
    connection: Connection,
    allocation_records_path: String,
}


impl SqliteAssetRepository {
    pub fn new(db_path: &str) -> Result<Self, Error> {
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let connection = Connection::open(db_path)?;
        let repository = Self { connection, allocation_records_path: "data/allocation_records".into() };
        repository.init_schema()?;
        Ok(repository)
    }

    fn load_asset_ron(&self, asset_id: i64) -> Result<AllocationAsset, Error> {
        let (name, reference_type_str, reference_value): (String, String, String) = self.connection
            .query_row(
                "SELECT name, reference_type, reference_value
                FROM assets
                WHERE id = ?1",
                rusqlite::params![asset_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;

        let reference_type: AssetReferenceType = reference_type_str.parse().unwrap();
        let mut stmt = self.connection
            .prepare(
                "SELECT ac.name, acv.name, acva.ratio
                FROM asset_category_value_assignments acva
                JOIN asset_category_values acv
                    ON acv.id = acva.asset_category_value_id
                JOIN asset_categories ac
                    ON ac.id = acv.asset_category_id
                WHERE acva.asset_id = ?1
                ORDER BY ac.name, acv.name",
            )?;

        let rows = stmt
            .query_map(rusqlite::params![asset_id], |row| {
                Ok((
                    row.get::<_, String>(0)?, // category
                    row.get::<_, String>(1)?, // value
                    row.get::<_, f64>(2)?,    // ratio
                ))
            })?;

        let mut map: BTreeMap<String, Vec<AllocationCategoryValue>> = BTreeMap::new();
        for row in rows {
            let (category, value, ratio) = row?;
            map.entry(category)
                .or_default()
                .push(AllocationCategoryValue {
                    name: value,
                    ratio,
                });
        }

        let categories = map
            .into_iter()
            .map(|(name, values)| AllocationAssetCategory { name, values })
            .collect();

        Ok(AllocationAsset {
            name,
            reference: AssetReference {
                r#type: reference_type,
                value: reference_value,
            },
            categories,
        })
    }

    fn init_schema(&self) -> Result<(), Error> {
        self.connection
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS assets (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    reference_type TEXT NOT NULL,
                    reference_value TEXT NOT NULL
                );
                "#,
                [],
            )?;

        self.connection
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS asset_categories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL
                );
                "#,
                [],
            )?;

        self.connection.execute(
            r#"
            CREATE TABLE IF NOT EXISTS asset_category_values (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                asset_category_id INTEGER NOT NULL,
                name              TEXT NOT NULL,
                FOREIGN KEY (asset_category_id) REFERENCES asset_categories(id)
            )
            "#,
            [],
        )?;

        self.connection.execute(
            r#"
                CREATE TABLE IF NOT EXISTS asset_category_value_assignments (
                    asset_id INTEGER NOT NULL,
                    asset_category_value_id INTEGER NOT NULL,
                    ratio DECIMAL(5,4) CHECK (ratio >= 0 AND ratio <= 1) NOT NULL,
                    PRIMARY KEY (asset_id, asset_category_value_id),
                    FOREIGN KEY (asset_id) REFERENCES assets(id),
                    FOREIGN KEY (asset_category_value_id) REFERENCES asset_category_values(id)
                )
                "#,
            [],
        )?;

        Ok(())
    }
}

fn get_latest_allocation_record_paths(
    dir: &Path,
    limit: usize,
) -> Result<Vec<PathBuf>, Error> {
    if !dir.exists() {
        return Ok(Vec::new());
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
                    .is_some_and(|stem| {
                        jiff::civil::Date::strptime("%Y-%m-%d", stem).is_ok()
                    })
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

fn get_allocation_record(path: &Path) -> Result<AllocationRecord, Error> {
    Ok(ron::from_str(&fs::read_to_string(path)?)?)
}

impl AssetRepository for SqliteAssetRepository {
    fn get_latest_allocation_records(
        &self,
        limit: usize,
    ) -> Result<Vec<AllocationRecord>, Error> {
        get_latest_allocation_record_paths(Path::new(self.allocation_records_path.as_str()), limit)?
            .into_iter()
            .map(|path| get_allocation_record(&path))
            .collect()
    }

    fn get_category_values(&self, category_id: i64) -> Result<Vec<CategoryValue>, Error> {
        Ok(self.connection
            .prepare("
                SELECT id, asset_category_id, name
                FROM asset_category_values
                WHERE asset_category_id = ?
                ORDER BY name")
            .and_then(|mut stmt| {
                stmt.query_map(params![category_id], |row| {Ok(CategoryValue { id: row.get(0)?, name: row.get(2)? })})?
                    .collect()
            })?)
    }

    fn get_categories_without_values(&self) -> Result<Vec<Category>, Error> {
        Ok(self.connection
            .prepare("
                SELECT id, name
                FROM asset_categories
                ORDER BY name ASC")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| {Ok(Category { id: row.get(0)?, name: row.get(1)?, values: Vec::new() })})?
                    .collect()
            })?)
    }

    fn add_category_value(&mut self, category_id: i64, value_name: &str) -> Result<(), Error> {
        self.connection.execute(
            "INSERT INTO asset_category_values (asset_category_id, name)
            VALUES (?1, ?2)",
            rusqlite::params![category_id, value_name],
        )?;
        Ok(())
    }

    fn add_asset(&mut self, asset: &Asset, catgy_assignms: &Vec<CategoryAssignment>) -> Result<(), Error> {
        let tx = self.connection.transaction()?;
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
            tx.execute("
                INSERT INTO asset_category_value_assignments
                (asset_id, asset_category_value_id, ratio)
                VALUES (?1, ?2, ?3)",
                params![asset_id, assignm.value_id, assignm.ratio],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn add_category(&mut self, name: &str) -> Result<i64, Error> {
        self.connection
            .execute(
                "INSERT INTO asset_categories (name) VALUES (?1)",
                params![name],
            )?;
        Ok(self.connection.last_insert_rowid())
    }

    fn get_assets(&self) -> Result<Vec<Asset>, Error> {
        let mut stmt = self.connection
            .prepare(
                "SELECT id, name, reference_type, reference_value
                 FROM assets
                 ORDER BY name ASC"
            )?;

        let rows = stmt
            .query_map([], |row| {
                let reference_type_str: String = row.get(2)?;
                let reference_type: AssetReferenceType = reference_type_str.parse().unwrap();
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

    fn add_allocation_record(&mut self, record: &AllocationRecordInput) -> Result<(), Error>
    {
        let mut positions = Vec::new();
        for position in &record.positions {
            let asset = self.load_asset_ron(position.asset_id)?;
            positions.push(AllocationPosition {
                asset,
                amount: position.amount,
            });
        }
        let ron_record = AllocationRecord {
            date: record.date.to_string(),
            positions,
        };
        fs::create_dir_all(self.allocation_records_path.as_str())?;
        let path = Path::new(self.allocation_records_path.as_str())
            .join(format!("{}.ron", ron_record.date));

        let pretty = PrettyConfig::default();
        let ron = ron::ser::to_string_pretty(&ron_record, pretty)?;
        fs::write(path, ron)?;
        Ok(())
    }

    fn get_category_name_by_id(&self, category_id: i64) -> Result<String, Error> {
        Ok(self.connection.query_row(
            "SELECT name FROM asset_categories WHERE id = ?1",
            rusqlite::params![category_id],
            |row| row.get(0),
        )?)
    }
}