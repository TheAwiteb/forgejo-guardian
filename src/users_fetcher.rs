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
use tokio::time::sleep as tokio_sleep;
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

/// Maximum retries for fetching the users
const MAX_RETRIES: u8 = 10;
/// Base seconds for the retry interval
const RETRY_INTERVAL: u8 = 30;

/// Wait for the interval to pass, if the cancellation token is cancelled,
/// return true, after the interval has passed return false
async fn wait_interval(req_interval: u32, cancellation_token: &CancellationToken) -> bool {
    tracing::debug!(
        "Reached the request limit for old users checker. Waiting for {req_interval} seconds.",
    );
    tokio::select! {
        _ = tokio_sleep(Duration::from_secs(req_interval.into())) => false,
        _ = cancellation_token.cancelled() => true,
    }
}

/// Get the new instance users, the vector may be empty if there are no new
/// users
///
/// Forgejo use intger ids for the users, so we can use the last user id to get
/// the new users.
async fn get_new_users(
    request_client: &reqwest::Client,
    last_user_id: usize,
    config: &Config,
    cancellation_token: CancellationToken,
) -> Vec<ForgejoUser> {
    let mut page = 1;
    let mut reqs = 0;
    let mut retries = 0;
    let mut users = Vec::new();

    loop {
        if reqs >= config.expressions.req_limit || cancellation_token.is_cancelled() {
            if wait_interval(config.expressions.req_interval, &cancellation_token).await {
                break;
            }
            reqs = 0;
        }
        reqs += 1;
        let page_users = match forgejo_api::get_users(
            request_client,
            &config.forgejo.instance,
            &config.forgejo.token,
            config.expressions.limit,
            page,
            "newest",
        )
        .await
        {
            Ok(mut page_users) => {
                retries = 0;
                page_users.retain(|u| u.id > last_user_id);
                page_users
            }
            Err(err) => {
                retries += 1;
                tracing::error!("Failed to fetch new users page {page}: {err}");
                if retries >= MAX_RETRIES {
                    tracing::error!(
                        "Failed to fetch new users page {page} after {MAX_RETRIES} retries."
                    );
                    return users;
                }
                tracing::info!("Retrying in {} seconds.", RETRY_INTERVAL * retries);
                tokio_sleep(Duration::from_secs((RETRY_INTERVAL * retries).into())).await;
                continue;
            }
        };
        if page_users.is_empty() {
            tracing::info!("Done fetching all new users, the total is {}", users.len());
            break;
        }
        users.extend(page_users);
        page += 1;
    }
    users
}

/// Get the newest user id from the instance
async fn get_newest_user_id(
    request_client: &reqwest::Client,
    config: &Config,
) -> GuardResult<usize> {
    Ok(get_users(
        request_client,
        &config.forgejo.instance,
        &config.forgejo.token,
        1,
        1,
        "newest",
    )
    .await?
    .first()
    .map(|u| u.id)
    .unwrap_or(1))
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
    cancellation_token: CancellationToken,
    sus_sender: Sender<UserAlert>,
    ban_sender: Sender<UserAlert>,
) {
    let mut reqs = 0;
    let new_users = get_new_users(
        &request_client,
        last_user_id.load(Ordering::Relaxed),
        &config,
        cancellation_token.clone(),
    )
    .await;

    if new_users.is_empty() {
        return;
    }

    if let Some(uid) = new_users.iter().max_by_key(|u| u.id).map(|u| u.id) {
        tracing::debug!("New last user id: {uid}");
        last_user_id.store(uid, Ordering::Relaxed);
    }

    for user in new_users {
        if (reqs + 4) > config.expressions.req_limit || cancellation_token.is_cancelled() {
            if wait_interval(config.expressions.req_interval, &cancellation_token).await {
                break;
            }
            reqs = 0;
        }
        reqs += check_new_user(
            user,
            &database,
            &request_client,
            &config,
            false,
            (config.telegram.is_enabled() || config.matrix.is_enabled()).then_some(&sus_sender),
            (config.telegram.is_enabled() || config.matrix.is_enabled()).then_some(&ban_sender),
        )
        .await;
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
    let request_client = Arc::new(reqwest::Client::new());
    let last_user_id = if let Ok(last_id) = get_newest_user_id(&request_client, &config).await {
        Arc::new(AtomicUsize::new(last_id))
    } else {
        tracing::error!("Failed to get newest user id");
        return;
    };

    tracing::info!("Starting users fetcher");
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.expressions.interval.into())) => {
                tokio::spawn(check_new_users(
                    Arc::clone(&last_user_id),
                    Arc::clone(&request_client),
                    Arc::clone(&database),
                    Arc::clone(&config),
                    cancellation_token.clone(),
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

    let client = reqwest::Client::new();
    let mut retries = 0;
    let mut reqs = 0;
    let mut page = 1;

    'main_loop: loop {
        // Enter the block if we cancelled, so will break
        if reqs >= config.expressions.req_limit || cancellation_token.is_cancelled() {
            if wait_interval(config.expressions.req_interval, &cancellation_token).await {
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
            tracing::error!("Failed to fetch old users");
            retries += 1;
            if retries >= MAX_RETRIES {
                tracing::error!("Failed to fetch old users after {MAX_RETRIES} retries.");
                return;
            }
            tracing::info!("Retrying in {} seconds.", RETRY_INTERVAL * retries);
            tokio_sleep(Duration::from_secs((RETRY_INTERVAL * retries).into())).await;
            continue;
        };
        retries = 0;
        if users.is_empty() {
            tracing::info!("No more old users to check, all instance users are checked.");
            break;
        }

        for user in users {
            if (reqs + 4) > config.expressions.req_limit || cancellation_token.is_cancelled() {
                if wait_interval(config.expressions.req_interval, &cancellation_token).await {
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
