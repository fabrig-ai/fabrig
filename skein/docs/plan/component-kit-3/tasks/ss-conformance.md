+++
id = "ss-conformance"
part = "stage-sites"
kind = "migration"
needs = ["io-stage"]
files = ["skein/crates/skein-sim/src/tests/**", "skein/tests/conformance/real/**"]
effort = "high"
minutes = 40
+++

## Goal

The simulator's given-listener test and the real-kernel conformance suite drive io through `skein_io::stage` where they drive its stage by hand. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `crates/skein-sim/src/tests/given_listener.rs` (one).
- `tests/conformance/real/src/bin/signal_sender_fixture.rs` (one), `tests/suite/adopted_listener.rs` (two), `tests/suite/given_listener.rs` (one), `tests/support/deadline.rs` (one), `tests/support/given_listener.rs` (one).

No world surfaces here: these crates hold no referee or host of their own.

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs`.

## Owns and may change

- Owns `skein/crates/skein-sim/src/tests/**`, `skein/tests/conformance/real/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - A test binary that needs what the gate's profiles do not give (a real browser, a Forgejo binary, a tier profile of `tools/tiers/run`) is built with `heavy cargo nextest run -p PKG --no-run`, and the report names it as not run.
  - The conformance suite checks the simulator against the real kernel step by step: a site whose assertion sits between two single io steps keeps them ("One step alone" in `notes/stage-sites.md`).

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-sim -p skein-conformance-real --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-sim -p skein-conformance-real`. Commit and push.

## Report

As `notes/stage-sites.md` says.
