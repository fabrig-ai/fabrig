+++
id = "temper-cut"
part = "temper-cut"
kind = "feature"
needs = ["window-pages", "turn-cut"]
files = ["temper/crates/temper-service/src/store/route.rs", "temper/crates/temper-service/src/store/ranges.rs", "temper/crates/temper-service/src/tests/store_route.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/archive.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/tests/engine/domain/tests/suite/preparation.rs", "temper/tests/engine/domain/src/store.rs", "temper/docs/design/domain/root.status.md", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 140
+++

## Goal

temper carries a window's cut turn to the core: the store's cut page
(`turn-cut`) reaches it as `TranscriptLoad::Cut`, and a turn the store
carried whole but past the root's own page bounds is handed on cut the
same way, so a goal whose window holds a large tool result starts fresh
from its tail at once, counting nothing. On the trunk (with `turn-cut`
merged) the service answers a cut page as unread (`StoreInput::Unloaded`,
the core's `Unread`: `transcript: its window could not be read`, a
transient try, then a fresh start without the tail), and a row past
`root.loads.transcript_bytes`, or alone past `root.loads.bytes`, ends its
page in `route.rs`, `page_failed`, whose `stop` reports the last commit
`Uncommitted`. Failing rows first.

Acceptance:

- The failing rows (Steps, 1) are committed first, alone, and the report
  quotes what each did on the trunk.
- A window whose turn passes the store's page room, or the root's row or
  page bound, prepares at once, fresh: no brief fails, no try is spent,
  the engine keeps running, and the brief's `TranscriptTail` section holds
  `[a turn too large to load]` in that turn's place.
- A store `Unread` (a frame unread or refused) still reaches the core as
  `Unread`, and the archive's row and text guards still answer `Failed`.
- The **Not yet** entry `8: a window's turn handed on cut ...` in
  `temper/docs/design/domain/root.status.md` is removed, its lines alone.
- Two status sentences say what is now true (a departure from the
  implementer's text and, for jig's, from the one-tree rule: both describe
  this plan's temper code): in `temper/docs/design/domain/root.status.md`,
  the paragraph *Section 8's transcript load has no known gap at an
  activation* ends `A window's later page is below.`: replace that
  sentence with `A window's later pages keep its first page's slot, and a
  turn too large for a page reaches the core cut.`; in
  `jig/docs/design/domain/engine.status.md`, the paragraph *Section 9.1's
  room as capacity holds at temper's activation* ends `A window's later
  page without one still fails its preparation there
  (`temper/docs/design/domain/root.status.md`, 8).`: replace it with `A
  window's later pages keep its first page's slot there
  (`temper/docs/design/domain/root.md`, 8).`

## Inputs

- `temper/docs/design/domain/root.md`, 8: *Pages*, *Loads for the core*,
  *An issued transcript load that fails* (its cut turn, this plan's), *A
  window's pages keep their load slot*.
- `jig/docs/design/protocol/store.md`, 8.3 (*A window's turn past its
  page's room*) and `jig/docs/design/domain/engine.md`, 7.2 (*A turn too
  large for a page*).
- `jig/docs/plan/transcript-window/notes/contracts.md`: the store's `cut`,
  `TranscriptLoad::Cut`, and temper's default shape.
- `jig/docs/plan/transcript-window/notes/common.md`.
- `jig/docs/plan/transcript-window/inputs/request.md`, items 1 and 2.

## Owns and may change

- Owns the files in `files`.
- Reader files, call sites only (a `Some(..)` texts literal's new element
  type): `temper/crates/temper-service/src/engine.rs`,
  `temper/crates/temper-service/src/tests/fixtures.rs`,
  `temper/crates/temper-engine-domain/src/route.rs`,
  `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper/crates/temper-engine-domain/src/tests/restore_fit.rs`,
  `temper/tests/engine/domain/src/*.rs`, `temper/tests/system/src/composed.rs`,
  and any other the compiler names.
- Pre-approved:
  - **temper's shape:** the contracts' default (`TurnText` in
    `boundary.rs`, `TranscriptTexts = Box<[TurnText]>`), or another
    temper-internal shape that says a page's one turn is cut and its
    length; named in the report.
  - **The service's own cut:** in `route.rs`, `page`, a non-startup
    `TaskTranscript` page whose first kept row would pass
    `root.loads.transcript_bytes`, or `root.loads.bytes` alone, is
    answered that row cut (its fields, its transcript emptied, its
    length), alone, `next` at its key, instead of `page_failed`; a later
    row past the bounds still ends the page before it, as now. If the
    root's load seam (`loads.rs`, `check`) could still cut a transcript
    row the service passed, say where, and leave the seam's translation.
  - **The world's bounds for a row:** the walking world's
    `root.journal.transcript_bytes` (8192) may refuse a turn past
    `root.loads.transcript_bytes` (8192) or the page's room before it is
    committed. A row may build its world from the shipped standard
    profile's `root.loads` and `root.journal`
    (`temper_service::limits::profile`, as `parked_chats` takes shipped
    capacities); if the world still cannot commit a turn a row needs, take
    the largest it can, say so, and keep the row. Never change a figure
    of a fixture or a profile.
- Not yours: `jig/**` but the one status sentence, every figure, the
  load seam's slot rule (`window-pages`).

## Steps

1. **The failing table, first.** In
   `temper/tests/engine/domain/tests/suite/preparation.rs`, a table test
   `a_window_with_a_large_turn_starts_fresh_from_the_tail_that_fits`: a
   parked chat whose window holds the row's turns, woken by a message.
   Rows, by the large turn's size against the bounds in force (state them
   in the report):
   - just inside `root.loads.transcript_bytes`: loaded whole (a guard;
     resumed or fresh by the resume limit, no failure either way);
   - just past it, within the store's page room: fails on the trunk
     (`page_failed`, `stop`); say what the trunk did;
   - past the store's page room (twice it, or the largest the world
     commits): fails on the trunk (unread, a transient try);
   - three short turns, the large one, one short turn (a window of
     several pages around a cut turn): fails on the trunk.
   Each row asserts the next assignment comes at once, fresh
   (`run.policy.resume` false, `transcript` empty), its `TranscriptTail`
   section holds `[a turn too large to load]` in the cut turn's place
   with the short turns' texts around it (as `upgrade_transcripts.rs`
   reads a tail), no `Fact::BriefFailed`, tries `tasks::Tries::NONE`,
   and the root still ready.
2. **The cut through temper,** in `boundary.rs` (the shape),
   `temper-service`'s `route.rs` (`up`: a cut page, now carried rather
   than unread; `page`: the service's own cut) and `ranges.rs` (`texts`:
   a cut turn has no rendering), `archive.rs` (`transcript_loaded`: a page
   whose one turn is cut is the core's `TranscriptLoad::Cut`, its `done`
   as a page's), `restart.rs` (the texts it hands on). Module comments
   cite `temper/docs/design/domain/root.md`, 8.
3. **The service's rows,** in `temper/crates/temper-service/src/tests/store_route.rs`:
   a store page with `cut: Some(bytes)` reaches the root as one cut turn;
   a page whose first row passes the root's row bound is answered that
   row cut, never `stop`; `Unread` and `Corrupt` still answer
   `StoreInput::Unloaded`.
4. Break each rule once by hand (the cut page answered unloaded again;
   the service's own cut back to `page_failed`), see a row fail for it,
   restore; name both in the report.
5. **Runs.** `heavy cargo nextest run -p temper-service -E 'test(/store_route/)'`;
   `heavy cargo nextest run -p temper-engine-domain --lib`;
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^preparation::/) | test(/^upgrade_transcripts::/) | test(/^loads::/)'`.
   What a transcript page carries changes for every reader: also
   `heavy cargo nextest run -p temper-engine-domain-world` whole,
   `heavy cargo nextest run -p temper-service`, and
   `heavy cargo check -p temper-system-world --tests` (or the package of
   `temper/tests/system`, by its `Cargo.toml`). Check each run's count.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-service -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`;
   the status entry and the two sentences; commit saying why (temper
   hands a large window turn on cut, so the run starts fresh from its
   tail rather than failing or stopping the engine); the citations check;
   push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: each row's result on the
trunk, quoted, and the bounds in force; temper's shape and the service's
cut as you built them; the rules broken by hand and what failed; the runs
and their counts; the branch and commit pushed; implementation and check
minutes apart.
