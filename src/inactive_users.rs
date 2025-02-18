// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use reqwest::Client;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::{
    config::{BanAction, Config},
    error::GuardResult,
    forgejo_api::{self, ForgejoUser, Sort},
};

const LIMIT: u32 = 30;

/// Returns true if the user has no tokens or `is_enabled` is false.
///
/// If there is an error while fetching the tokens, it will return false.
async fn is_empty_tokens(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
    is_enabled: bool,
) -> bool {
    if !is_enabled {
        return true;
    }

    match forgejo_api::is_empty_tokens(client, instance, token, username).await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!("Error while get user `@{}` tokens: {err}", username);
            false
        }
    }
}

/// Returns true if the user has no apps or `is_enabled` is false.
///
/// If there is an error while fetching the apps, it will return false.
async fn is_empty_apps(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
    is_enabled: bool,
) -> bool {
    if !is_enabled {
        return true;
    }

    match forgejo_api::is_empty_apps(client, instance, token, username).await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!("Error while get user `@{}` tokens: {err}", username);
            false
        }
    }
}

/// Returns true if the tokens and apps are empty
async fn is_empty_tokens_and_apps(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
    tokens_enabled: bool,
    apps_enabled: bool,
) -> bool {
    is_empty_tokens(client, instance, token, username, tokens_enabled).await
        && is_empty_apps(client, instance, token, username, apps_enabled).await
}

/// Returns true if the user is inactive
pub async fn is_inactive(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
    tokens_enabled: bool,
    apps_enabled: bool,
) -> GuardResult<bool> {
    Ok(
        forgejo_api::is_empty_feeds(client, instance, token, username).await?
            && is_empty_tokens_and_apps(
                client,
                instance,
                token,
                username,
                tokens_enabled,
                apps_enabled,
            )
            .await,
    )
}

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

    match is_inactive(
        req_client,
        &config.forgejo.instance,
        &config.forgejo.token,
        &user.username,
        config.inactive.check_tokens,
        config.inactive.check_oauth2,
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
                // activity feed, purge request and tokens (if sended)
                return 2
                    + usize::from(config.inactive.check_tokens)
                    + usize::from(config.inactive.check_oauth2);
            }
        }
        Err(err) => {
            tracing::error!("{err}");
        }
        _ => {}
    }

    1 // only activity feed request
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
            &Sort::Oldest,
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
            if (reqs + 4) > config.inactive.req_limit.into() {
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
