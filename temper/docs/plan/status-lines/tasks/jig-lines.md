+++
id = "jig-lines"
part = "jig"
kind = "docs"
needs = []
files = [
  "jig/docs/design/domain/engine.status.md",
  "jig/docs/design/client/domain/contract.status.md",
  "jig/docs/design/client/domain/client.status.md",
  "jig/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 50
+++

# jig: the backfill's design gaps in four status files

## Goal

The debt backfill found six jig design gaps that the code shows and no status file holds
(`temper/docs/plan/status-lines/inputs/request.md`, jig 1 to 6). Write one entry for each, in
the file named, after checking the code. Docs only.

Acceptance:

- Each gap below that the code still shows has one entry in its file, in the form
  `notes/common.md` sets; each it no longer shows is under `## Dropped` in the report.
- No other line of the four files changes, but for their date lines (today's).
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.
- `git diff --stat origin/main` shows the four files alone.

## Inputs

- `temper/docs/plan/status-lines/notes/common.md` (the standing rules: form, labels, dates).
- `temper/docs/plan/status-lines/inputs/request.md`, section `jig`.
- `DESIGN.md`, 2. `jig/docs/design/examples.status.md` (the code's-side form).
- The design sections each gap is against, below, read for its label only.

## Owns and may change

- The four files of `files`, each only by the entries below and its date line.
- Nothing else. `jig/docs/design/STATUS.md` stays as it is: its "departs in a few single items"
  list already names `domain/engine.md` and `client/domain/client.md`, and no "have no gap"
  list names these documents.

Pre-approved:

- **Overlap with another draft:** `public-reading-build/public-status` changes
  `contract.status.md`'s last line and `engine.status.md`'s **Not yet** bullet. So in
  `contract.status.md` the new `- **Departs:**` bullet goes directly before the first
  `- **Not yet:**` bullet (after the date line's blank line), not after the last, so the two
  branches share no hunk; say so in the commit message. In `engine.status.md` the entries go at
  the end of the **Departs:** bullet, as the note says; touch nothing in the **Not yet:** bullet.
- If a gap's design section turns out to cover it already (the design has the text the request
  says it lacks), drop it and report it.
- If the code shows a gap in a different file than the request names, write it with the file
  the code has now and say so in the report.

## Steps

1. Read the note; `date -u +%F`.
2. For each gap, check the code, find its label, write the entry:
   1. **Wire fields the engine never fills** (jig 1). `jig-protocol-engine-clients/src/translate/refusals.rs`
      sends `None` for `LackingTaskKind`'s `executor`, `LackingDepth`'s and `LackingTasks`'
      `needed`, `LackingDeadline`'s `asked` and `latest`, a failed or cancelled ending's
      `reason`, and a decided ending's `party` and `how`. Two entries, one per file:
      - `contract.status.md` (labelled with `contract.md`'s sections that define those
        fields): the client wire carries them and the engine fills none, so a client always
        reads them empty; either the engine fills them or a client wire version drops them;
        no reason is recorded.
      - `engine.status.md` (labelled with `engine.md`'s section on the refusals and endings
        it gives): the same fact from the engine's side, citing `contract.status.md`'s entry
        by path rather than repeating it.
   2. **Over-ceiling limits capped silently** (jig 2), in `client.status.md`.
      `jig-client-protocol-link/src/limits.rs` caps its limits with `.min(w::CEILINGS....)`
      (count them: the request says 15); the entry says what `client.md` says of a limit over
      the wire's ceiling (or that it has no text on it), that the link caps it without a word,
      and that the backfill recommends refusing it. The code's side is
      `jig/docs/development/debt/D009-limits-mapped-by-hand.md`.
   3. **Public `Core` methods with no design text** (jig 3), in `engine.status.md`:
      `effect_owner` and `application_part_request` (`jig-domain-core/src/routing/flight.rs`),
      `call_room` (`runs/calls.rs`), and the test-only `next_finishing_deadline` and
      `finishing_due` (`routing/timers.rs`). Check each with `rg -n '<name>' jig/docs/design`:
      a method the design does name leaves the entry. Label: `engine.md`'s section on the
      core's interface.
   4. **The root fires one due core child per pass** (jig 4), in `engine.status.md`:
      `jig-domain-core/src/routing/timers.rs`'s `due` gives one `Timer` a call, and a root
      fires what it gives once a pass (`rg -n '\.due\(|finishing_due' jig temper --glob '*.rs'`
      finds the callers; the quality run's note,
      `jig/docs/development/debt/reviews/2026-10-31-jig-quality-run.md`, item 5, says the core
      world's `fire_due` and temper's `settle` had to loop for it). The entry says `engine.md`
      has no line on how many due children a pass fires, and what the code does.
   5. **The brief and notes worlds draw no seeds and check no heap** (jig 5), in
      `testing.status.md`, only if `jig/docs/design/domain/testing.md`, 3 asks seeds and a heap
      check of every child's world. Read section 3 and the two worlds (`rg -l 'brief|notes'
      jig/tests/*/Cargo.toml`); if 3 asks it of each child's world and they do not, write a
      **Departs:** entry labelled 3; if 3 does not ask it, drop it and report what 3 says.
   6. **One charter slot** (jig 6), in `engine.status.md`: with one charter slot, accepting
      proposed settings answers `ProposalDecided { Accepted }`, not `Full` (the quality run's
      section 6, "A deleted test leaves a behaviour question"). Read `engine.md`'s section on a
      proposal's acceptance against charter capacity, and the acceptance code
      (`rg -n 'ProposalDecided' jig/crates/jig-domain-core/src`). The entry says what the
      design says, what the code answers, and that whether a capacity check is missing or the
      expectation is wrong is the designer's decision (a bug, if the design asks `Full`).
3. Set the date lines; `git diff` shows only the entries and dates.
4. No build or test: `rg -n 'engine.status|contract.status|client.status|testing.status' jig --glob '*.rs'`
   must find nothing (if it finds a test, run it and report it).
5. Commit (one commit), then the citation check; fix your lines until it reports nothing new.
6. Push your branch.

jig 7's two stale texts are not this task's: `temper/docs/design/protocol/configuration.md`'s
`hello_bounds` is `temper-lines`', and `jig/docs/design/protocol/llm.md`, 5.6's moved-record
reader is gone from the text since commit `6532420160`.

## Report

As the implementer's text and the note say: each entry written (file, label, first words), each
dropped gap with what you found, the labels you chose where the design had no section on the
subject, and the citation check's output.
