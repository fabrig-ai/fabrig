+++
id = "args-core-party"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/routing/party/**", "jig/crates/jig-domain-core/src/routing/application_routes.rs", "jig/crates/jig-domain-core/src/routing/effect/topics.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/settings/restart.rs", "jig/crates/jig-domain-core/src/settings/replaced.rs", "jig/crates/jig-domain-core/src/tests/settings/replaced.rs"]
effort = "medium"
minutes = 60
+++

# jig-domain-core: the party and settings routes take their arguments folded

## Goal

Part of jig D016: ten sites of `jig-domain-core`'s `routing/` lose their `too_many_arguments` exception, by `notes/arguments.md`'s folds. Their callers are `routing/mod.rs`'s dispatch, and `settings/application.rs` for `decide_judged`.

The sites: `routing/settings/dispatch.rs`'s `start`, `decide_judged`, `decide_with`, `propose_start`; `routing/party/key_routes.rs`'s `start`; `routing/party/deployment_acts.rs`'s `start`; `routing/party/projects.rs`'s `make_project_act`; `routing/application_routes.rs`'s `start`; `routing/effect/topics.rs`'s `subscribe`; `settings/restart.rs`'s `revision_writes_with_hold`.

Acceptance:

- `rg -n 'too_many_arguments' jig/crates/jig-domain-core/src/routing/settings jig/crates/jig-domain-core/src/routing/party jig/crates/jig-domain-core/src/routing/application_routes.rs jig/crates/jig-domain-core/src/routing/effect/topics.rs jig/crates/jig-domain-core/src/settings/restart.rs` reads nothing.
- `heavy cargo nextest run -p jig-domain-core --lib` and `-p jig-core-world` pass, with no assertion changed.
- About +30 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header; `routing/mod.rs` in this part's call sites alone, lines near 454 to 490 at the base.
- **`settings/application.rs`:** `settings-name` changes this file in this plan. Its `Name` may be on the trunk when you start; change only the `decide_judged` call.
- **Stays off:** `routing/run/**`, `routing/view/**`, `effects.rs`, `settings/check.rs`, `acts/`: other `args-*` tasks hold them.

## Steps

1. `Signed` and its four sites, then `Judged`, then the existing types, then the two single sites.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
