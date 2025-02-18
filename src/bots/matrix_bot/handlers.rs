// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use matrix_sdk::{
    event_handler::Ctx,
    ruma::{
        events::{
            reaction::OriginalSyncReactionEvent,
            room::message::{MessageType, OriginalSyncRoomMessageEvent},
        },
        EventId,
    },
    Room,
    RoomState,
};
use reqwest::Client;

use super::{utils, MatrixBot};
use crate::{
    bots::{matrix_bot::users_handler, UserAlert},
    config::RegexReason,
    db::{EventsTableTrait, IgnoredUsersTableTrait},
    forgejo_api,
};

/// Ban command handler
pub async fn ban_command_handler(
    event_id: &EventId,
    bot: &MatrixBot,
    moderator: &str,
    username: &str,
) {
    if username.is_empty() {
        return;
    }

    tracing::info!("{moderator} requesting a ban request for `@{username}`");
    let Ok(user) = forgejo_api::get_user(username, &bot.config.forgejo).await else {
        bot.reply_to(event_id, t!("messages.user_not_found", username = username))
            .await;
        return;
    };
    if user.is_admin {
        bot.reply_to(event_id, t!("messages.can_not_ban_admin"))
            .await;
        return;
    }
    users_handler::send_ban_request(
        bot,
        UserAlert::new(
            user,
            RegexReason::new(
                Vec::new(),
                Some(
                    t!(
                        "messages.ban_command_reason",
                        moderator = moderator,
                        prefix = "!"
                    )
                    .into_owned(),
                ),
            ),
        ),
        &bot.config.expressions.ban_action,
    )
    .await;
}

impl MatrixBot {
    pub async fn on_room_reaction(
        event: OriginalSyncReactionEvent,
        room: Room,
        Ctx(bot): Ctx<MatrixBot>,
    ) {
        if room.state() != RoomState::Joined
            || bot.client.user_id().is_some_and(|u| u == event.sender)
        {
            return;
        }

        let moderator = event.sender.as_str();
        let reaction = &event.content.relates_to.key;
        let reply_to_event_id = event.content.relates_to.event_id.clone();
        let Some(reply_to_event) = utils::get_msg_event(&room, &reply_to_event_id).await else {
            return;
        };
        let Some(msg_text) = utils::get_image_caption(&reply_to_event.content) else {
            return;
        };
        if reply_to_event.sender != bot.client.user_id().expect("already logged") {
            return;
        }

        let username = match bot.db.get_username(&reply_to_event_id) {
            Ok(Some(username)) => username,
            Ok(None) => {
                tracing::warn!(
                    "{moderator} react to `{reply_to_event_id}` while there is no user for this \
                     event, the event may deleted after the action"
                );
                return;
            }
            Err(err) => {
                tracing::error!("Failed to get username from db: {err}");
                return;
            }
        };

        if reaction == &bot.ban_reaction() {
            let ban_status = if bot.config.dry_run
                || forgejo_api::ban_user(
                    &Client::new(),
                    &bot.config.forgejo.instance,
                    &bot.config.forgejo.token,
                    &username,
                    &bot.config.expressions.ban_action,
                )
                .await
                .is_ok()
            {
                tracing::info!(
                    "Moderation team has banned @{username}, the moderator is {moderator}",
                );
                t!("messages.ban_success")
            } else {
                t!("messages.ban_failed")
            };
            let new_caption = format!("{ban_status} ({moderator})\n\n{msg_text}");
            bot.edit_msg_caption(
                &reply_to_event_id,
                new_caption,
                Some([event.sender.clone()]),
            )
            .await;
        } else if reaction == &bot.ignore_reaction() {
            let new_caption = format!("{} ({moderator})\n\n{msg_text}", t!("messages.ban_denied"));
            bot.edit_msg_caption(
                &reply_to_event_id,
                new_caption,
                Some([event.sender.clone()]),
            )
            .await;
            bot.db.add_ignored_user(&username).ok();
        }

        bot.db.remove_event(&reply_to_event_id).ok();
    }

    pub async fn on_room_message(
        event: OriginalSyncRoomMessageEvent,
        room: Room,
        Ctx(bot): Ctx<MatrixBot>,
    ) {
        if room.state() != RoomState::Joined
            || bot.client.user_id().is_some_and(|u| u == event.sender)
        {
            return;
        }
        let MessageType::Text(text) = &event.content.msgtype else {
            return;
        };
        let moderator = event.sender.as_str();

        if text.body == "!ping" {
            bot.reply_to(&event.event_id, "Pong!").await;
        }
        if let Some(("!ban", username)) = text.body.split_once(" ") {
            ban_command_handler(&event.event_id, &bot, moderator, username).await;
        }
    }
}
