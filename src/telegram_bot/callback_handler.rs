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
