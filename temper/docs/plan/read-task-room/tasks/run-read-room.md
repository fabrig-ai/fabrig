+++
id = "run-read-room"
part = "run-read-room"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/reads/run_task.rs",
  "jig/crates/jig-domain-core/src/reads.rs",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/crates/jig-domain-core/src/routing/run/calls.rs",
  "temper/crates/temper-engine-domain/src/limits.rs",
  "temper/crates/temper-engine-domain/src/route/tools.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/startup/calls_tests.rs",
  "temper/tests/engine/domain/tests/suite/host_calls.rs",
  "jig/docs/design/domain/engine.status.md",
  "temper/docs/design/domain/root.status.md",
  "temper/docs/design/domain/agent.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 90
+++

# A run's `read_task` reads in its own room

## Goal

A run's `read_task` is cut at `documents.text_bytes`, the most a client decodes: `run_task::value` in `jig/crates/jig-domain-core/src/reads/run_task.rs` passes it to `chunk`. So a box's run takes one call per 512 bytes of a spec, each a turn at a coding model's prices. Let the root name a room with the read, so one call answers what the run's reply can hold. A person's client still reads at `documents.text_bytes`.

Rendering needs no change: `temper_engine_smith::answer_bounded` already cuts at the reply's room and prints `next:`.

Acceptance:

- With `documents.text_bytes` 512 and a local agent's reply of 2,048, a spec of 1,500 bytes comes back from one `read_task` whole, with no `next:`.
- A spec of 5,000 bytes, same bounds: every answer is at most 2,048 bytes, each continues from the printed `next`, the pieces join to the spec, and it takes 3 calls, where 512-byte pieces take 10.
- With `host_reply_bytes` `None` (no local agent's metadata), the read's words are at most `documents.text_bytes`, as now.
- With `root.journal.transcript_bytes` below the reply, a read is answered and reads to the end, never refused and never past the door. `Decision::deliver` refuses a delivery past that bound (`host_within`, `temper-engine-domain/src/decision.rs`), and `host_route::read` asserts its door took it, so this bound matters.
- jig: a read's words are at most its room and at most `call_answer_bytes`; `documents.text_bytes` does not bound them; `answer_bound` covers a read at that most.
- A person's client is unchanged: `jig/crates/jig-domain-core/src/tests/documents/task_specification.rs` passes without an edit.
- A started service's domain limits carry the agent's `host_reply_text` (`temper-service/src/startup/calls_tests.rs`).
- The report says what one read holds at `temper init`'s limits, measured with the real `size_of`s (a throwaway print is fine): words at most 10,662, plus `answer_carrier_bound`; and that times `fleet.calls` as the most held at once.

## Inputs

- `temper/docs/plan/read-task-room/notes/common.md`.
- `temper/docs/design/domain/root.md`, 5.2 (*A task read's room*).
- `temper/docs/design/domain/agent.md`, 4.3 (*A coordinator's task reads*, *An answer fits what the agent takes*) and 10.
- `jig/docs/design/domain/engine.md`, 7.3 (*Task reads*).
- `temper/docs/design/protocol/setup.md`, 8.1 and 16.

## Owns and may change

Owns the files in the header, each for what the sketches say.

May change, only so:

- `temper/tests/engine/domain/src/walking.rs` and `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`: the literal's `host_reply_bytes: None`.
- `jig/crates/jig-domain-core/src/reads/settings.rs`, or wherever a match on `Destination` names `RunTask`'s fields: that arm alone.
- `temper/crates/temper-engine-domain/src/decision.rs`: nothing is expected here; do not widen `host_within`.
- `temper/crates/temper-engine-smith/src/answers.rs`: only if a new test shows `task_read_words` cuts wrongly; say so in the report.

Departs from the implementer's text: this task edits the status files in its header, and only so. Delete the **Not yet** entry naming `read_task`'s room from each of `jig/docs/design/domain/engine.status.md` (7.3), `temper/docs/design/domain/root.status.md` (5.2), `temper/docs/design/domain/agent.status.md` (4.3) and `temper/docs/design/protocol/setup.status.md` (8.1). In `temper/docs/design/STATUS.md`, make "so a client reads a spec in pieces of it, and so does a run until `read_task` has its own room" read "so a client reads a spec in pieces of it". Run `python3 -I -B tools/citations/citations.py --base origin/main`. Any other `docs/design/` is not yours.

Options, each pre-approved:

- `run`, `load` and `loaded` in `run_task.rs` pass too many arguments for clippy: gather `task`, `field`, `from` and `room` in a small `Copy` struct private to the crate, also carried by `Destination::RunTask`. Default: do that.
- A test in `run_task.rs` sets `limits.documents.text_bytes` to size a chunk: pass that figure as the room, and raise the test's `limits.call_answer_bytes` if its 256 would cut first.
- The engine world's bounds refuse these specs (`tasks.spec_bytes` is 64 in `walking.rs`, and a brief's room may also bind): raise what is needed in this test's own bounds, never in the shared `limits()`. If 5,000 bytes will not fit the world, take smaller figures keeping spec above reply above 512, and say which.
- A `size_of` pin moves because `NamedAction` or `Pending` grew: follow it, and list it in the report.

Not this task: the worker's hello or any way for an engine to learn a remote agent's reply bound (`agent.md`, 10); `tool_payload`; a client's `text`; `documents.text_bytes` anywhere; the playground's documents; `read_task`'s schema, description or help.

## Steps

Outer layer, jig:

```rust
// jig/crates/jig-domain-core/src/vocabulary.rs
ReadTask {
    field: crate::reads::TaskTextField,
    task: u64,
    from: u64,
    /// The most words this read's answer holds, as its application names it (engine.md, 7.3).
    room: u32,
},

// jig/crates/jig-domain-core/src/reads/run_task.rs, in `value`
let words = super::task_text::chunk(row.number, field, source, from, room.min(limits.call_answer_bytes))

/// The most a task read's carrier and metadata take, without its words.
#[must_use]
pub fn answer_carrier_bound(limits: &Limits) -> Option<u64> // today's answer_bound less documents.text_bytes

/// Maximum task-read carrier and payload under the admitted limits.
#[must_use]
pub fn answer_bound(limits: &Limits) -> Option<u64> // answer_carrier_bound plus call_answer_bytes
```

Outer layer, temper:

```rust
// temper/crates/temper-engine-domain/src/limits.rs, on Limits
/// The longest host answer the local agent takes, its `host_reply_text`;
/// `None` in a process with no local agent's metadata.
pub host_reply_bytes: Option<u32>,

/// The most words one task read holds (temper/docs/design/domain/root.md, 5.2).
pub(crate) fn task_read_room(limits: &Limits) -> u32 {
    // None: documents.text_bytes.
    // Some(reply): reply, within call_answer_bytes, within
    // journal.transcript_bytes less run_task::answer_carrier_bound(&core_limits(limits)).
}

// temper/crates/temper-engine-domain/src/route/tools.rs, in relay_payload
action: core::NamedAction::ReadTask { task, field, from, room: crate::limits::task_read_room(&domain.limits) },
```

`temper-service` keeps one holder of the figure, the domain's limits:

- `limits.rs`, `engine_candidate`: `host_reply_bytes: None`.
- `system.rs`: `limits.host_reply_bytes = settings.host_reply_bytes;` beside `limits.memory_bytes`, and drop `engine.host_reply_bytes = ...`.
- `engine.rs`: drop the `host_reply_bytes` field and its initializer.
- `route.rs`, `render_host`: read `engine.env.limits.host_reply_bytes.unwrap_or(u32::MAX)`.
- `startup/calls_tests.rs`, the assertion at its line 80: `engine.env.limits.host_reply_bytes`.
- `settings::Engine.host_reply_bytes` stays.

Tests, the failing one first. Add the field and `room` with `task_read_room` answering `documents.text_bytes` in both arms, so everything compiles; write the tests; run them and see the first fail for its words being cut at 512. Then write the room.

```rust
// temper/tests/engine/domain/tests/suite/host_calls.rs
// temper/docs/design/domain/root.md, 5.2.
const WINDOW: u32 = 512;
const REPLY: u32 = 2048;

fn coordinator_within(bounds: engine::Limits) -> (Driver, engine::Assignment) // coordinator() with these bounds
fn read_room_bounds(reply: Option<u32>) -> engine::Limits // added_deps::limits(), WINDOW, reply, spec room
fn read_whole(driver, run, child) -> (Vec<u8>, u32) // follows next, rendering each answer with answer_bounded(&answer, REPLY); the words and the calls

#[test] fn a_run_reads_a_spec_past_the_clients_window_in_one_call()
#[test] fn a_spec_past_the_room_continues_by_offset_and_comes_back_whole()
#[test] fn without_a_local_agents_reply_a_read_stays_in_the_documents_window()
#[test] fn a_read_stays_within_the_journal_bound()
```

`decoded_task_call` gives the typed answer, so the third test reads `words.bytes.len()` from `engine::CallAnswer::TaskRead`, and the fourth checks each answer's `call_answer_bytes`-counted size through what the door admits: the read is answered. In `run_task.rs`'s tests, add one with `documents.text_bytes` 4 and room 64, reading a longer source whole, one where `call_answer_bytes` cuts below the room, and one that `answer_bound` covers the largest read.

This changes what a shared item means: run the tests of its readers.

- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(host_calls)'`, then the whole `-p temper-engine-domain-world`.
- `heavy cargo nextest run -p jig-domain-core --lib`; `-p temper-engine-domain`; `-p temper-engine-smith`; `-p temper-service`.
- `heavy cargo check -p jig-test-domain -p jig-test-service --tests` and any other dependent that names `reads::run_task::answer_bound` or `NamedAction`.
- `TEMPER_BUNDLE=none` as the machines set it; check the count of each run.

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core -p temper-engine-domain -p temper-service -p temper-engine-domain-world --all-targets -- -D warnings`; the citations check; commit small and push.

## Report

The usual, and: the failure the first test showed before the room was written; each rule broken by hand and what failed; the figures of what one read holds at `temper init`'s limits, and at most at once; any world bound raised; whether `answers.rs` needed anything. The proof beyond this task is fabrig/qa's next goal's `read_task` count: say nothing of it as done.
