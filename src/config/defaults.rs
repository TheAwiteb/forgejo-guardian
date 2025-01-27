// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

/// A constant function that returns `true`.
pub const fn bool_true() -> bool {
    true
}

/// Defult configuration for expressions section
pub mod expressions {
    use crate::config::BanAction;

    /// Default interval for checking for new users.
    pub const fn interval() -> u32 {
        300
    }
    /// Default limit for the amount of users to check per fetch.
    pub const fn limit() -> u32 {
        100
    }

    /// Default ban action to take when banning a user.
    pub const fn ban_action() -> BanAction {
        BanAction::Purge
    }

    pub const fn req_limit() -> u32 {
        200
    }

    pub const fn req_interval() -> u32 {
        10 * 60
    }
}

/// Default configuration for inactive section.
pub mod inactive {
    pub const fn enabled() -> bool {
        false
    }
    pub const fn days() -> u64 {
        30
    }
    pub const fn req_limit() -> u16 {
        200
    }
    pub const fn req_interval() -> u32 {
        10 * 60
    }
    pub const fn interval() -> u32 {
        // 7 days
        7 * 24 * 60 * 60
    }
}
