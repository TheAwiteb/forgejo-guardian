// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use matrix_sdk::ruma::EventId;
use redb::{Database, TableDefinition};

use crate::error::GuardResult;

/// Events table, stores the event id and the username, the key is the event id
/// and the value is the username.
const EVENTS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("events");

#[easy_ext::ext(EventsTableTrait)]
impl Database {
    // Add a new event to the database
    pub fn add_event(&self, event: &EventId, username: &str) -> GuardResult<()> {
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(EVENTS_TABLE)?;
            table.insert(event.to_string().as_str(), username)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Remove an event from the database
    pub fn remove_event(&self, event: &EventId) -> GuardResult<()> {
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(EVENTS_TABLE)?;
            table.remove(event.to_string().as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // Get the username of an event, if it exists
    pub fn get_username(&self, event: &EventId) -> GuardResult<Option<String>> {
        let read_txn = self.begin_read()?;
        let table = read_txn.open_table(EVENTS_TABLE)?;
        Ok(table
            .get(event.to_string().as_str())
            .map(|o| o.map(|g| g.value().to_string()))?)
    }
}
