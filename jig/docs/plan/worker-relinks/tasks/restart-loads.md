+++
id = "restart-loads"
part = "restart-loads"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/loads.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/crates/temper-engine-domain/src/archive.rs", "temper/tests/engine/domain/tests/suite/preparation.rs", "temper/docs/design/domain/root.status.md", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 100
+++

## Goal

A task whose transcript window would find every load slot taken waits,
due, instead of failing its preparation: the root answers its activation
busy, as it already does when the archive's read table is full, so no
transient try is spent on room (`temper/docs/design/domain/root.md`, 8,
*An issued transcript load that fails*: "A transcript load is issued only
with read room: without it the root answers the activation `busy` and the
task stays due"; jig's `jig/docs/design/domain/engine.md`, 9.1: room is
capacity, never transient). This is fault 3's restart of the request:
right after a restart about 23 briefs failed with `transcript: its window
could not be loaded`, and five tasks were held. Failing row first.

Acceptance:

- The failing row (Steps, 1) is committed first, alone, and the report
  quotes its failure on the trunk.
- `activate` answers busy when no load slot is free, as when the archive's
  read table is full; a due task then stays due and is activated again
  once a slot frees, its window loaded whole.
- With `root.loads.loads` at the shipped 2, three or more tasks due
  together each load their window and resume it: none fails its
  preparation, none spends a transient try, none is held.
- `an_unissued_transcript_load_remains_preparation_failure` in
  `archive.rs`'s tests keeps its rule only where it still holds (Steps,
  3), renamed to say it.
- The **Not yet** entry `8: a transcript load that finds every load slot
  taken ...` in `temper/docs/design/domain/root.status.md`, and the
  **Departs** entry `9.1: a transcript's load refused for want of a load
  slot counts as transient ...` in `jig/docs/design/domain/engine.status.md`,
  are removed, their lines alone. That departs from the implementer's
  text, which closes `docs/design/` to you, and from the one-tree rule for
  the jig entry: both record this code's departure, and are this task's.

## Inputs

- `temper/docs/design/domain/root.md`, 8 (*Loads for the core*, *An issued
  transcript load that fails*) and 9.1 (*Restart*, step 7: every due task
  is activated).
- `jig/docs/design/domain/engine.md`, 9.1 (*A brief that fails*: capacity,
  transient, and the transcript rows of its table) and 7.2 (*Transcripts*,
  the window's loads).
- `jig/docs/design/domain/engine.status.md`, the **Departs** 9.1 entry: the
  departure this task closes, in the code's words.
- `jig/docs/plan/worker-relinks/notes/common.md`.
- `jig/docs/plan/worker-relinks/inputs/request.md`, item 3.

## Owns and may change

- Owns the files in `files`.
- Pre-approved:
  - **A room query on `Loads`** (`loads.rs`): `pub fn has_room(&self) ->
    bool`, or the name you choose, counting a retired, unreclaimed slot as
    taken, as `begin` does.
  - **If the core's activation is not where every transcript load starts**
    (a message to a parked chat, a resume after a hold), give each such
    path the same answer: busy and due, never `Failed`, and name each in
    the report.
  - **If the `Driver` restarts** (as `restore_fit.rs` and `switch.rs` make
    a restarted root), the row restarts it with the tasks parked; if not,
    the row makes the tasks due together in one pass, which is the
    restart's case without the restart, and says so.
  - **A change in jig's core** (`jig-domain-core`'s preparation, a new
    `TranscriptLoad` variant) is not needed for this rule; if the rows
    show it is, that is a `Question:`, with the failing row pushed.
- Not yours: `temper-service` (its limits stay: `root.loads.loads` is
  2), `jig/**` but the one status entry, every figure.

## Steps

1. **The failing row, first, alone.** In
   `temper/tests/engine/domain/tests/suite/preparation.rs`:
   `tasks_due_together_past_the_load_slots_each_load_their_window`. With
   the shipped limits (`root.loads.loads` 2), three chats each answer a
   turn and park (`Driver::open_chat`, `driver.answer(.., tasks::End::Parked, ..)`,
   as `a_transcript_read_landing_after_a_message_ended_its_preparation_is_dropped`
   does); then each is made due in the same pass (a message to each, or
   the restart, `Owns and may change`). Settle. Assert every chat's next
   assignment carries its window (the parked turn), no
   `Fact::BriefFailed` (or the root's preparation-failure fact) names
   `transcript: its window could not be loaded`, and no task's retry
   count moved. On the trunk the third fails its preparation.
2. **The rule.** In `route.rs`, `activate`: `busy` is the archive's read
   table full **or** no load slot free. The comment above it says why: the
   load the activation allows finds its read slot and its load slot.
3. **The unit test,** in `archive.rs`'s tests:
   `an_unissued_transcript_load_remains_preparation_failure` saturates the
   load slots and calls `request_load` directly, below the activation. If
   that path is still reachable once Steps 2 holds (a load requested
   without an activation), keep the test and rename it to the case it
   still pins; if it is not, turn it into the activation's case: with the
   load slots full, the activation is answered busy, no load is
   requested, and the core hears no `TranscriptLoaded`. Name which in the
   report.
4. Break the rule once by hand (drop the load-slot half of `busy`), see
   the row fail for that reason, restore, and name it in the report.
5. **Runs.** `heavy cargo nextest run -p temper-engine-domain --lib`,
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^preparation::/) | test(/^loads::/) | test(/^restore_fit::/) | test(/^switch::/)'`.
   This changes when an activation is answered busy, which every story
   that prepares a run sees: also
   `heavy cargo nextest run -p temper-engine-domain-world` whole, and
   `heavy cargo nextest run -p temper-service -E 'test(/restart|preparation|brief/)'`.
   Check each run's count.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-world
   --all-targets -- -D warnings`; remove both status entries; commit saying
   why (tasks due together wait for a load slot instead of spending a try on
   room, so a restart holds none); the citations check; push.

## Report

As the implementer's text has it, and: the row's failure on the trunk;
whether the row restarts the root or makes the tasks due in one pass; each
path that starts a transcript load and what it answers now; the rule
broken for the bite and the failure seen; what became of the unit test.
