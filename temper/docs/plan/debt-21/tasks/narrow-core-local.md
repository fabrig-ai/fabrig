+++
id = "narrow-core-local"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/party/holdings.rs", "jig/crates/jig-domain-core/src/routing/party/ended.rs", "jig/crates/jig-domain-core/src/routing/task.rs", "jig/crates/jig-domain-core/src/routing/flight.rs", "jig/crates/jig-domain-core/src/routing/effect/procedure.rs", "jig/crates/jig-domain-core/src/documents/objects.rs"]
effort = "medium"
minutes = 45
+++

# jig-domain-core: ten sites match once

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. Ten sites re-match, within a function or its helper, what the function has already matched. Each takes a local fix.

Acceptance:

- `rg -c 'unreachable!'` reads, under `jig/crates/jig-domain-core/src`: `routing/party/holdings.rs` 1, `routing/task.rs` 1, `routing/effect/procedure.rs` 0, `documents/objects.rs` 0, `routing/party/ended.rs` 0.
- `heavy cargo nextest run -p jig-domain-core --lib` and `heavy cargo nextest run -p jig-core-world --test suite` pass with no assertion changed. `heavy cargo check -p jig-ops-domain -p temper-engine-domain --tests` passes.
- The branch's net production lines don't rise by more than the cut's estimate of 15.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`

## The sites

- `routing/party/holdings.rs:243, 253`: one match.
- `routing/task.rs:55, 64, 143, 154, 167`: borrow the fields, then `flight::request_token(&mut event)`, or a new `take_reply` beside the private `flight::reply_to` (`routing/flight.rs`).
- `routing/effect/procedure.rs:82`: `apply` takes the fields.
- `documents/objects.rs:607`: bind `after` per arm.
- `routing/party/ended.rs:125`: an `Option` kind replaces the guard.

## Owns and may change

- **Owns:** the files in `files`. `routing/task/save.rs` stays out: its `Stored` lists are five different sets.
- **Stay off** every other file in the crate: other tasks of this plan hold them.

## Steps

1. The sites, file by file.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the counts; any site that stayed, and why.
