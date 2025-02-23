# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## unreleased
### Added
-  Lazy purge ([**#55**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/55)) ([`a1d6e20`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/a1d6e20eda29fe096dad07fc59635b43090b850b))
### Changed
-  Move `inactive.check_tokens|check_oauth2` to global scope ([**#54**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/54)) ([`a9eb1c9`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/a9eb1c9e99acc74b039dc29c03bcf70679141966))
    - **BC**:  Check #54
-  Sotre last 7 users in updated_users fetcher ([`81b9d0d`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/81b9d0dcb14a86049b77363c2c56cb4112e3f902))
### Fixed
-  Prevent multiple alerts for the same user update ([`41efebc`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/41efebc5832343a7ec1dfb779c4303ab494cf633))

## [0.6.0](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.5.1..v0.6.0) - 2025-02-18
### Added
-  Add German translation ([**#37**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/37)) ([`768c6a8`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/768c6a84893784b40e3b920f02d835a00903f349))
-  Add `safe_mode` to prevent banning active users and notify moderation team ([**#35**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/35)) ([`c5a75d1`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/c5a75d16b5da579f7ac4a5e97cafd628f28cf4da))
-  Add support for Matrix bot alongside Telegram bot ([**#36**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/36)) ([`2e1c1d5`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/2e1c1d5fe43154b6b41d3c02b2c997b8d296b12b))
-  Don't send alerts of ignored users ([**#38**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/38)) ([`cb85d6e`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/cb85d6ef742a7e80425b9e763ff61e93145c8c54))
-  Option to hide users email ([**#47**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/47)) ([`41995f1`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/41995f13ba3e1e953869dca09d5ee141dc8b3d6e))
-  Update fetcher to fetch updated users ([**#42**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/42)) ([`4741909`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/474190902f999862bf6ac261150d661b1b2624cd))
-  bot command to ban a user ([**#43**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/43)) ([`54c551b`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/54c551bf9a119d7600f93db2c38dd60a740721fd))
### Changed
-  New local `not_specified` used for the regex reason ([`2c76dce`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/2c76dce21e613686702039c10645c4aefd1e5330))
-  Rename `only_new_users` to `check_existing_users` ([**#48**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/48)) ([`e7f92d7`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/e7f92d7f8f312512a17d1f0f2c5a25c0d61d3ba2))
    - **BC**:  Rename `expressions.only_new_users` to `expressions.check_existing_users`
-  Fetch all new users instead of only the first page ([**#39**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/39)) ([`f79b5c8`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/f79b5c8f77cdf889a0f5977c8f8f0503cc039bac))
### Fixed
-  Fix safe mode condition ([`f741862`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/f7418628a43772dad86340a40764f855ca473864))

## [0.5.1](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.5.0..v0.5.1) - 2025-01-29
### Added
-  Enhance ban/sus logs to display the location ([**#33**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/33)) ([`f39e874`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/f39e874446baf647c3697e5fa8287a604b99b7f5))

## [0.5.0](https://git.4rs.nl/awiteb/forgejo-guardian/compare/v0.4.1..v0.5.0) - 2025-01-28
### Added
-  Ability to check user tokens and oauth2 apps ([**#25**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/25)) ([`1e90760`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/1e907609cd9fae24e58cbe9eab99cc4b88459cb3))
    - **BC**:  The minimum value for `inactive.req_limit` changed to 4
-  Ability to enter seconds in the interval without `s` suffix ([**#28**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/28)) ([`c9dfc6e`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/c9dfc6e57acdfbdbc2485d729ce24edcab292224))
-  Make `expressions.interval` suffixably ([**#29**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/29)) ([`e32aca7`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/e32aca7164669532ca08ee356fc01aaf6185aa67))
### Fixed
-  Guardian checks all instance users when `expressions.only_new_users` is `false` ([**#27**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/27)) ([`cf05a68`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/cf05a68c22f1eeffc3308856c7117ff6d82855da))
-  Prevent async deadlock when Telegram is disabled ([**#31**](https://git.4rs.nl/awiteb/forgejo-guardian/issues/31)) ([`2ee7849`](https://git.4rs.nl/awiteb/forgejo-guardian/commit/2ee784916305d32705d6a667ce1979c47f67874f))

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
