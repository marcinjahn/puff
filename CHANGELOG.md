# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- `puff add` and `puff forget` now print usage help instead of silently exiting when invoked
  without file paths

## [1.0.0] - 2026-04-12

### Added

- Support for managing entire directories as first-class units — `puff add config/` moves the
  directory to puff's data store and replaces it with a single directory symlink
- `--dir` flag on `add` command to create a fresh managed directory that doesn't exist yet
- Auto-detection of files vs directories when running `puff add` on existing paths
- Auto-absorb: `puff add config/` silently merges individually managed files already inside
  `config/` into the directory
- `puff forget` and `puff forget --delete` work on managed directories
- Guards that prevent adding/forgetting individual files inside a managed directory
- `puff init`, `puff link`, and `puff project forget` handle managed directories correctly
  (directory symlinks, backup of conflicting real directories, recursive restore)
- Support for managing files located in subdirectories of a project
- `--version` flag to print the version number with commit hash
- `status` command to print whether current directory is a part of puff-managed project and list
  managed files if it is
- `completions` command to generate shell completions (dynamic, with project name completion for
  `link` and `project forget`)
- support multiple files provided to `add` and `forget` commands
- a `cd` command that spawns subshell at the puff's projects directory - user can backup the files
- `link` command to create bring puff-managed files into other directories than the main project directory (useful for git
- modernize commands that require user input (such as `puff init`) with better UX
  worktrees, jj workspaces, or any secondary working copy)
- e2e tests

### Changed

- `puff status` now shows "Managed items:" (was "Managed files:") with directories displayed
  with a trailing `/`
- `puff link` now reports "item(s)" instead of "file(s)"
- Renamed `rm` command to `forget`
- Renamed `project rm` command to `project forget`
- Updated all dependencies; migrated to Rust 2024 edition
- Improved error messages
- Fixed `puff add` printing a false success message when a file exists in both the project
  directory and puff's registry (conflict now properly errors out)
- projects are now stored in `XDG_DATA_HOME/puff/projects` instead of `XDG_CONFIG_HOME/puff/configs`

### Removed

- Removed `symlink` crate dependency in favor of the standard library

## [0.1.7] - 2022-02-07

### Added

- CI status badge in README

## [0.1.6] - 2022-01-31

### Fixed

- Edge cases in the `add` command

## [0.1.5] - 2022-01-27

### Added

- `--git-ignore` flag on the `add` command to automatically add the file to `.gitignore`
- GitHub Releases for binary distribution

## [0.1.4] - 2022-01-27

### Added

- CI workflow via GitHub Actions

## [0.1.3] - 2022-01-27

Initial release with core functionality:

- `puff add` — start managing a gitignored file
- `puff init` — recreate symlinks in a project (e.g. after cloning on a new machine)
- `puff list` — list managed files and projects
- `puff rm` — stop managing a file
- `puff project rm` — stop managing a project

[unreleased]: https://github.com/marcinjahn/puff/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/marcinjahn/puff/compare/v0.1.7...v1.0.0
[0.1.7]: https://github.com/marcinjahn/puff/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/marcinjahn/puff/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/marcinjahn/puff/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/marcinjahn/puff/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/marcinjahn/puff/releases/tag/v0.1.3
