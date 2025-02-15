// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use reqwest::StatusCode;
use url::Url;

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
    #[error("Matrix Client: {0}")]
    MatrixClient(#[from] matrix_sdk::Error),
    #[error("Database error: {0}")]
    RedbError(#[from] redb::Error),
    /// Invalid response from Forgejo
    #[error("Invalid response from Forgejo, the error `{0:?}` from `{1}`")]
    InvalidForgejoResponse(String, Url),
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
    #[error("Matrix Error: {0}")]
    Matrix(String),
    /// Other errors, for custom errors
    #[error("{0}")]
    Other(String),
}

impl From<redb::TransactionError> for GuardError {
    fn from(err: redb::TransactionError) -> Self {
        Self::RedbError(err.into())
    }
}

impl From<redb::StorageError> for GuardError {
    fn from(err: redb::StorageError) -> Self {
        Self::RedbError(err.into())
    }
}

impl From<redb::TableError> for GuardError {
    fn from(err: redb::TableError) -> Self {
        Self::RedbError(err.into())
    }
}

impl From<redb::CommitError> for GuardError {
    fn from(err: redb::CommitError) -> Self {
        Self::RedbError(err.into())
    }
}
