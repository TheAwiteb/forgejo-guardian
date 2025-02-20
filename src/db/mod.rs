// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::path::Path;

use redb::Database;

use crate::error::GuardResult;

mod alerted_users;
mod events;
mod ignored_users;

pub use alerted_users::*;
pub use events::*;
pub use ignored_users::*;

/// Initialize the database, creating it if it doesn't exist.
pub fn init_db(db_path: &Path) -> GuardResult<Database> {
    Ok(Database::create(db_path).map_err(redb::Error::from)?)
}
