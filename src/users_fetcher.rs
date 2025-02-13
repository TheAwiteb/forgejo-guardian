// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use redb::Database;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{
    bots::UserAlert,
    config::Config,
    db::IgnoredUsersTableTrait,
    error::GuardResult,
    forgejo_api::{self, get_users, ForgejoUser},
    inactive_users,
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

/// Check if the user is protected from being banned
async fn is_user_protected(
    request_client: &reqwest::Client,
    config: &Config,
    user: &ForgejoUser,
    ban_sender: &Option<&Sender<UserAlert>>,
) -> GuardResult<bool> {
    Ok(config.expressions.safe_mode
        && ban_sender.is_some()
        && !inactive_users::is_inactive(
            request_client,
            &config.forgejo.instance,
            &config.forgejo.token,
            &user.username,
            config.inactive.check_tokens,
            config.inactive.check_oauth2,
        )
        .await?)
}

/// Check if ban or suspect a new user, returns the number of sended requests
async fn check_new_user(
    user: ForgejoUser,
    database: &Database,
    request_client: &reqwest::Client,
    config: &Config,
    overwrite_ban_alert: bool,
    sus_sender: Option<&Sender<UserAlert>>,
    ban_sender: Option<&Sender<UserAlert>>,
) -> u32 {
    if let Ok(true) = database.is_ignored(&user.username) {
        tracing::info!("Ignore an ignored user `@{}`", user.username);
        return 0;
    }

    if let Some(re) = config.expressions.ban.is_match(&user) {
        if is_user_protected(request_client, config, &user, &ban_sender)
            .await
            .unwrap_or_default()
        //  | ^^^^^
        //  | If there is an error don't send ban request
        {
            ban_sender
                .unwrap()
                .send(UserAlert::new(user, re).safe_mode())
                .await
                .ok();
            return 3;
        }

        tracing::info!("@{} has been banned because `{re}`", user.username);
        if config.dry_run {
            // If it's a dry run, we don't need to ban the user
            if config.expressions.ban_alert && ban_sender.is_some() {
                ban_sender
                    .unwrap()
                    .send(UserAlert::new(user, re))
                    .await
                    .ok();
            }
            return 0;
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
        } else if config.expressions.ban_alert && ban_sender.is_some() && !overwrite_ban_alert {
            ban_sender
                .unwrap()
                .send(UserAlert::new(user, re))
                .await
                .ok();
        }
        return if config.expressions.safe_mode && ban_sender.is_some() {
            4
        } else {
            1
        };
    } else if let Some(re) = sus_sender.and(config.expressions.sus.is_match(&user)) {
        tracing::info!("@{} has been suspected because `{re}`", user.username);
        sus_sender
            .unwrap()
            .send(UserAlert::new(user, re))
            .await
            .ok();
    }
    0
}

/// Check for new users and send the suspected users to the channel and ban the
/// banned users
async fn check_new_users(
    last_user_id: Arc<AtomicUsize>,
    request_client: Arc<reqwest::Client>,
    database: Arc<Database>,
    config: Arc<Config>,
    sus_sender: Sender<UserAlert>,
    ban_sender: Sender<UserAlert>,
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

            if is_first_fetch {
                return;
            }

            for user in new_users {
                check_new_user(
                    user,
                    &database,
                    &request_client,
                    &config,
                    false,
                    (config.telegram.is_enabled() || config.matrix.is_enabled())
                        .then_some(&sus_sender),
                    (config.telegram.is_enabled() || config.matrix.is_enabled())
                        .then_some(&ban_sender),
                )
                .await;
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
    database: Arc<Database>,
    cancellation_token: CancellationToken,
    sus_sender: Sender<UserAlert>,
    ban_sender: Sender<UserAlert>,
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
                    Arc::clone(&database),
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

/// Check for old users and ban them if they match the ban expressions. This
/// will not sned any alerts
pub async fn old_users(
    config: Arc<Config>,
    database: Arc<Database>,
    ban_sender: Sender<UserAlert>,
    cancellation_token: CancellationToken,
) {
    tracing::info!("Starting old users fetcher");

    let wait_interval = || {
        async {
            tracing::debug!(
                "Reached the request limit for old users checker. Waiting for {} seconds.",
                config.expressions.req_interval
            );
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(config.expressions.req_interval.into())) => false,
                _ = cancellation_token.cancelled() => true,
            }
        }
    };

    let client = reqwest::Client::new();
    let mut reqs = 0;
    let mut page = 1;

    'main_loop: loop {
        // Enter the block if we cancelled, so will break
        if reqs >= config.expressions.req_limit || cancellation_token.is_cancelled() {
            if wait_interval().await {
                break;
            }
            reqs = 0
        }
        reqs += 1;

        let Ok(users) = forgejo_api::get_users(
            &client,
            &config.forgejo.instance,
            &config.forgejo.token,
            config.expressions.limit,
            page,
            "newest",
        )
        .await
        else {
            tracing::error!("Falid to fetch old users");
            continue;
        };

        if users.is_empty() {
            tracing::info!("No more old users to check, all instance users are checked.");
            break;
        }

        for user in users {
            if (reqs + 4) > config.expressions.req_limit || cancellation_token.is_cancelled() {
                if wait_interval().await {
                    break 'main_loop;
                }
                reqs = 0;
            }

            reqs += check_new_user(
                user,
                &database,
                &client,
                &config,
                true,
                None,
                Some(&ban_sender),
            )
            .await;
        }

        page += 1;
    }
}
