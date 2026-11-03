+++
id = "relay-stall"
part = "relay"
kind = "feature"
needs = ["relay-foreman"]
files = ["temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge-change/src/domain.rs", "temper/crates/temper-engine-domain-forge-change/src/tests.rs", "temper/crates/temper-engine-domain-forge-change/src/tests/**", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/**", "temper/tests/engine/domain/tests/suite/forge_routes/questions.rs", "temper/docs/design/domain/forge.status.md"]
effort = "medium"
minutes = 45
+++

# relay-stall: a change waiting on an agent's open question does not stall

## Goal

A follow-up to tasks `relay-up` and `relay-foreman` (`domain/forge.md`, 8.4 "Stalls" and
8.11 "Its agents may ask"): while a question of the change's current agent is open (with
its foreman, asked of its requester, or held for dependencies), the change's wait for that
agent is no stall, so a question its person takes hours to settle never holds the change
`Stalled`. The stall counts again from the step that relays the last answer down.

Acceptance:

- The pure crate's `Facts` gains `asking: bool`, documented "An open question of the
  change's current agent stands (domain/forge.md, 8.4 and 8.11)."; every literal of `Facts`
  gains `asking: false`.
- Where a phase waits on its producing, repair or resolution delegate (`producing`,
  `delegated` and the `waiting` they reach), `asking` true waits with no deadline (`Decision::Wait { until: None }`)
  and sets the change's `since` to `now`, so the stall counts from the last step that saw a
  question open; every other wait is as today.
- The step's facts set `asking` from the row: `row.questions.open` holds a question whose
  asker is the task of `row.delegate`. The relay is applied before the pure step reads
  `asking` in the same step, so the step that relays the last answer down sees it false
  with `since` at that step.
- Status: remove `forge.status.md`'s entry "8.4, 8.11: a change waiting on an agent whose
  question is open stalls ...".

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4 ("Stalls") and 8.11 ("Its agents may ask").
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C2 and C7.

## Owns and may change

Owns the files in the header, each hunk to `asking`.

- `Facts` literals across the tree (the connector's and root's fixtures, the forge world),
  each gaining `asking: false` only; `rg 'Facts \{' temper` lists them; name each.
- If the relay is applied after the pure step today, move only the order of those two calls
  and say so; if that order cannot move, compute `asking` from the relayed `questions`
  instead and say so.

## Steps

1. The field, the waits, the facts, the order; the status line.
2. Tests:
   - in the pure crate: `an_agent_with_an_open_question_never_stalls` (producing with its
     delegate pending, `now` far past `since + stall`, `asking` true: a wait with no
     deadline, `since` now, not held); `the_stall_counts_from_the_last_question` (the next
     step with `asking` false and `now` past the new `since + stall` holds `Stalled`, and
     one before it waits until then).
   - in `temper/tests/engine/domain/tests/suite/forge_routes/questions.rs`:
     `a_question_open_past_the_stall_holds_nothing` (the world's clock moved past the
     stall while the question is up; no hold; after the answer the stall counts from it).
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge-change`, then `heavy cargo
   nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::questions::/) | test(/forge_routes::foremen::/)'` (check the
   counts). This changes when a change stalls, so its readers' tests run: `heavy cargo
   nextest run -p temper-engine-domain-forge` and `-p temper-engine-forge-world`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-engine-domain
   -p temper-engine-domain-world --all-targets -- -D warnings`. Commit ("A change waiting on
   an open question does not stall") and push.

## Report

As the implementer's text says.
