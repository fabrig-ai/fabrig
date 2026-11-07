+++
id = "tui-goldens"
part = "tui"
kind = "feature"
needs = ["tui-view", "screen-render"]
files = [
  "temper/crates/temper-client-tui/src/tests/golden/**",
  "temper/crates/temper-client-tui/src/tests/goldens.rs",
  "temper/crates/temper-client-tui/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# temper-client-tui: the TUI's screens as golden frames

## Goal

The TUI's two screens, laid by skein's renderer in the TUI's theme, kept
as golden frames: the first screen at two sizes, dark and light, and the
sign-in screen. They fix the TUI's look for review and for the recording
that comes later, and they are the first test that runs the TUI's view and
skein's renderer together.

Acceptance:

1. Five golden files in `temper/crates/temper-client-tui/src/tests/golden/`,
   each one `skein_ui_screen::Frame::print` (`skein/docs/design/ui-screen.md`,
   section 4), written by hand from the design and the trees, never by a
   writer switch:

   | File | Screen | Scheme | Size (columns by rows) |
   |---|---|---|---|
   | `first-80x24-dark.txt` | first | `Dark` | 80 by 24 |
   | `first-80x24-light.txt` | first | `Light` | 80 by 24 |
   | `first-40x12-dark.txt` | first | `Dark` | 40 by 12 |
   | `first-40x12-light.txt` | first | `Light` | 40 by 12 |
   | `sign-in-80x24-dark.txt` | signing in | `Dark` | 80 by 24 |

   - **The first screen:** the fixture root live with `me`, the party
     "Owner", its projects "Hello" and "Demo"; `Tui::new(b"127.0.0.1:8367")`
     told `Link { link: Live }`; the focus `Focus::default()`.
   - **Signing in:** a root signed out (an unlinked root started with no
     `with`); the TUI told `Typed { value: b"http://127.0.0.1:9000/ji" }`
     and `Refused { refusal: OtherAddress { address: b"127.0.0.1:9000" } }`;
     the focus `Focus::default()`, so that the renderer honours the
     field's focus request (the caret at the value's end, the cursor
     shown, the value masked).
2. A table test, `src/tests/goldens.rs`, `the_tuis_screens_are_their_goldens`:
   each row builds the view (`temper_client_tui::build`), lays it with
   `skein_ui_screen::render(tree, &theme(scheme), size, &mut focus, &mut
   frame)` into `Frame::new(size)`, and compares `frame.print()` with the
   file byte for byte (`include_bytes!`), printing both on a mismatch.
3. What the goldens show agrees with
   `temper/docs/design/client/README.md`, 6.14 and 6.15: the framed top
   titled "temper" in its border, "Connected to 127.0.0.1:8367 as Owner"
   and "Projects: Hello · Demo", the bottom row " q to quit" from the
   start and "● Live" at the end, one column of padding; the sign-in
   screen's lines, its masked field and its refusal; each paint as
   `notes/tui.md`, section 4, names it. At 40 by 12 the first line fills
   the inner width (36 columns) exactly.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, section 2
  (`render`, `Frame`, `Focus`).
- `temper/docs/plan/tui-connected/notes/tui.md`, sections 2, 3 and 4:
  the trees, the words, the paints.
- `temper/docs/design/client/README.md`, 6.14 (the two screens'
  illustrations) and 6.15.
- `skein/docs/design/ui-screen.md`, section 4 (the print's form) and
  section 5 (how the renderer lays the bands, a framed region's title, a
  spread row, a field, a secret, the cursor).
- `temper-client-tui`'s own test fixtures (`src/tests/fixtures.rs`,
  `tui-view`'s): the roots and the view's words.

## Owns and may change

**Owned:** the `files` above: the goldens, their test module, and its `mod`
line in `src/tests.rs`.

**Readers and neighbours:** `src/tests/fixtures.rs` may gain a helper that
makes the first screen's root with the party "Owner" and the projects
"Hello" and "Demo" (or takes the names as arguments): a helper only, the
other tests untouched. No other file of the crate changes.

**Options, each with its default:**

- A golden that the renderer lays otherwise than you wrote it from the
  design: read which is right by `skein/docs/design/ui-screen.md`, section
  5. If the renderer departs from its design, keep the golden as the design
  says, leave that row failing, report it under `## Left` with the
  renderer's output, and say which section it departs from: it is
  `screen-render`'s to repair, not yours. If the TUI's tree departs from
  `notes/tui.md`, section 2, the same, as `tui-view`'s.
- The first screen's `me` at 40 by 12: if a line breaks there that the
  design says fits, check the inner width (40 less two border columns and
  two padding columns) before anything else.

## Steps

1. Write each golden by hand: the text from the trees and the words, the
   paints' letters in the order they first appear (section 4 of
   `ui-screen.md`), the paints' lines, the cursor's line.
2. The table test of acceptance 2. Bite: swap the dark and light themes
   in one row once, and drop the title from the border's paint once; see
   the rows fail on the paints' lines, and restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-tui --lib`. Commit ("temper-client-tui: the
   TUI's screens as golden frames"), push.

## Report

As the implementer's text says, and: any golden you left failing and why,
and the test's time.

## Answers

### 2026-11-07: Resume after screen-render exports `render`, or leave this task deferred? (tui-connected/tui-goldens-1)

Resume; do not defer and do not write a renderer here. `screen-render` has
since pushed its renderer: `origin/tui-connected/screen-render` at
`66473e90e2` ("skein-ui-screen: the renderer, regions placed by name and the
body kept on the focus") exports `pub use render::render;` from
`skein/crates/skein-ui-screen/src/lib.rs`. Merge that branch into yours
(`git merge --no-edit origin/tui-connected/screen-render`), keep the five
hand-written goldens of `e0aef6c9e1`, and run acceptance and the bites. A
row the renderer lays otherwise than the design is handled by this spec's
first option (kept as the design says, left failing, reported under
`## Left` with the renderer's output and the `ui-screen.md` section): it is
not a reason to ask again.
