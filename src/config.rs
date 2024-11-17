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

use std::fmt::Display;

use regex::Regex;
use serde::{de, Deserialize};
use teloxide::types::ChatId;
use toml::Value;
use url::Url;

use crate::telegram_bot::Lang;

/// Function to create a custom error for deserialization from its string
fn custom_de_err<'de, D>(err: impl ToString) -> D::Error
where
    D: de::Deserializer<'de>,
{
    de::Error::custom(err.to_string())
}

/// Deserialize a string into a `url::Url`
///
/// This will check if the url is `http` or `https` and if it is a valid url
fn deserialize_str_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: de::Deserializer<'de>,
{
    let url = Url::parse(&String::deserialize(deserializer)?).map_err(custom_de_err::<'de, D>)?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(de::Error::custom("URL scheme must be http or https"));
    }
    Ok(url)
}

/// Parse the `re` key in the table, which can be a string or an array of string
fn parse_re<'de, D>(toml_value: &Value) -> Result<Vec<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    match toml_value {
        Value::String(str_re) => Ok(vec![str_re.to_owned()]),
        Value::Array(re_vec) => {
            re_vec
                .iter()
                .map(|str_re| {
                    str_re.as_str().map(String::from).ok_or_else(|| {
                        <D::Error as de::Error>::custom(format!(
                            "expected an array of string, found `{str_re}`"
                        ))
                    })
                })
                .collect()
        }
        value => {
            Err(<D::Error as de::Error>::custom(format!(
                "expected a string value or an array of string for `re`, found `{value}`"
            )))
        }
    }
}

/// Parse the vector of string regex to `Vec<Regex>`
fn parse_re_vec<'de, D>(re_vec: Vec<String>) -> Result<Vec<Regex>, D::Error>
where
    D: de::Deserializer<'de>,
{
    re_vec
        .into_iter()
        .map(|re| re.parse().map_err(custom_de_err::<'de, D>))
        .collect()
}

/// Deserialize `RegexReason`
fn deserialize_regex_reason<'de, D>(deserializer: D) -> Result<Vec<RegexReason>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let Ok(toml_value) = Vec::<Value>::deserialize(deserializer) else {
        return Err(de::Error::custom(
            "expected an array contains strings or arrays of string or tables with the keys `re` \
             and optional `reason`",
        ));
    };

    toml_value
        .into_iter()
        .map(|value| {
            if let Value::Table(table) = value {
                let re_vec = table.get("re").map(parse_re::<D>).ok_or_else(|| {
                    <D::Error as de::Error>::custom(
                        "The table must contain a `re` key with a string value or an array of \
                         string",
                    )
                })??;
                let reason = table
                    .get("reason")
                    .map(|reason| {
                        reason.as_str().map(String::from).ok_or_else(|| {
                            <D::Error as de::Error>::custom(format!(
                                "expected a string value for `reason`, found `{reason}`"
                            ))
                        })
                    })
                    .transpose()?;

                // Warn for unused keys
                for key in table.keys() {
                    if !["re", "reason"].contains(&key.as_str()) {
                        tracing::warn!("Unused key `{key}` in the configuration");
                    }
                }

                Ok(RegexReason::new(parse_re_vec::<D>(re_vec)?, reason))
            } else if matches!(value, Value::String(_) | Value::Array(_)) {
                Ok(RegexReason::new(
                    parse_re_vec::<D>(parse_re::<D>(&value)?)?,
                    None,
                ))
            } else {
                Err(de::Error::custom(format!(
                    "unexpected value in the regex list, expected a string or an array of string \
                     or a table with `re` (string) and optional `reason` (string), found `{value}`"
                )))
            }
        })
        .collect()
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

/// The telegram bot configuration
#[derive(Deserialize)]
pub struct Telegram {
    /// Telegram bot token
    pub token:     String,
    /// Chat to send the alert in
    pub chat:      ChatId,
    /// Send an alert when ban a user
    #[serde(default)]
    pub ban_alert: bool,
    /// Bot language
    pub lang:      Lang,
}

/// The regular expression with the reason
#[derive(Debug, Clone)]
pub struct RegexReason {
    /// The regular expression
    pub re_vec: Vec<Regex>,
    /// Optional reason
    pub reason: Option<String>,
}

/// The expression
#[derive(Deserialize, Debug, Default)]
pub struct Expr {
    /// The regular expressions that the action will be performed if they are
    /// present in the username
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_reason")]
    pub usernames: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user full_name
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_reason")]
    pub full_names: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user biography
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_reason")]
    pub biographies: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user email
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_reason")]
    pub emails: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user website
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_reason")]
    pub websites: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user location
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_regex_reason")]
    pub locations: Vec<RegexReason>,
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
    /// Dry run, without banning the users
    #[serde(default)]
    pub dry_run:        bool,
    /// Only checks new users
    #[serde(default)]
    pub only_new_users: bool,
    /// Configuration for the forgejo guard itself
    pub forgejo:        Forgejo,
    /// Configuration of the telegram bot
    pub telegram:       Telegram,
    /// The expressions, which are used to determine the actions
    #[serde(default)]
    pub expressions:    Exprs,
}

impl RegexReason {
    /// Create a new `RegexReason` instance
    fn new(re: Vec<Regex>, reason: Option<String>) -> Self {
        Self { re_vec: re, reason }
    }
}

impl Display for RegexReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for re in &self.re_vec {
            write!(f, "{re} ").ok();
        }
        if let Some(ref reason) = self.reason {
            write!(f, " ({reason})").ok();
        };
        Ok(())
    }
}
