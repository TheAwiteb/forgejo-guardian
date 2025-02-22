// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{sync::Arc, time::Duration};

use redb::Database;
use tokio_util::sync::CancellationToken;

use crate::{config::Config, db::PurgedUsersTableTrait};

/// Purge purged users
pub async fn purge_purged_users(
    database: &Database,
    config: &Config,
    request_client: &reqwest::Client,
    cancellation_token: CancellationToken,
) {
    if let Err(err) = database
        .purge_users(request_client, config, cancellation_token)
        .await
    {
        tracing::error!("Failed to purge users: {err}")
    };
}

/// The lazy purge worker
pub async fn worker(
    database: Arc<Database>,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
) {
    tracing::info!("Starting lazy purge worker");
    let request_client = Arc::new(reqwest::Client::new());

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.lazy_purge.interval.into())) => {
                purge_purged_users(&database, &config, &request_client, cancellation_token.clone()).await;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("Lazy purged worker has been stopped successfully.");
                break
            }
        };
    }
}
