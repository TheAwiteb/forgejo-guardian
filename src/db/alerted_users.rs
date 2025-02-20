// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use redb::{Database, TableDefinition};

use crate::error::GuardResult;

/// A table containing alerted users, with the username as the key and no value.
const ALERTED_USERS_TABLE: TableDefinition<&str, ()> = TableDefinition::new("alerted_users");

#[easy_ext::ext(AlertedUsersTableTrait)]
impl Database {
    /// Add a new alerted user to the database
    pub fn add_alerted_user(&self, username: &str) -> GuardResult<()> {
        tracing::info!("Adding alerted user: {username}");
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(ALERTED_USERS_TABLE)?;
            table.insert(username, ())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Remove alerted user, if exist
    pub fn remove_alerted_user(&self, username: &str) -> GuardResult<()> {
        tracing::info!("Removing alerted user: {username}");
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(ALERTED_USERS_TABLE)?;
            table.remove(username)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Returns `true` if the user already alerted to the moderation team
    pub fn is_alerted(&self, username: &str) -> GuardResult<bool> {
        let read_txn = self.begin_read()?;
        let table = read_txn.open_table(ALERTED_USERS_TABLE)?;
        Ok(table.get(username).map(|o| o.is_some())?)
    }
}
