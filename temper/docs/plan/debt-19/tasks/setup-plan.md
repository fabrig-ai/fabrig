+++
id = "setup-plan"
part = "setup-vocabulary"
kind = "migration"
needs = []
files = ["skein/crates/skein-forge-setup/src/plan.rs", "skein/crates/skein-forge-setup/src/lib.rs", "skein/crates/skein-forge-setup/src/reconcile.rs", "skein/crates/skein-forge-setup/src/reconcile/**", "skein/crates/skein-forge-setup/src/github/**", "skein/crates/skein-forge-setup/src/reachable.rs", "skein/crates/skein-forge-setup/src/create.rs", "skein/crates/skein-forge-setup/src/adopt.rs", "skein/crates/skein-forge-setup/src/take_over.rs", "skein/crates/skein-forge-setup/src/check.rs", "skein/crates/skein-forge-setup/src/access.rs"]
effort = "high"
minutes = 40
+++

# skein: forge-setup's shared words have a module of their own, and `stop` is written once

## Goal

The first of four steps of `skein/docs/development/debt/D014-forge-setup-vocabulary.md`. `plan.rs` takes `Need`, `Outcome`, `Event`, `Reply`, `Failure`, `Stage` and `Why` from Forgejo's reconcile, and the plans' `Need::Stop(Failure { .. })` is built
in one function. GitHub's plans stop depending on Forgejo's module for them.

Acceptance:

- `rg -n 'enum (Need|Event|Reply|Stage|Why|Outcome)\b|struct Failure\b' skein/crates/skein-forge-setup/src -g '!host/**' -g '!tests/**'` lists only `plan.rs`. `host::{Need, Event}` are other types and don't change.
- `rg -n 'Need::Stop\(Failure' skein/crates/skein-forge-setup/src -g '!tests/**' -g '!*tests.rs'` lists one line (6 at base, and `github/app.rs`'s).
- `rg -n 'reconcile::' skein/crates/skein-forge-setup/src/github -g '!*tests*'` names only `hook_url`, if it must.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` passes with no test changed: their spelling `reconcile::Why` holds by re-export. Every stop has the same stage and subject as at base; if one must change, stop and ask.
- Net production falls (estimate -10).

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/development/debt/D014-forge-setup-vocabulary.md`
- `skein/docs/design/forge-setup.md`, 4.2; `skein/docs/design/forge.md`, 4.4 (the neutral pieces' precedent)

## Owns and may change

- **Owns:** the files in `files`. `pub mod plan;` in `lib.rs`.
- **Public paths stay:** `reconcile.rs` keeps `pub use crate::plan::{Event, Failure, Need, Outcome, Reply, Stage, Why};`. temper spells `reconcile::Why` 75 times, `Stage` 28, `Outcome` 25 and `Failure` 14, and `temper-service/src/lib.rs` publishes `SetupStage` and `SetupWhy`: none of them changes.
- **`Reply` moves as it is,** both dialects' bodies in it: `setup-replies` splits it. The helpers `reply` and `account` stay where they are.
- **`stop`:** `plan::stop(forge: u16, step: Stage, subject: &[u8], why: Why) -> Need`, which is `github/app.rs`'s `app::stop` lifted. A plan's own method stays, and is thin: each clears different fields (`admin`, `shared.secret`, `taken`, `token`, `user`), which can't be shared without a closure or a trait. Same for `create.rs`'s four-way stage and `adopt.rs`'s two-way subject.
- **The host's `Failure { .. }` literals** (`host.rs`, `run.rs`, `states.rs`) aren't `Need`s and stay.

## Steps

1. `plan.rs`, by moves; the re-exports; `github/` and the others read `crate::plan`.
2. `plan::stop`, and every plan's `stop` calls it; `app::stop` goes.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p temper-service -p temper-fake -p temper-init -p temper-forge-setup -p temper-github-app -p temper-protocol-records -p temper-codecs-world -p temper-engine-domain-forge`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, each `stop`'s shape after, and the net lines.
