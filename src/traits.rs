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

use regex::Regex;

use crate::{config::Expr, forgejo_api::ForgejoUser};

/// Trait for checking if a user matches one of the expressions
pub trait ExprChecker {
    /// Returns the first matching expression, if any
    fn is_match(&self, user: &ForgejoUser) -> Option<Regex>;
}

impl ExprChecker for Expr {
    fn is_match<'a>(&'a self, user: &ForgejoUser) -> Option<Regex> {
        let one_of = |hay, exprs: &'a Vec<Regex>| exprs.iter().find(|re| re.is_match(hay));
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