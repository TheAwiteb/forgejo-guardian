// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::fmt;

/// The location where the regex got matched
#[derive(Debug, Clone)]
pub enum Locations {
    // This is the location before match it
    Unknown,
    Username,
    FullName,
    Biographie,
    Email,
    Website,
    Location,
}

impl fmt::Display for Locations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locations::Unknown => write!(f, "N/A"),
            Locations::Username => write!(f, "username"),
            Locations::FullName => write!(f, "full name"),
            Locations::Biographie => write!(f, "biography"),
            Locations::Email => write!(f, "email"),
            Locations::Website => write!(f, "website"),
            Locations::Location => write!(f, "location"),
        }
    }
}
