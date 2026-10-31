+++
id = "ss-forge"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/testing/skein-throwaway-forgejo/**", "skein/tests/forge/conformance/**", "skein/tests/forge/setup/**"]
effort = "high"
minutes = 40
+++

## Goal

The throwaway Forgejo's HTTP host and the forge conformance and setup worlds run their io stage on `skein_io::stage`. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `testing/skein-throwaway-forgejo/src/http/host.rs` (one).
- `tests/forge/conformance/src/component/hosted.rs` (two).
- `tests/forge/setup/src/closing.rs` (one).

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): the throwaway's `Host` impl and the conformance world's `Hosted` (`tests/forge/conformance/src/component/hosted.rs`).

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/testing/skein-throwaway-forgejo/**`, `skein/tests/forge/conformance/**`, `skein/tests/forge/setup/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - A test binary that needs what the gate's profiles do not give (a real browser, a Forgejo binary, a tier profile of `tools/tiers/run`) is built with `heavy cargo nextest run -p PKG --no-run`, and the report names it as not run.
  - No probe against a real forge: the conformance suite's tests that need a running Forgejo are built, not run.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-throwaway-forgejo -p forgejo-conformance -p skein-forge-setup-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-throwaway-forgejo -p forgejo-conformance -p skein-forge-setup-world`. Commit and push.

## Report

As `notes/stage-sites.md` says.
