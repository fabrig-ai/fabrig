+++
id = "core-small"
part = "core-small"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/timers.rs", "jig/crates/jig-domain-core/src/tests/routing/deadlines.rs", "jig/crates/jig-domain-core/src/tests/public/reads/fixtures.rs", "jig/crates/jig-domain-core/src/tests/public/reads/races.rs", "jig/testing/jig-test-domain/src/tests/public.rs", "jig/crates/jig-domain-tasks/src/boundary.rs", "jig/crates/jig-domain-tasks/src/tests/control.rs"]
effort = "medium"
minutes = 20
+++

# jig: two `Core` headers with no production caller, and a test module mid-file

## Goal

Finding 10's jig half. `Core::cold_header` and `Core::commit_header` have no production caller outside `Core`:
`decision_header` is what both roots call, and `Counters::commit_header` is what temper's decision calls and stays.
`jig-domain-tasks/src/boundary.rs` has a test module in the middle of a 2,275-line file.

Acceptance:

- `rg -n 'cold_header' jig temper --glob '*.rs'` finds nothing.
- `rg -n 'fn commit_header' jig/crates/jig-domain-core/src` finds one function, on `Counters`.
- `rg -n 'cfg\(test\)' jig/crates/jig-domain-tasks/src/boundary.rs` finds nothing.
- `temper/crates/temper-engine-domain/src/decision.rs` is untouched.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/design/domain/engine.md`, 5.4, for the deployment row.

## Owns and may change

- **Owns:** `jig/crates/jig-domain-core/src/routing/timers.rs` (`impl Core`'s two methods),
  `jig/crates/jig-domain-tasks/src/boundary.rs` (the `tests` module and nothing else).
- **May change, call spelling only:** `jig/crates/jig-domain-core/src/tests/public/reads/fixtures.rs` (six uses),
  `.../tests/public/reads/races.rs` (one use), `jig/testing/jig-test-domain/src/tests/public.rs` (one use): each reads
  `core.counters.commit_header()`. `Core::counters` is a public field.
- **May change:** `jig/crates/jig-domain-core/src/tests/routing/deadlines.rs`:
  `cold_header_is_taken_once_and_never_before_the_load` keeps every step and every assertion's meaning, read through
  `core.decision_header(false)`, which returns the row itself rather than a `Write::Save` of it. Rename it
  `a_dirty_header_is_taken_once_and_never_before_the_load`.
- **May change:** `jig/crates/jig-domain-tasks/src/tests/control.rs`, or another file of that directory holding `Party`
  tests: `party_sender_labels` moves there, unchanged.
- If `Write` or another import of `timers.rs` falls out of use, drop it. If a `Core` method other than these two turns
  out to have no caller, leave it and say so in the report.

## Steps

1. `timers.rs`: delete both `impl Core` methods. `Counters::commit_header` stays.
2. The test spellings above.
3. Move `party_sender_labels` beside the other `Party` tests; `boundary.rs` loses its `#[cfg(test)]` module.
4. `heavy cargo nextest run -p jig-domain-core --lib -E 'test(deadlines) or test(reads)'`, then `heavy cargo nextest run
   -p jig-domain-tasks --lib -E 'test(party_sender_labels)'` and `heavy cargo nextest run -p jig-test-domain --lib -E
   'test(a_read_waits_for_newer_commits_before_its_completing_step)'`.
5. `heavy cargo check -p temper-engine-domain --tests`: temper reads `Counters`, not `Core`, and this is the proof.
6. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-domain-tasks -p jig-test-domain --all-targets -- -D
   warnings`; the citations check. Commit and push.

## Report

The common note's items. No new rule, so nothing to break by hand; say the rename's old and new names. `party_sender_labels`
counts as a test moved out of a production file, not a production saving.
