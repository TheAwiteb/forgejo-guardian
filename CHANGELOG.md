# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## unreleased
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
