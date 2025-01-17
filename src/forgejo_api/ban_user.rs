// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use reqwest::{Body, Method, Request};

use crate::{
    config::BanAction,
    error::{GuardError, GuardResult},
};

fn purge_req(instance: &url::Url, token: &str, username: &str) -> Request {
    super::build_request(
        Method::DELETE,
        instance,
        token,
        &format!("/api/v1/admin/users/{username}?purge=true"),
    )
}

fn suspend_req(instance: &url::Url, token: &str, username: &str) -> Request {
    let mut req = super::build_request(
        Method::PATCH,
        instance,
        token,
        &format!("/api/v1/admin/users/{username}"),
    );
    req.headers_mut().insert(
        "Content-Type",
        "application/json".try_into().expect("Is valid"),
    );
    *req.body_mut() = Some(Body::from(r#"{"prohibit_login": true}"#));
    req
}

/// Ban a user from the instance, purging their data.
pub async fn ban_user(
    client: &reqwest::Client,
    instance: &url::Url,
    token: &str,
    username: &str,
    ban_action: &BanAction,
) -> GuardResult<()> {
    let req = if ban_action.is_purge() {
        purge_req(instance, token, username)
    } else {
        suspend_req(instance, token, username)
    };

    let res = client.execute(req).await?;
    tracing::debug!("Ban user response of {ban_action:?}: {:?}", &res);

    if !res.status().is_success() {
        return Err(GuardError::FailedToBan(res.status()));
    }

    Ok(())
}
