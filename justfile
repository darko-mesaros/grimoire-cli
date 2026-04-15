PATTERNS_DIR := env_var_or_default("PATTERNS_DIR", "")

build:
    cargo build

release:
    cargo build --release

install: release
    cp target/release/grimoire ~/.cargo/bin/grimoire

list:
    cargo run -- list

list-json:
    cargo run -- list --json

search *ARGS:
    cargo run -- search {{ARGS}}

get NAME:
    cargo run -- get "{{NAME}}"

browse:
    cargo run -- browse

setup-kiro:
    cargo run -- setup-kiro

setup-kiro-dry:
    cargo run -- setup-kiro --dry-run

clean:
    cargo clean
