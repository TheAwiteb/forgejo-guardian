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

use std::{process::ExitCode, sync::Arc, time::Duration};

use tokio::{signal::ctrl_c, sync};
use tokio_util::sync::CancellationToken;

pub mod config;
pub mod error;
pub mod forgejo_api;
pub mod traits;
pub mod users_fetcher;
pub mod utils;

async fn try_main() -> error::GuardResult<()> {
    let config = Arc::new(utils::get_config()?);
    let cancellation_token = CancellationToken::new();
    // Suspicious users are sent and received in this channel, users who meet the
    // `alert` expressions
    let (sus_sender, _sus_receiver) = sync::mpsc::channel::<forgejo_api::ForgejoUser>(100);

    tracing::info!("The instance: {}", config.forgejo.instance);
    tracing::debug!("The config exprs: {:#?}", config.expressions);

    tokio::spawn(users_fetcher::users_fetcher(
        Arc::clone(&config),
        cancellation_token.clone(),
        sus_sender.clone(),
    ));

    // TODO: Sus worker, who will receive sus users

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
