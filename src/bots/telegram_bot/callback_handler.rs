// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::sync::Arc;

use redb::Database;
use reqwest::Client;
use teloxide::{
    prelude::*,
    types::{
        CallbackQuery,
        InlineKeyboardButton,
        InlineKeyboardButtonKind,
        InlineKeyboardMarkup,
        MaybeInaccessibleMessage,
    },
};

use crate::{
    config::Config,
    db::{AlertedUsersTableTrait, IgnoredUsersTableTrait, PurgedUsersTableTrait},
    forgejo_api,
};

/// Inline keyboard with a single button that links to the Forgejo Guardian
/// repository. The undo button will be added if the username is `Some`
fn source_inline_keyboard(text: &str, username: Option<&str>) -> InlineKeyboardMarkup {
    let mut keyboard = vec![[InlineKeyboardButton::new(
        text,
        InlineKeyboardButtonKind::Url(
            url::Url::parse("https://git.4rs.nl/awiteb/forgejo-guardian").expect("Valid url"),
        ),
    )]];

    if let Some(username) = username {
        keyboard.push([InlineKeyboardButton::new(
            t!("buttons.undo"),
            InlineKeyboardButtonKind::CallbackData(format!("u {username}")),
        )]);
    }

    InlineKeyboardMarkup::new(keyboard)
}

/// Handle callback queries from the inline keyboard.
pub async fn callback_handler(
    bot: Bot,
    callback_query: CallbackQuery,
    config: Arc<Config>,
    database: Arc<Database>,
) -> ResponseResult<()> {
    let Some(callback_data) = callback_query.data else {
        return Ok(());
    };

    let Some((command, data)) = callback_data.split_once(' ') else {
        // Invalid callback data
        return Ok(());
    };

    match command {
        // Ban
        "b" if !database.is_layz_purged(data).is_ok_and(|y| y) => {
            // ban the user
            let button_text = if config.dry_run
                || config.lazy_purge.enabled
                || forgejo_api::ban_user(
                    &Client::new(),
                    &config.forgejo.instance,
                    &config.forgejo.token,
                    data,
                    &config.expressions.ban_action,
                )
                .await
                .is_ok()
            {
                if config.lazy_purge.enabled {
                    database.add_purged_user(data).ok();
                } else {
                    database.remove_alerted_user(data).ok();
                }

                let moderator = callback_query
                    .from
                    .username
                    .map(|u| format!("@{u}"))
                    .unwrap_or_else(|| format!("id={}", callback_query.from.id));

                tracing::info!("Moderation team has banned @{data}, the moderator is {moderator}",);
                t!("messages.ban_success")
            } else {
                t!("messages.ban_failed")
            };

            if let Some(MaybeInaccessibleMessage::Regular(msg)) = callback_query.message {
                bot.edit_message_reply_markup(msg.chat.id, msg.id)
                    .reply_markup(source_inline_keyboard(
                        &button_text,
                        config.lazy_purge.enabled.then_some(data),
                    ))
                    .await?;
            }
        }
        // Ignore
        "i" => {
            if let Some(MaybeInaccessibleMessage::Regular(msg)) = callback_query.message {
                bot.edit_message_reply_markup(msg.chat.id, msg.id)
                    .reply_markup(source_inline_keyboard(&t!("messages.ban_denied"), None))
                    .await?;
            }
            database.add_ignored_user(data).ok();
            database.remove_alerted_user(data).ok();
        }
        // Undo a purge
        "u" if config.lazy_purge.enabled && database.is_layz_purged(data).is_ok_and(|y| y) => {
            if let Some(MaybeInaccessibleMessage::Regular(msg)) = callback_query.message {
                bot.edit_message_reply_markup(msg.chat.id, msg.id)
                    .reply_markup(source_inline_keyboard(&t!("messages.undo_success"), None))
                    .await?;
            }
            database.remove_purged_user(data).ok();
        }
        _ => {}
    };

    Ok(())
}
