+++
id = "ss-io"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/tests/io/**"]
effort = "high"
minutes = 60
+++

## Goal

io's own world drives io through `skein_io::stage` in its hosts, and its referees and hosts move onto the world surfaces. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `tests/io/src/append.rs`, `src/output.rs`, `src/processes.rs`, `src/usage.rs`, `src/world.rs` (one each).
- `tests/io/tests/suite/memory.rs` (one) and `tests/suite/process_memory.rs` (two): these meter io's steps; see "One step alone" in `notes/stage-sites.md`. A meter per `resume` stays; a host loop that only drives io moves.

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): the nine referees and nine `Host` impls in `tests/io/src`.

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/tests/io/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - `tests/io/tests/suite/stage.rs` (task `io-stage`) stays as it is.
  - The review notes tests/io runs two world frameworks and six file owners retype one Host skeleton (theme 4): consolidating them is a later run's.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-io-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-io-world`. Commit and push.

## Report

As `notes/stage-sites.md` says.
