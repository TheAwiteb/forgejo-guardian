// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

//! Simple SDK for Forgejo API, only for banning users and getting users.

mod ban_user;
mod get_users;
mod user;

pub use ban_user::*;
pub use get_users::*;
use reqwest::{Method, Request};
pub use user::*;

/// Build a request with the given method, instance, token and endpoint.
pub fn build_request(method: Method, instance: &url::Url, token: &str, endpoint: &str) -> Request {
    let url = instance.join(endpoint).unwrap();
    let mut req = Request::new(method, url);

    req.headers_mut().insert(
        "Authorization",
        format!("token {token}").try_into().expect("Is valid"),
    );
    req.headers_mut()
        .insert("accept", "application/json".try_into().expect("Is valid"));

    req
}
