// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use reqwest::{Client, Method};
use url::Url;

use crate::{
    error::{GuardError, GuardResult},
    forgejo_api,
};

/// Returns whether the tokens is empty.
pub async fn is_empty_tokens(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
) -> GuardResult<bool> {
    let req = forgejo_api::build_request(
        Method::GET,
        instance,
        token,
        &format!("/api/v1/users/{username}/tokens"),
    );
    let url = req.url().clone();
    let res = client.execute(req).await?;

    if !res.status().is_success() {
        return Err(GuardError::InvalidForgejoResponse(
            format!("Status code: {status}", status = res.status()),
            url,
        ));
    }

    tracing::debug!("Get tokens response: {res:?}");

    Ok(res.text().await.unwrap_or_default().trim() == "[]")
}

/// Returns whether the apps is empty
pub async fn is_empty_apps(
    client: &Client,
    instance: &Url,
    token: &str,
    username: &str,
) -> GuardResult<bool> {
    let req = forgejo_api::build_request(
        Method::GET,
        instance,
        token,
        &format!("/api/v1/user/applications/oauth2?sudo={username}"),
    );
    let url = req.url().clone();
    let res = client.execute(req).await?;

    if !res.status().is_success() {
        return Err(GuardError::InvalidForgejoResponse(
            format!("Status code: {status}", status = res.status()),
            url,
        ));
    }

    tracing::debug!("Get apps response: {res:?}");

    Ok(res.text().await.unwrap_or_default().trim() == "[]")
}
