+++
id = "stage-ui"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/crates/skein-ui-text/src/terminal.rs", "skein/examples/ui/server/src/main.rs", "skein/tests/ui-web/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# The UI's io stages on the shared stage

## Goal

Draft, 3.3: the text UI's terminal, the UI example's server and the
ui-web test peer write io's stage by hand; they call `skein_io::stage`
instead, by `notes/stage.md`. Two of them interleave their own work
between steps, so the defaults decide.

Acceptance:

- Each of `skein/crates/skein-ui-text/src/terminal.rs:254`,
  `skein/examples/ui/server/src/main.rs:403` (in `impl Host for Web`,
  from 392) and `skein/tests/ui-web/src/harness_peer.rs:139` (`impl Host
  for Peer`, from 128) is moved, or left by a named default with the
  test that pins it. Expected: `harness_peer.rs` moves. `terminal.rs`
  takes completions before the ready list and handles its events after
  each step; `main.rs` interleaves one step at a time and handles its
  events after each: `stage.md`'s interleaving and per-step-events
  defaults decide each.
- `io_resume` falls by the sites moved.
- The hosts' `next_deadline`s (`main.rs:442`, `harness_peer.rs:168`)
  follow `stage.md` item 2; the terminal's inherent `next_deadline`
  (`terminal.rs:198`) stays.
- `heavy cargo nextest run -p skein-ui-text -p skein-ui-text-tests -p
  skein-ui-example-server -p skein-ui-web-tests` passes (counts named);
  clippy clean.
- If `terminal.rs` changes: `heavy cargo check --tests -p
  temper-client-shell-terminal -p jig-client-world` (its other trees'
  readers) passes, and `heavy cargo nextest run -p
  temper-client-shell-terminal` passes.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/design/ui-text.md` (the terminal) for the words.
- `skein_io::stage` on your base.

## Owns and may change

- The files in the header, `skein/scripts/old-forms.txt`. In
  skein-ui-text, only `terminal.rs`.
- The terminal's pop loops use what they pop: leave them.

## Steps

1. Each site, item 1 and the defaults; deadlines, item 2.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`; for each site left, the default and the test.
