+++
id = "lower-stories"
part = "lower"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/tests/suite/root_routes/added_dependencies.rs"]
effort = "medium"
minutes = 40
+++

# Two behaviours get their lower story first

## Goal

Two real-loop fuzzy stories can go once what only they hold has a story in the world that owns it (`temper/docs/design/domain/testing.md`, 2.4; `plan.md`, *Belonging*). Write those two stories in the engine's domain world. `stories-go` removes the two real-loop stories after this lands.

| Story that goes | What only it holds | Its lower story |
|---|---|---|
| `fuzzy::case_6_1` | One delegation of four resolves two batch-local references into the fourth's dependencies, and not the third's | `one_batch_of_four_resolves_its_local_references_and_gates_only_the_fourth` |
| `fuzzy::case_6_2` | A delegate kept after finishing keeps its journal as a prefix, and its next attempt, after Met, is given the turns it committed | `a_kept_delegate_resumes_with_its_committed_turns_and_its_journal_whole` |

Acceptance:

- Both tests are in `temper/tests/engine/domain/tests/suite/root_routes/added_dependencies.rs`, focused, each within the in-memory world test's guide (`skein/docs/foundation/testing-strategy.md`, 8).
- Each bites: break the rule by hand once, run it, see it fail for the rule's reason, and restore it. The report names the rule and the failure.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(added_dependencies)'` passes, with the count checked.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`.
- jig's `jig/docs/design/domain/tasks.md`, 5.7 and 7, on added dependencies and keeping a task.
- `temper/docs/design/domain/testing.md`, 5.3's rows 6.1 and 6.2, for what the two stories promised.
- `temper/tests/real-loop/src/case/oracle.rs`, `step_6_1` (its dependency-fact check) and `step_6_2` (its saved-journal and kept-report checks): read what they assert, and hold the same.
- The file's own `both_prerequisite_orders_and_completion_orders_gate_only_the_fourth_delegate` and `a_live_delegate_is_kept_until_both_added_dependencies_are_met`: your models. `root_routes.rs`'s `a_plan_of_spikes_a_choice_and_changes_runs_in_dependency_order` uses `engine::Dependency::Batch`.

## Owns and may change

Owns that one test file. A small helper may be pulled out of the first model test for the settings both share, in the same file. No product code changes.

Sketches. The bodies are yours:

```rust
#[test]
fn one_batch_of_four_resolves_its_local_references_and_gates_only_the_fourth() {
    // The model test's settings, and one call_batch of four report delegates:
    // the fourth with [engine::Dependency::Batch(0), engine::Dependency::Batch(1)].
    // The fourth's saved dependencies are the first two's numbers, and not the third's.
    // It has no assignment, and waits on both. The first three are assigned.
    // Finish one: still none. Finish the other: one, with the third still running.
}

#[test]
fn a_kept_delegate_resumes_with_its_committed_turns_and_its_journal_whole() {
    // As the second model: amend a live delegate to depend on a sibling. It commits a
    // recognisable turn (protocol::turn), then finishes, and is kept. Save its milestone
    // rows from the store. The sibling finishes, and Met starts a next attempt.
    // That Assignment's `transcript` holds the committed turn's bytes, and each saved
    // milestone row is still there, unchanged, before the new ones.
}
```

Options, each decided:

- **The fixture's charter doesn't resume:** set `policy.resume` as this suite's other resume tests do. Don't change a default.
- **A kept report's `finish` call:** on the engine's side it's a committed turn's bytes, so a recognisable turn is the same check. The real-loop story's mutation of the fake's history stays with that story.
- **One of the two can't be held here:** write the other, and your report's first lines after `Done:` say which behaviour stays unheld and why. `stories-go` then keeps that story. Don't add product code to hold it.

## Steps

1. Read the oracle's checks and the two model tests.
2. Write the two tests; make each bite.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`; the run in the acceptance; commit and push.

## Report

What each test holds, the rule you broke for each and what it printed, and your implementation and check minutes apart.
