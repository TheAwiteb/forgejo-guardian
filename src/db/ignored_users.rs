// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use redb::{Database, TableDefinition};

use crate::error::GuardResult;

/// A table containing ignored users, with the username as the key and no value.
pub(super) const IGNORED_USERS_TABLE: TableDefinition<&str, ()> =
    TableDefinition::new("ignored_users");

#[easy_ext::ext(IgnoredUsersTableTrait)]
impl Database {
    /// Add a new ignored user to the database
    pub fn add_ignored_user(&self, username: &str) -> GuardResult<()> {
        tracing::info!("Adding ignored user: {username}");
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(IGNORED_USERS_TABLE)?;
            table.insert(username, ())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Returns `true` if the user exists
    pub fn is_ignored(&self, username: &str) -> GuardResult<bool> {
        let read_txn = self.begin_read()?;
        let table = read_txn.open_table(IGNORED_USERS_TABLE)?;
        Ok(table.get(username).map(|o| o.is_some())?)
    }
}
