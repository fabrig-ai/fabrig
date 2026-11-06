+++
id = "tasks-tuples"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/moving.rs", "jig/crates/jig-domain-tasks/src/control.rs", "jig/crates/jig-domain-tasks/src/proposals.rs", "jig/crates/jig-domain-tasks/src/funders.rs", "jig/crates/jig-domain-tasks/src/domain.rs"]
effort = "medium"
minutes = 20
+++

# jig-domain-tasks: four same-typed tuples get names

## Goal

Finding 5 in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. A tuple parameter of one type names nothing, and its call site can swap two values unseen.

Acceptance:

- These parameters are named structs, private or `pub(crate)` beside the function that takes them:

  | Function | Parameter now | Struct |
  |---|---|---|
  | `moving::apply` | `(person, period, pool_budget, period_budget)` | `Funding { person, period, pool_budget, period_budget }` |
  | `control::dependency_reached` | `(project, task, dependency)` | `Reaching { project, task, dependency }` |
  | `proposals::stalled` | `(proposer, number)` | `Stalled { proposer, number }` |
  | `funders::carve` | `(period, budget)` | `Carving { period, budget }` |

- `rg -n '\((person, period, pool_budget|project, task, dependency|proposer, number|period, budget)[^)]*\): \(' jig/crates/jig-domain-tasks/src` reads 0.
- **The other tuple parameters stay,** and so do the parcels the review lists: the request names these only. Clippy's threshold doesn't change.
- About +10 lines; no test changes.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 5

## Owns and may change

- **Owns:** the files in the header. A name that's already taken in its module gets a near one, and your report says so.

## Steps

1. The structs, one function at a time, unpacked at the callee's first line.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-tasks --lib`. Commit and push.

## Report

The common note's items. This is a move: no rule to break by hand.
