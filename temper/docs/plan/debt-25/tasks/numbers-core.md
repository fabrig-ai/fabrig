+++
id = "numbers-core"
part = "numbers"
kind = "feature"
needs = []
files = ["jig/tests/core/**", "jig/testing/jig-core-kit/src/fixtures.rs", "jig/testing/jig-core-kit/src/lib.rs"]
effort = "high"
minutes = 60
+++

# The core world's fixtures and acts live once

## Goal

Jig D035's core part. The core world has none of the Measure's 37, but it's the widest repeat of "each world names its fixtures once". `OWNER` is written seven times with the kit's `PARTY` value and comment. `CONNECTOR` is written five times, with none in the kit. `phase` is copied five times. A `core::Event::SignIn` literal stands at 24 sites, all `held: None`.

Acceptance:

- `OWNER` reads the kit's `PARTY`. Local `const OWNER: u64 = 1` goes, except where the file means something else by it. `CONNECTOR` is named once in `jig-core-kit/src/fixtures.rs`.
- `phase` (five copies) lives once, and `link`, `accepted`, `change` and `run` (two each) live once where their bodies are the same.
- `jig-core-kit/src/fixtures.rs` has a sign-in builder named for the verb, and the 24 `core::Event::SignIn` literals with `held: None` read it. A literal that sets a field differently keeps that field visible through the builder.
- `request_event` (`support/reads.rs`) goes if its body is `party_ask`'s (`fixtures.rs`).
- The seven same-named `ask`s stay: they're different verbs.
- The test lines fall by about 130. The builder is the least certain part of that: if it doesn't save lines at most sites, take its sites by hand and say so. Expected values don't change.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`
- `jig/docs/design/domain/testing.md`, 4

## Owns and may change

- **Owns:** the files in the header; in `jig-core-kit`, only those two files. `jig-core-kit/src/conformance.rs` is other tasks'.
- **Other readers of `jig-core-kit`** (the conformance, store, ops and client kits) read an added name and nothing else changes. Keep existing names as they are.
- **`projects.rs`'s `OWNER_SUBJECT` and the `OWNER_ROLE`s** are other numbers, and they stay.

## Steps

1. The kit's names and builder, then one group of `jig/tests/core` files at a time. Use `heavy cargo check -p jig-core-world --tests` while errors last.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-core-kit -p jig-core-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-core-world -p jig-core-kit`, then `heavy cargo check -p jig-conformance-world -p jig-ops-world --tests`. Commit and push.

## Report

The common note's items; net test lines; how many sign-in sites read the builder and how many stayed. These are moves: no rule to break by hand.
