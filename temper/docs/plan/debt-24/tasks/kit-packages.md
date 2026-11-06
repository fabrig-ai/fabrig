+++
id = "kit-packages"
part = "skein-kits"
kind = "interface"
needs = []
files = ["Cargo.toml", "Cargo.lock", "skein/testing/skein-json-kit/**", "skein/testing/skein-http-kit/**", "skein/testing/skein-tls-kit/**", "skein/testing/skein-llm-kit/**", "skein/testing/skein-forge-kit/**", "jig/testing/jig-test-connector-kit/**", "jig/testing/jig-core-kit/**", "jig/testing/jig-client-kit/**", "jig/testing/jig-engine-clients-kit/**", "jig/testing/jig-client-protocol-kit/**", "jig/testing/jig-protocol-accounts-kit/**", "jig/testing/jig-channel-kit/**"]
effort = "medium"
minutes = 20
+++

# Twelve empty kits

## Goal

Make every kit `skein/docs/design/testing.md`, 6 and `jig/docs/design/domain/testing.md`, 7 name, empty, so the tasks that fill them touch neither the root manifest nor each other's lines.

Acceptance:

- Five kits under `skein/testing/`: `skein-json-kit`, `skein-http-kit`, `skein-tls-kit`, `skein-llm-kit`, `skein-forge-kit`. Seven under `jig/testing/`: `jig-test-connector-kit`, `jig-core-kit`, `jig-client-kit`, `jig-engine-clients-kit`, `jig-client-protocol-kit`, `jig-protocol-accounts-kit`, `jig-channel-kit`.
- Each has a `Cargo.toml` with a one-line `description` from the design's list and the workspace's inherited fields as `skein/testing/skein-protocol-kit/Cargo.toml` has them, `publish` included; its `[lints]` set by `python3 -I -B tools/lints/lints.py --set harness <path>`; a `clippy.toml`, the `ordinary` copy (`tools/lints/design.md`, 4); and a `src/lib.rs` of one `//!` line.
- The root `Cargo.toml` lists each as a member beside its tree's kits, and in `[workspace.dependencies]`; `Cargo.lock` comes from `cargo metadata --offline --format-version 1 >/dev/null`.
- `python3 -I -B tools/lints/lints.py` passes, and `heavy cargo nextest run -p fabrig-boundary` and `-p jig-boundary` do.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `tools/lints/design.md`, 2.2, 4, 5

## Owns and may change

- **Owns:** the files in the header.
- **A boundary test that reads a list of packages** may gain a kit's line, if a test says so; report it.
- **A kit doesn't depend on anything yet.** `heavy cargo check` on an empty kit is enough.

## Steps

1. The packages; the root manifest; the lock.
2. `cargo fmt`; the lints tool; `heavy cargo clippy` naming the twelve `--all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items. No rule, so nothing to break by hand.
