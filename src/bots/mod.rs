// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{borrow::Cow, sync::Arc};

use redb::Database;
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
    DeDe,
}

impl Lang {
    /// Get the language as a string
    pub fn as_str(&self) -> &str {
        match self {
            Lang::EnUs => "en-us",
            Lang::ArSa => "ar-sa",
            Lang::RuRu => "ru-ru",
            Lang::DeDe => "de-de",
        }
    }
}

/// Type to represent a user alert
pub struct UserAlert {
    /// The user that has been alerted, suspect or banned
    user:      ForgejoUser,
    /// The reason why the user has been alerted
    reason:    RegexReason,
    /// Is the user active, for ban this will send a ban request. For sus user
    /// this will add an active notice
    is_active: bool,
}

impl UserAlert {
    /// Create a new user alert
    pub fn new(user: ForgejoUser, reason: RegexReason) -> Self {
        Self {
            user,
            reason,
            is_active: false,
        }
    }

    /// Mark the user as active
    pub fn is_active(mut self, yes: bool) -> Self {
        self.is_active = yes;
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
pub fn user_details(
    msg: &str,
    user: &ForgejoUser,
    re: &RegexReason,
    action: &str,
    config: &Config,
) -> String {
    let user_email = if config.hide_user_email {
        t!("messages.hidden")
    } else {
        Cow::Borrowed(user.email.as_str())
    };

    t!(
        msg,
        action = action,
        user_id = user.id,
        username = user.username,
        email = user_email,
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
    database: Arc<Database>,
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
        database,
        config,
        telegram,
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    ));
}

/// Run the matrix bot in a separate task
pub fn run_matrix_bot(
    database: Arc<Database>,
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
        database,
        config,
        matrix,
        cancellation_token.clone(),
        sus_receiver,
        ban_receiver,
    ));
}

/// Run the enabled bot, if any
pub fn run_bots(
    database: Arc<Database>,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<UserAlert>,
    ban_receiver: Receiver<UserAlert>,
) {
    if let Some(telegram) = config.telegram.data().cloned() {
        run_telegram_bot(
            database,
            config,
            telegram,
            cancellation_token,
            sus_receiver,
            ban_receiver,
        )
    } else if let Some(matrix) = config.matrix.data().cloned() {
        run_matrix_bot(
            database,
            config,
            matrix,
            cancellation_token,
            sus_receiver,
            ban_receiver,
        )
    }
}
