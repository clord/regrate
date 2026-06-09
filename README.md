# Regrate migration management

[![Crates.io](https://img.shields.io/crates/v/regrate.svg)](https://crates.io/crates/regrate)
[![Docs.rs](https://docs.rs/regrate/badge.svg)](https://docs.rs/regrate)
[![CI](https://github.com/clord/regrate/workflows/CI/badge.svg)](https://github.com/clord/regrate/actions)

## Introduction

Most migration tools name migrations with serial numbers or timestamps, so
two people can land conflicting migrations without ever noticing. Regrate
names each migration by hashing the previous one: if anyone else claims the
next slot in the chain, you get an ordinary git merge conflict instead of a
silent ordering bug — and `regrate resolve` turns that conflict back into a
work-in-progress migration you can re-commit on top of theirs.

Regrate doesn't talk to your database and doesn't track what has been
applied; it manages an ordered, tamper-evident directory of migration
scripts and runs the command of your choice over them, in order.

## Quick start

    # set up a shell-based migration repo (also: postgres, mysql)
    regrate init shell

    # start a migration and edit its scripts
    regrate create
    $EDITOR regrate/current/up.sh
    $EDITOR regrate/current/down.sh

    # run committed migrations in order; --current includes your WIP one
    regrate run --current sh {up}

    # you can run any command, with substitutions
    regrate run echo {name}

    # if your migrations are SQL files, pass them to your client
    regrate run sqlite3 mydb.sqlite ".read {up}"

    # happy with it? seal it into the chain and commit with git
    regrate commit -m "add two columns"
    git add -A && git commit -m "add two columns migration"

If someone else pushed a migration first, your `git rebase` (or merge) will
conflict inside `regrate/store`. Then:

    # keep theirs in the store, move your version back to regrate/current
    regrate resolve

    # finish the rebase/merge, re-test your migration, and re-commit it
    regrate commit -m "add two columns (rebased)"

## Commands

| command | what it does |
|---|---|
| `regrate init <shell\|postgres\|mysql>` | create the `regrate/` directory, config, and script templates |
| `regrate create` | start a new `regrate/current` migration from the template |
| `regrate commit -m <msg>` | seal `current` into the store under its chain name |
| `regrate run [--current] <cmd>...` | run `<cmd>` once per migration, in order |
| `regrate valid` | verify the store matches the name chain; lists orphans |
| `regrate resolve` | after a git conflict in the store, keep upstream's migration and move yours back to `current` |
| `regrate generate <shell>` | emit shell completions |

### `run` substitutions and environment

Arguments to `regrate run` that exactly match a placeholder are replaced
for each migration: `{name}`, `{path}`, `{up}`, `{down}`, `{next-name}`,
`{next-path}`. Placeholders inside larger strings are deliberately not
expanded.

Each invocation also receives environment variables: `REGRATE_INDEX`
(zero-based position in the chain), `REGRATE_NAME`, `REGRATE_PATH`, and —
for committed migrations — `REGRATE_NEXT_NAME` and `REGRATE_NEXT_PATH`.

Your scripts should be idempotent, as they are executed in order from the
start every time. Regrate does not track the applied version; in a stateful
environment, a script can compare `REGRATE_INDEX` (or `REGRATE_NAME`)
against a value it stores to decide whether it should run.

## How it works

The "current version" is left open: you can keep editing it, and the name
it will receive is always well known in advance. To compute the next name,
hash the current name together with the current migration's files (sorted
by path, with file names and sizes mixed in, so the result doesn't depend
on filesystem order). Repeating this from a fixed seed yields the whole
chain of names.

Because names are derived from contents, committed migrations are no longer
editable: editing one invalidates the name of every migration after it.
`regrate valid` detects this, and `regrate run` refuses to run a broken
chain rather than silently skipping unreachable migrations. `regrate valid`
works well as a pre-commit or CI check.

And because the next name is the same for everyone, two people committing
"the next migration" produce a git conflict in the same store path —
which `regrate resolve` knows how to untangle.

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* Run `cargo install regrate`

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
