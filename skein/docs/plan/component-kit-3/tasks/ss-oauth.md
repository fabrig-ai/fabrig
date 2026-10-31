+++
id = "ss-oauth"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/tests/oauth-accounts/**"]
effort = "high"
minutes = 55
+++

## Goal

The oauth-accounts world's hosts run their io stage on `skein_io::stage`, and its dispatchers and referees move onto the world surfaces. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `tests/oauth-accounts/src/sign_in.rs`, `src/sign_in/browser.rs`, `src/world.rs` (one each).

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): its six referees, seven `Host` impls, its two `enum Process` dispatchers (`src/sign_in.rs`, `src/world.rs`) and the two wrappers (`src/private_cuts.rs`, `src/sign_in/cuts.rs`: each keeps its `Host` if it adds behaviour to a method).

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/tests/oauth-accounts/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - The review notes two owner adapters, two `Process` enums and two restart wrappers here (theme 4): merging them is a later run's; this task only moves them onto the surfaces.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-oauth-accounts-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-oauth-accounts-world`. Commit and push.

## Report

As `notes/stage-sites.md` says.
