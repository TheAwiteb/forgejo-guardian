// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::fmt;

use serde::{
    de::{self, Visitor},
    Deserialize,
    Deserializer,
};

/// A boolean that is always `true`.
pub struct True;
/// A boolean that is always `false`.
pub struct False;

/// A visitor for a boolean that is always `true` or `false`.
struct BoolVisitor<const BOOL: bool>;

impl<const BOOL: bool> Visitor<'_> for BoolVisitor<{ BOOL }> {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if BOOL {
            formatter.write_str("a `true` boolean")
        } else {
            formatter.write_str("a `false` boolean")
        }
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if BOOL && value {
            Ok(true)
        } else if !BOOL && !value {
            Ok(false)
        } else if BOOL {
            Err(de::Error::custom("expecting `true` boolean, found `false`"))
        } else {
            Err(de::Error::custom("expecting `false` boolean, found `true`"))
        }
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(BoolVisitor::<true>)?;
        Ok(Self)
    }
}

impl<'de> Deserialize<'de> for False {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(BoolVisitor::<false>)?;
        Ok(Self)
    }
}
