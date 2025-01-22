// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use reqwest::Client;
use tokio_util::sync::CancellationToken;

use crate::{
    config::{BanAction, Config},
    forgejo_api::{self, ForgejoUser},
};

const LIMIT: u32 = 30;

/// Check if the user is inactive.
async fn check_user(req_client: &Client, config: &Config, user: ForgejoUser) -> usize {
    if user.is_admin
        || config.inactive.exclude.contains(&user.username)
        || config.inactive.source_id_exclude.contains(&user.source_id)
        || (!config.inactive.source_id.is_empty()
            && !config.inactive.source_id.contains(&user.source_id))
    {
        tracing::info!(
            "{admin_user} `@{}` with source_id `{}` is excluded from the inactive check.",
            user.username,
            user.source_id,
            admin_user = if user.is_admin { "Admin" } else { "User" }
        );
        return 0;
    }

    match forgejo_api::is_empty_feeds(
        req_client,
        &config.forgejo.instance,
        &config.forgejo.token,
        &user.username,
    )
    .await
    {
        Ok(true) => {
            tracing::info!("User `@{}` is inactive.", user.username);
            if !config.dry_run {
                if let Err(err) = forgejo_api::ban_user(
                    req_client,
                    &config.forgejo.instance,
                    &config.forgejo.token,
                    &user.username,
                    &BanAction::Purge,
                )
                .await
                {
                    tracing::error!("Error while ban inactive user `@{}`: {err}", user.username);
                }
                return 2; // heatmap and purge request}
            }
        }
        Err(err) => {
            tracing::error!("{err}");
        }
        _ => {}
    }

    1 // only heatmap request
}

/// Check all the instance users and delete the inactive ones.
async fn inactive_checker(
    cancellation_token: CancellationToken,
    req_client: &Client,
    config: &Config,
) {
    let wait_interval = || {
        async {
            tracing::debug!(
                "Reached the request limit for inactive users checker. Waiting for {} seconds.",
                config.inactive.req_interval
            );
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(config.inactive.req_interval.into())) => false,
                _ = cancellation_token.cancelled() => true,
            }
        }
    };

    let mut reqs: usize = 0;
    let mut page = 1;
    let now = SystemTime::now();
    let days_secs = config.inactive.days * 24 * 60 * 60;
    'main_loop: loop {
        if reqs >= config.inactive.req_limit.into() {
            if wait_interval().await {
                break;
            }

            reqs = 0;
        }
        if cancellation_token.is_cancelled() {
            break;
        }

        reqs += 1;
        let users = forgejo_api::get_users(
            req_client,
            &config.forgejo.instance,
            &config.forgejo.token,
            LIMIT,
            page,
            "oldest",
        )
        .await;
        let users = match users {
            Ok(users) => {
                if users.is_empty() {
                    tracing::info!("No more inactive users to check.");
                    break;
                }
                users.into_iter().filter(|u| {
                    let Ok(since_now) = now.duration_since(u.created.into()) else {
                        return false;
                    };
                    since_now.as_secs() >= days_secs
                })
            }
            Err(err) => {
                tracing::error!("Error while fetching users: {err}");
                break;
            }
        };
        for user in users {
            // +2 Because the next check need 1~2 requests
            if (reqs + 2) > config.inactive.req_limit.into() {
                if wait_interval().await {
                    tracing::warn!("Inactive users checker stopped while checking users.");
                    break 'main_loop;
                }
                reqs = 0
            }
            reqs += check_user(req_client, config, user).await;
        }
        page += 1;
    }
}

/// The handler for the inactive users checker.
pub async fn handler(config: Arc<Config>, cancellation_token: CancellationToken) {
    tracing::info!("Starting inactive users checker");
    let request_client = Arc::new(reqwest::Client::new());

    // Run the first check, then wait for the interval.
    // Because the first check is not dependent on the interval.
    inactive_checker(cancellation_token.clone(), &request_client, &config).await;
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.inactive.interval.into())) => {
                inactive_checker(cancellation_token.clone(), &request_client, &config).await;
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("Inactive users checker has been stopped successfully.");
                break
            }
        };
    }
}
