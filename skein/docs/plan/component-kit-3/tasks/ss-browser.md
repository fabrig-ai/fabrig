+++
id = "ss-browser"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/testing/skein-fake-person-browser/**", "skein/testing/skein-lab-browser/**", "skein/tests/browser/**"]
effort = "high"
minutes = 50
+++

## Goal

The browser harnesses' io stages run on `skein_io::stage`. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `testing/skein-fake-person-browser/src/harness.rs`, `src/harness/capacity.rs`, `src/harness/peers/tests.rs` (one each).
- `testing/skein-lab-browser/src/runtime.rs` and `tests/support/mod.rs` (one each).
- `tests/browser/tests/support/real.rs` (two).

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): fake-person-browser's `Host` impl.

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/testing/skein-fake-person-browser/**`, `skein/testing/skein-lab-browser/**`, `skein/tests/browser/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - A test binary that needs what the gate's profiles do not give (a real browser, a Forgejo binary, a tier profile of `tools/tiers/run`) is built with `heavy cargo nextest run -p PKG --no-run`, and the report names it as not run.
  - The review counts the Chromium host loop three times across these crates (theme 4); consolidating it is a later run's, not this task's.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-fake-person-browser -p skein-lab-browser -p skein-browser-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-person-browser -p skein-lab-browser -p skein-browser-world`. Commit and push.

## Report

As `notes/stage-sites.md` says.
