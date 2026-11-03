+++
id = "playground-wasm32"
part = "composition"
kind = "feature"
needs = ["story-a"]
files = [
  "temper/crates/temper-playground/**",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# `temper-playground` builds for wasm32

## Goal

"`temper-playground` depends on product crates alone, on every target ... Its build for wasm32
proves it" (`temper/docs/design/playground.md`, 3). The crate, its stories with it, builds for
`wasm32-unknown-unknown`. The wasm32 builds in the gate, the worker's entry and the bundle are a
later plan's; this task makes the build pass, or names each crate of another tree that stops it.

Acceptance:

- `heavy cargo check --target wasm32-unknown-unknown -p temper-playground` passes, and `heavy
  cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings` too; or the
  report lists, for each crate outside temper's tree that fails, its first error (file, line, the
  item), and every temper crate in the path builds.
- Natively nothing changes: `heavy cargo nextest run -p temper-playground` passes with the
  trunk's count.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C1.
- `temper/docs/design/playground.md`, 3 ("A library on every target and a module on wasm32").
- `skein/docs/design/testing.md`, the portable crates' list (~489-506), read only.

## Owns and may change

- `temper-playground`: target gates (`#[cfg(target_arch = "wasm32")]` /
  `#[cfg(not(target_arch = "wasm32"))]`) and target-specific dependency tables, where an item is
  native-only. The composition's processes and documents stay one on both targets: nothing of the
  world's behaviour differs by target.
- A temper crate on the path (`temper/crates/*`) that fails for wasm32: a target gate on the
  native-only item, or a target-specific dependency line, each the least that builds, with no
  change on the native target. Before editing one, check whether another plan's task owns it now
  (`git log -3 origin/main -- <file>` and the trunk); if a change there would be more than a gate,
  stop and ask, naming the crate and the error.
- skein's, jig's and smith's crates are not yours: list them under `## Left`.

## Steps

1. `heavy cargo check --target wasm32-unknown-unknown -p temper-playground`; read the first errors
   by crate.
2. The gates in temper's crates, one crate at a time, each followed by the same check and by
   `heavy cargo check -p <crate> --tests` natively.
3. `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-playground -p <each crate
   edited> --all-targets -- -D warnings`; the focused tests of each crate edited. Commit; push.

## Report

As the implementer's text says, and: each crate gated and why, and each crate of another tree that
fails, with its first error.
