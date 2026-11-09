# transcript-window: rules every task keeps

- **The request** is `jig/docs/plan/transcript-window/inputs/request.md`:
  on a deployment at fabrig `69cff99edb`, a goal's coordinator, about an
  hour old, was held again and again with `brief failed ... counts=transient
  reason=transcript: its window could not be loaded`; a goal a few minutes
  old was held the same way; five tasks failed the same way for about a
  minute after each of two restarts. The evidence is outside the
  repository: the request's words are all there is. Read it for the why;
  your spec and the design sections it names are what you build.
- **Which words come from where.** The core says `transcript: its window
  could not be loaded` for `TranscriptLoad::Failed` and for a page row
  `append_transcript` refuses
  (`jig/crates/jig-domain-core/src/routing/run/preparation.rs`,
  `transcript_loaded`); it says `... could not be read` for
  `TranscriptLoad::Unread`. temper's root answers `Failed` from
  `temper/crates/temper-engine-domain/src/archive.rs`, `transcript_failed`:
  a continuation page refused a load slot (`restart.rs`, `request_load`),
  a page the load seam cut, texts that do not match their rows, a row
  that is not a turn. A store answer that fails (`Unread`, `Corrupt`, a
  malformed page) reaches the core as `Unread` (`restart.rs`,
  `load_outputs`).
- **Failing test first.** Each fix is pinned by a test that fails on the
  trunk for the fault's reason before the fix and passes after it. Commit
  the failing test first, on its own, saying which fault it shows; then
  the fix. The report quotes the failure seen before the fix.
- **Candidates are ruled in or out by test, never by assumption.** A spec
  that names candidates gives each a row; a row that passes on the trunk
  rules its candidate out and stays as a guard. The report says, per
  candidate, ruled in or out and by which row.
- **No figure changes.** Rows, bytes, slots, budgets and bounds stay as
  configured (`jig/crates/jig-protocol-config/src/profile/*.rs`,
  `temper/crates/temper-service/src/limits.rs`, every fixture's limits).
  The deployment set `documents.text_bytes` to 512: that bound sizes a
  task read's room, not a transcript's page. If a test shows a figure
  itself is wrong, stop and ask (`Question:`) with the measurement.
- **No stored format changes.** A turn's record, a transcript's frame and
  every store schema stay as they are; needing one is a `Question:`.
- **Tests stay cheap.** A new case of one behaviour is a row of an
  existing table test, or of the table your spec makes from one test. A
  story goes in a world below the real loop on a simulated clock. No test
  reaches a real host or network; no sleep.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a process or live tier
  story, a walk, a render check or a throwaway driver.
- **Every build and test through `heavy`,** one command at a time, with
  `TEMPER_BUNDLE=none` (the machines' cargo config sets it). `cargo fmt`
  builds nothing and runs directly.
- **Trees:** jig never names temper; skein names neither. A change to
  skein is a `Question:`; none is planned.
- **Status lines:** the design is written (`## Inputs` names it). A task
  removes only the **Not yet** entries its spec names, its lines alone.
  That departs from the implementer's text, which closes `docs/design/`
  to you: those entries are your task's.
- **Before calling a focused or fuzzy failure "outside the change,"** run
  it once alone on the pristine base. If it passes there and fails on your
  branch, it is your branch's.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` for each tree
  you touch, once before pushing. A raised count is yours to fix.
- **Citations:** after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root. It reads HEAD, not the working tree.
- **Decide, don't ask.** A small edit just outside your file list (a
  compiler-forced arm, a call site's new field) is made and named in the
  report. A `Question:` is only for what truly blocks your acceptance.
