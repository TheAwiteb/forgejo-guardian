<div align="center">

# Forgejo Guardian

Simple Forgejo instance guardian, banning users and alerting admins based on certain regular expressions (regex)

<!-- [![Forgejo CI Status](https://git.4rs.nl/awiteb/forgejo-guardian/badges/workflows/ci.yml/badge.svg)](https://git.4rs.nl/awiteb/forgejo-guardian)
[![Forgejo CD Status](https://git.4rs.nl/awiteb/forgejo-guardian/badges/workflows/cd.yml/badge.svg)](https://git.4rs.nl/awiteb/forgejo-guardian) -->

[![agplv3-or-later](https://www.gnu.org/graphics/agplv3-88x31.png)](https://www.gnu.org/licenses/agpl-3.0.html)

</div>

## Installation

You can let [cargo](https://doc.rust-lang.org/cargo/) build the binary for you, or build it yourself. <!-- You can also download the pre-built binaries from the [releases](https://git.4rs.nl/awiteb/forgejo-guardian/releases) page. -->

### Build it

#### `cargo-install`

> [!TIP]
> This will install the binary in `~/.cargo/bin/forgejo-guardian`. Make sure to add this directory to your `PATH`.
> If you want to update it, run `cargo install ...` again.

```sh
cargo install --git https://git.4rs.nl/awiteb/forgejo-guardian
```

#### `cargo-install` (from source)

> [!TIP]
> Then when you want to update it, pull the changes and run `cargo install --path .` again.

```sh
git clone https://git.4rs.nl/awiteb/forgejo-guardian
cd forgejo-guardian
cargo install --path .
```

#### Build (from source)

> [!TIP]
> The binary will be in `./target/release/forgejo-guardian`.

```sh
git clone https://git.4rs.nl/awiteb/forgejo-guardian
cd forgejo-guardian
cargo build --release
```

## Configuration

We use `TOML` format for configuration, the default configuration file is `/app/forgejo-guardian.toml`, but you can specify a different one with `FORGEJO_GUARDIAN_CONFIG` environment variable.

### Structure

In our configuration file you can have the following sections and the global sections:

- `forgejo`: Forgejo instance configuration
- `expressions`: Regular expressions to match against
- `telegram`: Telegram bot configuration

#### Global sections

The global section is the one that doesn't have a name, and it's in the top of the configuration file, with the following fields:

- `dry_run`: If set to `true`, the guardian will not ban the users, but will only alert the admins (default: `false`)

#### `forgejo`

Forgejo configuration section, with the following fields:

- `instance_url`: Forgejo instance URL (must be HTTPS or HTTP)
- `token`: Token to use to get the new users and ban them, requires `read:admin` and `write:admin` scopes.

```toml
[forgejo]
instance_url = "https://forgejo.example"
token = "your-token"
```

#### `expressions`

Expressions configuration section, with the following fields:

- `ban`: Regular expressions to match against to ban the user
- `sus`: Regular expressions to match against to alert the admins

`ban` and `sus` are tables, and each one have the following fields:

- `usernames`: Regular expressions to match against the usernames
- `full_names`: Regular expressions to match against the full names
- `biographies`: Regular expressions to match against the biographies
- `emails`: Regular expressions to match against the emails
- `websites`: Regular expressions to match against the websites
- `locations`: Regular expressions to match against the locations

```toml
[expressions.ban]
usernames = ['^admin.*$']

[expressions.sus]
usernames = ['^mod.*$']
```

#### `telegram`

Telegram bot configuration section, with the following fields:

- `token`: Telegram bot token
- `chat`: Chat ID to send the alerts to (Can be a group or a channel or a user)
- `ban_alert`: Send a notification when a user is banned (default: `false`)
- `lang`: Language to use for the alerts (Currently only `ar-sa`, `en-us` and `ru-ru` are supported)

```toml
[telegram]
token = "your-token"
chat = 00000000000
lang = "en-us"
```

## Running the guardian

After you have the configuration file ready, you can run the guardian with the following command:

```sh
FORGEJO_GUARDIAN_CONFIG=/path/to/your/config.toml forgejo-guardian
```

You can remove the `FORGEJO_GUARDIAN_CONFIG` environment variable from the command if it's already set, or the file in the default location `/app/forgejo-guardian.toml`.

### Adding a new language

If you would like to contribute by adding a new language, you can do that by adding your language file in the `locales` directory, and then add it to `Lang` enum in `src/telegram_bot/mod.rs` file. Then you can use it in the configuration file.

## License

This project is licensed under the [AGPL-3.0-or-later](https://www.gnu.org/licenses/agpl-3.0.html) license.
