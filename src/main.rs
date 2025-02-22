// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

#[macro_use]
extern crate rust_i18n;

use std::{process::ExitCode, sync::Arc, time::Duration};

use forgejo_api::Sort;
use tokio::{signal::ctrl_c, sync};
use tokio_util::sync::CancellationToken;

pub mod bots;
pub mod config;
pub mod db;
pub mod error;
pub mod forgejo_api;
pub mod inactive_users;
pub mod lazy_purge;
pub mod traits;
pub mod users_fetcher;
pub mod utils;

i18n!("locales", fallback = "en-us");

async fn try_main() -> error::GuardResult<()> {
    let config = Arc::new(utils::get_config()?);
    let cancellation_token = CancellationToken::new();
    // Suspicious users are sent and received in this channel, users who meet the
    // `alert` expressions
    let (sus_sender, sus_receiver) = sync::mpsc::channel::<bots::UserAlert>(100);
    // Banned users (already banned if `ban_alert`is set to true) and ban request
    // are sent and received in this channel, this to alert the admins on
    // Telegram and Matrix
    let (ban_sender, ban_receiver) = sync::mpsc::channel::<bots::UserAlert>(100);
    let database = Arc::new(db::init_db(&config.database)?);

    tracing::info!("Forgejo instance: {}", config.forgejo.instance);
    tracing::info!("Dry run: {}", config.dry_run);
    tracing::info!(
        "Inactive users checker enabled: {}",
        config.inactive.enabled
    );
    tracing::info!("Telegram enabled: {}", config.telegram.is_enabled());
    tracing::info!("Matrix enabled: {}", config.matrix.is_enabled());
    tracing::info!(
        "Ban expressions enabled: {}",
        config.expressions.ban.enabled
    );
    tracing::info!(
        "Sus expressions enabled: {}",
        config.expressions.sus.enabled
    );
    tracing::info!(
        config = "lazy_purge",
        "Lazy purge enabled: {}",
        config.lazy_purge.enabled
    );
    tracing::debug!("The config exprs: {:#?}", config.expressions);

    if config.inactive.enabled {
        tracing::info!(
            config = "inactive",
            "Consider inactive after: {} day{s}",
            config.inactive.days,
            s = if config.inactive.days >= 2 { "s" } else { "" }
        );
        tracing::info!(
            config = "inactive",
            "requests limit: {}",
            config.inactive.req_limit
        );
        tracing::info!(
            config = "inactive",
            "Interval when hitting the limit: {} seconds",
            config.inactive.req_interval
        );
        tracing::info!(
            config = "inactive",
            "Interval between each check: {} seconds",
            config.inactive.interval,
        );

        tokio::spawn(inactive_users::handler(
            Arc::clone(&config),
            cancellation_token.clone(),
        ));
    }

    if config.expressions.ban.enabled || config.expressions.sus.enabled {
        tracing::info!(
            config = "expressions",
            "Ban action: {}",
            config.expressions.ban_action
        );
        tracing::info!(
            config = "expressions",
            "Safe mode: {}",
            config.expressions.safe_mode
        );
        tracing::info!(
            config = "expressions",
            "check existing users: {}",
            config.expressions.check_existing_users
        );
        tracing::info!(
            config = "expressions",
            "Fetch updated users: {}",
            config.expressions.check_updated_users
        );
        tracing::info!(
            config = "expressions",
            "Interval between each fetch: {} seconds",
            config.expressions.interval
        );
        tracing::info!(
            config = "expressions",
            "Users to fetch per request: {}",
            config.expressions.limit
        );
        tracing::info!(
            config = "expressions",
            "Request limit for user fetcher: {}",
            config.expressions.req_limit
        );
        tracing::info!(
            config = "expressions",
            "Interval when hitting the limit for user fetcher: {} seconds",
            config.expressions.req_interval
        );

        tokio::spawn(users_fetcher::users_fetcher(
            Sort::Newest,
            Arc::clone(&config),
            Arc::clone(&database),
            cancellation_token.clone(),
            sus_sender.clone(),
            ban_sender.clone(),
        ));

        if config.expressions.check_updated_users {
            tokio::spawn(users_fetcher::users_fetcher(
                Sort::RecentUpdate,
                Arc::clone(&config),
                Arc::clone(&database),
                cancellation_token.clone(),
                sus_sender.clone(),
                ban_sender.clone(),
            ));
        }

        if config.expressions.check_existing_users {
            tokio::spawn(users_fetcher::old_users(
                Arc::clone(&config),
                Arc::clone(&database),
                ban_sender,
                sus_sender,
                cancellation_token.clone(),
            ));
        }

        if config.lazy_purge.enabled {
            tracing::info!(
                config = "lazy_purge",
                "Interval between each fetch: {} seconds",
                config.lazy_purge.interval
            );
            tracing::info!(
                config = "lazy_purge",
                "Request limit for user fetcher: {}",
                config.lazy_purge.req_limit
            );
            tracing::info!(
                config = "lazy_purge",
                "Interval when hitting the limit for user fetcher: {} seconds",
                config.lazy_purge.req_interval
            );
            tracing::info!(
                config = "lazy_purge",
                "Purge after: {} seconds",
                config.lazy_purge.purge_after
            );
            tokio::spawn(lazy_purge::worker(
                Arc::clone(&database),
                Arc::clone(&config),
                cancellation_token.clone(),
            ));
        }
    }

    bots::run_bots(
        Arc::clone(&database),
        Arc::clone(&config),
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    );

    tokio::select! {
        _ = ctrl_c() => {
            cancellation_token.cancel();
        }
        _ = cancellation_token.cancelled() => {}
    };

    tracing::info!("Waiting for graceful shutdown");
    tokio::time::sleep(Duration::from_secs(3)).await;

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_max_level(utils::get_log_level())
        .init();

    if let Err(err) = try_main().await {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
