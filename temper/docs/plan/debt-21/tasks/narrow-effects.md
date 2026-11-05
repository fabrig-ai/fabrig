+++
id = "narrow-effects"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/effects.rs"]
effort = "high"
minutes = 60
+++

# jig-domain-core: a completing effect's kind is settled once

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. `complete()` takes `Propose` out at its first match, and then `check`, `charge`, `writers_ready`, `acceptance_room` and `complete_accept` each match `EffectOrigin` again. A private enum built at the one match carries it, and eleven sites go.

Acceptance:

- A private `enum Making { Projection, Call, Accept, Procedure }` holds each variant's fields, and is built at the one match in `complete()`. Those five functions take it.
- `EffectOrigin` is unchanged: `jig/examples/ops/domain/src/route.rs`, `temper/crates/temper-engine-domain/src/route/forge_route.rs` and `jig/testing/jig-test-domain/src/lib.rs` build its `Propose`.
- `rg -c 'unreachable!' jig/crates/jig-domain-core/src/effects.rs` reads 4: `:934, 968, 977`, which need a public variant's change, and the state's invariant.
- `heavy cargo nextest run -p jig-domain-core --lib` and `heavy cargo nextest run -p jig-core-world --test suite` pass with no assertion changed. The suite's `effects` and `effect_fences` pin the order of a refusal and a charge. `heavy cargo check -p jig-ops-domain -p temper-engine-domain --tests` passes.
- Net production lines: the cut's estimate is about +15.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/domain/connectors.md`, 4.1 and 4.5

## The sites

`effects.rs:616, 637, 652, 709, 720, 733, 744, 772, 936, 955, 1095`. At `:637`, `Projection` becomes an arm doing nothing.

## Owns and may change

- **Owns:** `effects.rs`. No other file changes.

## Steps

1. `Making`; `complete()`; each function in turn.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the count; any site that stayed, and why.
