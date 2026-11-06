+++
id = "typed-outside"
part = "replay"
kind = "feature"
needs = ["outside-derives", "core-answers"]
files = ["jig/tests/conformance/src/replay.rs", "jig/tests/conformance/src/lib.rs", "jig/tests/conformance/src/scenarios/queries.rs", "jig/tests/conformance/src/scenarios/lifecycle.rs", "jig/tests/conformance/src/restart.rs", "jig/tests/conformance/tests/suite/snapshot.rs", "jig/tests/conformance/tests/suite/fuzzy/snapshot.rs"]
effort = "high"
minutes = 40
+++

# A resumed cut is compared as typed state

## Goal

Jig D034's first clause. `replay::outside` formats the world with `{:?}`, and both snapshot tests compare the strings, so a mismatch says only that two long strings differ. `Outside` compares values and names the first difference.

Acceptance:

- `replay::outside` returns `Outside`, as `notes/contracts.md` has it. `first_difference` names the field and its first differing `{:#?}` line.
- The two snapshot tests share one `compare`, and both read `first_difference`, panicking with the scenario, cut, seed and that difference. **This is the spec's named exception to "assertions stay":** what they compare doesn't get looser, and `Secret` and `Buffer` now compare by value where `Debug` showed lengths.
- `busy_events` holds `Vec<String>`, and its one reader stays on `.len()` (`restart.rs`).
- Net test lines don't grow: about +40 for `Outside`, about -20 for the shared `compare`. Say how close you got.
- Put a one-field difference in by hand (change one field of one `Outside` before comparing) and see `first_difference` name that field. Restore.

**If typed equality finds a difference `Debug` hid,** a resumed cut really differs from a fresh run. That's a test whose outcome changes: stop and ask, with the field and the seed. Don't exclude the field.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`, `Outside`
- `jig/docs/development/debt/D034-world-replay-cost.md`

## Owns and may change

- **Owns:** the files in the header.
- **A missing derive in a kit under `jig/testing`** is one line, allowed. A step crate's stops and asks.
- **The world's own scenario states,** `State` in `jig/tests/conformance/src/scenarios/queries.rs` and `lifecycle.rs`, take `PartialEq, Eq` on their derive line and nothing else.

## Steps

1. `Outside` and `first_difference`, with `busy_events`, then the shared `compare`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-conformance-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-conformance-world -E 'test(snapshot)'`, then `heavy cargo nextest run -p jig-conformance-world --profile fuzzy -E 'test(snapshot)'`, then the package whole. Commit and push.

## Report

The common note's items; the field you changed and what `first_difference` said; net test lines.

## Answers

- 2026-11-06. May this task add `PartialEq, Eq` to `State` in `scenarios/queries.rs` and `scenarios/lifecycle.rs`, or should `outside-derives` supply them? This task adds them. Both files are now in its header, and both edits stay on the derive line. They are the conformance world's own types, not kits under `jig/testing`, so `outside-derives`, already merged, keeps its scope. `Neighbours` holds these states, so `Outside`'s acceptance can't hold without them. Their fields already have `Eq` (`reads::Cursor` and `people::Outcome` included), so no step crate changes. Exclude no field. The one-line `jig-test-store-files` dependency in `jig/tests/conformance/Cargo.toml`, and its `Cargo.lock` entry, stand. Continue from the pushed `debt-25/typed-outside`: compile, run the spec's checks and the by-hand difference, and compact the +7 test lines as far as you can. If typed equality then finds a difference, the spec's stop and ask still holds.
