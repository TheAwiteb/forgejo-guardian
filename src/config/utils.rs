// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use serde::de;

/// Function to create a custom error for deserialization from its string
pub fn custom_de_err<'de, D>(err: impl ToString) -> D::Error
where
    D: de::Deserializer<'de>,
{
    de::Error::custom(err.to_string())
}
