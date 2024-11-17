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

//! Telegram bot module, to alert the admins of the bot of any suspicious users.

mod callback_handler;
mod message_handler;
mod users_handler;

use std::sync::Arc;

use callback_handler::callback_handler;
use serde::Deserialize;
use teloxide::{dispatching::UpdateFilterExt, prelude::*};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use crate::{
    config::{Config, RegexReason},
    forgejo_api::ForgejoUser,
};

/// Language of the telegram bot
#[derive(Deserialize)]
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

/// Start the telegram bot
pub async fn start_bot(
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    sus_receiver: Receiver<(ForgejoUser, RegexReason)>,
    ban_receiver: Receiver<(ForgejoUser, RegexReason)>,
) {
    tracing::info!("Starting the telegram bot");

    let bot = Bot::new(&config.telegram.token);
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(Message::filter_text().endpoint(message_handler::text_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    tokio::spawn(users_handler::users_handler(
        bot.clone(),
        Arc::clone(&config),
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
