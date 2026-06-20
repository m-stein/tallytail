use core_lib::{
    AdaptCategoryInput, AllocationRecord, Asset, AssetReference, AssetReferenceType,
    CategoryAssignment, ConfigureCatgoriesInput, Currency, GetAllocDiagramDataArgs,
    ListedTransaction, NewCategoryInput, TransactionType, add_asset_args::AddAssetArgs,
    allocation_diagram_data::AllocationDiagramData, category::Category,
    category_value::CategoryValue, log_transaction_input::LogTransactionInput,
};
use eyre::eyre;
use rusqlite::{params, types::FromSqlError};
use rust_decimal::Decimal;
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

pub fn log_transaction(input: LogTransactionInput) -> eyre::Result<()> {
    let transaction = validate_log_transaction_input(input)?;
    let db_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/transactions.sdb");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = rusqlite::Connection::open(db_path)?;
    ensure_transactions_schema(&connection)?;
    insert_transaction(&connection, transaction)
}

pub fn list_transactions() -> eyre::Result<Vec<ListedTransaction>> {
    let db_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/transactions.sdb");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = rusqlite::Connection::open(db_path)?;
    ensure_transactions_schema(&connection)?;
    list_transactions_raw(&connection)
}

#[derive(Debug)]
struct Transaction {
    r#type: TransactionType,
    currency: Currency,
    date: jiff::civil::Date,
    isin: String,
    quantity: Decimal,
    share_price: Decimal,
    order_value: Decimal,
}

fn validate_log_transaction_input(input: LogTransactionInput) -> eyre::Result<Transaction> {
    if input.date > jiff::Zoned::now().date() {
        return Err(eyre!("Transaction date must not be in the future"));
    }
    let isin = normalize_isin(&input.isin)?;
    let quantity = parse_transaction_decimal("Quantity", &input.quantity)?;
    let share_price = parse_transaction_decimal("Share price", &input.share_price)?;
    let order_value = parse_transaction_decimal("Order value", &input.order_value)?;

    if quantity <= Decimal::ZERO {
        return Err(eyre!("Quantity must be greater than 0"));
    }
    if share_price <= Decimal::ZERO {
        return Err(eyre!("Share price must be greater than 0"));
    }
    let trade_value = quantity
        .checked_mul(share_price)
        .ok_or_else(|| eyre!("Quantity * share price is too large"))?;
    match input.r#type {
        TransactionType::Buy if order_value < trade_value => {
            return Err(eyre!(
                "Order value must be greater than or equal to quantity * share price"
            ));
        }
        TransactionType::Sell if order_value > trade_value => {
            return Err(eyre!(
                "Order value must be less than or equal to quantity * share price"
            ));
        }
        _ => {}
    }

    Ok(Transaction {
        r#type: input.r#type,
        currency: input.currency,
        date: input.date,
        isin,
        quantity,
        share_price,
        order_value,
    })
}

fn parse_transaction_decimal(field_name: &str, input: &str) -> eyre::Result<Decimal> {
    input
        .trim()
        .parse::<Decimal>()
        .map_err(|_| eyre!("{field_name} must be a valid decimal number"))
}

fn normalize_isin(input: &str) -> eyre::Result<String> {
    let isin = input.trim().to_ascii_uppercase();
    if !is_valid_isin(&isin) {
        return Err(eyre!("ISIN must be a valid 12-character ISIN"));
    }
    Ok(isin)
}

fn is_valid_isin(isin: &str) -> bool {
    let bytes = isin.as_bytes();
    if bytes.len() != 12 {
        return false;
    }
    if !bytes[0].is_ascii_uppercase() || !bytes[1].is_ascii_uppercase() {
        return false;
    }
    if !bytes[2..11].iter().all(u8::is_ascii_alphanumeric) || !bytes[11].is_ascii_digit() {
        return false;
    }
    let mut digits = Vec::with_capacity(24);
    for byte in bytes {
        if byte.is_ascii_digit() {
            digits.push(byte - b'0');
        } else if byte.is_ascii_uppercase() {
            let mut value = byte - b'A' + 10;
            digits.push(value / 10);
            value %= 10;
            digits.push(value);
        } else {
            return false;
        }
    }
    let mut sum = 0_u32;
    let mut double = false;
    for digit in digits.iter().rev() {
        let mut value = u32::from(*digit);
        if double {
            value *= 2;
            value = (value / 10) + (value % 10);
        }
        sum += value;
        double = !double;
    }
    sum.is_multiple_of(10)
}

fn ensure_transactions_schema(connection: &rusqlite::Connection) -> eyre::Result<()> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            isin TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS currencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS transaction_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS dates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date_id INTEGER NOT NULL,
            type_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            currency_id INTEGER NOT NULL,
            quantity TEXT NOT NULL,
            share_price TEXT NOT NULL,
            order_value TEXT NOT NULL,
            created_at_date_id INTEGER NOT NULL,
            created_at_time TEXT NOT NULL,
            FOREIGN KEY (date_id) REFERENCES dates(id),
            FOREIGN KEY (type_id) REFERENCES transaction_types(id),
            FOREIGN KEY (asset_id) REFERENCES assets(id),
            FOREIGN KEY (currency_id) REFERENCES currencies(id),
            FOREIGN KEY (created_at_date_id) REFERENCES dates(id)
        );
        ",
    )?;
    Ok(())
}

fn get_or_create_id(
    connection: &rusqlite::Connection,
    table_name: &str,
    column_name: &str,
    value: &str,
) -> eyre::Result<i64> {
    connection.execute(
        &format!("INSERT OR IGNORE INTO {table_name} ({column_name}) VALUES (?1)"),
        params![value],
    )?;
    let id = connection.query_row(
        &format!("SELECT id FROM {table_name} WHERE {column_name} = ?1"),
        params![value],
        |row| row.get(0),
    )?;
    Ok(id)
}

fn transaction_type_code(transaction_type: TransactionType) -> &'static str {
    match transaction_type {
        TransactionType::Buy => "BUY",
        TransactionType::Sell => "SELL",
    }
}

fn insert_transaction(
    connection: &rusqlite::Connection,
    transaction: Transaction,
) -> eyre::Result<()> {
    let asset_id = get_or_create_id(connection, "assets", "isin", &transaction.isin)?;
    let transaction_date = transaction.date.to_string();
    let date_id = get_or_create_id(connection, "dates", "date", &transaction_date)?;
    let now = jiff::Zoned::now();
    let created_at_date = now.date().to_string();
    let created_at_date_id = get_or_create_id(connection, "dates", "date", &created_at_date)?;
    let created_at_time = now.time().to_string();
    let currency_code = transaction.currency.to_string();
    let currency_id = get_or_create_id(connection, "currencies", "code", &currency_code)?;
    let type_id = get_or_create_id(
        connection,
        "transaction_types",
        "code",
        transaction_type_code(transaction.r#type),
    )?;

    connection.execute(
        "
        INSERT INTO transactions
            (
                date_id,
                type_id,
                asset_id,
                currency_id,
                quantity,
                share_price,
                order_value,
                created_at_date_id,
                created_at_time
            )
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            date_id,
            type_id,
            asset_id,
            currency_id,
            transaction.quantity.to_string(),
            transaction.share_price.to_string(),
            transaction.order_value.to_string(),
            created_at_date_id,
            created_at_time,
        ],
    )?;
    Ok(())
}

fn list_transactions_raw(
    connection: &rusqlite::Connection,
) -> eyre::Result<Vec<ListedTransaction>> {
    let mut statement = connection.prepare(
        "
        SELECT
            dates.date,
            transaction_types.code,
            assets.isin,
            transactions.quantity,
            transactions.share_price,
            transactions.order_value
        FROM transactions
        JOIN dates ON dates.id = transactions.date_id
        JOIN transaction_types ON transaction_types.id = transactions.type_id
        JOIN assets ON assets.id = transactions.asset_id
        ORDER BY dates.date DESC, transactions.id DESC
        LIMIT 50
        ",
    )?;
    let transactions = statement
        .query_map([], |row| {
            Ok(ListedTransaction {
                date: row.get(0)?,
                r#type: row.get(1)?,
                isin: row.get(2)?,
                quantity: row.get(3)?,
                share_price: row.get(4)?,
                order_value: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(transactions)
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

#[cfg(test)]
mod tests {
    use super::*;
    use core_lib::TransactionType;
    use jiff::Zoned;

    fn valid_input() -> LogTransactionInput {
        LogTransactionInput {
            r#type: TransactionType::Buy,
            currency: Currency::Eur,
            date: Zoned::now().date(),
            isin: "US0378331005".to_string(),
            quantity: "2.5".to_string(),
            share_price: "100.00".to_string(),
            order_value: "250.00".to_string(),
        }
    }

    #[test]
    fn accepts_valid_isin_with_check_digit() {
        assert!(is_valid_isin("US0378331005"));
    }

    #[test]
    fn rejects_invalid_isin_check_digit() {
        assert!(!is_valid_isin("US0378331006"));
    }

    #[test]
    fn rejects_future_transaction_date() {
        let mut input = valid_input();
        input.date = Zoned::now().tomorrow().unwrap().date();

        let err = validate_log_transaction_input(input)
            .unwrap_err()
            .to_string();

        assert!(err.contains("future"));
    }

    #[test]
    fn rejects_order_value_below_quantity_times_share_price() {
        let mut input = valid_input();
        input.order_value = "249.99".to_string();

        let err = validate_log_transaction_input(input)
            .unwrap_err()
            .to_string();

        assert!(err.contains("quantity * share price"));
    }

    #[test]
    fn accepts_sell_order_value_below_quantity_times_share_price() {
        let mut input = valid_input();
        input.r#type = TransactionType::Sell;
        input.order_value = "249.99".to_string();

        validate_log_transaction_input(input).unwrap();
    }

    #[test]
    fn rejects_sell_order_value_above_quantity_times_share_price() {
        let mut input = valid_input();
        input.r#type = TransactionType::Sell;
        input.order_value = "250.01".to_string();

        let err = validate_log_transaction_input(input)
            .unwrap_err()
            .to_string();

        assert!(err.contains("quantity * share price"));
    }

    #[test]
    fn rejects_quantity_less_than_or_equal_to_zero() {
        for quantity in ["0", "-1"] {
            let mut input = valid_input();
            input.quantity = quantity.to_string();

            let err = validate_log_transaction_input(input)
                .unwrap_err()
                .to_string();

            assert!(err.contains("Quantity must be greater than 0"));
        }
    }

    #[test]
    fn rejects_share_price_less_than_or_equal_to_zero() {
        for share_price in ["0", "-1"] {
            let mut input = valid_input();
            input.share_price = share_price.to_string();

            let err = validate_log_transaction_input(input)
                .unwrap_err()
                .to_string();

            assert!(err.contains("Share price must be greater than 0"));
        }
    }

    #[test]
    fn rejects_invalid_quantity_decimal_format() {
        let mut input = valid_input();
        input.quantity = "not-a-decimal".to_string();

        let err = validate_log_transaction_input(input)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Quantity must be a valid decimal number"));
    }

    #[test]
    fn rejects_invalid_share_price_decimal_format() {
        let mut input = valid_input();
        input.share_price = "not-a-decimal".to_string();

        let err = validate_log_transaction_input(input)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Share price must be a valid decimal number"));
    }

    #[test]
    fn rejects_invalid_order_value_decimal_format() {
        let mut input = valid_input();
        input.order_value = "not-a-decimal".to_string();

        let err = validate_log_transaction_input(input)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Order value must be a valid decimal number"));
    }
}
