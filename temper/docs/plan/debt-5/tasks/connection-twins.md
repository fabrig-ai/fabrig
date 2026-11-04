+++
id = "connection-twins"
part = "skein-forge-root"
kind = "feature"
needs = ["forge-plan-failure"]
files = [
  "skein/crates/skein-forge-connection/src/connection.rs",
  "skein/crates/skein-forge-connection/src/component.rs",
  "skein/crates/skein-forge-connection/src/component/github_calls.rs",
  "skein/crates/skein-forge-connection/src/limits.rs",
  "skein/crates/skein-forge-connection/src/connection/tests.rs",
  "skein/crates/skein-forge-connection/src/tests/github_files/fake.rs",
]
effort = "medium"
minutes = 20
+++

# skein D010: forge-connection stops translating the twins

## Goal

`skein-forge-connection` uses the root's `Failure` and `Limits`, so `connection::Failure` and `github_documents` go. About -24 lines. D010 stays open.

Acceptance:

- `rg -n 'enum Failure|fn github_documents' skein/crates/skein-forge-connection/src` reads nothing.
- `rg -n 'forgejo::(time|decode::Malformed)' skein/crates/skein-forge-connection/src` reads nothing.
- `heavy cargo nextest run -p skein-forge-connection` passes, its count above zero.
- `heavy cargo check --tests -p skein-forge-machine-world -p skein-forge-setup -p skein-fake-forge-github -p temper-service` passes; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/forge.md`, 4.4; `skein/docs/design/forge-connection.md`, for what names `connection::Failure`, if any.

## Owns and may change

- Owns: the files in the header, in `skein-forge-connection`.
- `connection::Failure` is `pub(crate)` inside a private module, so nothing outside reads it.
- These stay; they are not twins: `Body::closes` (`connection.rs`, near 77-90, D013's), `Retention` to `Range` (near 751-756), each dialect's own `Bounds` construction, and `Unusable::{Documents, GitHubDocuments}`, which temper matches.
- Pre-approved: `pub(crate) use skein_forge::plan::Failure;` in `connection.rs`, if `component.rs` keeps its path.
- `skein-forge-setup/src/host/limits.rs` has a like copy, and stays (`temper/docs/plan/debt-5/notes/forge-root.md`).

## Steps

1. `connection.rs`: `connection::Failure` (near 54-59) goes, for `skein_forge::plan::Failure`. The two mappings near 919-926 become `Failed(failure)` in each arm. `component.rs:531` and `connection/tests.rs:75` follow.
2. `limits.rs`: `github_documents` (near 574-586) copies six fields of a type that is now one. It goes, and its callers call `documents(limits)`: `connection.rs` 261 and 648, `component/github_calls.rs:341`, `limits.rs` 304 and 589, `tests/github_files/fake.rs:351`.
3. `connection.rs:4` and `:695` name `skein_forge::decode` and `skein_forge::time`.
4. Run what the acceptance names; commit (`D010: forge-connection uses the root's Failure and Limits`, saying why) and push.

## Report

The lines added and removed, and any other field-by-field translation of a twin you found.
