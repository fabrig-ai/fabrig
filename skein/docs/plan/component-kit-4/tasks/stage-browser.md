+++
id = "stage-browser"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts"]
files = ["skein/testing/skein-fake-person-browser/**", "skein/testing/skein-lab-browser/**", "skein/tests/browser/**"]
effort = "high"
minutes = 60
+++

## Goal

The browser kits' hosts (the fake person's browser, the lab browser) and the browser world's real support run their io stage through `skein_io::stage`, so every one of them
checks the kernel's submission room before taking work
(`skein/docs/design/draft/component-kit.md, 3.3`; testing-strategy.md, 6),
and their worlds use the world surfaces where they forward or merge by hand
(3.5).

Acceptance:

- Each site below calls `skein_io::stage::up`/`down` instead of its own
  loops, or keeps `io::resume` for a reason `notes/migrations.md`
  allows, named in the report. The `io_resume` count falls by the sites
  moved.
- In the packages you own: `Host` impls whose `next_deadline` hand-merges
  a mechanism deadline with their policy deadline implement
  `next_mechanism_deadline` instead; trivial `Referee` overrides go.
- The tests of every package you own pass, and those of their direct
  dependents within skein check (`heavy cargo check --tests -p ...`).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and
  `notes/migrations.md`.
- `skein/crates/skein-io/src/stage.rs` (the contract);
  `skein/crates/skein-shell/src/drive.rs` (`Host`, `Forward`);
  `skein/testing/skein-world/src/referee.rs`.
- The sites (`rg -n 'io::resume'` in each path):
  - `skein/testing/skein-fake-person-browser/src/harness.rs`, `src/harness/capacity.rs`, `src/harness/peers/tests.rs`;
  - `skein/testing/skein-lab-browser/src/runtime.rs`, `tests/support/mod.rs`;
  - `skein/tests/browser/tests/support/real.rs` (two).

## Owns and may change

- Owns `testing/skein-fake-person-browser/`, `testing/skein-lab-browser/`, `tests/browser/`.
- The Chromium host loop is written three times across these (the review, theme 4): do not merge those copies here (a later run's); move only their io stage.

## Steps

1. Move each site (notes/migrations.md, the io stage), the narrowest test
   of its package after each.
2. The world surfaces in the packages you own (notes/migrations.md).
   The real-browser tests need a browser; run what the gate profile runs
   (`heavy cargo nextest run -p ...` as is) and report what it skipped.
3. `cargo fmt`; `heavy cargo clippy -p skein-fake-person-browser -p skein-lab-browser -p skein-browser-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-person-browser -p skein-lab-browser -p skein-browser-world`; the dependents' checks
   (`cargo tree -i <package> --workspace -e normal,dev --depth 1` finds
   them).
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each site: moved, or kept and why.
