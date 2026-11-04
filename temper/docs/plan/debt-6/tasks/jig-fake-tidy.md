+++
id = "jig-fake-tidy"
part = "jig-review"
kind = "feature"
needs = ["jig-delegate-story"]
files = ["jig/testing/jig-client-scripted-engine/src/requests.rs", "jig/testing/jig-client-scripted-engine/src/queries.rs"]
effort = "medium"
minutes = 20
+++

# jig: the scripted engine's standing and phase class say what they mean

## Goal

The review's Part B, findings 6 and 7. In `requests.rs`, the `waiting` match builds a `Recipient`, and
the test below it re-derives from the match's own arm that an escalation held by a task gives nobody
standing. It computes standing directly as an `Option<bool>`, mirroring the core's
`escalation_standing`. In `queries.rs`, `Phase::Ended.class()` is `PhaseClass::Ended` written the long
way, at its two uses. Behaviour does not change.

This task shares no code with `jig-delegate-story`; it follows it because the request puts findings 1
and 2 first.

Acceptance:

- `rg -n 'Phase::Ended\.class\(\)' jig/testing/jig-client-scripted-engine/src/queries.rs` reads nothing.
- `rg -n 'matches!\(waiting' jig/testing/jig-client-scripted-engine/src/requests.rs` reads nothing.
- `a_task_holder_gives_its_requester_standing_only_for_a_proposal` and the crate's other tests pass with
  no test changed.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/crates/jig-domain-core/src/admission/escalation.rs`'s `escalation_standing`.

## Owns and may change

- Owns: `src/requests.rs`'s `waiting`, and `src/queries.rs`'s `line`.
- `Recipient` stays: `recipient` and its other callers read it.

## Steps

1. The shape, with today's answers: a proposal held by a person or a task asks `recipient`, as an
   escalation held by a role does; an escalation held by a person is `person == party`; one held by a task
   is `false`; a proposal held by policy, and every other value, is `None`.

   ```rust
   let standing = match value { /* the arms above */ };
   if standing == Some(false) {
       return Ok((Answer::Refused { /* the Moved::Waiting refusal, unchanged */ }, Vec::new()));
   }
   ```

2. `queries.rs`: write `PhaseClass::Ended` at both.
3. `cargo fmt`; `heavy cargo clippy -p jig-client-scripted-engine --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-client-scripted-engine -p jig-client-world`; the citations check. One
   commit for each finding. Push.

## Report

Net lines. No test changed, so no hand-break.
