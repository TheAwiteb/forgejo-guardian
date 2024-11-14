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

use reqwest::Method;

use crate::error::GuardResult;

/// Ban a user from the instance, purging their data.
pub async fn ban_user(
    client: &reqwest::Client,
    instance: &url::Url,
    token: &str,
    username: &str,
) -> GuardResult<()> {
    let res = client
        .execute(super::build_request(
            Method::DELETE,
            instance,
            token,
            &format!("/api/v1/admin/users/{username}?purge=true"),
        ))
        .await?;
    tracing::debug!("Ban user response: {:?}", &res);
    tracing::debug!("Body: {}", res.text().await.unwrap_or_default());

    Ok(())
}
