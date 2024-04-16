default:
  @just --list --unsorted --justfile '{{justfile()}}'

fmt:
  cargo fmt --all --check

clippy:
  cargo clippy -- \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -A clippy::significant_drop_tightening \
    -A clippy::no_effect_underscore_binding

build:
  cargo build

test:
  cargo nextest run --hide-progress-bar --success-output immediate --failure-output immediate