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
