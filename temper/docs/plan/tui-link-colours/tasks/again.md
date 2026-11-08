+++
id = "again"
part = "again"
kind = "feature"
needs = ["binary-tui", "tui-colours", "joint"]
files = ["temper/tools/journeys/src/tui_again.rs", "temper/tools/journeys/src/tui_again/tests.rs", "temper/docs/design/client/tui/journeys/**", "temper/docs/design/client/README.status.md", "temper/docs/design/client/tui/journeys.status.md"]
effort = "high"
minutes = 90
+++

# tui-link-colours: `sign-in-approved` recorded again, and the plan's status made whole

## Goal

With the whole plan merged, record the TUI's journey `sign-in-approved` again with
`temper-client-journeys again`, the TUI in its own runtime directory so that it finds no
engine's link file and still walks the browser's approval, and with no `COLORFGBG`, so
that it asks the terminal its background and draws dark; reprint; and delete the status
entries the earlier tasks left for this one.

Acceptance:

- `tui_again.rs`, as `temper/docs/design/client/tui/journeys.md`, 8.2: the TUI's
  environment has `XDG_RUNTIME_DIR` as the scratch directory's `tui-runtime`, made
  `0o700` with the others, while the box keeps `runtime`; and no `COLORFGBG`, even when
  `again`'s own environment has one. Rows in
  `tui_again/tests.rs`, `tui_environment_keeps_the_opener_inside_the_scratch_directory`
  or beside it: the TUI's `XDG_RUNTIME_DIR` is `tui-runtime`, never the box's; a given
  `COLORFGBG` is dropped. A failing test first.
- `heavy cargo run -p temper-client-journeys -- again sign-in-approved` (generation, not a
  walk: `notes/common.md`, the first rule, on retries), its result added by `again`
  itself: the new `recording` holds `local` among its start's facts, then `read` and
  `found none` before its first `approval-ask`, `screen 24 80 dark`, the walk's keys and a
  `browse` before its approval, and no `noted`; `01.txt` still the approval's frame and
  `02.txt` the first screen; `01.html`, `02.html`, `theme.css` reprinted. The journey's
  `README.md` and `walk` change only if a mark's words did, and the report says which.
- The reprint (`heavy cargo run -p temper-client-journeys`) leaves the browser's journeys
  byte for byte as they are.
- Status, the entries left for this task, as stated departures from the implementer's
  text:
  - `temper/docs/design/client/README.status.md`: delete the **Not yet** entries "6.13
    and 6.14, *Signed in from the engine's link* and *Expired is not signed out*, in the
    host: ...", "6.15, *The terminal's own colours* and *Its glyphs and its focus*: ..."
    and "6.11, 6.12 and 6.15, in the binary: ...", and, once no entry of this plan's is left there, the intro
    paragraph's added words "but for the **Not yet** entries of the sign-in from the
    engine's link and the terminal's own colours below", ending its sentence at "themes
    and closing." If another of this plan's entries is still there, its task didn't land
    it: leave it and say so.
  - `temper/docs/design/client/tui/journeys.status.md`: delete the **Not yet** entries
    "6.4: ..." and "8.2 and 9: ...", and the intro's added words "but the **Not yet**
    entries below", leaving "No known implementation gap."; any other entry of this
    plan's still there stays, and the report says so.

## Inputs

- `temper/docs/design/client/tui/journeys.md`: 3.2 (the table, *The start's facts*), 6.1
  and 6.2, 8.1 and 8.2 (all of it), 9.
- `temper/docs/design/client/README.md`: 6.12 (*The engine's link, kept at rest*, *Where*)
  and 6.15 (*The scheme*).
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 7.

## Owns and may change

- Owns the files in `files`. The browser's `again` (`again.rs`) and journeys are another
  owner's: leave them.
- If `again` stops on a moving screen three times, each on a fresh fake box, widen the
  mark's wait in the recording driver (`tui_again.rs`) and say so.
- If the recording shows the TUI signed in from a file (no approval), the TUI found the
  box's: that is a red of this task's environment, not a new journey.
- If `again` needs Chromium or a build the builder lacks, report `Failed:` with what it
  said; never record by hand.

## Steps

1. The environment's rows; `heavy cargo nextest run -p temper-client-journeys -E
   'test(/tui_again/)'`; see them fail; the change.
2. `again sign-in-approved`; check the recording's lines above with `grep`; the reprint.
3. The status entries.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-journeys --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-journeys -p temper-client-shell-tui`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
5. Commit, push.

## Report

As the implementer's text says; how many tries `again` took, the recording's first lines
up to its first `approval-ask` (no secret is in them: the recording is redacted), and
the status entries deleted or left.
