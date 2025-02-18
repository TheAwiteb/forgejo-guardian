// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::fmt;

use reqwest::Method;

use super::ForgejoUser;
use crate::error::{GuardError, GuardResult};

/// Sort order for the users
#[derive(Clone, Copy)]
pub enum Sort {
    Newest,
    RecentUpdate,
    Oldest,
}

impl Sort {
    /// Returns `true` if the sort order is `newest`
    pub const fn is_newest(&self) -> bool {
        matches!(self, Self::Newest)
    }

    /// Returns `true` if the sort order is `recentupdate`
    pub const fn is_recent_update(&self) -> bool {
        matches!(self, Self::RecentUpdate)
    }

    /// Returns `true` if the sort order is `oldest`
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Newest => "newest",
            Self::RecentUpdate => "recentupdate",
            Self::Oldest => "oldest",
        }
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Returns the first page of users from the instance
pub async fn get_users(
    client: &reqwest::Client,
    instance: &url::Url,
    token: &str,
    limit: u32,
    page: u32,
    sort: &Sort,
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
