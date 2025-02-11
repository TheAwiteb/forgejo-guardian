// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{borrow::Cow, sync::Arc};

use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::{
    config::{BanAction, Config, MatrixData, RegexReason, TelegramData},
    forgejo_api::ForgejoUser,
};

pub mod matrix_bot;
pub mod telegram_bot;

/// Language of the bots
#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lang {
    EnUs,
    ArSa,
    RuRu,
}

impl Lang {
    /// Get the language as a string
    pub fn as_str(&self) -> &str {
        match self {
            Lang::EnUs => "en-us",
            Lang::ArSa => "ar-sa",
            Lang::RuRu => "ru-ru",
        }
    }
}

/// Type to represent a user alert
pub struct UserAlert {
    /// The user that has been alerted, suspect or banned
    user:      ForgejoUser,
    /// The reason why the user has been alerted
    reason:    RegexReason,
    /// Safe mode is enabled
    safe_mode: bool,
}

impl UserAlert {
    /// Create a new user alert
    pub fn new(user: ForgejoUser, reason: RegexReason) -> Self {
        Self {
            user,
            reason,
            safe_mode: false,
        }
    }

    /// Set a value to the safe mode
    pub fn safe_mode(mut self) -> Self {
        self.safe_mode = true;
        self
    }
}

/// If the text is empty, return a not found message
pub fn not_found_if_empty(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        t!("words.not_found")
    } else {
        Cow::Borrowed(text)
    }
}

/// Generate a user details message
pub fn user_details(msg: &str, user: &ForgejoUser, re: &RegexReason, action: &str) -> String {
    t!(
        msg,
        action = action,
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
            .unwrap_or_else(|| t!("words.not_specified").into_owned()),
    )
    .into_owned()
}

/// Get the action word from the ban action
pub fn action_word(ban_action: &BanAction) -> String {
    if ban_action.is_purge() {
        t!("words.purge").into_owned()
    } else {
        t!("words.suspend").into_owned()
    }
}

/// Run the telegram bot in a separate task
pub fn run_telegram_bot(
    config: Arc<Config>,
    telegram: TelegramData,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<UserAlert>,
    ban_receiver: Receiver<UserAlert>,
) {
    tracing::info!(config = "telegram", "Bot lang: {}", telegram.lang.as_str());
    tracing::info!(config = "telegram", "Receiver chat ID: {}", telegram.chat);

    rust_i18n::set_locale(telegram.lang.as_str());

    tokio::spawn(telegram_bot::start_bot(
        config,
        telegram,
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    ));
}

/// Run the matrix bot in a separate task
pub fn run_matrix_bot(
    config: Arc<Config>,
    matrix: MatrixData,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<UserAlert>,
    ban_receiver: Receiver<UserAlert>,
) {
    tracing::info!(config = "matrix", "Bot lang: {}", matrix.lang.as_str());
    tracing::info!(config = "matrix", "Bot username: {}", matrix.username);
    tracing::info!(config = "matrix", "Homeserver: {}", matrix.homeserver);
    tracing::info!(config = "matrix", "Room: {}", matrix.room);

    rust_i18n::set_locale(matrix.lang.as_str());

    tokio::spawn(matrix_bot::start_bot(
        config,
        matrix,
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    ));
}

/// Run the enabled bot, if any
pub fn run_bots(
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<UserAlert>,
    ban_receiver: Receiver<UserAlert>,
) {
    if let Some(telegram) = config.telegram.data().cloned() {
        run_telegram_bot(
            config,
            telegram,
            cancellation_token,
            sus_receiver,
            ban_receiver,
        )
    } else if let Some(matrix) = config.matrix.data().cloned() {
        run_matrix_bot(
            config,
            matrix,
            cancellation_token,
            sus_receiver,
            ban_receiver,
        )
    }
}
