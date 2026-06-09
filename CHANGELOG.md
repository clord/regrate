# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-06-09

### Added
- `regrate resolve` is now implemented: after a git merge/rebase conflict in
  the store, it keeps upstream's migration (staged) and moves your version
  back to `regrate/current` for re-committing.
- `regrate valid` verifies that every store directory is reachable from the
  name chain and reports orphans (e.g. an edited committed migration).
- `regrate run --current` now actually runs the work-in-progress migration
  after the committed ones.
- `REGRATE_INDEX` is exported to run commands (zero-based chain position).
- `down` script templates for shell, postgres, and mysql.
- Integration test suite covering the full workflow, name determinism,
  tamper detection, and conflict resolution.

### Changed
- **Breaking:** migration names (v2) now hash files in sorted order and
  include relative file names and sizes. Previously the hash depended on
  filesystem iteration order, so the same migration could produce different
  names on different machines; renaming a file also went undetected.
- `regrate run` refuses to run when the store contains migrations that are
  unreachable from the name chain, instead of silently skipping them.
- Run commands receive `REGRATE_*` variables via the child environment only,
  rather than mutating regrate's own environment.
- Upgraded to clap 4 and current versions of all dependencies.
- CI workflows migrated off the archived actions-rs actions.

### Fixed
- `regrate create` copied template subdirectories to the wrong location.
- `regrate init --path` no longer changes the process working directory
  (which was left changed on some error paths).
- Template files are now truncated on overwrite and the unix-only
  permission handling is gated, unblocking Windows builds.
