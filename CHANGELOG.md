# Change Log

Changes to this project will be written here.

Version number are changed in accordance with the [Semantic Versioning Specification](https://semver.org/). (maybe)

***

## [1.0.1](https://github.com/Fairy-Phy/estella/tree/1.0.1) - 2022-06-16

### Fixed

* Fix debug command bug: `estella.insert_sub`

  Change to `estella.sub_insert`

## [1.0.0](https://github.com/Fairy-Phy/estella/tree/1.0.0) - 2022-06-12

### Added

* Repository created

* Project push

* DB Table

  * main-account

  * sub-account

  * confirmed-account

  * pending-account

  * user-data

  * guild-config

  * (level) <- unused. Leveling system is not available Estella project

* commands (public)

  * `/estella user reserve (id) (name) [reason]`

  * `/estella user sub_application (id) (name)`

  * `/estella user find [id]`

  * `/estella version`

  * `/estella ping`

  * `/estella config`

* commands (bot owner only debug command)

  * `estella.test`

  * `estella.rep`

  * `estella.insert (uid) (name) (version) (is_sc) (is_leave)`

  * `estella.g_init`

  * `estella.insert_sub (uid) (name) (main_uid)`

  * `estella.create`

  * `estella.delete (command_id)`

  * `estella.logout`
