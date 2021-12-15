# Regrate migration management

## Introduction

Manage migrations, apply them in order, deal with merge conflicts.

    # create a new shell-based migration
    regrate init --shell
    # Edit the up and down scripts (bash since --shell was used)
    edit regrate/latest/up.sh
    edit regrate/latest/down.sh
    # Run the current migration (to test it out) by passing them to bash in order
    regrate run -- bash {}
    # Once the migration is working right, we can commit it.
    regrate commit -m "add two columns"
    # With our migrations complete, we can push it up!
    git add -A && git commit -m "initial migration"
    # Try to push it. first rebase onto latest...
    git rebase main ## ERROR, conflicts! 
    # Someone else already pushed a migration. you can either:
    # 1. merge the migrations together (you're in dev and willing to reset your db)
    # 2. better idea would be to move your local changes to a new migration:
    regrate resolve
    # regrate-resolve moves your local changes into a new migration, 
    # leaving your peer's as the base.
    edit regrate/latest/up.sh # your peer added one of your columns!
    regrate commit -m "add one column"
    git add -A && git rebase --continue
    git push # Yay!

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

[![Crates.io](https://img.shields.io/crates/v/regrate.svg)](https://crates.io/crates/regrate)
[![Docs.rs](https://docs.rs/regrate/badge.svg)](https://docs.rs/regrate)
[![CI](https://github.com/clord/regrate/workflows/CI/badge.svg)](https://github.com/clord/regrate/actions)
[![Coverage Status](https://coveralls.io/repos/github/clord/regrate/badge.svg?branch=main)](https://coveralls.io/github/clord/regrate?branch=main)

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install regrate`

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
