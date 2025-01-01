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

use super::ForgejoUser;
use crate::error::{GuardError, GuardResult};

/// Returns the first page of users from the instance
pub async fn get_users(
    client: &reqwest::Client,
    instance: &url::Url,
    token: &str,
    limit: u32,
) -> GuardResult<Vec<ForgejoUser>> {
    let req = super::build_request(
        Method::GET,
        instance,
        token,
        &format!("/api/v1/admin/users?limit={limit}&sort=newest"),
    );
    let res = client
        .execute(req.try_clone().expect("There is no body"))
        .await?;

    if !res.status().is_success() {
        return Err(GuardError::InvalidForgejoResponse(
            format!("Status code: {status}", status = res.status()),
            req,
        ));
    }

    tracing::debug!("Get users response: {res:?}");

    serde_json::from_str(&res.text().await?)
        .map_err(|err| GuardError::InvalidForgejoResponse(err.to_string(), req))
}
