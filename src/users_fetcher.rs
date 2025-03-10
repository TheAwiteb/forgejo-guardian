// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{sync::Arc, time::Duration};

use redb::Database;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio::time::sleep as tokio_sleep;
use tokio_util::sync::CancellationToken;

use crate::inactive_users::is_inactive;
use crate::{
    bots::UserAlert,
    config::Config,
    db::{AlertedUsersTableTrait, IgnoredUsersTableTrait, PurgedUsersTableTrait},
    error::GuardResult,
    forgejo_api::{self, ForgejoUser, Sort},
    inactive_users,
    traits::ExprChecker,
    utils,
};

/// Maximum retries for fetching the users
const MAX_RETRIES: u64 = 10;
/// Base seconds for the retry interval
const RETRY_INTERVAL: u64 = 30;
/// Users count in recentupdate sort
const UPDATED_USERS_COUNT: u8 = 7;

/// Get the instance users, the vector may be empty if there are no users
///
/// Forgejo use intger ids for the users, so we can use the last user id to get
/// the users.
async fn get_users(
    sort: &Sort,
    request_client: &reqwest::Client,
    last_users_ids: &Mutex<Vec<usize>>,
    config: &Config,
    cancellation_token: CancellationToken,
) -> Vec<ForgejoUser> {
    let mut page = 1;
    let mut reqs = 0;
    let mut retries = 0;
    let mut found_last_updated = false;
    let mut users = Vec::new();

    loop {
        if reqs >= config.expressions.req_limit || cancellation_token.is_cancelled() {
            if utils::wait_interval(config.expressions.req_interval, &cancellation_token).await {
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
            sort,
        )
        .await
        {
            Ok(mut page_users) => {
                retries = 0;
                let users_ids_lock = last_users_ids.lock().await;
                let first_user_id = users_ids_lock.first().unwrap_or(&1);

                match sort {
                    Sort::Newest => {
                        page_users.retain(|u| &u.id > first_user_id);
                        page_users
                    }
                    Sort::RecentUpdate => {
                        let is_last_page = found_last_updated
                            || page_users.iter().any(|u| users_ids_lock.contains(&u.id));
                        let page_users: Vec<_> = page_users
                            .into_iter()
                            .take_while(|u| !found_last_updated && !users_ids_lock.contains(&u.id))
                            .collect();
                        found_last_updated = is_last_page;
                        page_users
                    }
                    _ => unreachable!("Oldest will not use this function"),
                }
            }
            Err(err) => {
                retries += 1;
                tracing::error!("Failed to fetch {sort} users page {page}: {err}");
                if retries >= MAX_RETRIES {
                    tracing::error!(
                        "Failed to fetch {sort} users page {page} after {MAX_RETRIES} retries."
                    );
                    return users;
                }
                tracing::info!("Retrying in {} seconds.", RETRY_INTERVAL * retries);
                tokio_sleep(Duration::from_secs(RETRY_INTERVAL * retries)).await;
                continue;
            }
        };
        if page_users.is_empty() {
            tracing::info!(
                "Done fetching all {sort} users, the total is {}",
                users.len()
            );
            break;
        }
        users.extend(page_users);
        page += 1;
    }
    users
}

/// Get the least user id from the instance. This will returns the last 7 users
/// ids for `recentupdate` sort
async fn get_least_users_ids(
    sort: &Sort,
    request_client: &reqwest::Client,
    config: &Config,
) -> GuardResult<Vec<usize>> {
    let limit = if sort.is_recent_update() {
        UPDATED_USERS_COUNT
    } else {
        1
    };

    let ids: Vec<_> = forgejo_api::get_users(
        request_client,
        &config.forgejo.instance,
        &config.forgejo.token,
        limit.into(),
        1,
        sort,
    )
    .await?
    .into_iter()
    .map(|u| u.id)
    .collect();

    if ids.is_empty() {
        return Ok(vec![1]);
    }

    Ok(ids)
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
            config.check_tokens,
            config.check_oauth2,
        )
        .await?)
}

/// Check if ban or suspect a user, returns the number of sended requests
#[allow(clippy::too_many_arguments)]
async fn check_user(
    sort: &str,
    user: ForgejoUser,
    database: &Database,
    request_client: &reqwest::Client,
    config: &Config,
    overwrite_ban_alert: bool,
    sus_sender: Option<&Sender<UserAlert>>,
    ban_sender: Option<&Sender<UserAlert>>,
) -> u32 {
    let username = user.username.clone();
    let (is_ignored, is_alerted, is_lazy_purged) = (
        database.is_ignored(&username).is_ok_and(|y| y),
        database.is_alerted(&username).is_ok_and(|y| y),
        database.is_lazy_purged(&username).is_ok_and(|y| y),
    );

    if is_ignored || is_alerted || is_lazy_purged {
        tracing::info!(
            "({sort}) Skipped {a_an} {reason} user `@{username}`",
            a_an = if is_lazy_purged { "a" } else { "an" },
            reason = if is_ignored {
                "ignored"
            } else if is_alerted {
                "alerted"
            } else {
                "lazy purged"
            }
        );
        return 0;
    }

    if let Some(re) = config.expressions.ban.is_match(&user) {
        if is_user_protected(request_client, config, &user, &ban_sender)
            .await
            .unwrap_or_default()
        {
            if !is_alerted {
                database.add_alerted_user(&username).ok();
                ban_sender
                    .unwrap()
                    .send(UserAlert::new(user, re).is_active(true))
                    .await
                    .ok();
            }
            return 3;
        }

        tracing::info!("({sort}) @{} has been banned because `{re}`", username);
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

        match forgejo_api::ban_user(
            request_client,
            &config.forgejo.instance,
            &config.forgejo.token,
            &username,
            &config.expressions.ban_action,
        )
        .await
        {
            Ok(_) => {
                if config.expressions.ban_alert && ban_sender.is_some() && !overwrite_ban_alert {
                    ban_sender
                        .unwrap()
                        .send(UserAlert::new(user, re))
                        .await
                        .ok();
                }
                database.remove_alerted_user(&username).ok();
            }
            Err(err) => {
                tracing::error!("({sort}) Error while banning a user: {err}");
            }
        }
        return if config.expressions.safe_mode && ban_sender.is_some() {
            4
        } else {
            1
        };
    } else if let Some(re) = sus_sender.and(config.expressions.sus.is_match(&user)) {
        tracing::info!("({sort}) @{} has been suspected because `{re}`", username);
        database.add_alerted_user(&username).ok();

        let is_active = config.expressions.active_sus_notice
            && !is_inactive(
                request_client,
                &config.forgejo.instance,
                &config.forgejo.token,
                &username,
                config.check_tokens,
                config.check_oauth2,
            )
            .await
            .is_ok_and(|y| y);

        sus_sender
            .unwrap()
            .send(UserAlert::new(user, re).is_active(is_active))
            .await
            .ok();

        if config.expressions.active_sus_notice {
            return 3;
        }
    }
    0
}

/// Check for users and send the suspected users to the channel and ban the
/// banned users
#[allow(clippy::too_many_arguments)]
async fn check_users(
    sort: Sort,
    last_users_ids: Arc<Mutex<Vec<usize>>>,
    request_client: Arc<reqwest::Client>,
    database: Arc<Database>,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    sus_sender: Sender<UserAlert>,
    ban_sender: Sender<UserAlert>,
) {
    let mut reqs = 0;
    let users = get_users(
        &sort,
        &request_client,
        &last_users_ids,
        &config,
        cancellation_token.clone(),
    )
    .await;

    if users.is_empty() {
        return;
    }

    {
        let mut ids = last_users_ids.lock().await;

        match sort {
            Sort::Newest => {
                if let Some(uid) = users.iter().max_by_key(|u| u.id).map(|u| u.id) {
                    tracing::debug!("{sort} last user id: {uid}");
                    ids.remove(0);
                    ids.push(uid);
                }
            }
            Sort::RecentUpdate => {
                *ids = users
                    .iter()
                    .map(|u| u.id)
                    .take(UPDATED_USERS_COUNT.into())
                    .collect();
            }
            _ => unreachable!(),
        }
    }

    for user in users {
        if (reqs + 4) > config.expressions.req_limit || cancellation_token.is_cancelled() {
            if utils::wait_interval(config.expressions.req_interval, &cancellation_token).await {
                break;
            }
            reqs = 0;
        }

        if (sort.is_recent_update() && user.is_new(config.expressions.interval)) || (user.is_admin)
        {
            continue;
        }

        reqs += check_user(
            sort.as_str(),
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

/// The users fetcher, it will check for users every period and send the
/// suspected users to the channel
pub async fn users_fetcher(
    sort: Sort,
    config: Arc<Config>,
    database: Arc<Database>,
    cancellation_token: CancellationToken,
    sus_sender: Sender<UserAlert>,
    ban_sender: Sender<UserAlert>,
) {
    let request_client = Arc::new(reqwest::Client::new());
    let last_users_ids =
        if let Ok(last_ids) = get_least_users_ids(&sort, &request_client, &config).await {
            Arc::new(Mutex::new(last_ids))
        } else {
            tracing::error!("Failed to get {sort} user id");
            return;
        };

    tracing::info!("Starting {sort} users fetcher");
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.expressions.interval.into())) => {
                tokio::spawn(check_users(
                    sort,
                    Arc::clone(&last_users_ids),
                    Arc::clone(&request_client),
                    Arc::clone(&database),
                    Arc::clone(&config),
                    cancellation_token.clone(),
                    sus_sender.clone(),
                    ban_sender.clone(),
                ));
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("{sort} users fetcher has been stopped successfully.");
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
    sus_sender: Sender<UserAlert>,
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
            if utils::wait_interval(config.expressions.req_interval, &cancellation_token).await {
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
            &Sort::Newest,
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
            tokio_sleep(Duration::from_secs(RETRY_INTERVAL * retries)).await;
            continue;
        };
        retries = 0;
        if users.is_empty() {
            tracing::info!("No more old users to check, all instance users are checked.");
            break;
        }

        for user in users {
            tokio_sleep(Duration::from_secs(2)).await;
            if (reqs + 4) > config.expressions.req_limit || cancellation_token.is_cancelled() {
                if utils::wait_interval(config.expressions.req_interval, &cancellation_token).await
                {
                    break 'main_loop;
                }
                reqs = 0;
            }

            reqs += check_user(
                "oldest",
                user,
                &database,
                &client,
                &config,
                true,
                config
                    .expressions
                    .check_sus_existing_users
                    .then_some(&sus_sender),
                Some(&ban_sender),
            )
            .await;
        }

        page += 1;
    }
}
