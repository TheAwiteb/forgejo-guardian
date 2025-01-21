// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Forgejo user
#[derive(Deserialize, Debug)]
pub struct ForgejoUser {
    /// User id, incremental integer
    pub id:         usize,
    /// Avatar URL
    pub avatar_url: url::Url,
    /// HTML URL
    pub html_url:   url::Url,
    /// Is admin
    pub is_admin:   bool,
    /// The login source id
    pub source_id:  u32,
    /// Username
    #[serde(rename = "login")]
    pub username:   String,
    /// Full name
    pub full_name:  String,
    /// Biography (AKA bio, profile description)
    #[serde(rename = "description")]
    pub biography:  String,
    /// Email
    pub email:      String,
    /// Website
    pub website:    String,
    /// Location
    pub location:   String,
    /// Created date of the user
    pub created:    DateTime<Utc>,
}
