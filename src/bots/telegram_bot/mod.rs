// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

//! Telegram bot module, to alert the admins of the bot of any suspicious users.

mod callback_handler;
mod message_handler;
mod users_handler;

use std::sync::Arc;

use callback_handler::callback_handler;
use teloxide::{dispatching::UpdateFilterExt, prelude::*};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use super::UserAlert;
use crate::config::{Config, TelegramData};

/// Start the telegram bot
pub async fn start_bot(
    config: Arc<Config>,
    telegram: TelegramData,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<UserAlert>,
    ban_receiver: Receiver<UserAlert>,
) {
    tracing::info!("Starting the telegram bot");

    let telegram = Arc::new(telegram);
    let bot = Bot::new(&telegram.token);
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(Message::filter_text().endpoint(message_handler::text_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    tokio::spawn(users_handler::users_handler(
        bot.clone(),
        Arc::clone(&config),
        Arc::clone(&telegram),
        cancellation_token,
        sus_receiver,
        ban_receiver,
    ));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
