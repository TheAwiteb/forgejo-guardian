// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use std::{env, fs, path::PathBuf, str::FromStr, time::Duration};

use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;

use crate::{
    config::{parse_invalid, Config, Matrix, Telegram, CONFIG_PATH_ENV, DEFAULT_CONFIG_PATH},
    error::{GuardError, GuardResult},
};

/// Checks for warnings in the config
fn check_warnings(config: &Config) {
    if config.inactive.enabled && (config.inactive.req_interval > config.inactive.interval) {
        tracing::warn!(
            "The inactive request interval is greater than the inactive interval, \
             `inactive.req_interval` is intended to prevent hitting the rate limit *during* the \
             process."
        );
    }

    if config.expressions.sus.enabled
        && !config.telegram.is_enabled()
        && !config.matrix.is_enabled()
    {
        tracing::warn!(
            "The suspicious users expressions are enabled but the Telegram and Matrix bot is \
             disabled, the suspicious users will not be alerted"
        );
    }
}

/// Checks for errors in the config
fn check_errors(config: &Config) -> GuardResult<()> {
    if let Telegram::Invalid(toml_value) = &config.telegram {
        return Err(GuardError::Other(format!(
            "Configuration Error: {}",
            parse_invalid::invalid_telegram(toml_value,)
        )));
    }
    if let Matrix::Invalid(toml_value) = &config.matrix {
        return Err(GuardError::Other(format!(
            "Configuration Error: {}",
            parse_invalid::invalid_matrix(toml_value,)
        )));
    }

    if config.expressions.safe_mode {
        if !config.expressions.ban_action.is_purge() {
            return Err(GuardError::Other(
                "Safe mode is enabled, but the ban action is not set to `purge`, there is no \
                 point in enabling safe mode if the ban action is not set to `purge`"
                    .to_owned(),
            ));
        }
        if config.expressions.req_limit < 4 {
            return Err(GuardError::Other(
                "Safe mode is enabled, it's need more requests to check if the user is inactive \
                 or not, so the request limit must be greater than 3"
                    .to_owned(),
            ));
        }
        if !config.telegram.is_enabled() && !config.matrix.is_enabled() {
            return Err(GuardError::Other(
                "Safe mode is enabled, but Telegram and Matrix bot is disabled, the safe mode \
                 need to send a ban request to the moderation team"
                    .to_owned(),
            ));
        }
    }

    if config.telegram.is_enabled() && config.matrix.is_enabled() {
        return Err(GuardError::Other(
            "Both Telegram and Matrix bot is enabled, only one can be enabled at a time".to_owned(),
        ));
    }

    if !config.expressions.ban_action.is_purge() && config.lazy_purge.enabled {
        return Err(GuardError::Other(
            "Lazy purge is enabled, but the ban action is not set to `purge`".to_owned(),
        ));
    }

    Ok(())
}

/// Checks if the Forgejo token is specified as an environment variable.
///
/// If the token starts with the prefix `env.`, the remainder of the token is
/// treated as the name of an environment variable from which the actual token
/// value is retrieved.
fn check_forgejo_token(config: &mut Config) -> GuardResult<()> {
    if config.forgejo.token.starts_with("env.") {
        let (_, env_var) = config.forgejo.token.split_once('.').expect("unreachable");
        let env_var = env::var(env_var).map_err(|_| {
            GuardError::Other(format!("Environment variable `{env_var}` not found"))
        })?;
        config.forgejo.token = env_var;
    }

    Ok(())
}

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
    let mut config =
        toml::from_str(&fs::read_to_string(&config_path)?).map_err(GuardError::from)?;

    check_errors(&config)?;
    check_warnings(&config);
    check_forgejo_token(&mut config)?;

    Ok(config)
}

/// Wait for the interval to pass, if the cancellation token is cancelled,
/// return true, after the interval has passed return false
pub async fn wait_interval(req_interval: u32, cancellation_token: &CancellationToken) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(req_interval.into())) => false,
        _ = cancellation_token.cancelled() => true,
    }
}
