// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{borrow::Cow, sync::Arc, time::Duration};

use matrix_sdk::{config::SyncSettings, Client as MatrixClient, Room};
use redb::Database;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

mod handlers;
mod matrix_api;
mod users_handler;
mod utils;

use super::UserAlert;
use crate::{
    config::{Config, MatrixData},
    error::{GuardError, GuardResult},
};

/// Maximum retries for matrix sync
const MAX_RETRIES: u64 = 10;
/// Base seconds for the retry interval
const RETRY_INTERVAL: u64 = 5;

#[derive(Clone)]
pub struct MatrixBot {
    client:          MatrixClient,
    config:          Arc<Config>,
    db:              Arc<Database>,
    moderation_room: Room,
}

impl MatrixBot {
    /// Create a new matrix bot
    pub async fn new(
        config: Arc<Config>,
        matrix: &MatrixData,
        database: Arc<Database>,
    ) -> GuardResult<Self> {
        let client = MatrixClient::builder()
            .homeserver_url(&matrix.homeserver)
            .build()
            .await
            .map_err(|err| GuardError::Other(err.to_string()))?;

        client
            .matrix_auth()
            .login_username(&matrix.username, &matrix.password)
            .initial_device_display_name("Forgejo Guardian <git.4rs.nl/awiteb/forgejo-guardian>")
            .await?;

        client.sync_once(SyncSettings::new()).await?;

        let moderation_room = client
            .get_room(&matrix.room)
            .ok_or_else(|| GuardError::Matrix("Falied to get the moderation room".to_owned()))?;

        Ok(Self {
            config,
            client,
            db: database,
            moderation_room,
        })
    }

    /// Returns the ban reaction
    pub fn ban_reaction(&self) -> Cow<'_, str> {
        t!("buttons.ban", action = self.config.expressions.ban_action)
    }

    /// Returns the ignore reaction
    pub fn ignore_reaction(&self) -> Cow<'_, str> {
        t!("buttons.ignore")
    }

    /// Returns the undo reaction
    pub fn undo_reaction(&self) -> Cow<'_, str> {
        t!("buttons.undo")
    }

    /// Run the matrix bot, this will join the moderation room and start
    /// listening to events
    pub async fn run(self, cancellation_token: CancellationToken) {
        let mut retries = 0;
        let client = self.client.clone();
        for room in client.invited_rooms() {
            if room.room_id() == self.moderation_room.room_id() {
                room.join()
                    .await
                    .expect("Could not join the moderation room");
            }
        }

        client.add_room_event_handler(self.moderation_room.room_id(), Self::on_room_message);
        client.add_room_event_handler(self.moderation_room.room_id(), Self::on_room_reaction);
        client.add_event_handler_context(self);

        loop {
            if cancellation_token.is_cancelled() || retries > MAX_RETRIES {
                break;
            }

            if let Err(err) = client.sync(SyncSettings::default()).await {
                retries += 1;
                tracing::error!(
                    "Falied to sync the matrix bot (retries {retries}/{MAX_RETRIES}): {err}"
                )
            }
            tracing::info!("Retrying in {} seconds.", RETRY_INTERVAL * retries);
            tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL * retries)).await;
        }
    }
}

/// Start the matrix bot
pub async fn start_bot(
    database: Arc<Database>,
    config: Arc<Config>,
    matrix: MatrixData,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<UserAlert>,
    ban_receiver: Receiver<UserAlert>,
) {
    tracing::info!("Starting the matrix bot");

    let bot = match MatrixBot::new(Arc::clone(&config), &matrix, database).await {
        Ok(bot) => bot,
        Err(err) => {
            tracing::error!("Falied to run the matrix bot: {err}");
            return;
        }
    };

    tokio::spawn(users_handler::users_handler(
        bot.clone(),
        config,
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    ));

    bot.run(cancellation_token.clone()).await
}
