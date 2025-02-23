// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::path::Path;

use redb::{Database, TableDefinition, TableHandle, WriteTransaction};

use crate::error::GuardResult;

mod alerted_users;
mod events;
mod ignored_users;
mod lazy_purge;

pub use alerted_users::*;
pub use events::*;
pub use ignored_users::*;
pub use lazy_purge::*;

/// Open a table in a write transaction, creating it if it doesn't exist.
fn open_table<K, V>(write_txn: &WriteTransaction, table: TableDefinition<K, V>) -> GuardResult<()>
where
    K: redb::Key,
    V: redb::Value,
{
    write_txn.open_table(table)?;
    Ok(())
}

/// Initialize the database, creating it if it doesn't exist.
pub fn init_db(db_path: &Path) -> GuardResult<Database> {
    let db = Database::create(db_path).map_err(redb::Error::from)?;
    let write_txn = db.begin_write()?;

    open_table(&write_txn, ALERTED_USERS_TABLE)?;
    open_table(&write_txn, EVENTS_TABLE)?;
    open_table(&write_txn, IGNORED_USERS_TABLE)?;
    open_table(&write_txn, PURGED_USERS_TABLE)?;

    tracing::info!(
        "Database tables: {:?}",
        write_txn
            .list_tables()?
            .map(|t| t.name().to_owned())
            .collect::<Vec<_>>()
    );

    write_txn.commit()?;
    Ok(db)
}
