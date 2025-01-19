// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use toml::Value;

/// Check if a key is present in a table and if it's the correct type
fn check_key(
    table_name: &str,
    table: &toml::Table,
    key: &str,
    ty: &str,
    ty_fn: impl FnOnce(&toml::Value) -> bool,
    is_required: bool,
) -> Result<(), String> {
    if let Some(value) = table.get(key) {
        if !ty_fn(value) {
            return Err(format!(
                "`{table_name}.{key}` must be a {ty}, found `{value}`"
            ));
        }
    } else if is_required {
        return Err(format!(
            "Missing key `{table_name}.{key}`, it must be a {ty}"
        ));
    }
    Ok(())
}

/// Returns readable error messages for invalid telegram configuration.
///
/// In serde, untagged enums produce unclear error messages.
pub fn invalid_telegram(value: &Value) -> String {
    // check if it's a table
    let Some(table) = value.as_table() else {
        return format!("`telegram` must be a table, found `{}`", value);
    };
    let is_enabled = table
        .get("enabled")
        .map_or_else(|| false, |v| v.as_bool().unwrap_or_default());
    // check enabled field
    if let Err(err) = check_key(
        "telegram",
        table,
        "enabled",
        "boolean",
        Value::is_bool,
        true,
    ) {
        return err;
    }
    // check token
    if let Err(err) = check_key(
        "telegram",
        table,
        "token",
        "string",
        Value::is_str,
        is_enabled,
    ) {
        return err;
    }
    // check chat
    if let Err(err) = check_key(
        "telegram",
        table,
        "chat",
        "number",
        Value::is_integer,
        is_enabled,
    ) {
        return err;
    }
    // check lang
    if let Err(err) = check_key(
        "telegram",
        table,
        "lang",
        "string",
        Value::is_str,
        is_enabled,
    ) {
        return err;
    }

    unreachable!(
        "The telegram configuration is invalid, all keys are checked and one of them should \
         returns the error message"
    );
}
