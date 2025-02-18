// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{Me, ReplyParameters},
    utils::command::BotCommands,
};

use crate::{
    bots::telegram_bot::users_handler,
    config::{Config, RegexReason},
    forgejo_api,
};

#[derive(BotCommands, Clone, Debug, PartialEq)]
#[command(rename_rule = "lowercase")]
enum Command {
    Start,
    #[command(aliases = ["h", "?"])]
    Help,
    Ping,
    Ban(String),
}

/// Help and start commands handler
pub async fn help_start_handler(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, t!("messages.help_start"))
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}

/// Ban command handler
pub async fn ban_handler(
    config: &Config,
    bot: &Bot,
    msg: &Message,
    username: String,
) -> ResponseResult<()> {
    if username.is_empty() {
        return Ok(());
    }

    let moderator = msg
        .from
        .as_ref()
        .map(|u| {
            u.username
                .as_ref()
                .map(|n| format!("@{n}"))
                .unwrap_or_else(|| u.full_name())
        })
        .unwrap_or_else(|| "N/A".to_owned());

    tracing::info!("{moderator} requesting a ban request for `@{username}`");
    let Ok(user) = forgejo_api::get_user(&username, &config.forgejo).await else {
        bot.send_message(
            msg.chat.id,
            t!("messages.user_not_found", username = username),
        )
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
        return Ok(());
    };
    if user.is_admin {
        bot.send_message(msg.chat.id, t!("messages.can_not_ban_admin"))
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }
    users_handler::send_ban_request(
        bot,
        config.telegram.data().expect("telegram is enabled"),
        &RegexReason::new(
            Vec::new(),
            Some(
                t!(
                    "messages.ban_command_reason",
                    moderator = moderator,
                    prefix = "/"
                )
                .into_owned(),
            ),
        ),
        user,
        config,
    )
    .await
}

/// Handle text messages
pub async fn text_handler(
    bot: Bot,
    me: Me,
    msg: Message,
    config: Arc<Config>,
) -> ResponseResult<()> {
    if msg.forward_origin().is_some() {
        return Ok(());
    }

    let text = msg.text().expect("Is a text handler");
    let Ok(command) = Command::parse(text, me.username()) else {
        return Ok(());
    };

    match command {
        Command::Help | Command::Start => help_start_handler(&bot, &msg).await?,
        Command::Ping => {
            bot.send_message(msg.chat.id, "Pong!")
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
        }
        Command::Ban(username)
            if config
                .telegram
                .data()
                .is_some_and(|d| d.chat == msg.chat.id) =>
        {
            ban_handler(&config, &bot, &msg, username).await?
        }
        _ => {}
    };

    Ok(())
}
