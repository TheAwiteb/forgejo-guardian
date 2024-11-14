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
}
