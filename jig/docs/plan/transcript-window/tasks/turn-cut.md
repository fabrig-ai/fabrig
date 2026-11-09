+++
id = "turn-cut"
part = "turn-cut"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-store/src/boundary.rs", "jig/crates/jig-protocol-store/src/transcripts/read.rs", "jig/crates/jig-protocol-store/src/transcripts/read/**", "jig/crates/jig-protocol-store/src/tests/transcripts.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/runs/state.rs", "jig/crates/jig-domain-core/src/runs/transcript_text.rs", "jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/tests/runs/windows.rs", "jig/testing/jig-test-store/src/route.rs", "jig/tests/core/tests/suite/runs.rs", "jig/docs/design/protocol/store.status.md", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 160
+++

## Goal

A window's turn too large for a load page is loaded cut, never refused:
jig's store hands it on with its fields and its length, alone in its page,
and the core keeps its place with a fixed marker in the tail and starts
the run fresh from that tail, counting nothing. On the trunk the store
answers such a page `Unread` (`transcripts/read.rs`, `accept` stops the
page, `settle` finds it empty), so an open window holding a large tool
result fails its brief once, `transcript: its window could not be read`,
spending a transient try, and its retry starts fresh without the tail.
Failing rows first.

Acceptance:

- The failing rows (Steps, 1) are committed first, alone, and the report
  quotes their failure on the trunk.
- A window whose turn passes its page's room prepares at once, fresh, no
  brief failed, no try spent, no refusal; its brief's tail holds
  `[a turn too large to load]` in that turn's place and the other turns'
  texts as before (each within the tail's budget).
- The store and core shapes are those of
  `jig/docs/plan/transcript-window/notes/contracts.md`, exactly.
- A window read never answers `Unread` for a turn's size; a frame unread
  or refused, a window starting at another turn and a missing window
  still answer `Unread`, as their existing tests hold.
- temper still builds and keeps its behaviour: its store routing answers
  a cut page as it answered the store's `Unread` (`Owns and may change`).
- The **Not yet** entries `7.2: a turn too large for a page ...` in
  `jig/docs/design/domain/engine.status.md` and `8.3: a window's turn
  past its page's room ...` in `jig/docs/design/protocol/store.status.md`
  are removed, their lines alone.

## Inputs

- `jig/docs/design/protocol/store.md`, 8.3: *Resuming*, *A window's turn
  past its page's room* (this plan's), *A page's turns* (a turn past the
  page's bound has its fields and length read: the party path whose header
  read, `Step::Sized`, you can reuse), *Direct complete-turn requests*.
- `jig/docs/design/domain/engine.md`, 7.2: *Resuming*, *A turn too large
  for a page* (this plan's), *A window closed by an upgrade*; 9 (the
  brief's tail) and 9.1 (the table's transcript row).
- `jig/docs/plan/transcript-window/notes/contracts.md`: the shapes, binding.
- `jig/docs/plan/transcript-window/notes/common.md`.
- `jig/docs/plan/transcript-window/inputs/request.md`, items 1 and 2.

## Owns and may change

- Owns the files in `files`.
- Reader files, the new field only (`cut: None`, or `..` in a pattern),
  compiler-forced: `jig/testing/jig-test-store-files/src/store/readback.rs`,
  `jig/testing/jig-test-store-files/src/store/tests.rs`,
  `jig/crates/jig-protocol-store/src/tests/frame_patches.rs`,
  `jig/tests/store/src/world.rs`, `jig/tests/store/tests/support/frame_reads.rs`,
  `jig/tests/store/tests/suite/readers.rs`,
  `jig/tests/store/tests/suite/transcript_golden.rs`,
  `jig/tests/store/tests/suite/upgrades/*.rs`, and any other pattern or
  literal of `Event::Turns` the compiler names. The `Transcript` literal
  in `jig/crates/jig-domain-core/src/tests/start.rs` takes `cut: false`.
- **temper, the required field's lines** (a departure from the one-tree
  rule, stated): in `temper/crates/temper-service/src/store/route.rs`,
  `up`, the `Event::Turns` arm matches `cut: None`, and a page with
  `cut: Some(_)` joins the `Unread` arm (`StoreInput::Unloaded` for its
  flight), so temper answers it as it answered the store's `Unread`; in
  `temper/crates/temper-service/src/tests/store_route.rs`, the literal's
  `cut: None`. Nothing else in `temper/**`.
- Pre-approved:
  - **The store's read:** reuse the party path's header-only read
    (`Step::Sized`, `party::header`) for a window's turn past the room, or
    a header-only step of your own; a frame its machines must patch whole
    (`whole_from_head`) is read whole when within the room, and past it,
    cut from its header as a party page's turn is. If a frame past the
    room can only be cut by reading its body, stop and ask.
  - **The core's admission:** a new `#[must_use] pub fn` on `Core` beside
    `append_transcript` (such as `append_cut_transcript(task, turn, bytes)
    -> bool`) sharing its checks; or one function taking an enum; your
    call, named in the report.
  - **jig's other roots:** the reference root (`jig-test-store/src/route.rs`)
    maps a cut page to `TranscriptLoad::Cut` (contracts). If
    `jig/examples/ops` or `jig/testing/jig-test-service` reads window pages
    through its own mapping, give it the same; if it only forwards store
    events, leave it.
- Not yours: every figure (the page's room, `resume_bytes`, `tail_bytes`,
  the store's page bytes), every stored format, `docs/help/`.

## Steps

1. **The failing rows, first.** In `jig/tests/core/tests/suite/runs.rs`,
   a scenario of the `runs_scenarios` bundle per row (fresh seeds), made
   from `a_chat_past_the_resume_limit_starts_fresh`: a chat's window,
   parked, then a message wakes it. R is the window page's room in this
   world: the reference root asks `resume_bytes` (256 in
   `jig-test-profile`), bounded by the store's page bytes; state R in the
   report. Rows, by the window's turns:
   - one turn of R − 4 bytes: resumes, the turn whole (a guard);
   - one turn of R bytes: loaded whole, past the resume limit by its
     length's frame: fresh, its text in the tail (a guard);
   - one turn of R + 1 bytes: just past the room: fresh at once, the tail
     `[a turn too large to load]`; fails on the trunk;
   - turns of 100, then 4096 (or the largest the world's bounds admit,
     at least twice R), then 50 bytes: a large tool result amid others:
     fresh at once, the tail holds the marker between the others' texts,
     newest last, within the tail's budget; fails on the trunk.
   Each failing row asserts no brief failed, the task's tries unchanged
   (`tasks::Tries`), no refusal counted, the assignment at once (no
   `retry_due`), `run.policy.resume` false, its transcript empty.
2. **The store's rows,** in `jig/crates/jig-protocol-store/src/tests/transcripts.rs`,
   a table of window reads over a room of R bytes: a turn of R (whole);
   R + 1 alone (cut: `cut: Some(R + 1)`, its fields, an empty transcript,
   `next` past its frame); small, large, small (page 1 the small one
   with `next` at the large one's frame; page 2 the large one cut; page
   3 the last); the window's opening turn past the room (cut, still
   checked as the opening turn); none answers `Unread`. Plus a guard: a
   window starting at another turn still answers `Unread`.
3. **The core's rows,** in `jig/crates/jig-domain-core/src/tests/runs/windows.rs`:
   a cut turn counts `bytes + 4`, sets `cut`, keeps the marker as its text
   and no body; a window with a cut turn resumes nothing
   (`resumed_turns` empty, `transcript_oversized`); a cut turn out of
   order or opening the wrong turn is refused; `TranscriptLoad::Cut` with
   `done` starts the brief fresh with its tail. The request's candidate "a
   window whose rows or bytes pass `append_transcript`'s bound" is ruled
   out by the existing `many_one_byte_turns_start_fresh_when_their_lengths_pass_the_limit`
   and `an_oversized_resume_turn_keeps_its_text_independently`: name them
   in the report.
4. **The store,** per the contracts: `boundary.rs` (the field and its
   doc), `transcripts/read.rs` (a window's turn past the room: alone, cut;
   a page already holding a turn ends before it).
5. **The core,** per the contracts: `vocabulary.rs` (the variant),
   `runs/transcript_text.rs` (`CUT_TURN_TEXT`), `lib.rs` (its export),
   `runs/state.rs` (`Transcript.cut`, the admission, `transcript_oversized`,
   `resumed_turns`), `routing/run/preparation.rs` (`transcript_loaded`'s
   arm; the claim's `resume`). Module comments cite
   `jig/docs/design/domain/engine.md`, 7.2.
6. **The reference root,** `jig-test-store/src/route.rs`: a cut page is
   `TranscriptLoad::Cut`, then the next page as after any page.
7. Break each rule once by hand (the store answering `Unread` again; the
   core resuming a cut window), see a row fail for it, restore; name both
   in the report.
8. **Runs.** `heavy cargo nextest run -p jig-protocol-store --lib`;
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/runs::/)'`;
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/runs::/) | test(/closed_windows::/) | test(/store_adapter::/)'`.
   The store's window read changes meaning for its readers: also
   `heavy cargo nextest run -p jig-store-world`,
   `heavy cargo nextest run -p jig-test-store-files --lib`,
   `heavy cargo nextest run -p jig-core-world`,
   `heavy cargo check -p temper-service --tests`,
   `heavy cargo nextest run -p temper-service -E 'test(/store_route/)'`,
   and `heavy cargo check -p temper-engine-domain-world --tests`. Check
   each run's count.
9. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-protocol-store -p jig-domain-core -p jig-test-store -p jig-core-world -p jig-store-world -p jig-test-store-files -p temper-service --all-targets -- -D warnings`;
   remove both status entries; commit saying why (a large turn in a
   window is loaded cut and the run starts fresh from its tail, rather
   than the window failing); the citations check; push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: the failing rows' output on
the trunk, quoted; R in the world; the request's candidates ruled in or
out, each by its row; the store's read and the core's admission as you
built them; the rules broken by hand and what failed; temper's two lines;
the runs and their counts; the branch and commit pushed; implementation
and check minutes apart.

## Answers

2026-11-09, to run `turn-cut-1` (its rows at `b73e31364c` on this branch).

- **A frame behind the binary's versions, past the room: read whole, or
  header-only patching first?** Read it whole and patch it, then judge it,
  as `jig/docs/design/protocol/store.md`, 8.6, *Bounds on patched bytes*,
  already says for every read kind: the head-only path serves frames at
  the binary's versions alone. This replaces the pre-approval's "stop if
  cutting needs a body read" for behind frames; build no header-only
  patcher. A behind frame goes through `whole_from_head` and the machines
  as today; its charge is its patched size; past the window's remaining
  room it is cut from the patched record: its fields, `transcript`
  emptied, `cut: Some(` the patched transcript's length `)`, alone in its
  page. A current frame past the room is cut from its head
  (`extend_header`, `Step::Sized`), its body never read. A patched record
  past its machine's bytes or nodes still halts (`Room`) and is unread,
  as 8.6 says: that is a refused frame, not a turn's size. Every real
  set is empty until the baseline, so no machine is made and every frame
  takes the head path in the worlds; the behind path keeps its existing
  tests, with a window row past the room expecting the cut.
- **The room's accounting: transcript bytes, or the whole frame?** The
  whole frame, as 8.6 says a direct Window/Page read charges (metadata,
  payload and its 12 bytes of framing); change no accounting and no
  design. The spec's rows wrongly sized the room by transcript bytes;
  they are sized by the frame's charge instead. Let F be a turn's frame
  less its transcript in this world (state it in the report), so a turn
  of transcript length T charges T + F:
  - Steps, 1, the world's rows (`runs.rs`): replace the R − 4 and R
    guards with one guard, a turn charging exactly R (T = R − F): loaded
    whole and resumed, the turn whole in the run's transcript; then the
    row charging R + 1 (T = R − F + 1): fresh at once, the marker in the
    tail, failing on the trunk; and the 100 / 4096 / 50 row as written
    (100 and 50 fit the room whole). The "loaded whole, past the resume
    limit" guard has no single-turn case under frame accounting (T + 4 <
    T + F for any whole turn): drop it; the two core tests Steps, 3 names
    keep that path.
  - Steps, 2, the store's table: "a turn of R" is a turn charging R
    (whole); "R + 1 alone" a turn charging R + 1, answered
    `cut: Some(R + 1 − F)`; the rest as written.
  - `tests::transcripts::current_and_patched_frames_share_the_exact_direct_read_bound`
    keeps its charges for every read kind; its `Window` cases one byte
    under now expect the cut page (the turn's fields, `cut:
    Some(` its transcript's length `)`), not `Unread`, as 8.3 now says a
    window's read is never unread for a turn's size. Its other kinds keep
    `Unread`.

2026-11-09, to run `turn-cut-2`.

- **May turn-cut change the three oversized-window guards in
  `jig/tests/store/tests/suite/upgrades/upgrade_edges.rs`?** Yes, as
  recommended: they assert the trunk's `Unread` for a window turn's size,
  which `jig/docs/design/protocol/store.md`, 8.3, *A window's turn past
  its page's room*, and this task's acceptance now forbid, so this task's
  own acceptance cannot hold without them. In
  `native_pages_without_room_end_as_unread`,
  `machine_pages_charge_framing_before_accepting_any_turn` and
  `accepted_prefix_continues_without_skipping_after_an_unread`, the
  `Window` reads with row room ("zero bytes" and "one byte short"
  included) expect one cut turn: its fields, an empty transcript,
  `cut: Some(` its transcript's length `)`, and `next` as the read's
  cursor past its frame. Keep each test's charges and cursor checks; the
  "zero rows" room and every `Page` read keep `Unread`. Change nothing
  else in the file beyond the new field; test names may stay.
