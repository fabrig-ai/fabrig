+++
id = "trait-unused"
part = "conformance"
kind = "interface"
needs = []
files = ["jig/testing/jig-conformance/src/harness.rs", "jig/tests/conformance/src/**", "jig/tests/ops/src/conformance/**", "jig/tests/ops/src/joint/engine.rs", "temper/tests/conformance/src/lib.rs"]
effort = "high"
minutes = 40
+++

# `Application` drops what nothing reads

## Goal

Jig D030's first narrowing. `Key` is read only where it's declared. `Write` is `Durable::Write` again. `configuration_heap` is dead: temper's returns 0, which is the default. `Record` and `Input::Restore` serve one adopter, whose restore is already an event of its root.

Acceptance:

- `type Key`, `type Write`, `type Record` and `fn configuration_heap` leave `Application`, and the harness's `configuration_heap` field and sum go with them. `Output` reads `<A::Store as Durable>::Write`.
- `Input` loses `R` and `Restore`. `Ops` makes `Input::Event(root::Event::Store(root::Store::Restore(row)))` where it made `Input::Restore`, and the `Input::Restore` arms in `Testing` and temper's adapter go.
- The item's Measure reads 21 (`awk '/pub trait Application/,/^}/' jig/testing/jig-conformance/src/harness.rs | grep -cE '^\s+fn '`), and the trait has 7 types.
- No test's assertion changes. Net lines fall by about 35.

**Restored rows now go through `A::inbound`,** because the harness calls it on `Input::Event`. By reading it's a no-op for those rows: check `Ops::inbound` and the joint adapter's. The trace's line changes from `input Restore(` to `input Event(`, and `jig/tests/ops/tests/suite/trace.rs` still accepts it: this is pre-approved. Any other change to a test's events stops and asks.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`
- `jig/docs/development/debt/D030-conformance-not-drop-in.md`
- `jig/docs/design/domain/testing.md`, 5

## Owns and may change

- **Owns:** the files in the header. The `Input<` readers (`scenarios.rs`, `protocol.rs`, `funding.rs`, `scripts.rs`, `documents.rs` under ops, and `negative.rs`, `store.rs` under conformance) change in their type's spelling only.
- **temper:** in `temper/tests/conformance/src/lib.rs`, only the lines that name these items, by the plan's one crossing.

## Steps

1. The harness, then each adopter. Use `heavy cargo check -p jig-conformance-world -p jig-ops-world -p temper-conformance-world --tests` while errors last.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `(cd temper && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-conformance -p jig-conformance-world -p jig-ops-world -p temper-conformance-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-conformance -p jig-conformance-world -p jig-ops-world -p temper-conformance-world`. Commit and push.

## Report

The common note's items; the Measure. This removes what nothing reads: no rule to break by hand.
