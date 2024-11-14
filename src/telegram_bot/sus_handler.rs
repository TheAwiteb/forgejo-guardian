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

use std::{borrow::Cow, sync::Arc};

use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, InputFile},
};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::{config::Config, forgejo_api::ForgejoUser};

/// Create an inline keyboard of the suspicious user
fn make_sus_inline_keyboard(sus_user: &ForgejoUser) -> InlineKeyboardMarkup {
    let button = |text: &str, callback: String| {
        InlineKeyboardButton::new(text, InlineKeyboardButtonKind::CallbackData(callback))
    };

    InlineKeyboardMarkup::new([[
        button(
            t!("buttons.ban").as_ref(),
            format!("b {}", sus_user.username),
        ),
        button(t!("buttons.ignore").as_ref(), "ignore -".to_owned()),
    ]])
}

fn not_found_if_empty(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        t!("words.not_found")
    } else {
        Cow::Borrowed(text)
    }
}

/// Send a suspicious user alert to the admins
pub async fn send_sus_alert(
    bot: &Bot,
    sus_user: ForgejoUser,
    config: &Config,
) -> ResponseResult<()> {
    let keyboard = make_sus_inline_keyboard(&sus_user);

    bot.send_photo(config.telegram.chat, InputFile::url(sus_user.avatar_url))
        .caption(t!(
            "messages.sus_alert",
            user_id = sus_user.id,
            username = sus_user.username,
            full_name = not_found_if_empty(&sus_user.full_name),
            bio = not_found_if_empty(&sus_user.biography),
            website = not_found_if_empty(&sus_user.website),
            profile = sus_user.html_url,
        ))
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Handle the suspicious users
pub async fn sus_users_handler(
    bot: Bot,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    mut sus_receiver: Receiver<ForgejoUser>,
) {
    loop {
        tokio::select! {
            Some(sus_user) = sus_receiver.recv() => {
                send_sus_alert(&bot, sus_user, &config).await.ok();
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("sus users handler has been stopped successfully.");
                break;
            }
        }
    }
}
