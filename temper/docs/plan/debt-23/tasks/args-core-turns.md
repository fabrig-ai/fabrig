+++
id = "args-core-turns"
part = "payloads"
kind = "feature"
needs = ["args-core-party"]
files = ["jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/routing/run/payloads.rs", "jig/crates/jig-domain-core/src/routing/run/turns.rs", "jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/routing/run/jobs.rs", "jig/crates/jig-domain-core/src/routing/view/routes.rs"]
effort = "medium"
minutes = 50
+++

# jig-domain-core: the turn, answer and view routes take their arguments folded

## Goal

Part of jig D016: seven sites of `jig-domain-core`'s `routing/` lose their `too_many_arguments` exception, by `notes/arguments.md`'s folds.

The sites: `routing/run/payloads.rs`'s `route_answer_payload` and `route_turn_payload`; `routing/run/turns.rs`'s `route_accepted_turn`; `routing/run/preparation.rs`'s `workspace_prepared`; `routing/run/jobs.rs`'s `prepared`; `routing/view/routes.rs`'s `run_text` and `run_input`.

Acceptance:

- `rg -n 'too_many_arguments' jig/crates/jig-domain-core/src/routing/run/payloads.rs jig/crates/jig-domain-core/src/routing/run/turns.rs jig/crates/jig-domain-core/src/routing/run/preparation.rs jig/crates/jig-domain-core/src/routing/run/jobs.rs jig/crates/jig-domain-core/src/routing/view/routes.rs` reads nothing.
- `route_accepted_turn`'s `TurnRecord` is built before the call and dropped unread on `Accepted::Already`: that builds nothing costly today, as `transcript` already moves in. If it would clone anything it doesn't now, stop and take another fold.
- `heavy cargo nextest run -p jig-domain-core --lib` and `-p jig-core-world` pass, with no assertion changed.
- About +25 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header; `routing/mod.rs` in this part's call sites alone, lines near 530 to 585 at the base.
- **Other drafts:** `ui-chat-draft`'s `note-scope` changes `routing/mod.rs` and `routing/run/preparation.rs`: fab merges its trunk in as it lands. Find functions by name; if a signature here has grown, fold what's there.
- **`preparation.rs`'s `too_many_lines`** exception is adjacent: see the note.
- **Stays off:** `routing/run/calls.rs`, `routing/run/delegation.rs`, `effects.rs`: `args-core-calls` holds them.

## Steps

1. `TurnProof` and `TurnRecord`; then `Answer`, `Prepared`, `JobItems`, `Written`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
