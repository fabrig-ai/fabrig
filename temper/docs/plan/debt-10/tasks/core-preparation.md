+++
id = "core-preparation"
part = "tidying"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/runs/state.rs", "jig/crates/jig-domain-core/src/tests/runs/notes_room.rs", "jig/crates/jig-domain-core/src/tests/start.rs", "jig/crates/jig-domain-core/src/tests/runs/**", "jig/tests/core/tests/suite/restart.rs", "jig/tests/core/tests/suite/runs.rs", "temper/crates/temper-engine-domain/src/archive.rs", "temper/crates/temper-service/src/tests/store_route.rs"]
effort = "medium"
minutes = 40
+++

# A1, A2, A4: the core's preparation

## Goal

Three small cuts left by moving `PrepareAgent` out of the core. Behaviour stays unchanged.

Acceptance:

- **A1.** `route_prepared_agent` is `prepare_agent`, private, and has no check of its row's phase: `route_activate`
  checks that just before. Search for the old name finds nothing.
- **A2.** `Core::begin_transcript(task, previous_attempt, window)` takes `window: Option<(u64, u32)>` and derives none.
  Neither product caller assigns `.window` after it.
- **A4.** `a_busy_root_still_prepares_an_agent_with_nothing_to_load` has no branch on its input and no closed-window
  row; that row's assertions are a case of `a_busy_root_keeps_an_agent_with_a_window_to_load_due`.
- Every test passes with no assertion changed but A4's move.
- **Size cap:** net at most -10 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/domain/engine.md`, 7.2, for what a preparation loads.

**What this cut checked.** A1: one caller, in `route_activate`; between its phase check and the call stand only
`recurring_template`, `run_admission` and `transcript_from`. A2: `begin_transcript` sets `window` from the proof, and
both product callers overwrite it (`prepare_agent` with `from`, `transcript_unread` with `None`). A4: the name says
"prepares", and the `Some(true)` row asserts that it doesn't.

## Owns and may change

- Owns: `routing/run/preparation.rs`, `runs/state.rs`, `tests/runs/notes_room.rs`.
- `begin_transcript` is `pub`, and tests in other crates call it. The new argument is pre-approved: the request asks
  for it. May change, the call alone: `jig-domain-core`'s tests, `jig/tests/core/tests/suite/{restart,runs}.rs`,
  `temper/crates/temper-engine-domain/src/archive.rs`, `temper/crates/temper-service/src/tests/store_route.rs`. At each,
  pass what `begin_transcript` derived there: `None` where the test holds no proof with a window, otherwise that
  proof's `(attempt, turn)`.
- **If `run_admission` can move a task out of `Due`,** the guard is live: keep it, and say so.
- **If a test depends on the derived `window` in a way one argument can't state,** add a test-only helper in that
  test's own file, and say so.

## Steps

1. A1, then `heavy cargo nextest run -p jig-domain-core --lib -E 'test(notes_room)'`.
2. A2: the argument, its two product callers, then the test callers. `heavy cargo check -p jig-domain-core -p
   jig-core-world -p temper-engine-domain -p temper-service --tests` while errors last.
3. A4.
4. `heavy cargo nextest run -p jig-domain-core -p jig-core-world`, `-p temper-engine-domain --lib -E 'test(archive)'`,
   `-p temper-service --lib -E 'test(store_route)'`. fmt, clippy, old forms, citations. Commit, push:
   one commit per finding.

## Report

As the implementer's text says, with the net lines. A1 and A2 pin no new rule; A4 moves a row.
