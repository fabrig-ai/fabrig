+++
id = "ss-ui"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/crates/skein-ui-text/**", "skein/examples/ui/server/**", "skein/tests/ui-web/**"]
effort = "high"
minutes = 45
+++

## Goal

The text terminal's host, the UI example's server and the ui-web world's peer run their io stage on `skein_io::stage`. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `crates/skein-ui-text/src/terminal.rs` (one: its loop reaps completions before the ready list and handles events after each step; see "Order" and "Between steps" in `notes/stage-sites.md`).
- `examples/ui/server/src/main.rs` (one).
- `tests/ui-web/src/harness_peer.rs` (one).

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): the `Host` impls of `examples/ui/server` and `tests/ui-web`.

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/crates/skein-ui-text/**`, `skein/examples/ui/server/**`, `skein/tests/ui-web/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - `skein/docs/design/ui-text.md` describes the terminal's loop: if its order is the design's own (not io.md 2's), keep it and say so.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-ui-text -p skein-ui-example-server -p skein-ui-web-tests --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-ui-text -p skein-ui-example-server -p skein-ui-web-tests`. Commit and push.

## Report

As `notes/stage-sites.md` says.
