+++
id = "room-wait"
part = "full-outbox"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/projection.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 60
+++

## Goal

A goal's issue projection whose next write meets the forge outbox at its
bound waits for room instead of asking to be projected again at once: it
sets its due time at `wall + issue_policy.interval`, as `fail_projection`
does for a failed write but without failing, and emits `ProjectAfter {
goal, when: Some(..) }`. It also marks that a projection met the full
outbox, for the root's backstop (`requeue-backstop`) to read. This is the
fix for the deployment's spin (`notes/common.md`).

The idiom chosen is the timer, not a wake when an entry frees: every wait
of a projection in this crate is a `projection_due` timer (a failed
write's retry, `issues::Decision::Wait`), restore already re-arms it for a
row with nothing in flight, and no path in the forge wakes a waiter when
another entry leaves the outbox.

Acceptance:

- `project` (`temper-engine-domain-forge/src/domain.rs`) calls
  `wait_for_room` where an effect meets `d.entries.len() ==
  env.limits.entries`; an entry number already in the outbox keeps
  `ProjectAfter { goal, when: None }`. The row is not failed, holds no
  pending entry, and keeps its saved `desired`; nothing is saved, failed
  or settled for the wait.
- `Domain` has the volatile `full_outbox_met` mark, `full_outbox_met()`
  and `begin_step()`, exactly as `notes/contracts.md` writes them.
- One table test in `src/tests/projection.rs` (Steps, 2) holds the rule;
  its `full` row fails before the fix, on its assertion.
- The **Not yet** entry 12, *A full outbox defers it*, in
  `temper/docs/design/domain/forge.status.md` that names `project` and
  "`ProjectAfter` with no time" is removed, its lines alone (the other 12
  entry, naming `forge_route.rs`, is `requeue-backstop`'s).

## Inputs

- `temper/docs/design/domain/forge.md`, 12: the bullets *It fits its
  bounds, and never holds the goal* (a failed write's retry after the
  interval) and *A full outbox defers it*, the rule this task builds.
- `temper/docs/design/domain/forge.md`, 6: *The outbox is bounded*.
- `temper/docs/plan/projection-full-outbox/notes/contracts.md`: *The wait*
  and *The step's mark*, binding.
- `temper/docs/plan/projection-full-outbox/notes/common.md`.
- `temper/docs/plan/projection-full-outbox/inputs/request.md`.

## Owns and may change

- Owns the files in `files`; in `domain.rs`, `project`, the new
  `wait_for_room`, the `Domain` field, its constructor's literal and the
  two accessors beside `projection_after` and `projection_settled`.
- Pre-approved: if an existing test of this crate, or of
  `temper/tests/engine/forge` or `temper/tests/engine/domain`, asserted
  `ProjectAfter { when: None }` or an at-once re-projection where an
  outbox was full, change that assertion to the rule (the projection is
  due at `wall + interval`, and goes out once the wall reaches it with
  room) and name it in the report; its other steps stay.
- Pre-approved: if a story in `temper/tests/engine/domain` now waits an
  interval it did not wait before, because its outbox fills, advance its
  wall past the interval where it waits, and name it.
- Not yours: `temper-engine-domain` (the root, `requeue-backstop`'s),
  `fail_projection`'s and `projection_outcome`'s behaviour, the outbox's
  bound and every other figure, any record or schema.

## Steps

1. **The failing test first** (Step 2's `full` row): write it, run it, see
   it fail because the request is `ProjectAfter { goal: GOAL, when: None }`
   where `Some(wall + interval)` is wanted. Then the fix.

2. **The table test,** in `src/tests/projection.rs`:
   `a_projection_meeting_a_full_outbox_waits_for_room`, one loop over rows.
   Each row starts from `started(view, issues::Projected::default(), None)`
   with a live view (`issues::GoalView { finished: None, ..goal_view() }`),
   so the next write is the issue's open. It fills the outbox with
   `LIMITS.entries` status entries (`Event::Enqueue { entry: status(n, 8) }`
   then `Event::Committed { entry: n }`, as
   `a_held_effect_asked_again_is_looked_for_first_even_with_an_otherwise_full_outbox`
   in `src/tests.rs` does), less one for `taken`, and checks
   `d.effect_room(&LIMITS, 0)` is what the row says.

   | row | outbox | asked with | then |
   |---|---|---|---|
   | `full` | at its bound | `Event::ProjectDesired { entry: NEXT, goal: GOAL }` | the requests hold `ProjectAfter { goal: GOAL, when: Some(wall + interval) }`, and no `ProjectAfter { when: None }`, `ProjectionEffect`, `ProjectionFailed` or `ProjectionSettled`; the issue row is not failed and has no pending; `d.next_wall_deadline()` is `wall + interval`; the outbox is unchanged; `d.full_outbox_met()`, and false after `d.begin_step()` |
   | `taken` | one under its bound | `ProjectDesired` with the number of an entry already in the outbox | `ProjectAfter { goal: GOAL, when: None }`, no effect; `!d.full_outbox_met()` |
   | `freed` | at its bound; after the `full` request, one entry leaves | the timer, then `ProjectDesired { entry: NEXT + 1, .. }` | the forge's `fire` before `wall + interval` asks nothing for `GOAL`; with the wall at `wall + interval`, `fire` emits `ProjectAfter { goal: GOAL, when: None }`, and the `ProjectDesired` that follows emits a `ProjectionEffect` whose row is `GOAL`'s |

   `interval` is `LIMITS.issue_policy.interval`. The timers are the
   forge's `fire` (`domain.rs`), not `resume`: call it as `requests` calls
   `resume`, through a small helper beside `requests`. To free an entry in
   `freed`, withdraw one the client has not sent (`Event::Withdraw`), or
   answer its write as `write` and `answer` in this file do; either, and
   assert `d.effect_room(&LIMITS, 0)` after. Advance the wall by setting
   `environment.wall` (and `now` with it, as `restart_inflight` does).

   Bites: put back `ProjectAfter { goal, when: None }` for the full
   outbox and see `full` fail; drop `d.full_outbox_met = true` and see
   `full` fail on the mark. Restore each and name both in the report.

3. **The fix,** `wait_for_room` and the mark, as `notes/contracts.md`
   writes them. In `project`, split today's
   `d.entries.len() == env.limits.entries || d.entries.contains_key(&number)`
   so the first calls `wait_for_room(d, env, goal, out)` and returns, and
   the second keeps `ProjectAfter { goal, when: None }`. The doc comments
   cite `temper/docs/design/domain/forge.md`, 12, at the function and the
   field, not on each line.

4. **Narrow runs, then the readers.** This changes what `project` asks of
   the root on a full outbox, so it runs the tests that read it:
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/projection/)'`
   - `heavy cargo nextest run -p temper-engine-domain-forge`
   - `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/projection/)'`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^issue_projection::/)'`
   - `heavy cargo nextest run -p temper-engine-forge-world`

   Check each run's count; one that selected nothing proves nothing.

5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain-forge --all-targets -- -D warnings`;
   remove the status entry; commit saying why (a projection meeting the
   full outbox waits an interval, so the root's step ends); push.

## Report

As the implementer's text has it, and: how `freed` frees its entry; the
rule broken for each bite and the failure seen; each existing test whose
assertion you changed, and how; the counts of the runs in step 4.
