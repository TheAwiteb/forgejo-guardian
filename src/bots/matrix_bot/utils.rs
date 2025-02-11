// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use matrix_sdk::{
    deserialized_responses::{TimelineEvent, TimelineEventKind},
    ruma::{
        events::{
            reaction::ReactionEventContent,
            relation::Annotation,
            room::message::{MessageType, RoomMessageEventContent},
            AnySyncMessageLikeEvent,
            AnySyncTimelineEvent,
            OriginalSyncMessageLikeEvent,
            SyncMessageLikeEvent,
        },
        EventId,
    },
    Room,
};

/// Get the message event from the room
pub async fn get_msg_event(
    room: &Room,
    event_id: &EventId,
) -> Option<OriginalSyncMessageLikeEvent<RoomMessageEventContent>> {
    if let Ok(TimelineEvent {
        kind: TimelineEventKind::PlainText { event },
        ..
    }) = room.event(event_id, None).await
    {
        return event.deserialize().ok().and_then(|event| {
            if let AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
                SyncMessageLikeEvent::Original(event),
            )) = event
            {
                return Some(event);
            }
            None
        });
    }
    None
}

/// Get the image caption from the message event content
pub fn get_image_caption(content: &RoomMessageEventContent) -> Option<&str> {
    if let MessageType::Image(image) = &content.msgtype {
        return Some(&image.body);
    }
    None
}

/// Make a reaction event content
pub fn make_reaction(to: &EventId, reaction: &str) -> ReactionEventContent {
    ReactionEventContent::new(Annotation::new(to.to_owned(), reaction.to_owned()))
}
