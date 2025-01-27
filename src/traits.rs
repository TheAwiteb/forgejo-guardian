// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use crate::{
    config::{Expr, RegexReason},
    forgejo_api::ForgejoUser,
};

/// Trait for checking if a user matches one of the expressions
pub trait ExprChecker {
    /// Returns the first matching expression, if any
    fn is_match(&self, user: &ForgejoUser) -> Option<RegexReason>;
}

impl ExprChecker for Expr {
    fn is_match<'a>(&'a self, user: &ForgejoUser) -> Option<RegexReason> {
        if !self.enabled {
            return None;
        }

        let one_of = |hay: &str, exprs: &'a Vec<RegexReason>| {
            // Join the user bio into a single line
            // ref: https://git.4rs.nl/awiteb/forgejo-guardian/issues/2
            let hay = if hay.contains('\n') {
                hay.split('\n').collect::<Vec<_>>().join(" ")
            } else {
                hay.to_string()
            };
            exprs
                .iter()
                .find(|re_re| re_re.re_vec.iter().all(|re| re.is_match(&hay)))
        };
        [
            one_of(&user.username, &self.usernames),
            one_of(&user.full_name, &self.full_names),
            one_of(&user.biography, &self.biographies),
            one_of(&user.email, &self.emails),
            one_of(&user.website, &self.websites),
            one_of(&user.location, &self.locations),
        ]
        .into_iter()
        .find_map(|v| v)
        .cloned()
    }
}
