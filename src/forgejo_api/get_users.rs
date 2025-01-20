// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use reqwest::Method;

use super::ForgejoUser;
use crate::error::{GuardError, GuardResult};

/// Returns the first page of users from the instance
pub async fn get_users(
    client: &reqwest::Client,
    instance: &url::Url,
    token: &str,
    limit: u32,
    page: u32,
    sort: &str,
) -> GuardResult<Vec<ForgejoUser>> {
    let req = super::build_request(
        Method::GET,
        instance,
        token,
        &format!("/api/v1/admin/users?limit={limit}&page={page}&sort={sort}"),
    );
    let url = req.url().clone();

    let res = client.execute(req).await?;

    if !res.status().is_success() {
        return Err(GuardError::InvalidForgejoResponse(
            format!("Status code: {status}", status = res.status()),
            url,
        ));
    }

    tracing::debug!("Get users response: {res:?}");

    serde_json::from_str(&res.text().await?)
        .map_err(|err| GuardError::InvalidForgejoResponse(err.to_string(), url))
}
