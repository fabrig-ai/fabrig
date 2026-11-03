+++
id = "room-wait"
part = "room"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/routing/**", "jig/crates/jig-domain-core/src/tests/**", "jig/crates/jig-protocol-store/schema/tasks-v9.schema", "jig/crates/jig-protocol-store/src/**", "jig/crates/jig-protocol-store/golden/tasks/**"]
effort = "high"
minutes = 240
+++

# A procedure's batch refused for live room waits for that room

## Goal

A procedure step whose batch jig refuses for the live tasks in the deployment or in its
project (a count that frees as tasks end) makes its task wait for that room, durably and in
the order the refusals came, and an ending task wakes the oldest waiter its place serves, as
jig's `jig/docs/design/domain/connectors.md`, 6.2 ("when room it was refused for frees") has
it. Today the step is refused (`Ask::StepRefused`) and the procedure is stepped again only
when some later fact moves it, so a goal of many changes on a box whose live tasks bind loses
changes to refusals. Every other refusal (the tree's allotment, delegates, inbox, depth,
authority, spend, shape) stays a refusal.

Acceptance:

- **Which refusals wait** (`jig-domain-tasks`): a `Problem` gains `freeing_count(&self) ->
  Option<LiveRoom>`: `Some(LiveRoom::Deployment)` for a batch (`CapacityAction::Batch`) refused
  for the deployment's live tasks (`CapacityCause::Tasks`, `Refusal::Live`),
  `Some(LiveRoom::Project(p))` for one refused for its project's (`CapacityCause::ProjectTasks`,
  `Refusal::Project`, the capacity's project), `None` for everything else, matched
  exhaustively over the action and the cause so a new one is decided where it is added.
- **The wait, durable:** `pub enum LiveRoom { Deployment, Project(u32) }` and `pub struct
  RoomWait { pub room: LiveRoom, pub order: u64 }`; the live task's outcome record gains
  `room_wait: Option<RoomWait>`. A procedure task whose batch is refused with a freeing count
  takes the next order (one past the largest any live task holds) and stays live, idle,
  waiting; the refusal is still told to its procedure as today, with why.
- **The wake:** when a task ends, the oldest waiter (lowest order) whose room the ending frees
  (any waiter for the deployment's room; for a project's, those of the ending task's project)
  is woken to step again on the facts as they are then (`wake_procedure`), one waiter per
  ending; a wake reserves nothing: the step's batch is checked afresh, and a refusal again
  takes a new order at the back. A waiter that steps, is cancelled or ends drops its wait.
- **Across a restart:** the waits are restored from the live rows; a row whose wait is not a
  procedure's, names another project than its task's, has order 0, or shares its order with
  another row is refused at restore as the domain's other invalid rows are.
- **The store** (`jig-protocol-store`): `tasks-v9.schema` gains, in place (the version staying
  9 until the first release: `jig/docs/design/protocol/store.md`, 5), `record RoomProject {
  project: u32 }`, `enum LiveRoom { deployment, project: RoomProject }`, `record RoomWait {
  room: LiveRoom order: u64 }`, and `room_wait: option RoomWait` appended last to `record
  TaskOutcome`; its generated code regenerated with skein's generator as the crate's other
  schema changes were, its codec mapping (`src/codec/tasks/…`), and its golden bytes
  regenerated in place (new records' goldens added, changed ones rewritten). Any fixture or
  profile price the larger row needs (`jig-protocol-config`'s profiles' store sizes) is a
  compiler- or test-forced change, listed in the report.
- **Tests** in `jig-domain-tasks` (a `tests/room.rs` module): a procedure's batch refused for
  the deployment's live tasks waits and is woken when a task ends, and its batch then made; the
  same for a project's live tasks, a task ending in another project waking none; two waiters
  woken one per ending in refusal order; a refusal for the tree's allotment, for delegates and
  for the inbox does not wait; an agent's `delegate` refused for room is refused, not waiting
  (`jig/docs/design/domain/tasks.md`, section 11); a restart rebuilds the waiters and keeps
  their order. In `jig-protocol-store`: the codec round-trips a `TaskOutcome` with and without
  a wait, and the goldens match.
- The focused tests of `jig-domain-tasks`, `jig-domain-core`, `jig-protocol-store` and
  `jig-protocol-config` pass, and `temper-engine-domain`'s (temper's procedures run on this:
  run its focused tests, and `heavy cargo check -p temper-engine-domain --tests`).

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `jig/docs/design/domain/connectors.md`, 6.2 (when a procedure steps: room it was refused for)
  and 6.4 (level-triggered); `jig/docs/design/domain/tasks.md`, section 4 (a batch's counts)
  and section 11 (an agent's refused batch is refused).
- `jig/docs/design/protocol/store.md`, 5 (schemas change in place until the first release).
- **Old code, a reference never a plan:** fabrig-ai's jig `github-pilot/t28-land-r4`, commit
  `6ae1d45ab` "Wait for live task room after procedure batch refusal", built this on an older
  tree: a `room.rs` in `jig-domain-tasks` with `LiveRoom`, `RoomWait`, `Problem::freeing_count`
  and `wait`/`freed`/`valid` as above; the step's refusal in `jig-domain-core`'s
  `routing/procedure_steps.rs` keeping the wait's request in its outcome; `closing.rs` calling
  `freed` with the ending task's project; the schema lines above. Today's tree differs (the
  record is `TaskOutcome`; `wake_procedure` and `publish` are in `src/domain.rs`); write it
  afresh against today's code.

## Owns and may change

- Owns the files of this spec's header for what acceptance names.
- Literals and matches elsewhere in jig that the new field or enum forces (fixtures in
  `jig/tests/**`, `jig/testing/**`, `jig-protocol-config`'s tests): the new field only, `None`,
  compiler-forced.
- temper: nothing; temper's re-stepping of a refused procedure
  (`temper-engine-domain/src/route/asks.rs`) is left as it is. If temper's tests fail because
  a procedure now waits where it was refused, stop and ask, naming the test.

## Steps

1. The types and `freeing_count`; the wait and the wake; the restore check.
2. The schema, the generator's output, the codec, the goldens.
3. The tests; run each narrowly (`heavy cargo nextest run -p jig-domain-tasks --lib -E
   'test(/room/)'`).
4. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p
   jig-domain-tasks -p jig-domain-core -p jig-protocol-store --all-targets -- -D warnings`; the
   focused tests named in acceptance; commit and push.

## Report

As the implementer's text asks.
