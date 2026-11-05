+++
id = "route-move"
part = "d011-route"
kind = "feature"
needs = ["route-limits", "connection-core"]
files = ["skein/crates/skein-fake-forge-http/**", "skein/crates/skein-fake-forge-protocol/Cargo.toml", "skein/crates/skein-fake-forge-protocol/src/git.rs", "skein/crates/skein-fake-forge-protocol/src/git/**", "skein/crates/skein-fake-forge-protocol/src/lib.rs", "skein/crates/skein-fake-forge-protocol/src/tests/git.rs", "skein/crates/skein-fake-forge-github/Cargo.toml", "skein/crates/skein-fake-forge-github/README.md", "skein/crates/skein-fake-forge-github/src/**", "Cargo.lock"]
effort = "medium"
minutes = 45
+++

# The fake GitHub stops standing on the fake Forgejo

## Goal

The title of `skein/docs/development/debt/D011-fake-github-on-forgejo-fake.md`. `git`, `Hosted` and `Faults` move to
`skein-fake-forge-http`, and `skein-fake-forge-github` drops its dependency on the Forgejo fake. Forgejo's crate
re-exports them, so temper's paths don't move until `forgejo-rename`.

Acceptance:

- `rg -n 'fake.forge.protocol' skein/crates/skein-fake-forge-github` finds nothing.
- `skein/crates/skein-fake-forge-http/src/git.rs` and `git/` hold the route; `skein-fake-forge-protocol/src/git.rs` and
  `git/` are gone, and Forgejo's `lib.rs` has `pub use skein_fake_forge_http::{Faults, Hosted, git};`.
- `rg -n -i 'forgejo|github' skein/crates/skein-fake-forge-http/src` finds nothing but comments.
- The packages of step 4 pass, the same tests as at base.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/plan/debt-15/notes/contracts.md`, 1, 3 and 5
- `skein/docs/design/fake-forge.md`, 12

## Owns and may change

- **Owns:** the moved code and `skein-fake-forge-http`'s `lib.rs` and `Cargo.toml` (it gains skein-json, `skein-forge` and
  `skein-fake-forge-domain`).
- **May change:** Forgejo's `Cargo.toml` and `lib.rs`; `G/**`, `use` lines alone; GitHub's `Cargo.toml`;
  `skein-fake-forge-github/README.md`'s `Faults` sentence, which names the new path; `Cargo.lock`.
- **Options:**
  - Tests in `P/tests/git.rs` that need no Forgejo `Service` move with the code, with the same assertions, to
    `skein-fake-forge-http/src/tests/git.rs`. The rest stay. A test on the fence stays.
  - `pub(crate)` items of `git` that Forgejo's `server/git.rs` reads become `pub`.
- **Not to touch:** either `server.rs` beyond `use` lines. No temper file changes: if one must, it's a question.

## Steps

1. `git mv` the route, so history follows. Then `Hosted` and `Faults`.
2. The re-export; GitHub's `use` lines; GitHub's `Cargo.toml`.
3. `git::status`'s table test stays in Forgejo's crate, since it reads `errors::answer`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p skein-fake-forge-http -p
   skein-fake-forge-protocol -p skein-fake-forge-github --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-fake-forge-http -p skein-fake-forge-protocol -p skein-fake-forge-github -p skein-fake-peers-tests -p
   temper-fake-git`; `heavy cargo check --tests -p temper-fake -p temper-playground -p temper`; the citations check.
   Commit and push.

## Report

The common note's items. The moved tests' names and how many stayed. No break applies to a move.

## Answers

2026-11-05. May this task change the qualified lookup `skein_fake_forge_protocol::git::name` in GitHub's
`service.rs`, which is not a `use` line? Yes: replace that one path with `skein_fake_forge_http::git::name`, the same
function at its new home, and change nothing else on that line. The acceptance (`rg -n 'fake.forge.protocol'
skein/crates/skein-fake-forge-github` finds nothing) can't hold without it. "`use` lines alone" stands for the rest of
`G/**`. Then drop `skein-fake-forge-protocol` from GitHub's `Cargo.toml`, update `Cargo.lock`, and run all of step 4,
whose five-package run and `cargo check --tests` the first run left unrun.
