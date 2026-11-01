+++
id = "jp-erase"
part = "jig-remove-project"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/erase.rs", "jig/crates/jig-domain-core/src/tests/erase.rs", "jig/crates/jig-protocol-store/src/boundary.rs", "jig/crates/jig-protocol-store/src/commits.rs", "jig/crates/jig-protocol-store/src/keys/**", "jig/crates/jig-protocol-store/src/transcripts/**", "jig/crates/jig-protocol-store/src/tests/erase.rs", "jig/tests/core/tests/suite/erasing.rs"]
effort = "high"
minutes = 100
+++

## Goal

Give jig one entry that erases an ended task whole, at once, as its
retention would at the horizon (jig's `protocol/store.md`, 11.1: "one
commit erases a task's full records"), its transcript with it. The
project's deletion (this part's later task) erases each of its ended
tasks through it; nothing calls it yet but the tests.

Acceptance:

- `jig_domain_core::erase::erase_task(core, env, work, owner: &Erased) -> Result<Requests, Unerasable>`
  (`pub(crate)`, a new module `erase.rs`, beside `forget::forget`), where
  `Erased` is built from the task's loaded ended record
  (`Erased::of_row(&tasks::TaskRecord) -> Result<Erased, Unerasable>`, as
  `forget::Owner::of_row`); for a task whose phase is `Ended` and which no
  live task names (as a delegate, an edge, a subscription), it emits in
  one decision every write that erases it:
  - every jig row keyed by the task, in every family of the core, tasks
    and people children (its record `Ended`, `Stub`, history, milestones,
    activity, terminals, calls, turns, run proofs, decision records,
    delegate edges, its result's update, its census and index rows
    `EndedByParty`/`EndedByProject` from the record's fields, its people
    read positions and ended read, through `people::Event::Forget` as
    `forget.rs` does, with `party: None` where no requester is a person);
  - its transcript, its places and its attempts' places, through the
    store's drop (below).
  It refuses (`Err(Unerasable)`, no write) for a live or closing task, or
  one still named by a live task. Loading the record is the caller's.
- The core's `Event` (`routing/vocabulary.rs`) gains `EraseTask { record:
  Box<tasks::TaskRecord> }`, beside `Forget`, routed in `routing/mod.rs`
  to `Erased::of_row` then `erase_task`; a refusal there emits only
  `Request::Decided`.
- The core's `Write` (`routing/vocabulary.rs`) gains `EraseTask { task:
  u64 }`, and jig-protocol-store's `Write` (`boundary.rs`) gains
  `EraseTask { task: u64 }`: in the commit, every row under the task's key
  prefixes, in each family whose keys start with the task (read
  `keys/*.rs` and `protocol/store.md`'s key tables), erased by range,
  plus `DropTranscript`'s work for its transcript, file removed before the
  commit is submitted as `DropTranscript`'s is. Rows not keyed by the task
  first (the indexes, the result's update, read positions) are erased by
  `erase_task` key by key from the record's fields.
- Every application routing that maps the core's `Write` to the store's
  (`temper-service`, `jig-test-service`, the ops example, the worlds'
  store adapters) maps `EraseTask` to the store's `EraseTask`.
- Tests: `jig-protocol-store` library tests (`src/tests/erase.rs`,
  registered as the other test modules are): a task's rows of every
  task-keyed family and its transcript file gone after the commit, a
  neighbouring task's rows untouched, a missing transcript accepted;
  `jig-core-world`'s `suite` binary, a new `erasing.rs`: a person's ended
  change task and a deployment-requested one each erased whole (the
  durable rows read after the commit name the task nowhere: scan every
  key and value as `jig/tests/conformance/src/scenarios/conversations.rs`'s
  `check_ending` does), a live task refused, a crash right after the
  commit restored with the task absent.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1,
  "Whole-task erasure at once").
- `jig/docs/design/protocol/store.md`, 8.5 (transcripts dropped whole),
  11.1 (retention), and its key tables for the task-keyed families;
  `jig/docs/design/domain/tasks.md`, the retention class.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  sections 1 and 2: every task-bearing family, `forget.rs`, the
  transcript drop and its tests.

## Owns and may change

- Owns: `jig/crates/jig-domain-core/src/erase.rs` (new; its `mod` line in
  `lib.rs`), `jig/crates/jig-domain-core/src/tests/erase.rs` (new, with
  its `mod` line); in `jig-protocol-store`: `boundary.rs`'s `Write`,
  `commits.rs`, `keys/` (range helpers for a task's prefixes),
  `transcripts/` (sharing the drop with `EraseTask`), `src/tests/erase.rs`
  (new); `jig/tests/core/tests/suite/erasing.rs` (new, its `mod` line in
  `main.rs`).
- May change: `jig/crates/jig-domain-core/src/routing/vocabulary.rs`, the
  new `Write::EraseTask` and `Event::EraseTask` variants, their doc lines
  and the arms they force in that file only (task `jp-acts` edits
  `Request` in the same file at the same time: touch nothing else there);
  `jig/crates/jig-domain-core/src/routing/mod.rs`, the `Event::EraseTask`
  arm; the application routings' `Write` translation (one arm each) in
  `temper/crates/temper-service`, `jig/testing/jig-test-service`,
  `jig/examples/ops`, and the worlds' store adapters; forced arms anywhere.
- Options: if a task-keyed family's rows are not contiguous under the task
  (so a range cannot cover them), erase them key by key from what the
  core holds, or load them first through the store's `Load`; take the
  first that works and say which in the report. If a family's row needs
  a field the record lacks to build its key, add the field to the erased
  owner's struct (as `forget::Owner` carries `label` and `ended_at`).
  Spend rows (`Ledger`, `ClosedLedger`, funding records) are not the
  task's to erase: leave them and say so.

## Steps

1. Read `forget.rs` and the store's key layout; list the task-keyed
   families and the index rows in a doc comment on `erase_task` (cite
   `jig/docs/design/protocol/store.md, 11.1`).
2. The store's `EraseTask` (range erase + transcript drop), its library
   tests: `heavy cargo nextest run -p jig-protocol-store --lib -E
   'test(/erase/)'`.
3. The core's `Write::EraseTask`, `erase_task`, the routings' arms.
4. `jig-core-world` cases: `heavy cargo nextest run -p jig-core-world
   --test suite -E 'test(/erasing/)'`.
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p
   jig-protocol-store --all-targets -- -D warnings`; `heavy cargo check
   --tests` for each package whose routing arm you added; the focused
   tests of `jig-domain-core`, `jig-protocol-store`, `jig-store-world`,
   `jig-core-world`. Commit and push.

## Report

As `notes/common.md` says; list the families erased by range and those
erased key by key, and any task-bearing row you found that the list
above misses.

## Answers

2026-11-01. Question (run jp-erase-1): extend the core with an ended-record
load continuation, a refusal channel and a test entrance for a number-only
`erase_task`, or take a loaded record and return `Result<Requests,
Unerasable>` with an authorized test entrance?

- **The second, shaped as `forget` is.** `erase_task` takes `&Erased`, built
  from the loaded ended `TaskRecord` by `Erased::of_row`, and returns
  `Result<Requests, Unerasable>`; the acceptance above is amended to say so.
  No load continuation here: the project's deletion (this part's later
  task) loads each ended task through the existing `Now::EndedTask` /
  `Event::EndedTaskLoaded` read with a `ReadFor` arm of its own, and calls
  `erase_task` from there.
- **The world's entrance is `Event::EraseTask { record }`**, as
  `Event::Forget { owner }` is `forget`'s: `jig-core-world` reads the task's
  `Ended` row from its store (or keeps the record its `Stored::Ended`
  carried) and steps the event. The typed `Unerasable` reasons are checked
  in the core's unit tests (`src/tests/erase.rs`); the world's "live task
  refused" case checks that no row changed. The vocabulary and `mod.rs`
  edits this needs are allowed above; `jp-acts` edits only `Request` there.
- **"Erased by range" in the store** means: within `EraseTask`'s commit, the
  store enumerates each task-first prefix with its own range reads and puts
  one `Erase` per key; no new skein-kv operation. The transcript goes as
  `DropTranscript`'s does, its attempts' places included, with the same
  later commits for places `DropTranscript` defers to today; say so in the
  report.
- **What the end scan excludes:** spend (`Ledger`, `ClosedLedger`, funding)
  and people's retained answers (`Answer`, `Acted`), which outlive the task
  (draft, section 5, "What outlives it"). Reply and proposal `Update` rows
  and goal `Projection` rows that name the task are its to erase, key by key
  from the record when it carries their keys; any that cannot be reached
  from the record goes under `## Left`, named, with its family.
