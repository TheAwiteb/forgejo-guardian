// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{
    config::{Config, RegexReason},
    error::GuardResult,
    forgejo_api::{self, get_users, ForgejoUser},
    traits::ExprChecker,
};

/// Get the new instance users, the vector may be empty if there are no new
/// users
///
/// Forgejo use intger ids for the users, so we can use the last user id to get
/// the new users.
async fn get_new_users(
    request_client: &reqwest::Client,
    last_user_id: usize,
    config: &Config,
) -> GuardResult<Vec<ForgejoUser>> {
    Ok(get_users(
        request_client,
        &config.forgejo.instance,
        &config.forgejo.token,
        config.expressions.limit,
        1,
        "newest",
    )
    .await?
    .into_iter()
    .filter(|u| u.id > last_user_id)
    .collect())
}

/// Check if ban or suspect a new user
async fn check_new_user(
    user: ForgejoUser,
    request_client: &reqwest::Client,
    config: &Config,
    sus_sender: &Sender<(ForgejoUser, RegexReason)>,
    ban_sender: &Sender<(ForgejoUser, RegexReason)>,
) {
    if let Some(re) = config.expressions.ban.is_match(&user) {
        tracing::info!("@{} has been banned because `{re}`", user.username);
        if config.dry_run {
            // If it's a dry run, we don't need to ban the user
            if config.expressions.ban_alert {
                ban_sender.send((user, re)).await.ok();
            }
            return;
        }

        if let Err(err) = forgejo_api::ban_user(
            request_client,
            &config.forgejo.instance,
            &config.forgejo.token,
            &user.username,
            &config.expressions.ban_action,
        )
        .await
        {
            tracing::error!("Error while banning a user: {err}");
        } else if config.expressions.ban_alert {
            ban_sender.send((user, re)).await.ok();
        }
    } else if let Some(re) = config.expressions.sus.is_match(&user) {
        tracing::info!("@{} has been suspected because `{re}`", user.username);
        sus_sender.send((user, re)).await.ok();
    }
}

/// Check for new users and send the suspected users to the channel and ban the
/// banned users
async fn check_new_users(
    last_user_id: Arc<AtomicUsize>,
    request_client: Arc<reqwest::Client>,
    config: Arc<Config>,
    sus_sender: Sender<(ForgejoUser, RegexReason)>,
    ban_sender: Sender<(ForgejoUser, RegexReason)>,
) {
    let is_first_fetch = last_user_id.load(Ordering::Relaxed) == 0;
    match get_new_users(
        &request_client,
        last_user_id.load(Ordering::Relaxed),
        &config,
    )
    .await
    {
        Ok(new_users) => {
            if !new_users.is_empty() {
                tracing::debug!("Found {} new user(s)", new_users.len());
            }

            if let Some(uid) = new_users.iter().max_by_key(|u| u.id).map(|u| u.id) {
                tracing::debug!("New last user id: {uid}");
                last_user_id.store(uid, Ordering::Relaxed);
            }

            if config.expressions.only_new_users && is_first_fetch {
                return;
            }

            for user in new_users {
                check_new_user(user, &request_client, &config, &sus_sender, &ban_sender).await;
            }
        }
        Err(err) => {
            tracing::error!("Error while fetching new users: {err}");
        }
    }
}

/// The users fetcher, it will check for new users every period and send the
/// suspected users to the channel
pub async fn users_fetcher(
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    sus_sender: Sender<(ForgejoUser, RegexReason)>,
    ban_sender: Sender<(ForgejoUser, RegexReason)>,
) {
    let last_user_id = Arc::new(AtomicUsize::new(0));
    let request_client = Arc::new(reqwest::Client::new());

    tracing::info!("Starting users fetcher");
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.expressions.interval.into())) => {
                tokio::spawn(check_new_users(
                    Arc::clone(&last_user_id),
                    Arc::clone(&request_client),
                    Arc::clone(&config),
                    sus_sender.clone(),
                    ban_sender.clone(),
                ));
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("Users fetcher has been stopped successfully.");
                break
            }
        };
    }
}
