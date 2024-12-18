#!/usr/bin/env -S just --justfile

set dotenv-filename := '.env-just'

smee_gh_app := env('SMEE_GH_APP', 'https://smee.io/WPgvb1aTMNPsas')
smee_runner := env('SMEE_RUNNER', 'https://smee.io/hyZtaQlRMpJ1pEt')

_default:
  @just --list --unsorted --justfile '{{justfile()}}'

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all --check

clippy:
  cargo clippy -- \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -A clippy::significant_drop_tightening \
    -A clippy::no_effect_underscore_binding \
    -A clippy::missing_errors_doc

build *FLAGS='':
  cargo build {{FLAGS}}

test:
  cargo nextest run --lib --bins --features tests-with-resources --hide-progress-bar --success-output immediate --failure-output immediate

integration-test:
  @export TIMEFORMAT='%3lR' && time just _integration-test-raw

_integration-test-raw:
  #!/usr/bin/env bash
  docker run --name it-postgres -p 5433:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=mysecretpassword -e POSTGRES_DB=postgres -d postgres:14-alpine
  trap 'docker rm $(docker stop it-postgres)' EXIT
  just integration-test-with-available-pg

integration-test-with-available-pg:
  cargo nextest run -v --test-threads 1 --test '*' --features tests-with-docker --hide-progress-bar --success-output immediate --failure-output immediate

run:
   cargo shuttle run

rund:
   export RUST_LOG="debug" && export RUST_BACKTRACE=1 && just run

shuttle-restart:
  cargo shuttle project restart --idle-minutes 0

install-smee:
  npm install -g smee-client

start-smee-gh:
  smee -u {{smee_gh_app}} -t http://127.0.0.1:8000/webhook/github -p 3000

start-smee-runner:
  smee -u {{smee_runner}} -t http://127.0.0.1:8000/webhook/github/runner -p 3001

stop-smee-runner:
  docker rm $(docker stop smee-client-runner)

start-pg:
  docker run --name it-postgres -p 5433:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=mysecretpassword -e POSTGRES_DB=postgres postgres:14-alpine

pg-admin:
 docker run -d --name pg-admin -p 5050:5050 \
   -v test_files/servers.json:/pgadmin4/servers.json \
   -e 'PGADMIN_LISTEN_PORT=5050' \
   -e 'PGADMIN_DEFAULT_EMAIL=toto@t.com' \
   -e 'PGADMIN_DEFAULT_PASSWORD=toto' \
   dpage/pgadmin4:latest
