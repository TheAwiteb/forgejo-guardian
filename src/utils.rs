// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

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
