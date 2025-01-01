// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use reqwest::Method;

use crate::error::{GuardError, GuardResult};

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

    if !res.status().is_success() {
        return Err(GuardError::FailedToBan(res.status()));
    }

    Ok(())
}
