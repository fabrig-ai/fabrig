+++
id = "core-answers"
part = "conformance"
kind = "feature"
needs = ["core-view"]
files = ["jig/testing/jig-conformance/src/core/peers.rs", "jig/testing/jig-conformance/Cargo.toml", "jig/tests/conformance/src/lib.rs", "jig/tests/ops/src/conformance/mod.rs", "jig/testing/jig-core-kit/src/protocol.rs", "jig/tests/ops/src/protocol/calls.rs"]
effort = "medium"
minutes = 25
+++

# The kit answers the grant and the settled call

## Goal

Jig D030's last peers. `peers::account` already answers the accounts' shared requests, but `Testing` and `Ops` each copy its grant by hand. `jig-core-kit/src/protocol.rs` and `jig/tests/ops/src/protocol/calls.rs` render a settled answer in two copies that differ only in `to`.

Acceptance:

- The hand copies in `Testing` and `Ops` call `peers::account`, each keeping its own validity (60 s and 7200 s).
- One render of a settled answer, taking `to`, lives in `jig_conformance::core`, and both callers read it, if it can read smith-free tools as `jig/tests/boundary/src/tools.rs` allows. If the manifest would have to name a crate that links smith, leave the render where it is and say so: the kit must not link smith (`jig/AGENTS.md`).
- No test changes. Net lines fall by about 10.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`
- `jig/docs/development/debt/D030-conformance-not-drop-in.md`
- `jig/AGENTS.md`

## Owns and may change

- **Owns:** the files in the header.
- **Decoding, the first period and `Drained` stay with the adopters:** `notes/contracts.md` says why.

## Steps

1. The grant, then the render.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-conformance -p jig-conformance-world -p jig-ops-world -p jig-core-kit --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-conformance -p jig-conformance-world -p jig-ops-world -p jig-core-kit -p jig-core-world`, and the boundary world if the manifest changed (`rg -n '^name' jig/tests/boundary/Cargo.toml`). Commit and push.

## Report

The common note's items; whether the render moved. This is a move: no rule to break by hand.
