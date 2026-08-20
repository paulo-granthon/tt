default:
    @just --list

build:
    cargo build --release

test:
    cargo test

lint:
    cargo clippy --all-targets -- -D warnings

install: build
    cargo install --path . --force
    ./install-completions.sh

completions:
    ./install-completions.sh
