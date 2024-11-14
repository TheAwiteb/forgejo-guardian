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
