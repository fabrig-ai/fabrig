# tui-box-polish: what every task holds

The plan carries out the user's five requests of 2026-11-09
(`temper/docs/plan/tui-box-polish/inputs/request.md`). The design is
`temper/docs/design/client/README.md`, 6.13 to 6.17, and skein's
`skein/docs/design/ui-screen.md`, sections 3 and 5.4. The shapes that
cross tasks are `temper/docs/plan/tui-box-polish/notes/contracts.md`.

- **Read** fabrig's `AGENTS.md` and `temper/AGENTS.md` (or
  `skein/AGENTS.md`), your spec's design sections, and this note.
- **Runs:** focused and fuzzy only, through `heavy`, with
  `TEMPER_BUNDLE=none`. No `tools/tiers/run`, no browser, process or
  live tier, no walk, render check or throwaway driver. The joint
  world, `temper-client-world`'s `tui` suite, is focused.
- **The browser's terminal is temper/ui's:** `temper-client-view`'s
  `shared` module and its words stay as they are. The TUI says its own
  words in `temper-client-tui`'s `words.rs`, under its owner.
- **Words:** a new `Word` goes at the end of the enum, of `ALL`, of
  `number()` (the next number) and of `english()`'s arms, in that
  order; `temper-client-tui`'s `tests/limits.rs` holds their bounds.
- **Goldens** (`temper-client-tui/src/tests/golden/*.txt`) are kept by
  hand: a mismatch prints the frame. Copy the printed frame only after
  checking that every difference is your change's, and say which
  goldens changed in the report. A new golden is a row of the existing
  golden test of its screen.
- **Journeys:** `temper-client-journeys`'s
  `tests::the_committed_journeys_are_their_print` checks the TUI's
  committed journeys (`temper/docs/design/client/tui/journeys/*/`). A
  change of frames is reprinted, generation, by `TEMPER_BUNDLE=none heavy
  cargo run -p temper-client-journeys`; a journey whose replay goes stale
  (a duty, a deadline or a key's event differs) is recorded again with
  `TEMPER_BUNDLE=none heavy cargo run -p temper-client-journeys -- again
  NAME`. Both are generation: run them, and name what changed.
- **Help pages** (`temper/docs/help/**`) are read by tests
  (`temper-docs`); only a spec that names one edits it.
- **Host tests** live in `temper-client-shell-tui/src/tests.rs`, one
  file several tasks of this plan extend: put your new test functions
  right after the function your spec names, so that merges stay apart.
- **Status entries:** each task removes its own **Not yet** entry, the
  one naming plan `tui-box-polish` and its parts, deleting only the
  entry's own lines, never a blank line around it; where an entry's
  part stays unbuilt, it narrows the entry instead and says why in the
  report. `docs/design/` is otherwise not yours.
- **Stored formats and the seam** may change freely before v0.1.0: no
  compatibility, no migration.
- **Before calling a focused or fuzzy failure outside your change,** run
  it once on the trunk's tip: if it passes there and fails on your
  branch, it is yours.
