default:
    just --list

build:
    cargo build

build-release:
    cargo build --release

check:
    cargo check

lint:
    cargo clippy

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

run *args:
    cargo run -- {{args}}

install:
    cargo install --path .

test-unit:
    cargo test

test-e2e: build-release
    PATH="$(pwd)/target/release:$PATH" bats tests/e2e/ --print-output-on-failure

test: test-unit test-e2e
