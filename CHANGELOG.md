# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for managing files located in subdirectories of a project
- `--version` flag to print the version number with commit hash
- `status` command to print whether current directory is a part of puff-managed project and list
  managed files if it is
- `completions` command to generate shell completions
- support multiple files provided to `add` and `forget` commands
- e2e tests

### Changed

- Renamed `rm` command to `forget`
- Renamed `project rm` command to `project forget`
- Updated all dependencies; migrated to Rust 2024 edition
- Improved error messages

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

[unreleased]: https://github.com/marcinjahn/puff/compare/v0.1.7...HEAD
[0.1.7]: https://github.com/marcinjahn/puff/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/marcinjahn/puff/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/marcinjahn/puff/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/marcinjahn/puff/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/marcinjahn/puff/releases/tag/v0.1.3
