// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

/// The environment variable of the config file path
pub(crate) const CONFIG_PATH_ENV: &str = "FORGEJO_GUARDIAN_CONFIG";
/// Defult config path location
pub(crate) const DEFAULT_CONFIG_PATH: &str = "/app/forgejo-guardian.toml";

use std::fmt::Display;

use regex::Regex;
use serde::Deserialize;
use teloxide::types::ChatId;
use url::Url;

use crate::telegram_bot::Lang;

mod defaults;
mod deserializers;
mod utils;

/// Ban action to take when banning a user
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BanAction {
    /// Purge the user (Forcibly delete user and any repositories,
    /// organizations, and packages owned by the user. All comments and issues
    /// posted by this user will also be deleted.)
    Purge,
    /// Suspend the user (Block this user from interacting with this service
    /// through their account and prohibit signing in.)
    Suspend,
}

#[derive(Deserialize)]
pub struct Inactive {
    /// Whether the feature is enabled
    #[serde(default = "defaults::inactive::enabled")]
    pub enabled:      bool,
    /// Number of inactive days to consider
    #[serde(default = "defaults::inactive::days")]
    pub days:         u64,
    /// Number of requests to send
    #[serde(default = "defaults::inactive::req_limit")]
    pub req_limit:    u16,
    /// Time interval in seconds for the request limit
    #[serde(
        default = "defaults::inactive::req_interval",
        deserialize_with = "deserializers::suffix_interval"
    )]
    pub req_interval: u32,
    /// Time interval in seconds to check for inactive users
    #[serde(
        default = "defaults::inactive::interval",
        deserialize_with = "deserializers::suffix_interval"
    )]
    pub interval:     u32,
}

/// The forgejo config of the guard
#[derive(Deserialize)]
pub struct Forgejo {
    /// The bot token
    ///
    /// Required Permissions:
    /// - `read:admin`: To list the users
    /// - `write:admin`: To ban the users
    /// - `read:user`: To get user heatmap
    pub token:    String,
    /// The instance, e.g. `https://example.com` or `https://example.com/` or `http://example.com:8080`
    #[serde(rename = "instance_url", deserialize_with = "deserializers::url")]
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
    #[serde(deserialize_with = "deserializers::regex_reason")]
    pub usernames: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user full_name
    #[serde(default)]
    #[serde(deserialize_with = "deserializers::regex_reason")]
    pub full_names: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user biography
    #[serde(default)]
    #[serde(deserialize_with = "deserializers::regex_reason")]
    pub biographies: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user email
    #[serde(default)]
    #[serde(deserialize_with = "deserializers::regex_reason")]
    pub emails: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user website
    #[serde(default)]
    #[serde(deserialize_with = "deserializers::regex_reason")]
    pub websites: Vec<RegexReason>,

    /// The regular expressions that the action will be performed if they are
    /// present in the user location
    #[serde(default)]
    #[serde(deserialize_with = "deserializers::regex_reason")]
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
    /// Interval to check for new users in seconds
    #[serde(default = "defaults::global::interval")]
    pub interval:       u32,
    /// Limit of users to fetch in each interval
    #[serde(default = "defaults::global::limit")]
    pub limit:          u32,
    /// Action to take when banning a user
    #[serde(default = "defaults::global::ban_action")]
    pub ban_action:     BanAction,
    /// Inactive users configuration
    #[serde(default)]
    pub inactive:       Inactive,
    /// Configuration for the forgejo guard itself
    pub forgejo:        Forgejo,
    /// Configuration of the telegram bot
    pub telegram:       Telegram,
    /// The expressions, which are used to determine the actions
    #[serde(default)]
    pub expressions:    Exprs,
}

impl BanAction {
    /// Returns `true` if the action is `Purge`
    pub fn is_purge(&self) -> bool {
        matches!(self, Self::Purge)
    }
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

impl Display for BanAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Purge => write!(f, "purge"),
            Self::Suspend => write!(f, "suspend"),
        }
    }
}

impl Default for Inactive {
    fn default() -> Self {
        Self {
            enabled:      defaults::inactive::enabled(),
            days:         defaults::inactive::days(),
            req_limit:    defaults::inactive::req_limit(),
            req_interval: defaults::inactive::req_interval(),
            interval:     defaults::inactive::interval(),
        }
    }
}
