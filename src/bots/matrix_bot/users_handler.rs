// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::sync::Arc;

use matrix_sdk::ruma::OwnedEventId;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use super::{database::EventsTableTrait, MatrixBot};
use crate::{
    bots::{action_word, user_details, UserAlert},
    config::{BanAction, Config},
};

/// Send an alert to the moderation room
async fn send_alert(
    bot: &MatrixBot,
    alert: &UserAlert,
    action: &BanAction,
    msg_name: &str,
) -> Option<OwnedEventId> {
    let caption = user_details(msg_name, &alert.user, &alert.reason, &action_word(action));
    bot.send_image(alert.user.avatar_url.clone(), caption).await
}

/// Send a suspicious alert and add the event to the database
pub async fn send_sus_alert(bot: &MatrixBot, alert: UserAlert, action: &BanAction) {
    let Some(event_id) = send_alert(bot, &alert, action, "messages.sus_alert").await else {
        return;
    };
    bot.send_ok_no_reaction(&event_id).await;
    if let Err(err) = bot.db.add_event(&event_id, &alert.user.username) {
        tracing::error!("{err}");
    }
}

/// Send a ban request alert and add the event to the database
pub async fn send_ban_request(bot: &MatrixBot, alert: UserAlert, action: &BanAction) {
    let Some(event_id) = send_alert(bot, &alert, action, "messages.ban_request").await else {
        return;
    };
    bot.send_ok_no_reaction(&event_id).await;
    if let Err(err) = bot.db.add_event(&event_id, &alert.user.username) {
        tracing::error!("{err}");
    }
}

/// Send a ban notify alert
pub async fn send_ban_notify(bot: &MatrixBot, alert: UserAlert, action: &BanAction) {
    send_alert(bot, &alert, action, "messages.ban_notify").await;
}

/// Handle the suspicious and banned users
pub async fn users_handler(
    bot: MatrixBot,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    mut sus_receiver: Receiver<UserAlert>,
    mut ban_receiver: Receiver<UserAlert>,
) {
    loop {
        tokio::select! {
            Some(alert) = sus_receiver.recv() => {
                send_sus_alert(&bot, alert, &config.expressions.ban_action).await;
            }
            Some(alert) = ban_receiver.recv() => {
                if alert.safe_mode {
                    send_ban_request(&bot, alert, &config.expressions.ban_action).await;
                } else {
                    send_ban_notify(&bot, alert, &config.expressions.ban_action).await;
                }
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("Matrix users handler has been stopped successfully.");
                break;
            }
        }
    }
}
