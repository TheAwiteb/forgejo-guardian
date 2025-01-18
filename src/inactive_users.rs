// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use reqwest::{Client, Method};
use tokio_util::sync::CancellationToken;

use crate::{
    config::{BanAction, Config},
    forgejo_api::{self, ForgejoUser},
};

const LIMIT: u32 = 30;

/// Check if the user is inactive.
async fn check_user(req_client: &Client, config: &Config, user: ForgejoUser) -> usize {
    let res = req_client
        .execute(forgejo_api::build_request(
            Method::GET,
            &config.forgejo.instance,
            &config.forgejo.token,
            &format!("/api/v1/users/{}/heatmap", user.username),
        ))
        .await;
    if let Ok(res) = res {
        if res.text().await.unwrap_or_default().trim() == "[]" {
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
            }
            return 2; // heatmap and purge request
        }
    }
    1 // only heatmap request
}

/// Check all the instance users and delete the inactive ones.
async fn inactive_checker(
    cancellation_token: CancellationToken,
    req_client: &Client,
    config: &Config,
) {
    let mut reqs: usize = 0;
    let mut page = 1;
    let now = SystemTime::now();
    let days_secs = config.inactive.days * 24 * 60 * 60;
    loop {
        if reqs >= config.inactive.req_limit.into() {
            tracing::debug!(
                "Reached the request limit for inactive users checker. Waiting for {} seconds.",
                config.inactive.req_interval
            );
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(config.inactive.req_interval.into())) => {}
                _ = cancellation_token.cancelled() => break
            };

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
                    tracing::debug!("No more inactive users to check.");
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
        let requests_count: usize =
            futures::future::join_all(users.map(|u| check_user(req_client, config, u)))
                .await
                .into_iter()
                .sum();
        reqs += requests_count;
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
