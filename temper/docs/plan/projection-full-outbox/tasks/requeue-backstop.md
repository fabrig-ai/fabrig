+++
id = "requeue-backstop"
part = "full-outbox"
kind = "feature"
needs = ["room-wait"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/tests/projection_room.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 60
+++

## Goal

The root's backstop: once a step has met the full forge outbox, a
projection asked for at once in that step (`ProjectAfter { goal, when:
None }`) is not projected again while the outbox stays full; its timer is
armed at `wall + interval` instead, so no path can spin the step. The
root clears the forge's mark as each step begins. And a root story shows
the deployment's case ending: a projection asked for with the outbox full
ends the step, due later.

Acceptance:

- `domain.forge.begin_step()` is called beside `domain.changed()` at the
  top of `step`, `fire` and `release` (`temper-engine-domain/src/domain.rs`).
- The `forge::Request::ProjectAfter` arm in `route/forge_route.rs` keeps
  its `when: Some` arm and its `pending` check, and for `when: None`
  pushes no work and calls `domain.forge.projection_after(goal, env.wall +
  interval)` when `domain.forge.full_outbox_met() &&
  !domain.forge.effect_room(&env.limits.forge, 0)`
  (`notes/contracts.md`, *The backstop*).
- `src/tests/projection_room.rs` holds a story and a table test (Steps,
  1 and 2); the table's `met` row fails before the guard, on its
  assertion.
- The **Not yet** entry 12, *A full outbox defers it*, in
  `temper/docs/design/domain/forge.status.md` that names
  `forge_route.rs` is removed, its lines alone.

## Inputs

- `temper/docs/design/domain/forge.md`, 12: *A full outbox defers it*,
  its last sentence above all; 6: *The outbox is bounded*.
- `temper/docs/design/domain/root.md`, 5.5: one step, one decision.
- `temper/docs/plan/projection-full-outbox/notes/contracts.md`: *The step's
  mark* and *The backstop*, binding.
- `temper/docs/plan/projection-full-outbox/notes/common.md`.
- `temper/crates/temper-engine-domain/src/tests/projection_settled.rs`:
  how a root test routes a forge request (`forge_route::outputs`) and
  reads `domain.work`; `src/tests/kinds.rs`, around
  `domain.forge.restore_begin`: how a root test restores forge records.

## Owns and may change

- Owns the files in `files`: in `domain.rs`, the three lines beside
  `domain.changed()` in `step`, `fire` and `release`; in `forge_route.rs`,
  the `ProjectAfter` arm; in `src/tests.rs`, the `mod projection_room;`
  line.
- Pre-approved: if another public entry of the root starts a step and
  routes forge work outside these three, add the same line there and name
  it in the report.
- Pre-approved: if an existing test of `temper-engine-domain` or
  `temper/tests/engine/domain` now sees a projection wait an interval in
  a step that met the full outbox, advance its wall past the interval
  where it waits, and name it.
- Not yours: `temper-engine-domain-forge` (`room-wait`'s; its accessors
  are as `notes/contracts.md` writes them: a change there is a question),
  the outbox's bound and every other figure.

## Steps

1. **The story,** `a_projection_meeting_a_full_outbox_ends_the_step`, in
   the new `src/tests/projection_room.rs` (module doc citing
   `temper/docs/design/domain/forge.md`, 6 and 12). A fixture builds the
   root from `resume_fixture::root()` and restores into `domain.forge`,
   between `restore_begin(env.wall)` and the forge's `Restored`,
   `ReadAfresh` and `SettleOutbox`: an adopted repository the projection
   may write issues in, a live goal's issue row (`desired` a live view,
   nothing projected, no pending, not failed) and `env.limits.forge.entries`
   outbox entries (`forge::Stored::Entry`) on that repository; then it
   checks `!domain.forge.effect_room(&env.limits.forge, 0)`. The story
   pushes `Work::Forge(forge::Event::ProjectDesired { entry, goal })` on
   `domain.work` and calls `crate::route::route_into` once: it returns;
   `domain.work` is empty; `domain.forge.next_wall_deadline()` is
   `env.wall + interval`; the issue row is not failed and has no pending;
   the outbox is still full; `domain.forge.full_outbox_met()`. Without
   `room-wait` this loops forever: that is why it is not this task's
   failing test.

2. **The table test, failing first:**
   `a_projection_asked_at_once_waits_in_the_step_that_met_the_full_outbox`,
   one loop over rows, each from the story's state (the outbox full, the
   mark set by the `route_into` above), then routing one
   `forge::Request::ProjectAfter { goal, when: None }` through
   `crate::route::forge_route::outputs` with the wall moved on by a
   nanosecond (`later`), so a timer the backstop arms is told from the one
   `room-wait` set:

   | row | before the request | then |
   |---|---|---|
   | `met` | nothing | no work on `domain.work`; `domain.forge.next_wall_deadline()` is `later + interval` |
   | `next step` | `domain.forge.begin_step()` | `domain.work` holds `Work::Forge(forge::Event::ProjectDesired { goal, .. })` |
   | `room` | one entry leaves the outbox (`forge::Event::Withdraw` through `forge::step`); `domain.forge.effect_room(.., 0)` | `domain.work` holds `Work::Forge(forge::Event::ProjectDesired { goal, .. })` |

   Write it and see `met` fail (the arm re-queues) before the guard. If
   `Withdraw` frees nothing from this fixture without a forge call's
   answer, free the entry as the forge's own tests do; if neither reaches
   from a root test, drop the `room` row and say so in the report.

   Then a row for the step's start, if the fixture runs `crate::fire`
   (its public timer entry) with the wall before the deadline: after
   it, `!domain.forge.full_outbox_met()`. If `fire` needs more of the
   root than the fixture holds, leave the row out and say so.

   Bites: drop the guard and see `met` fail; drop
   `!effect_room(..)` from it and see `room` fail; drop `begin_step()`
   from `fire` and see its row fail, if written. Restore each and name
   them in the report.

3. **The fix,** as `notes/contracts.md`, *The backstop*, writes it; the
   interval's sum saturates as `room-wait`'s `wait_for_room` does. A
   comment at the arm cites `temper/docs/design/domain/forge.md`, 12.

4. **Narrow runs, then the readers.** This changes when the root
   re-projects, so it runs the tests that read that:
   - `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/projection_room::/) | test(/projection_settled::/)'`
   - `heavy cargo nextest run -p temper-engine-domain --lib`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^issue_projection::/)'`

   Check each run's count; one that selected nothing proves nothing.

5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain --all-targets -- -D warnings`; remove
   the status entry; commit saying why (no re-projection can spin a step
   that met the full outbox); push.

## Report

As the implementer's text has it, and: the fixture's shape; which rows
you wrote and any you dropped, with why; the rule broken for each bite
and the failure seen; any existing test whose wall you advanced; the
counts of the runs in step 4.
