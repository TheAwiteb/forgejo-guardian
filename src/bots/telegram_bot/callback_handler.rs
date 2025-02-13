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

use crate::{config::Config, db::IgnoredUsersTableTrait, forgejo_api};

/// Inline keyboard with a single button that links to the Forgejo Guardian
/// repository.
fn source_inline_keyboard(text: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::new(
        text,
        InlineKeyboardButtonKind::Url(
            url::Url::parse("https://git.4rs.nl/awiteb/forgejo-guardian").expect("Valid url"),
        ),
    )]])
}

/// Handle callback queries from the inline keyboard.
pub async fn callback_handler(
    bot: Bot,
    database: Arc<Database>,
    callback_query: CallbackQuery,
    config: Arc<Config>,
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
        "b" => {
            // ban the user
            let button_text = if config.dry_run
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
                    .reply_markup(source_inline_keyboard(&button_text))
                    .await?;
            }
        }
        // Ignore
        "i" => {
            if let Some(MaybeInaccessibleMessage::Regular(msg)) = callback_query.message {
                bot.edit_message_reply_markup(msg.chat.id, msg.id)
                    .reply_markup(source_inline_keyboard(&t!("messages.ban_denied")))
                    .await?;
            }
            database.add_ignored_user(data).ok();
        }
        _ => {}
    };

    Ok(())
}
