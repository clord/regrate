# Regrate migration management

## Introduction

Manage migrations, apply them in order, deal with merge conflicts.

    # create a new shell-based migration
    regrate init shell

    # Edit the up and down scripts (bash since --shell was used)
    edit regrate/current/up.sh
    edit regrate/current/down.sh

    # Run migrations (including current) by executing them in order:
    regrate run --current {up}

    # You can run any old command and do substitution:
    regrate run --current echo {name}

    # if your migrations are in SQL files, you might want to pass those to a command:
    regrate run sqlite3 {up}

    # Once the migration is working right, we can commit it
    regrate commit -m "add two columns"

    # With our migrations complete, we can push it up!
    git add -A && git commit -m "initial migration"

    # Try to push it. first rebase onto current...
    git rebase main ## ERROR, conflicts!

    # Someone else already pushed a migration.
    # move your local changes to a new migration and try again.
    # in future we will have a command that resolves conflicts
    # regrate resolve
    # regrate-resolve moves your local changes into a new migration, 
    # leaving your peer's as the base.

In summary,
if anyone claims the next migration name
you get a merge conflict with their change.
Regrate can help resolve your conflict markers into a new migration.

Your scripts should be idempotent as they will be executed in order
from the start every time.
The current runtime version is not tracked by regrate.
To help, `regrate-run` will set `REGRATE_INDEX` for each step of the migration,
which can help with keeping track of whether a given script should run in a stateful
environment (on completion, script would update a shared variable to `REGRATE_INDEX`).

`regrate` will derive names from the contents of the migration scripts.
This means migration scripts are no longer editable once committed. `regrate-valid`
will complain about hash mismatches.
This can be used in a pre-commit hook to detect bad changes.

Also provided is a library that can be used to iterate migrations
in the current directory. This is helpful for runtime. In this case,
it is your responsibility to invoke the scripts as you see fit.
Packages for other languages also exist and let you run migrations.

[![Crates.io](https://img.shields.io/crates/v/regrate.svg)](https://crates.io/crates/regrate)
[![Docs.rs](https://docs.rs/regrate/badge.svg)](https://docs.rs/regrate)
[![CI](https://github.com/clord/regrate/workflows/CI/badge.svg)](https://github.com/clord/regrate/actions)
[![Coverage Status](https://coveralls.io/repos/github/clord/regrate/badge.svg?branch=main)](https://coveralls.io/github/clord/regrate?branch=main)

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* Run `cargo install regrate`

## How it works

The "current version" is left open.
You can make changes to it,
and its name is always well known.
If two people try to change the open version, it will be a merge conflict.
To compute the next version from the current version, take the current version string,
compute the hash of all migration scripts in that version,
and concatenate the version and the hash.
Hash the result and this is your next version name.
This sequence can be repeated to generate a list of names.

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
