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

use std::{fs, path::PathBuf, str::FromStr};

use tracing::level_filters::LevelFilter;

use crate::{
    config::{Config, CONFIG_PATH_ENV, DEFAULT_CONFIG_PATH},
    error::{GuardError, GuardResult},
};

/// Returns the log level from `RUST_LOG` environment variable
pub fn get_log_level() -> LevelFilter {
    std::env::var("RUST_LOG")
        .ok()
        .and_then(|s| LevelFilter::from_str(s.as_str()).ok())
        .unwrap_or(LevelFilter::INFO)
}

/// Returns the guard config
pub fn get_config() -> GuardResult<Config> {
    let config_path = if let Ok(path) = std::env::var(CONFIG_PATH_ENV) {
        PathBuf::from(path)
    } else if matches!(fs::exists(DEFAULT_CONFIG_PATH), Ok(true)) {
        PathBuf::from(DEFAULT_CONFIG_PATH)
    } else {
        return Err(GuardError::CantGetConfigFile);
    };

    tracing::info!("Config path: {}", config_path.display());
    toml::from_str(&fs::read_to_string(&config_path)?).map_err(Into::into)
}
