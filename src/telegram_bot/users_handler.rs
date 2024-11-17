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

use crate::{
    config::{Config, RegexReason},
    forgejo_api::ForgejoUser,
};

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

/// Generate a user details message
fn user_details(msg: &str, user: &ForgejoUser, re: &RegexReason) -> String {
    t!(
        msg,
        user_id = user.id,
        username = user.username,
        email = user.email,
        full_name = not_found_if_empty(&user.full_name),
        bio = not_found_if_empty(&user.biography),
        website = not_found_if_empty(&user.website),
        profile = user.html_url,
        reason = re
            .reason
            .clone()
            .unwrap_or_else(|| t!("words.not_found").to_string()),
    )
    .to_string()
}

/// Send a suspicious user alert to the admins
pub async fn send_sus_alert(
    bot: &Bot,
    re: &RegexReason,
    sus_user: ForgejoUser,
    config: &Config,
) -> ResponseResult<()> {
    let keyboard = make_sus_inline_keyboard(&sus_user);

    let caption = user_details("messages.sus_alert", &sus_user, re);
    bot.send_photo(config.telegram.chat, InputFile::url(sus_user.avatar_url))
        .caption(caption)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Send a ban notification to the admins chat
pub async fn send_ban_notify(
    bot: &Bot,
    re: &RegexReason,
    sus_user: ForgejoUser,
    config: &Config,
) -> ResponseResult<()> {
    let caption = user_details("messages.ban_notify", &sus_user, re);
    bot.send_photo(config.telegram.chat, InputFile::url(sus_user.avatar_url))
        .caption(caption)
        .await?;

    Ok(())
}

/// Handle the suspicious and banned users
pub async fn users_handler(
    bot: Bot,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    mut sus_receiver: Receiver<(ForgejoUser, RegexReason)>,
    mut ban_receiver: Receiver<(ForgejoUser, RegexReason)>,
) {
    loop {
        tokio::select! {
            Some((sus_user, re)) = sus_receiver.recv() => {
                send_sus_alert(&bot, &re, sus_user, &config).await.ok();
            }
            Some((banned_user, re)) = ban_receiver.recv() => {
                send_ban_notify(&bot, &re, banned_user, &config).await.ok();
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("sus users handler has been stopped successfully.");
                break;
            }
        }
    }
}
