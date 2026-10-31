+++
id = "ss-fake-peers"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/crates/skein-fake-peers/**", "skein/tests/fake-peers/**"]
effort = "high"
minutes = 40
+++

## Goal

skein-fake-peers' shared io face runs on `skein_io::stage`, and the `#[path]`-included copy of its ordering test goes (`tests/io`'s stage test covers it). (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `crates/skein-fake-peers/src/face.rs`: `Face::up` and `Face::down` → `stage::up` and `stage::down` (its `room() < 2` checks go with them; `Face::down`'s `reclaim` stays the face's).
- `tests/fake-peers/tests/suite/stages.rs`: the `#[path = "../../../../crates/skein-fake-peers/src/face.rs"]` module and `the_face_tells_ready_results_before_completions_and_timers` go; the file goes if nothing else is in it, with its `mod` line.

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): the crate's three `Host` impls (their `next_deadline`).

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/crates/skein-fake-peers/**`, `skein/tests/fake-peers/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-fake-peers -p skein-fake-peers-tests --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-peers -p skein-fake-peers-tests`. Commit and push.

## Report

As `notes/stage-sites.md` says.
