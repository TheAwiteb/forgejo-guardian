// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use matrix_sdk::{
    attachment::AttachmentConfig,
    room::edit::EditedContent,
    ruma::{
        events::{
            room::message::{AddMentions, ForwardThread, RoomMessageEventContent},
            Mentions,
        },
        EventId,
        OwnedEventId,
        OwnedUserId,
    },
};

use super::{utils, MatrixBot};

impl MatrixBot {
    pub async fn reply_to(&self, reply_to: &EventId, msg: impl Into<String>) {
        let Some(event) = utils::get_msg_event(&self.moderation_room, reply_to).await else {
            return;
        };

        let content = RoomMessageEventContent::text_plain(msg).make_reply_to(
            &event.into_full_event(self.moderation_room.room_id().to_owned()),
            ForwardThread::No,
            AddMentions::No,
        );
        if let Err(err) = self.moderation_room.send(content).await {
            tracing::error!("Falied to send a reply message: {err}");
        }
    }

    pub async fn send_image(
        &self,
        image: url::Url,
        caption: impl Into<String>,
    ) -> Option<OwnedEventId> {
        let err_msg = format!("Failed to get the user image: {image}");
        let filename = format!("{}.png", image.path().split('/').last().unwrap_or("image"));

        let Ok(image_res) = reqwest::get(image).await else {
            tracing::error!("{err_msg}");
            return None;
        };
        let Ok(image_bytes) = image_res.bytes().await else {
            tracing::error!("{err_msg}");
            return None;
        };

        match self
            .moderation_room
            .send_attachment(
                filename,
                &"image/png".parse().unwrap(),
                image_bytes.to_vec(),
                AttachmentConfig::new().caption(Some(caption.into())),
            )
            .await
        {
            Ok(res) => return Some(res.event_id.to_owned()),
            Err(err) => {
                tracing::error!("Falied to send an image message: {err}")
            }
        }
        None
    }

    pub async fn edit_msg_caption(
        &self,
        msg_id: &EventId,
        new_caption: impl Into<String>,
        mentions: Option<impl IntoIterator<Item = OwnedUserId>>,
    ) {
        let new_content = EditedContent::MediaCaption {
            caption:           Some(new_caption.into()),
            formatted_caption: None,
            mentions:          mentions.map(Mentions::with_user_ids),
        };
        let Ok(edit_event) = self
            .moderation_room
            .make_edit_event(msg_id, new_content)
            .await
        else {
            tracing::error!("Falied to create content edit event");
            return;
        };
        if let Err(err) = self.moderation_room.send(edit_event).await {
            tracing::error!("Falied to send content edit event: {err}");
        }
    }

    pub async fn send_ok_no_reaction(&self, event_id: &EventId) {
        if let Err(err) = self
            .moderation_room
            .send(utils::make_reaction(event_id, &self.ban_reaction()))
            .await
            .and(
                self.moderation_room
                    .send(utils::make_reaction(event_id, &self.ignore_reaction()))
                    .await,
            )
        {
            tracing::error!("Falied to send a reaction: {err}");
        }
    }
}
