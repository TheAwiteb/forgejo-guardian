// Simple Forgejo instance guardian, banning users and alerting admins based on
// certain regular expressions. Copyright (C) 2024 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://gnu.org/licenses/agpl.txt>.

#[macro_use]
extern crate rust_i18n;

use std::{process::ExitCode, sync::Arc, time::Duration};

use tokio::{signal::ctrl_c, sync};
use tokio_util::sync::CancellationToken;

pub mod config;
pub mod error;
pub mod forgejo_api;
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
    let (sus_sender, sus_receiver) = sync::mpsc::channel::<forgejo_api::ForgejoUser>(100);
    // Banned users (already banned) are sent and received in this channel, this
    // to alert the admins on Telegram if `ban_alert` is set to true
    let (ban_sender, ban_receiver) = sync::mpsc::channel::<forgejo_api::ForgejoUser>(100);

    tracing::info!("The instance: {}", config.forgejo.instance);
    tracing::info!("Dry run: {}", config.dry_run);
    tracing::debug!("The config exprs: {:#?}", config.expressions);

    rust_i18n::set_locale(config.telegram.lang.as_str());

    tokio::spawn(users_fetcher::users_fetcher(
        Arc::clone(&config),
        cancellation_token.clone(),
        sus_sender,
        ban_sender,
    ));

    tokio::spawn(telegram_bot::start_bot(
        Arc::clone(&config),
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    ));

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
