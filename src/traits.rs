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
