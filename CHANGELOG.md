# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## unreleased
### Added
-  Ability to check user tokens and oauth2 apps ([**#25**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/25)) ([`1e90760`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/1e907609cd9fae24e58cbe9eab99cc4b88459cb3))
    - **BC**:  The minimum value for `inactive.req_limit` changed to 4
-  Ability to enter seconds in the interval without `s` suffix ([**#28**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/28)) ([`c9dfc6e`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/c9dfc6e57acdfbdbc2485d729ce24edcab292224))
-  Make `expressions.interval` suffixably ([**#29**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/29)) ([`e32aca7`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/e32aca7164669532ca08ee356fc01aaf6185aa67))
### Fixed
-  Guardian checks all instance users when `expressions.only_new_users` is `false` ([**#27**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/27)) ([`cf05a68`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/cf05a68c22f1eeffc3308856c7117ff6d82855da))

## [0.4.1](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.4.0..v0.4.1) - 2025-01-22
### Added
-  Add support for including and excluding users ([**#22**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/22)) ([`f07fdab`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/f07fdaba7e9a37b87848e6d0bbb6b639c84cfd95))
### Fixed
-  Check for the user activities for more than last 365 days ([`e41b9b3`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/e41b9b36f67ff465731a20c1baaca9a9e6440bd0))
-  Exceed `inactive.req_limit` due to asynchronous user checking ([**#20**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/20)) ([`71a0c51`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/71a0c5114b69b4d15c2b9e41abbe80552ed0b234))

## [0.4.0](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.3.1..v0.4.0) - 2025-01-19
### Added
-  Ability to Enable/disable `sus` and `ban` expressions, and Telegram bot ([`a89a675`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/a89a675a210685553a2926c2a41a12c02ab33163))
    - **BC**:  `telegram.ban_alert` has been moved to the global scope
-  Ability to fetch Forgejo token from environment variables ([**#15**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/15)) ([`2bdbf4b`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/2bdbf4b5f234c8654b51b88a778394543c00f79e))
-  Clean up instance of inactive users ([**#9**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/9)) ([`d83e49b`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/d83e49bcf6ec606f334b9451ad7dc3430152a3bf))
### Fixed
-  Move unglobal fields to the expressions section ([**#18**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/18)) ([`a82cd4b`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/a82cd4bc5741a41ab76aef16967de6e1d72bfe50))
    - **BC**:  Move unglobal fields to the expressions section

## [0.3.1](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.3.0..v0.3.1) - 2025-01-17
### Added
-  Ability to suspend the user instead of deleting them permanently ([**#7**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/7)) ([`48e7057`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/48e70572e2b1b48321637e55fbdf25180ed8cccd))

## [0.3.0](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.2.0..v0.3.0) - 2025-01-16
### Added
-  Add `interval` and `limit` configurations ([`251fc20`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/251fc209a2d642c3b804a74b08d2bad32d7c3165))
-  Dockerize the guardian with `Dockerfile` and `docker-compose` ([`688ec77`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/688ec77c77c8b4a3d9e0b5d6e5715c175210e63c))

## [0.2.0](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.1.0..v0.2.0) - 2024-12-09
### Added
-  Add the email to user details message ([`b3397f6`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/b3397f63163b6679248a680a7ab423d7852df647))
-  Possibility to put an array of expressions and they must all matches ([`cc2f8a7`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/cc2f8a791b2c1be8cae2f6ba9dfd0a718d4d3c71))
-  Reason for banned and suspicious ([`c96b859`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/c96b859931d893751b15977f2ede7034b46628e7))
### Fixed
-  Matching users multiline description correctly ([`3d6b49c`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/3d6b49c01a61d6ee18da488dbc1d1fbf5caedf3c))

## 0.1.0 - 2024-11-16
### Added
-  Add Russian language ([`8dc8c0d`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/8dc8c0d2315d7d47f6f2605fcdfd62499a4c4460))
-  Add telegram bot to the config ([`68cd88e`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/68cd88e96af0cd92c10e30ec9675f003c89c436f))
-  Checks only new users configuration ([`f68ce0c`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/f68ce0c5bd86e1a637736219f0e952831fe8cc7b))
-  Dry run mode ([`c3972b3`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/c3972b356642c3977b4a2477e4a5f1acd3db868f))
-  Initialize `forgejo-guardian` ([`d12c45e`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/d12c45ed637b0ba1e42b73fe46520e65b0d0dfd9))
-  Notification when users are banned ([`6070ca0`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/6070ca035cb6f18b18a2e467240c06d6df3c6092))
-  Send sus alert via telegram ([`5bb6114`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/5bb6114aa77e629fcc0c12177b401ac7ab287db2))
### Fixed
-  Respect `telegram.ban_alert` configuration ([`9b533e7`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/9b533e7ea37741808e63500e6f7b3273cfcb8e5a))
-  Split the haystack lines ([`8803305`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/880330576dffa09909beee8c1ec3570f40915adc))

This changelog was generated by [git-cliff](https://github.com/orhun/git-cliff)
