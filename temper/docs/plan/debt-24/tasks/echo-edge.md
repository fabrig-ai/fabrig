+++
id = "echo-edge"
part = "skein-kits"
kind = "migration"
needs = []
files = ["skein/testing/skein-echo-client/**", "skein/tests/echo/src/scenarios.rs", "skein/tests/echo/tests/end_to_end.rs", "skein/examples/echo/shell/Cargo.toml", "skein/examples/echo/shell/tests/end_to_end.rs", "Cargo.lock"]
effort = "medium"
minutes = 25
+++

# skein: the echo's shell stops reading its world

## Goal

Skein D028's edge from `skein-echo-shell` to `skein-echo-world`. The item's list doesn't cover it. Its end-to-end test belongs to the shell's package, so Cargo supplies its binary, but it lives in the world's directory and takes three things from the world, and it uses only `Proc::client`.

Acceptance:

- `client_limits` and `plan` move from `skein/tests/echo/src/scenarios.rs` to `skein-echo-client` (`skein/testing/skein-echo-client`, the fake echo client). `scenarios.rs` re-exports them, so `census.rs` and the world's suites stand.
- `skein/tests/echo/tests/end_to_end.rs` moves to `skein/examples/echo/shell/tests/end_to_end.rs`, and the manifest's `[[test]] path` goes; the binary's name stays `end_to_end`.
- `Process::Client` holds a small `Host` local to the test, in place of `Proc`, doing what `Proc::Client`'s arms do: about 30 lines. Nothing else in the test changes.
- `skein-echo-shell`'s manifest has no `skein-echo-world` line. `rg -n '^skein-echo-world\b' skein -g Cargo.toml` reads nothing.
- `heavy cargo nextest run -p skein-echo-shell -p skein-echo-world -p skein-echo-client` passes; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`
- `skein/docs/design/examples.md`, 7
- `skein/docs/foundation/testing-strategy.md`, 2.9

## Owns and may change

- **Owns:** the files in the header.
- **`skein-echo-client` is on `step`.** `client_limits` and `plan` hold that crate's subset; a scoped `#[expect]` where they trip it. If `plan` can't sit there without `std`, its builder stays in `scenarios.rs` and takes a smaller step: say so.
- **A dependency the local `Host` needs** goes in the shell's dev-dependencies: `skein-echo-client` and what it reads.

## Steps

1. The two functions; the move; the local `Host`.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-echo-client -p skein-echo-shell -p skein-echo-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, net lines. A move: no rule to break by hand.
