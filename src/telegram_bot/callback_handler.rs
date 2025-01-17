// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::sync::Arc;

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

use crate::{config::Config, forgejo_api};

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
        "b" => {
            // ban the user
            let button_text = if config.dry_run
                || forgejo_api::ban_user(
                    &Client::new(),
                    &config.forgejo.instance,
                    &config.forgejo.token,
                    data,
                    &config.ban_action,
                )
                .await
                .is_ok()
            {
                tracing::info!("Suspicious user @{data} has been banned");
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
        "ignore" => {
            if let Some(MaybeInaccessibleMessage::Regular(msg)) = callback_query.message {
                bot.edit_message_reply_markup(msg.chat.id, msg.id)
                    .reply_markup(source_inline_keyboard(&t!("messages.ban_denied")))
                    .await?;
            }
        }
        _ => {}
    };

    Ok(())
}
