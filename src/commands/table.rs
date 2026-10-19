//! `bart-rs table` — dump a raw GTFS table.

use serde_json::{Map, Value};

use crate::cli::{Cli, TableArgs};
use crate::error::{Error, Result};

use super::print_table;

/// Print the rows of one of the feed's tables.
pub fn run(cli: &Cli, args: &TableArgs) -> Result<()> {
    if !crate::attributes::TABLE_NAMES.contains(&args.name.as_str()) {
        return Err(Error::msg(format!(
            "unknown table \"{}\"; expected one of: {}",
            args.name,
            crate::attributes::TABLE_NAMES.join(", ")
        )));
    }

    let path = crate::paths::current_feed_dir()?.join(format!("{}.txt", args.name));
    if !path.exists() {
        return Err(Error::FeedNotFound(path));
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_path(&path)
        .map_err(|e| Error::csv(&path, e))?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| Error::csv(&path, e))?
        .iter()
        .map(str::to_string)
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut json_rows: Vec<Value> = Vec::new();

    for record in reader.records() {
        let record = record.map_err(|e| Error::csv(&path, e))?;
        if rows.len() >= args.limit {
            break;
        }

        let values: Vec<String> = record.iter().map(str::to_string).collect();

        let mut object = Map::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            object.insert(header.clone(), Value::String(value.clone()));
        }
        json_rows.push(Value::Object(object));

        rows.push(values);
    }

    if cli.json {
        return super::print_json(&json_rows);
    }

    let header_refs: Vec<&str> = headers.iter().map(String::as_str).collect();
    print_table(&header_refs, &rows);
    Ok(())
}
