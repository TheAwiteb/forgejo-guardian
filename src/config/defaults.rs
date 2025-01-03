// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

/// Global configuration defaults.
pub mod global {
    /// Default interval for checking for new users.
    pub fn interval() -> u32 {
        300
    }
    /// Default limit for the amount of users to check per fetch.
    pub fn limit() -> u32 {
        100
    }
}
