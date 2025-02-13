// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, InputFile},
};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use super::UserAlert;
use crate::{
    bots::{action_word, user_details},
    config::{Config, RegexReason, TelegramData},
    forgejo_api::ForgejoUser,
};

/// Create an inline keyboard ask to ban or ignore the user
fn make_ban_ignore_keyboard(user: &ForgejoUser, action: &str) -> InlineKeyboardMarkup {
    let button = |text: &str, callback: String| {
        InlineKeyboardButton::new(text, InlineKeyboardButtonKind::CallbackData(callback))
    };

    InlineKeyboardMarkup::new([[
        button(
            t!("buttons.ban", action = action).as_ref(),
            format!("b {}", user.username),
        ),
        button(
            t!("buttons.ignore").as_ref(),
            format!("i {}", user.username),
        ),
    ]])
}

/// Send a suspicious user alert to the admins
pub async fn send_sus_alert(
    bot: &Bot,
    telegram: &TelegramData,
    re: &RegexReason,
    sus_user: ForgejoUser,
    config: &Config,
) -> ResponseResult<()> {
    tracing::info!("Sending suspicious user alert to the admins chat");

    let action = action_word(&config.expressions.ban_action);
    let keyboard = make_ban_ignore_keyboard(&sus_user, &action);

    let caption = user_details("messages.sus_alert", &sus_user, re, &action);
    bot.send_photo(telegram.chat, InputFile::url(sus_user.avatar_url))
        .caption(caption)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Send a ban notification to the admins chat
pub async fn send_ban_notify(
    bot: &Bot,
    telegram: &TelegramData,
    re: &RegexReason,
    sus_user: ForgejoUser,
    config: &Config,
) -> ResponseResult<()> {
    tracing::info!("Sending ban notification to the admins chat");

    let action = action_word(&config.expressions.ban_action);
    let caption = user_details("messages.ban_notify", &sus_user, re, &action);
    bot.send_photo(telegram.chat, InputFile::url(sus_user.avatar_url))
        .caption(caption)
        .await?;

    Ok(())
}

/// Send a ban request to the admins chat
pub async fn send_ban_request(
    bot: &Bot,
    telegram: &TelegramData,
    re: &RegexReason,
    user: ForgejoUser,
    config: &Config,
) -> ResponseResult<()> {
    tracing::info!("Sending ban request to the admins chat");

    let action = action_word(&config.expressions.ban_action);
    let keyboard = make_ban_ignore_keyboard(&user, &action);
    let caption = user_details("messages.ban_request", &user, re, &action);

    bot.send_photo(telegram.chat, InputFile::url(user.avatar_url))
        .caption(caption)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Handle the suspicious and banned users
pub async fn users_handler(
    bot: Bot,
    config: Arc<Config>,
    telegram: Arc<TelegramData>,
    cancellation_token: CancellationToken,
    mut sus_receiver: Receiver<UserAlert>,
    mut ban_receiver: Receiver<UserAlert>,
) {
    loop {
        tokio::select! {
            Some(alert) = sus_receiver.recv() => {
                send_sus_alert(&bot, &telegram, &alert.reason, alert.user, &config).await.ok();
            }
            Some(alert) = ban_receiver.recv() => {
                if alert.safe_mode {
                    send_ban_request(&bot,&telegram, &alert.reason, alert.user, &config).await.ok();
                } else {
                    send_ban_notify(&bot,&telegram, &alert.reason, alert.user, &config).await.ok();
                }
            }
            _ = cancellation_token.cancelled() => {
                tracing::info!("Telegram users handler has been stopped successfully.");
                break;
            }
        }
    }
}
