+++
id = "connector-names"
part = "world-verbs"
kind = "feature"
needs = ["connector-kit"]
files = ["jig/testing/jig-test-connector-kit/src/**", "jig/tests/test-connector/tests/**", "jig/tests/test-connector/src/**"]
effort = "medium"
minutes = 35
+++

# jig's test-connector world: a path speaks of its story

## Goal

Jig D035's test-connector part, 42 of the Measure's 262: `path(82, 1)` and `kind(..)`, where 29 calls already pass `seed` and the rest repeat one of 13 literal seeds.

Acceptance:

- A file that uses one literal seed names it once, and its calls pass the name, so `path(82, 1)` becomes `path(SEED, DOC)`, or a file-local `doc()` where one row repeats: read each file, as seeds differ between them. `application_documents.rs` (25) and `stories.rs` (15) first.
- The Measure over `jig/tests/test-connector` reads at most eight, and each left is in the report with why. `path(seed, 1)` doesn't match it, and it needs no change.
- `heavy cargo nextest run -p jig-test-connector-world` passes, with no assertion changed. Tests' net lines don't rise.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header.
- **A `sed` is safe only per file,** after reading it.

## Steps

A file at a time. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-test-connector-world --all-targets -- -D warnings`; the test above. Commit and push.

## Report

The common note's items; the Measure before and after. No new rule: nothing to break by hand.
