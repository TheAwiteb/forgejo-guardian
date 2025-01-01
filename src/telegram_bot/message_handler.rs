// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use teloxide::{
    prelude::*,
    types::{Me, ReplyParameters},
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone, Debug, PartialEq)]
#[command(rename_rule = "lowercase")]
enum Command {
    Start,
    #[command(aliases = ["h", "?"])]
    Help,
}

/// Help and start commands handler
pub async fn help_start_handler(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, t!("messages.help_start"))
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}

/// Handle text messages
pub async fn text_handler(bot: Bot, me: Me, msg: Message) -> ResponseResult<()> {
    let text = msg.text().expect("Is a text handler");
    let Ok(command) = Command::parse(text, me.username()) else {
        return Ok(());
    };

    match command {
        Command::Help | Command::Start => help_start_handler(&bot, &msg).await?,
    };

    Ok(())
}
