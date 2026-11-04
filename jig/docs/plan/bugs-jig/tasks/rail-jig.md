+++
id = "rail-jig"
part = "charter-rail"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-charter/src/charter.rs", "jig/crates/jig-protocol-charter/src/boundary.rs", "jig/crates/jig-protocol-charter/src/tests.rs", "jig/docs/design/domain/hosts.status.md"]
effort = "medium"
minutes = 40
+++

# jig's charter grants modify only with deliver

## Goal

temper's design says a run that cannot deliver cannot write; jig's says the workspace tools are
jig's and a root decides none of them. temper holds the rail by narrowing the charter in its root
(`temper/crates/temper-engine-smith/src/start.rs`), and any other root has none. The rail moves
into jig's one translation (`jig/docs/design/domain/hosts.md`, 8), where `deliver` is decided.
This task is jig's half; `rail-temper` removes temper's line after it.

Acceptance:

- `jig_protocol_charter::charter` grants `tools.modify` only where the run's `grants.deliver` is
  some: a finite run with a change contract follows its policy; a report, a verdict and every Open
  run get `false`, whatever the policy.
- `Charter.workspace`, what `assignment` assembles, still says what the policy says.
- The new test fails on the old code on the report case, and passes.
- The tests of every reader below stay green (this changes what a shared item means).

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/domain/hosts.md`, 8 (the `assignment` paragraph and the tools table).
- The code: `jig/crates/jig-protocol-charter/src/charter.rs` (`charter`, `grants`),
  `src/tests.rs` (`assignment_fixture`, `workspace_tools_follow_every_combination_of_run_policy`,
  `contract_text_is_bounded_and_change_checks_are_required` for a change contract).

## Owns and may change

- `jig/crates/jig-protocol-charter/src/charter.rs`: bind `deliver` before `run::Charter`, and grant
  `modify: charter.workspace.modify && deliver.is_some()`.
- `jig/crates/jig-protocol-charter/src/boundary.rs`: `WorkspaceTools`'s doc comment may say that
  these are the rights the policy grants and that translation grants `modify` only with `deliver`.
- `jig/crates/jig-protocol-charter/src/tests.rs`: the expectation in
  `workspace_tools_follow_every_combination_of_run_policy` (its fixture is a report, so the
  translated `modify` is `false`), one new test.
- `jig/docs/design/domain/hosts.status.md`: this plan's **Not yet** entry for 8.
- Options: if a world's scripted run relied on `modify` under a report or an Open run, and the
  story is about a change, give that charter a change contract and say so in the report. If the
  story needs a run that writes without a delivery, stop and ask. Files of `jig/tests/**`,
  `jig/testing/**` and `jig/examples/**` may change only for this.
- `temper/crates/temper-engine-smith/src/start.rs` keeps its line: it is `rail-temper`'s.

## Steps

1. The test in `src/tests.rs`, `modify_needs_a_delivery`: with `policy.modify` true, and
   `assignment` then `charter`:

   | contract | `ended_by` | `grants.tools.modify` |
   |---|---|---|
   | `Report` (the fixture's) | `Result` | false |
   | `Verdict` | `Result` | false |
   | `Change { connector: 1, kind: 1, words: 64 }` | `Result` | true |
   | `Change { .. }` | `Requester` | false |

   and with `policy.modify` false, the `Change` and `Result` row is false. Each `assert_eq!`
   message names its row. See the first row fail, then make the change.
2. `heavy cargo nextest run -p jig-protocol-charter --lib -E 'test(=tests::modify_needs_a_delivery)'`,
   then `heavy cargo nextest run -p jig-protocol-charter -p jig-ops-domain -p jig-test-service -p jig-core-world -p jig-ops-world`.
3. Its temper readers, whose rail still stands: `heavy cargo nextest run -p temper-engine-smith -p temper-service`.
4. `cargo fmt`; old forms; `heavy cargo clippy -p jig-protocol-charter --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks, with how the test failed on the old code, and any world whose
charter changed.
