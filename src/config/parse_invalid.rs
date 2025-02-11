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
/// Macro to use `check_key` function to check multiple keys in a table
///
/// ## Example
/// ```rust,no_run
/// checks! {
///     (table, "telegram")
///     ty="string"=>is_str; "token", "lang";
///     ty="number"=>is_integer; "chat";
/// };
/// ```
macro_rules! checks {
    (
        ($table:ident, $table_name:tt)
        $(ty=$keys_type:tt => $check_fn:ident, keys=[$($keys:tt),+]$(;)?);+
    ) => {
        let is_enabled = $table
            .get("enabled")
            .map_or_else(|| false, |v| v.as_bool().unwrap_or_default());

        if let Err(err) = check_key(
            $table_name,
            $table,
            "enabled",
            "boolean",
            Value::is_bool,
            true,
        ) {
            return err;
        }
        $(
            $(
                if let Err(err) = check_key(
                    $table_name,
                    $table,
                    $keys,
                    $keys_type,
                    Value::$check_fn,
                    is_enabled,
                ) {
                    return err;
                }
            )+
        )+
    };
}

/// Returns readable error messages for invalid telegram configuration.
///
/// In serde, untagged enums produce unclear error messages.
pub fn invalid_telegram(value: &Value) -> String {
    // check if it's a table
    let Some(table) = value.as_table() else {
        return format!("`telegram` must be a table, found `{}`", value);
    };

    checks! {
        (table, "telegram")
        ty="string"=>is_str, keys=["token", "lang"];
        ty="number"=>is_integer, keys=["chat"];
    }

    unreachable!(
        "The telegram configuration is invalid, all keys are checked and one of them should \
         returns the error message"
    );
}

pub fn invalid_matrix(value: &Value) -> String {
    // check if it's a table
    let Some(table) = value.as_table() else {
        return format!("`matrix` must be a table, found `{}`", value);
    };

    checks! {
        (table, "matrix")
        ty="string"=>is_str, keys=["host","username","password","room","lang"];
    }

    unreachable!(
        "The matrix configuration is invalid, all keys are checked and one of them should returns \
         the error message"
    );
}
