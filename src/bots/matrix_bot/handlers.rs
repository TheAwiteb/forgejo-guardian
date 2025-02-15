// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use matrix_sdk::{
    event_handler::Ctx,
    ruma::events::{
        reaction::OriginalSyncReactionEvent,
        room::message::{MessageType, OriginalSyncRoomMessageEvent},
    },
    Room,
    RoomState,
};
use reqwest::Client;

use super::{utils, MatrixBot};
use crate::{
    db::{EventsTableTrait, IgnoredUsersTableTrait},
    forgejo_api,
};

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

        if text.body == "!ping" {
            bot.reply_to(&event.event_id, "Pong!").await;
        }
    }
}
