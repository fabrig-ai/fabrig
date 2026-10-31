+++
id = "stage-ui"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts"]
files = ["skein/crates/skein-ui-text/src/terminal.rs", "skein/examples/ui/server/**", "skein/tests/ui-web/**"]
effort = "high"
minutes = 45
+++

## Goal

skein-ui-text's terminal, the UI example's server and the ui-web world's harness peer run their io stage through `skein_io::stage`, so every one of them
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
  - `skein/crates/skein-ui-text/src/terminal.rs` (production: the terminal host);
  - `skein/examples/ui/server/src/main.rs`;
  - `skein/tests/ui-web/src/harness_peer.rs`.

## Owns and may change

- Owns `crates/skein-ui-text/src/terminal.rs` (nothing else in skein-ui-text), `examples/ui/server/`, `tests/ui-web/`.
- skein-ui-text has two terminal backends (linux and darwin); move both if both hold the stage.

## Steps

1. Move each site (notes/migrations.md, the io stage), the narrowest test
   of its package after each.
2. The world surfaces in the packages you own (notes/migrations.md).

3. `cargo fmt`; `heavy cargo clippy -p skein-ui-text -p skein-ui-example-server -p skein-ui-web-tests --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-ui-text -p skein-ui-example-server -p skein-ui-web-tests`; the dependents' checks
   (`cargo tree -i <package> --workspace -e normal,dev --depth 1` finds
   them).
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each site: moved, or kept and why.
