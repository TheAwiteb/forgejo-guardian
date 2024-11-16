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

use reqwest::StatusCode;

use crate::config::{CONFIG_PATH_ENV, DEFAULT_CONFIG_PATH};

/// Result of the guard
pub type GuardResult<T> = Result<T, GuardError>;

#[derive(Debug, thiserror::Error)]
pub enum GuardError {
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// reqwest error
    #[error("Sending request error: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Invalid response from Forgejo
    #[error("Invalid response from Forgejo, the error `{0:?}` request `{1:?}`")]
    InvalidForgejoResponse(String, reqwest::Request),
    /// Faild to get the config file
    #[error(
        "The configuration file could not be accessed, its path is not in the `{CONFIG_PATH_ENV}` \
         environment variable nor is it in the default path `{DEFAULT_CONFIG_PATH}`"
    )]
    CantGetConfigFile,
    /// Faild to deserialize the config file
    #[error("Failed to deserialize the config: {0}")]
    FaildDeserializeConfig(#[from] toml::de::Error),
    /// Failed to ban the user
    #[error("Failed to ban the user, status code: {0}")]
    FailedToBan(StatusCode),
}
