+++
id = "ws-echo"
part = "world-surfaces"
kind = "migration"
needs = ["echo-first", "ws-shell"]
files = ["skein/examples/echo/service/**", "skein/tests/echo/**"]
effort = "high"
minutes = 30
+++

## Goal

The echo service's host and the echo world's referees, hosts and `enum Process` dispatcher move onto the world surfaces. Their io stage is task `echo-first`'s, merged. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- None: the io stage moved in `echo-first`.

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): the echo service's `Host` impl, `tests/echo`'s two referees and two `Host` impls, and its dispatcher (`tests/echo/tests/end_to_end.rs`).

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/examples/echo/service/**`, `skein/tests/echo/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-echo-service -p skein-echo-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-echo-service -p skein-echo-world`. Commit and push.

## Report

As `notes/stage-sites.md` says.
