+++
id = "journey-setup"
part = "journey-setup"
kind = "feature"
needs = []
files = ["temper/tools/journeys/src/tui_again.rs", "temper/tools/journeys/src/tui_again/**", "temper/tools/journeys/src/again.rs", "temper/tools/journeys/src/main.rs", "temper/docs/design/client/tui/journeys.status.md"]
effort = "high"
minutes = 120
+++

# tui-launchpad: a TUI journey's setup, walked in the browser before the TUI starts

## Goal

`temper-client-journeys again NAME` for a TUI journey walks the journey's `setup`, in
the browser's format, in the signed-in browser before it starts the TUI, so that a TUI
journey can open on a project with a model (`project-landing`'s steps) and reach a chat.
No journey is recorded by this task: the task `again` records the first one that has a
setup.

Acceptance:

- `tui_again::starting` reads, before any build or child, the walk and, where the
  journey's directory holds one, its `setup`, as the browser's `again::setup` reads one,
  its `after` read from the browser's journeys' directory (`again::setup` given the
  journey's own directory and the browser's for `after`, or a sibling of it in
  `again.rs`; the browser's callers keep their meaning). A TUI setup holding a `mark` is
  refused, naming its line; one whose `after` names no browser journey's setup is
  refused as the browser's is.
- `tui_again::again`'s walker: after the box's printed link is opened in Chromium, the
  setup's steps, then its `reload` where it has one, each as the browser's walker does
  them (its finds, its 60-second bound, its refusal naming the line and what it sought),
  then the TUI started and the walk followed as today. With no setup, the walker is as
  it is now, step for step.
- Rows, each a failing test first, in `temper/tools/journeys/src/tui_again/tests.rs`:
  `a_tui_walk_is_read_before_any_build_or_child` (or a test beside it) gains a journey
  with a `setup` `after` a browser journey in a scratch pair of directories, read whole
  with its parent's steps first; a setup with a `mark` refused naming its line; a setup
  whose `after` has no setup refused; a journey with no setup read as before.
- The walker's setup phase has no story of its own (`temper/docs/design/client/tui/journeys.md`,
  7: the first journey recorded with a setup proves it); it is compiled and its phases
  are reached by the unit rows above as far as they reach without a browser.
- `temper/docs/design/client/tui/journeys.status.md`: delete the **Not yet** entry "8.2:
  a TUI journey's `setup`, walked in the signed-in browser before the TUI starts, and
  7's rows for it." and nothing else (a stated departure from the implementer's text).

## Inputs

- `temper/docs/design/client/tui/journeys.md`: 7 (*The walk, natively*), 8.1, 8.2 (*A
  setup*).
- `temper/docs/design/client/journeys.md`: 8.2 (*A setup*: the format, `after`, the
  bounds).
- `temper/docs/plan/tui-launchpad/notes/common.md`.

## Owns and may change

- Owns `tools/journeys`' TUI `again` (`src/tui_again.rs` and `src/tui_again/`) for this
  wave; in `src/again.rs`, only what makes its setup reader take a directory for the
  journey and one for `after`, or share its setup stepping with the TUI's walker; in
  `src/main.rs`, only the line that passes the browser's directory to the TUI's
  `starting`, if its signature needs it.
- If sharing the browser walker's setup stepping would change the browser walker's
  behaviour, copy the stepping into the TUI's walker instead and say so.
- No journey's files change: `sign-in-approved` has no setup, and its walk stands.
- `temper-client-shell-web`'s `journey::Setup` and `journey::setup` are read as they
  are; if a TUI setup must be refused for a `mark`, refuse it in `tools/journeys`, not in
  the web shell.

## Steps

1. The rows; `heavy cargo nextest run -p temper-client-journeys --lib -E
   'test(/tui_again/)'`; see them fail.
2. The reader, then the walker's setup phase.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-journeys --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-journeys`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen. Say that no journey was
recorded.
