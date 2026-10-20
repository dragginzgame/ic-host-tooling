//! Borrowed external-package fields; callers retain lock admission and identity policy.

use toml::{Table, Value};

pub fn external_package(value: &Value) -> Result<Option<&Table>, &'static str> {
    let table = value.as_table().ok_or("package entry is not a table")?;
    Ok(table.contains_key("source").then_some(table))
}

pub fn required_string<'a>(table: &'a Table, field: &'static str) -> Result<&'a str, &'static str> {
    optional_string(table, field)?.ok_or(field)
}

pub fn optional_string<'a>(
    table: &'a Table,
    field: &'static str,
) -> Result<Option<&'a str>, &'static str> {
    table
        .get(field)
        .map(|value| value.as_str().ok_or(field))
        .transpose()
}
