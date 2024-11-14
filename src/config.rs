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

/// The environment variable of the config file path
pub(crate) const CONFIG_PATH_ENV: &str = "FORGEJO_GUARDIAN_CONFIG";
/// Defult config path location
pub(crate) const DEFAULT_CONFIG_PATH: &str = "/app/forgejo-guardian.toml";

use regex::Regex;
use serde::{de, Deserialize};
use url::Url;

/// Deserialize a string into a `url::Url`
///
/// This will check if the url is `http` or `https` and if it is a valid url
fn deserialize_str_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: de::Deserializer<'de>,
{
    let url = Url::parse(&String::deserialize(deserializer)?)
        .map_err(|e| de::Error::custom(e.to_string()))?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(de::Error::custom("URL scheme must be http or https"));
    }
    Ok(url)
}

/// Deserialize a vector of strings into a vector of `regex::Regex`
fn deserialize_regex_vec<'de, D>(deserializer: D) -> Result<Vec<Regex>, D::Error>
where
    D: de::Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|s| Regex::new(&s))
        .collect::<Result<_, _>>()
        .map_err(|e| de::Error::custom(e.to_string()))
}

/// The forgejo config of the guard
#[derive(Deserialize)]
pub struct Forgejo {
    /// The bot token
    ///
    /// Required Permissions:
    /// - `read:admin`: To list the users
    /// - `write:admin`: To ban the users
    pub token:    String,
    /// The instance, e.g. `https://example.com` or `https://example.com/` or `http://example.com:8080`
    #[serde(rename = "instance_url", deserialize_with = "deserialize_str_url")]
    pub instance: Url,
}

/// The expression
#[derive(Deserialize, Debug, Default)]
pub struct Expr {
    /// The regular expressions that the action will be performed if they are
    /// present in the username
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_vec")]
    pub usernames: Vec<Regex>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user full_name
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_vec")]
    pub full_names: Vec<Regex>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user biography
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_vec")]
    pub biographies: Vec<Regex>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user email
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_vec")]
    pub emails: Vec<Regex>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user website
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_vec")]
    pub websites: Vec<Regex>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user location
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_vec")]
    pub locations: Vec<Regex>,
}

/// the expressions
#[derive(Deserialize, Debug, Default)]
pub struct Exprs {
    /// Direct ban expressions.
    ///
    /// Users are directly banned if any of the expressions are true
    #[serde(default)]
    pub ban: Expr,

    /// Alert expressions.
    ///
    /// Moderators will be notified via Telegram if one of the expressions are
    /// true
    #[serde(default)]
    pub sus: Expr,
}

/// forgejo-guard configuration
#[derive(Deserialize)]
pub struct Config {
    /// Configuration for the forgejo guard itself
    pub forgejo:     Forgejo,
    /// The expressions, which are used to determine the actions
    #[serde(default)]
    pub expressions: Exprs,
}