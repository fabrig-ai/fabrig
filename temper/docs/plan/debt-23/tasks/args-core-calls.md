+++
id = "args-core-calls"
part = "payloads"
kind = "feature"
needs = ["args-core-turns"]
files = ["jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/routing/run.rs", "jig/crates/jig-domain-core/src/routing/run/calls.rs", "jig/crates/jig-domain-core/src/routing/run/delegation.rs", "jig/crates/jig-domain-core/src/routing/effect/procedure.rs", "jig/crates/jig-domain-core/src/routing/party/amendment_reads.rs", "jig/crates/jig-domain-core/src/effects.rs", "jig/crates/jig-domain-core/src/tests/acts/proposals.rs"]
effort = "high"
minutes = 70
+++

# jig-domain-core: the named calls, delegation and effects take their arguments folded

## Goal

Part of jig D016: ten sites of `jig-domain-core` lose their `too_many_arguments` exception, by `notes/arguments.md`'s folds. This is core's last, and the one that meets other drafts: it runs after their tasks on these files as far as this plan can place it.

The sites: `routing/run/calls.rs`'s `named_proposal_decide`, `named_propose`, `named_escalation`, `named_amend`; `routing/run/delegation.rs`'s `propose_batch`, `procedure_step`, `procedure_holdings`; `effects.rs`'s `propose_start`, `complete_proposal`, `procedure_propose`.

Acceptance:

- `rg -n 'too_many_arguments' jig/crates/jig-domain-core/src/routing/run/calls.rs jig/crates/jig-domain-core/src/routing/run/delegation.rs jig/crates/jig-domain-core/src/effects.rs` reads nothing.
- `Call` and `Proposing` live in `routing/run.rs`, which no other draft holds. `Proposing` carries `to`, `key`, `reason` and `as_holder` from `named_propose` through `propose_batch` and `effects.rs`'s `complete_proposal` without a new clone: the arguments move today, and the fold moves as one.
- `heavy cargo nextest run -p jig-domain-core --lib`, `-p jig-core-world` and `-p jig-ops-world` pass, with no assertion changed. The last because `procedure_propose` serves ops' procedures.
- About +35 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header; `routing/mod.rs` in this part's call sites alone, lines near 640 at the base; `routing/party/amendment_reads.rs` in `named_amend`'s one call; `tests/acts/proposals.rs` in `procedure_propose`'s one call.
- **Other drafts:** `note-parts`' `note-range` changes `routing/run/calls.rs`, and `ui-chat-draft`'s `note-answer` changes `effects.rs`. Their lines arrive with fab's merge at your start: find functions by name, and fold what's there.
- **If a site has grown past its fold,** by a field those drafts added, fold that too.

## Steps

1. `Call`, `AgentAmend`; then `ProcedureCall`; then `Proposing` and `Described`, across the three files at once.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
