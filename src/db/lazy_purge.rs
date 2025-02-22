// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

#![allow(async_fn_in_trait)]

use std::time::{SystemTime, UNIX_EPOCH};

use redb::{Database, ReadableTable, TableDefinition};
use tokio_util::sync::CancellationToken;

use crate::{
    config::{BanAction, Config},
    db::{AlertedUsersTableTrait, EventsTableTrait},
    error::GuardResult,
    forgejo_api,
    utils,
};

/// A table containing purged users, with the username as the key and purged at
const PURGED_USERS_TABLE: TableDefinition<&str, u64> = TableDefinition::new("purged_users");

/// Returns the current timestamp
fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs()
}

#[easy_ext::ext(PurgedUsersTableTrait)]
impl Database {
    /// Add a new purged user to the database
    pub fn add_purged_user(&self, username: &str) -> GuardResult<()> {
        tracing::info!("Adding purged user: {username}");
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(PURGED_USERS_TABLE)?;
            table.insert(username, timestamp_now())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Remove purged user from the database
    pub fn remove_purged_user(&self, username: &str) -> GuardResult<()> {
        let write_txn = self.begin_write()?;
        {
            let mut table = write_txn.open_table(PURGED_USERS_TABLE)?;
            table.remove(username)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Returns `true` if the user is lazy purged
    pub fn is_layz_purged(&self, username: &str) -> GuardResult<bool> {
        let read_txn = self.begin_read()?;
        let table = read_txn.open_table(PURGED_USERS_TABLE)?;
        Ok(table.get(username).map(|o| o.is_some())?)
    }

    /// Purge existing users
    pub async fn purge_users(
        &self,
        client: &reqwest::Client,
        config: &Config,
        cancellation_token: CancellationToken,
    ) -> GuardResult<()> {
        let now = timestamp_now();
        let purge_after = u64::from(config.lazy_purge.purge_after);

        let mut reqs = 0;
        let mut total_purged = 0;

        let usernames: Vec<_> = {
            let read_txn = self.begin_read()?;
            let table = read_txn.open_table(PURGED_USERS_TABLE)?;
            table
                .iter()?
                .filter_map(|e| {
                    e.ok().and_then(|(u, p)| {
                        (now >= p.value() + purge_after).then(|| u.value().to_owned())
                    })
                })
                .collect()
        };

        tracing::info!("Starting lazy purge");
        for username in usernames {
            if reqs > config.lazy_purge.req_limit || cancellation_token.is_cancelled() {
                if utils::wait_interval(config.lazy_purge.req_interval, &cancellation_token).await {
                    break;
                }
                reqs = 0;
            }

            if config.dry_run {
                tracing::info!("User @{username} has been lazy purged");
            } else if let Err(err) = forgejo_api::ban_user(
                client,
                &config.forgejo.instance,
                &config.forgejo.token,
                &username,
                &BanAction::Purge,
            )
            .await
            {
                tracing::error!("Failed to lazy purge `@{username}`: {err}");
                continue;
            }

            self.remove_purged_user(&username).ok();
            self.remove_alerted_user(&username).ok();
            self.remove_user_events(&username).ok();
            total_purged += 1;
        }
        tracing::info!("Done lazy purge, purged {total_purged} users");
        Ok(())
    }
}
