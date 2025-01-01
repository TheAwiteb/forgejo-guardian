// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

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
