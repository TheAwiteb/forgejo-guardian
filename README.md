<div align="center">

# Forgejo Guardian

Simple Forgejo instance guardian, banning users and alerting admins based on certain regular expressions (regex)

[![Forgejo CI Status](https://git.4rs.nl/awiteb/forgejo-guardian/badges/workflows/ci.yml/badge.svg)](https://git.4rs.nl/awiteb/forgejo-guardian)
[![Forgejo CD Status](https://git.4rs.nl/awiteb/forgejo-guardian/badges/workflows/cd.yml/badge.svg)](https://git.4rs.nl/awiteb/forgejo-guardian)
[![Minimum Forgejo version v10.0.0](https://img.shields.io/badge/Minimum_Forgejo_version-v10.0.0-brightgreen?style=flat&color=ff7f24&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyMTIgMjEyIiB3aWR0aD0iMzIiIGhlaWdodD0iMzIiPjxzdHlsZT5jaXJjbGUscGF0aHtmaWxsOm5vbmU7c3Ryb2tlOiMwMDA7c3Ryb2tlLXdpZHRoOjE1fXBhdGh7c3Ryb2tlLXdpZHRoOjI1fS5vcmFuZ2V7c3Ryb2tlOiNmNjB9LnJlZHtzdHJva2U6I2Q0MDAwMH08L3N0eWxlPjxnIHRyYW5zZm9ybT0idHJhbnNsYXRlKDYgNikiPjxwYXRoIGQ9Ik01OCAxNjhWNzBhNTAgNTAgMCAwIDEgNTAtNTBoMjAiIGNsYXNzPSJvcmFuZ2UiLz48cGF0aCBkPSJNNTggMTY4di0zMGE1MCA1MCAwIDAgMSA1MC01MGgyMCIgY2xhc3M9InJlZCIvPjxjaXJjbGUgY3g9IjE0MiIgY3k9IjIwIiByPSIxOCIgY2xhc3M9Im9yYW5nZSIvPjxjaXJjbGUgY3g9IjE0MiIgY3k9Ijg4IiByPSIxOCIgY2xhc3M9InJlZCIvPjxjaXJjbGUgY3g9IjU4IiBjeT0iMTgwIiByPSIxOCIgY2xhc3M9InJlZCIvPjwvZz48L3N2Zz4=)](https://codeberg.org/forgejo/forgejo/milestone/8377)

[![agplv3-or-later](https://www.gnu.org/graphics/agplv3-88x31.png)](https://www.gnu.org/licenses/agpl-3.0.html)

</div>

> [!NOTE]
> The minimum Forgejo version required is `v10.0.0`, because of this [PR](https://codeberg.org/forgejo/forgejo/pulls/6228).

## About

`forgejo-guardian` is a simple guardian for your Forgejo instance, it will ban
users based on certain regular expressions (regex) and alert the admins about
them. The alert will send to the admin via a Telegram bot.

### The expressions

See the [Configuration](#configuration) section for more information about the
expressions. The different between `ban` and `sus` is that the `ban` will ban
the user, and the `sus` will only alert the admins about the user and the admin
can decide to ban the user or not (via telegram inline keyboard). You can also
set `telegram.ban_alert` to `true` to send a notification when a user is banned.

### Ban action

The ban action can be `purge` or `suspend`, the default is `purge`. The `purge`
will delete the user and all their data, and the `suspend` will only suspend the
user, the suspended user can be unsuspended later by the admin from the
dashboard.

### Clean up instance of inactive users

The guardian can also clean up inactive users by setting `inactive.enabled` to
`true` in the configuration file and specifying the number of days in
`inactive.days` to consider a user inactive. The guardian will ban users who
have had no activity since they registered.

Inactivity feature need `read:user` and `read:admin` scopes.

## Docker

If you want to run the guardian in a docker container, you can find the
`Dockerfile` and `docker-compose.toml` in the
[docker](https://git.4rs.nl/awiteb/forgejo-guardian/src/branch/master/docker)
directory. Copy them or clone the repository, make sure to have `Dockerfile`,
`docker-compose.toml` and your configuration file `forgejo-guardian.toml` (see
[Configuration](#Configuration) section) in the same directory, then you can run
the following command:

```sh
docker-compose up -d # To run the guardian in the background (remove `-d` first time to see the logs and make sure everything is working)
```

## Without building the image

If you don't want to build the image yourself, you can use this docker-compose file:

```yaml
services:
    forgejo-guardian:
        image: git.4rs.nl/awiteb/forgejo-guardian:0.3
        volumes:
            - ./forgejo-guardian.toml:/app/forgejo-guardian.toml:ro
```

Make sure to have the `forgejo-guardian.toml` file in the same directory as the `docker-compose.yml` file, then you can run the following command:

```sh
docker-compose up -d
```

### Without docker-compose

#### Build the image

```sh
docker build -t forgejo-guardian .
docker run --rm -d -v $PWD/forgejo-guardian.toml:/app/forgejo-guardian.toml:ro forgejo-guardian
```

#### Without building the image

```sh
docker run --rm -d -v $PWD/forgejo-guardian.toml:/app/forgejo-guardian.toml:ro git.4rs.nl/awiteb/forgejo-guardian:0.3
```

## Installation

You can let [cargo](https://doc.rust-lang.org/cargo/) build the binary for you, or build it yourself. You can also download the pre-built binaries from the [releases](https://git.4rs.nl/awiteb/forgejo-guardian/releases) page.

### Build it

#### `cargo-install`

> [!TIP]
> This will install the binary in `~/.cargo/bin/forgejo-guardian`. Make sure to add this directory to your `PATH`.
> If you want to update it, rerun the command.

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

In our configuration file you can have the following sections and the global section:

-   `inactive`: Configuration for cleaning up inactive users
-   `forgejo`: Forgejo instance configuration
-   `expressions`: Regular expressions to match against
-   `telegram`: Telegram bot configuration

#### Global section

The global section is the one that doesn't have a name, and it's in the top of the configuration file, with the following fields:

-   `dry_run`: If set to `true`, the guardian will not ban the users, but will only alert the admins (default: `false`)
-   `only_new_users`: If set to `true`, the guardian will only check the new users, and not the existing ones (default: `false`)
-   `interval`: Interval to check for new users in seconds (default: `300`)
-   `limit`: Limit of users to fetch in each interval (default: `100`)
-   `ban_action`: The action to take when a user is banned, can be one of the following:
    -   `purge` (default): Forcibly delete user and any repositories, organizations, and
        packages owned by the user. All comments and issues posted by this user
        will also be deleted. (unduoable, the user will be permanently deleted)
    -   `suspend`: Block the user from interacting with the service through their
        account and prohibit signing in. The admins can later decide to
        reactivate the user, from the dashboard.

> [!TIP]
> Make sure to set `interval` and `limit` to a reasonable values based on your
> instance size and the number of new users. If your instance is small, you can
> set `interval` to a higher value (something like `600`) and `limit` to a lower
> value (something like `50`), so the guardian will fetch latest 50 users every
> 10 minutes, which should be enough for small instances.

```toml
dry_run = true
only_new_users = true
interval = 40
limit = 50
ban_action = "suspend"
```

#### `inactive`

Inactive users configuration section, with the following fields:

-   `enabled`: Enable the cleanup of inactive users, inactive feature need `read:user` scope (default: `false`)
-   `days`: Number of days to consider a user inactive (default: `30`)
-   `req_limit`: Maximum number of requests to send to the Forgejo instance within each interval (default: `200`)
-   `req_interval`: Time interval to pause after reaching the `req_limit` (default: `10m`)
-   `interval`: Time Interval to check for inactive users (default: `7d`)

The `inactive.req_interval` and `inactive.interval` have the following suffixes:

-   `s`: Seconds
-   `m`: Minutes
-   `h`: Hours
-   `d`: Days

```toml
[inactive]
enabled = true
days = 30
req_limit = 200
req_interval = "10m"
interval = "7d"
```

> [!NOTE]
>
> Forgejo itself has no rate limiting, but the reverse proxy may have rate
> limiting.

#### `forgejo`

Forgejo configuration section, with the following fields:

-   `instance_url`: Forgejo instance URL (must be HTTPS or HTTP)
-   `token`: Token to use to get the new users and ban them, requires
    `read:admin` and `write:admin` scopes. The token can be retrieved from an
    environment variable by prefixing the variable name with `env.`. For
    example, use `env.FORGEJO_TOKEN` to get the token from the `FORGEJO_TOKEN`
    environment variable.

```toml
[forgejo]
instance_url = "https://forgejo.example"
token = "your-token"
```

#### `expressions`

Expressions configuration section, with the following fields:

-   `ban`: Regular expressions to match against to ban the user
-   `sus`: Regular expressions to match against to alert the admins

`ban` and `sus` are tables, and each one have the following fields:

-   `usernames`: Regular expressions to match against the usernames
-   `full_names`: Regular expressions to match against the full names
-   `biographies`: Regular expressions to match against the biographies
-   `emails`: Regular expressions to match against the emails
-   `websites`: Regular expressions to match against the websites
-   `locations`: Regular expressions to match against the locations

Each field is an array of regular expressions, the regular expression can be one of the following:

-   String: The regular expression itself
-   Table: The regular expression and the reason, with the following fields:
    -   `re` (string, array of string): The regular expression (if it's an array of strings, all regex in that array should match to ban/sus the user)
    -   `reason` (optional string): The reason to ban/sus the user. This will be used in the notification message.

```toml
[expressions.ban]
usernames = ['^admin.*$']
websites = ['^https://example\.com$', { re = '^https://example2\.com$', reason = "Example 2 is not allowed" }, '^https://example3\.com$']

[expressions.sus]
usernames = ['^mod.*$']
```

#### `telegram`

Telegram bot configuration section, with the following fields:

-   `token`: Telegram bot token
-   `chat`: Chat ID to send the alerts to (Can be a group or a channel or a user)
-   `ban_alert`: Send a notification when a user is banned (default: `false`)
-   `lang`: Language to use for the alerts (Currently only `ar-sa`, `en-us` and `ru-ru` are supported)

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
