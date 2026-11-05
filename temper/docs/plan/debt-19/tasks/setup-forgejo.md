+++
id = "setup-forgejo"
part = "setup-vocabulary"
kind = "migration"
needs = ["setup-plan"]
files = ["skein/crates/skein-forge-setup/src/forgejo.rs", "skein/crates/skein-forge-setup/src/forgejo/**", "skein/crates/skein-forge-setup/src/lib.rs", "skein/crates/skein-forge-setup/src/reconcile.rs", "skein/crates/skein-forge-setup/src/reconcile/**", "skein/crates/skein-forge-setup/src/access.rs", "skein/crates/skein-forge-setup/src/adopt.rs", "skein/crates/skein-forge-setup/src/check.rs", "skein/crates/skein-forge-setup/src/copy.rs", "skein/crates/skein-forge-setup/src/create.rs", "skein/crates/skein-forge-setup/src/given.rs", "skein/crates/skein-forge-setup/src/limits.rs", "skein/crates/skein-forge-setup/src/reachable.rs", "skein/crates/skein-forge-setup/src/take_over.rs", "skein/docs/development/debt/D037-*.md"]
effort = "medium"
minutes = 25
+++

# skein: Forgejo's plans sit in `forgejo/`, beside `github/`

## Goal

The second of four steps of `skein/docs/development/debt/D014-forge-setup-vocabulary.md`, a move and nothing else: the crate stops being asymmetric on disk.

Acceptance:

- `skein/crates/skein-forge-setup/src/` holds at its root only `call.rs`, `item.rs`, `plan.rs`, `lib.rs`, `forgejo.rs` and `github.rs`, and the directories `forgejo/`, `github/`, `host/` and `tests/`, with `host.rs` and `tests.rs`.
- `git diff -M --stat` against your base shows the moved files at 90% similarity or more, and no test file changed.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` passes, with the same tests under the same names, and `heavy cargo check --tests` on the readers of step 3.
- Net production rises by at most 15 lines (estimate +12).

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/development/debt/D014-forge-setup-vocabulary.md`

## Owns and may change

- **Owns:** the files in `files`.
- **Public paths stay,** as `skein/docs/design/forge.md`, 4.4 has a dialect's: `mod forgejo;` is private, and `lib.rs` has `pub use forgejo::{adopt, check, copy, create, given, limits, reachable, reconcile, take_over};`. `access` stays private, as `forgejo/access.rs`. A module's name doesn't change, so `crate::reconcile::..` in tests holds.
- **`reachable` moves still speaking both dialects:** `setup-replies` splits it.
- **`skein/docs/development/debt/D037-*.md`:** its citation of `skein-forge-setup/src/limits.rs` follows the move. That's all you change in it; D037 isn't paid here.
- **`#[path]`s in `host/` and `tests/` stay,** since neither moves.

## Steps

1. `git mv`, one commit, compiling with `lib.rs`'s lines.
2. Fix anything left by name.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p temper-service -p temper-fake -p temper-init -p temper-forge-setup -p temper-github-app -p temper-protocol-records -p temper-codecs-world -p temper-engine-domain-forge`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items and the net lines.
