// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

#[macro_use]
extern crate rust_i18n;

use std::{process::ExitCode, sync::Arc, time::Duration};

use tokio::{signal::ctrl_c, sync};
use tokio_util::sync::CancellationToken;

pub mod config;
pub mod error;
pub mod forgejo_api;
pub mod inactive_users;
pub mod telegram_bot;
pub mod traits;
pub mod users_fetcher;
pub mod utils;

i18n!("locales", fallback = "en-us");

async fn try_main() -> error::GuardResult<()> {
    let config = Arc::new(utils::get_config()?);
    let cancellation_token = CancellationToken::new();
    // Suspicious users are sent and received in this channel, users who meet the
    // `alert` expressions
    let (sus_sender, sus_receiver) =
        sync::mpsc::channel::<(forgejo_api::ForgejoUser, config::RegexReason)>(100);
    // Banned users (already banned) are sent and received in this channel, this
    // to alert the admins on Telegram if `ban_alert` is set to true
    let (ban_sender, ban_receiver) =
        sync::mpsc::channel::<(forgejo_api::ForgejoUser, config::RegexReason)>(100);

    tracing::info!("Forgejo instance: {}", config.forgejo.instance);
    tracing::info!("Dry run: {}", config.dry_run);
    tracing::info!(
        "Inactive users checker enabled: {}",
        config.inactive.enabled
    );
    tracing::info!(
        "Telegram enabled: {}",
        config.telegram.is_enabled().is_some()
    );
    tracing::info!(
        "Ban expressions enabled: {}",
        config.expressions.ban.enabled
    );
    tracing::info!(
        "Sus expressions enabled: {}",
        config.expressions.sus.enabled
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
            "Only fetch new users: {}",
            config.expressions.only_new_users
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

        if config.expressions.sus.enabled && config.telegram.is_enabled().is_none() {
            tracing::warn!(
                "The suspicious users expressions are enabled but the Telegram bot is disabled, \
                 the suspicious users will not be alerted"
            );
        }

        tokio::spawn(users_fetcher::users_fetcher(
            Arc::clone(&config),
            cancellation_token.clone(),
            sus_sender,
            ban_sender,
        ));

        if !config.expressions.only_new_users {
            tracing::info!(
                config = "expressions",
                "Request limit for old users: {}",
                config.expressions.req_limit
            );
            tracing::info!(
                config = "expressions",
                "Interval when hitting the limit for old users: {} seconds",
                config.expressions.req_interval
            );

            tokio::spawn(users_fetcher::old_users(
                Arc::clone(&config),
                cancellation_token.clone(),
            ));
        }
    }

    if let Some(telegram) = config.telegram.is_enabled() {
        tracing::info!(config = "telegram", "Bot lang: {}", telegram.lang.as_str());
        tracing::info!(config = "telegram", "Receiver chat ID: {}", telegram.chat);

        rust_i18n::set_locale(telegram.lang.as_str());

        tokio::spawn(telegram_bot::start_bot(
            Arc::clone(&config),
            telegram.clone(),
            cancellation_token.clone(),
            sus_receiver,
            ban_receiver,
        ));
    }

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
