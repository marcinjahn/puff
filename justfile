default:
    just --list

# Build debug binary
build:
    cargo build

# Build release binary
build-release:
    cargo build --release

# Run cargo check
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy

# Format code
format:
    cargo fmt

# Check code formatting
format-check:
    cargo fmt --check

# Run puff with the given arguments
run *args:
    cargo run -- {{args}}

# Install puff locally via cargo
install:
    cargo install --path .

# Run unit tests
test-unit:
    cargo test

# Run e2e tests (builds release first)
test-e2e: build-release
    PATH="$(pwd)/target/release:$PATH" bats tests/e2e/ --print-output-on-failure

# Run script tests
test-scripts:
    bats tests/scripts/ --print-output-on-failure

# Run app tests (unit + e2e, no scripts)
test-app: test-unit test-e2e

# Run all tests
test: test-app test-scripts

# Trigger a new release (major, minor, or patch)
release bump:
    gh workflow run release.yml -f bump={{bump}}
