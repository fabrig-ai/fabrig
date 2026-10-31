+++
id = "stage-forge-worlds"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "world-kit", "counts", "abort-setup"]
files = ["skein/testing/skein-throwaway-forgejo/**", "skein/tests/forge/conformance/**", "skein/tests/forge/setup/src/closing.rs"]
effort = "high"
minutes = 45
+++

## Goal

The throwaway Forgejo's HTTP host, the forge conformance's hosted component and forge-setup's closing world run their io stage through `skein_io::stage`, so every one of them
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
  - `skein/testing/skein-throwaway-forgejo/src/http/host.rs`;
  - `skein/tests/forge/conformance/src/component/hosted.rs` (two);
  - `skein/tests/forge/setup/src/closing.rs` (its `pump` loop).

## Owns and may change

- Owns `testing/skein-throwaway-forgejo/`, `tests/forge/conformance/`, and in `tests/forge/setup/` only `src/closing.rs`.
- The throwaway Forgejo and the forge conformance talk to a real Forgejo; their tests that need one are not in the default profile: run what it runs, and report what it skipped.

## Steps

1. Move each site (notes/migrations.md, the io stage), the narrowest test
   of its package after each.
2. The world surfaces in the packages you own (notes/migrations.md).

3. `cargo fmt`; `heavy cargo clippy -p skein-throwaway-forgejo -p forgejo-conformance -p skein-forge-setup-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-throwaway-forgejo -p forgejo-conformance -p skein-forge-setup-world`; the dependents' checks
   (`cargo tree -i <package> --workspace -e normal,dev --depth 1` finds
   them).
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each site: moved, or kept and why.
