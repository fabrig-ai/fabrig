+++
id = "trait-policy"
part = "conformance"
kind = "interface"
needs = ["trait-unused"]
files = ["jig/testing/jig-conformance/src/harness.rs", "jig/testing/jig-conformance/src/core.rs", "jig/tests/conformance/src/lib.rs", "jig/tests/ops/src/conformance/mod.rs", "jig/tests/ops/src/joint/engine.rs", "temper/tests/conformance/src/lib.rs", "temper/tests/conformance/src/scenarios.rs"]
effort = "high"
minutes = 35
+++

# The harness reads the policy itself

## Goal

Jig D030's second narrowing. Three adopters keep a generation counter so the harness knows when to read `policy`, though `Policy` is `Eq`, and the harness can compare values. `recovery` is a second hand table, asserted against the one in `Policy.kinds`, which already states the class (`jig/docs/design/domain/testing.md`, 5).

Acceptance:

- `fn policy_generation` and `fn recovery` leave `Application`.
- The harness refreshes its copy when `A::policy(peers)` differs from the one it holds. The harness's `policy_generation` field and its snapshot's go, along with the core observer's counter and accessor (`core.rs`) and temper's `Peers.policy_generation` and its two bumps.
- The `recovery` tables in each adopter go. `Policy.kinds`, and whatever each adopter builds it from, stays the one statement of each kind's class. Remove the `declared recovery class` assertion with them.
- The Measure reads 19.
- No test's other assertion changes. Net lines fall by about 70.

**A policy edit without a bump would now refresh,** where today it doesn't. By reading there's none: `core.rs`'s three bumps and temper's two cover every edit, and `Testing`'s and the joint adapter's run before start. If a test's events or outcome change, stop and ask.

**The cost:** the harness compares policies once per iteration. If `jig-conformance-world`'s or `jig-ops-world`'s fuzzy suite takes noticeably more CPU, compare only after an input that can change it, and say so in your report.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`
- `jig/docs/development/debt/D030-conformance-not-drop-in.md`
- `jig/docs/design/domain/testing.md`, 5

## Owns and may change

- **Owns:** the files in the header; in temper's, only the lines that name these items.
- **A kind in `Policy.kinds` that its fake reads from a table of its own** (`jig/tests/ops/src/conformance/observations.rs`, `temper/tests/conformance/src/observations.rs`): that table stays, and the policy keeps building from it.

## Steps

1. `policy_generation`, with a run of the packages below, then `recovery`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `(cd temper && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-conformance -p jig-conformance-world -p jig-ops-world -p temper-conformance-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-conformance -p jig-conformance-world -p jig-ops-world -p temper-conformance-world`, then the same with `--profile fuzzy`. Commit and push.

## Report

The common note's items; the Measure; fuzzy CPU before and after if you measured a change. The `recovery` assertion's removal is named here as this spec's one exception to "no assertion changes". The refresh is a rule: stop refreshing on a change, and name the test that fails, then restore.
