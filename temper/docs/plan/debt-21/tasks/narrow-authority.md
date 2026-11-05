+++
id = "narrow-authority"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-authority/src/check.rs"]
effort = "medium"
minutes = 20
+++

# jig-domain-authority: a source's ceiling comes with the source

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. `check.rs` iterates a literal array of sources, and then matches each to find its ceiling, aborting on those the array never held. The array holds the pairs.

Acceptance:

- `jig-domain-authority/src/check.rs:35, 268, 663, 684` iterate `[(Source::Project, &policy.ceiling), …]`, and `rg -c 'unreachable!' jig/crates/jig-domain-authority/src/check.rs` reads nothing.
- `heavy cargo nextest run -p jig-domain-authority --lib` and `heavy cargo nextest run -p jig-core-world --test suite` pass with no assertion changed.
- Net production lines don't rise.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/domain/authority.md`, 8 and 9

## Owns and may change

- **Owns:** `check.rs`. Nothing else changes, and the order of the sources stays, since a check's first refusal is what it answers.

## Steps

1. The four loops.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-authority --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the count.
