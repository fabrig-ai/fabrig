+++
id = "ss-llm-tls"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "ws-shell"]
files = ["skein/tests/llm-connection/**", "skein/tests/tls/**"]
effort = "high"
minutes = 45
+++

## Goal

The llm-connection and TLS worlds' hosts run their io stage on `skein_io::stage`, and their `enum Process` dispatchers become `Dispatch`. (`skein/docs/design/draft/component-kit.md`, 3.3 and 3.5.) No
behaviour changes but those `notes/stage-sites.md` names: every site now
checks the kernel's room before each io step.

The io stage sites:

- `tests/llm-connection/src/simulated.rs` (one).
- `tests/tls/src/native.rs` (one) and `tests/tls/tests/suite/fake_peer.rs` (one).

In the same crates, onto the world surfaces (`notes/stage-sites.md`, "The world surfaces"): each crate's referee, its `Host` impls, and its `enum Process` dispatcher (`tests/llm-connection/src/simulated.rs`, `tests/tls/tests/suite/fake_peer.rs`).

Acceptance: each site above calls `skein_io::stage` or is named in the
report as left, with the note's reason; the counts it touches fall
(`notes/counts.md`); clippy and the owned packages' tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/stage-sites.md` (how to move, the
  world surfaces, the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); as merged: `skein/crates/skein-io/src/stage.rs` and `skein/crates/skein-shell/src/drive.rs` and `skein/testing/skein-world/src/referee.rs`.

## Owns and may change

- Owns `skein/tests/llm-connection/**`, `skein/tests/tls/**`.
- Nothing outside them: no public item outside test support changes; a
  `testing/` crate's public items stay as they are.
- Pre-approved:
  - The sites named here, and any other io stage in the owned crates (`rg -n 'io::resume|io::fire' <dir>`), which the report names.
  - `skein-tls-world::pki` and `skein-llm-world` are used by other test crates (theme 6): no public item of these crates changes, so their users need no change.

## Steps

1. Move each io stage site, then the world surfaces, as the note says;
   remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-llm-connection-world -p skein-tls-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-llm-connection-world -p skein-tls-world`. Commit and push.

## Report

As `notes/stage-sites.md` says.
