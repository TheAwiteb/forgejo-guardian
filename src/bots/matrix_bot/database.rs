// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::path::Path;

use matrix_sdk::ruma::EventId;
use redb::{Database, TableDefinition};

use crate::error::{GuardError, GuardResult};

/// Events table, stores the event id and the username, the key is the event id
/// and the value is the username.
const EVENTS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("events");

fn redb_err<T>(result: Result<T, impl Into<redb::Error>>) -> GuardResult<T> {
    result.map_err(|err| GuardError::from(err.into()))
}

pub fn init_db(db_path: &Path) -> GuardResult<Database> {
    Ok(Database::create(db_path).map_err(redb::Error::from)?)
}

#[easy_ext::ext(EventsTableTrait)]
impl Database {
    // Add a new event to the database
    pub fn add_event(&self, event: &EventId, username: &str) -> GuardResult<()> {
        let write_txn = redb_err(self.begin_write())?;
        {
            let mut table = redb_err(write_txn.open_table(EVENTS_TABLE))?;
            redb_err(table.insert(event.to_string().as_str(), username))?;
        }
        redb_err(write_txn.commit())
    }

    /// Remove an event from the database
    pub fn remove_event(&self, event: &EventId) -> GuardResult<()> {
        let write_txn = redb_err(self.begin_write())?;
        {
            let mut table = redb_err(write_txn.open_table(EVENTS_TABLE))?;
            redb_err(table.remove(event.to_string().as_str()))?;
        }
        redb_err(write_txn.commit())
    }

    // Get the username of an event, if it exists
    pub fn get_username(&self, event: &EventId) -> GuardResult<Option<String>> {
        let read_txn = redb_err(self.begin_read())?;
        let table = redb_err(read_txn.open_table(EVENTS_TABLE))?;
        redb_err(
            table
                .get(event.to_string().as_str())
                .map(|o| o.map(|g| g.value().to_string())),
        )
    }
}
