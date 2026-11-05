+++
id = "classify-close"
part = "classifiers"
kind = "feature"
needs = ["classify-people", "classify-tasks", "classify-core", "classify-ops"]
files = ["jig/crates/jig-domain-core/src/routing/party/people.rs", "jig/crates/jig-domain-core/src/start/links.rs", "jig/docs/development/debt/D015-exhaustive-lists.md"]
effort = "medium"
minutes = 20
+++

# jig: the core's lists of people's answers, and the item goes

## Goal

Pay `jig/docs/development/debt/D015-exhaustive-lists.md`. The core's two lists of `people::Outcome` read `Outcome::family()`, the figures are read once more, and the item goes, saying what stays.

Acceptance:

- `routing/party/people.rs`'s and `start/links.rs`'s `people::Outcome` or-lists match `family()`'s answer where it answers their question. A list whose arm needs a variant's own field stays, named in your report.
- `heavy cargo nextest run -p jig-domain-core --lib` and `heavy cargo nextest run -p jig-core-world --test suite` pass, with no assertion changed.
- The net production lines are at most 0. The item is deleted.
- Your report gives the **Measure** on people, on `jig-domain-core/src`, on `jig-domain-tasks/src` and on `jig/examples/ops`, the targets each task met, and what stays (the reasons are in the `classify-*` specs): routing lists, single-use lists, and `reads/*`, whose `tasks::Stored` lists are each a different set. If you find one question asked in two places that no task took, name it in your report rather than taking it.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/classifiers.md`: the rule, the run. **This task departs from its "pin first":** `classify-people`'s table pins `family()`.
- `jig/docs/development/debt/D015-exhaustive-lists.md`

## Owns and may change

- **Owns:** the files in `files`. Your citations check confirms nothing else cites the item. The item says it lowers D016 and D017: their files don't change.

## Steps

1. The two lists.
2. The figures.
3. Delete the item. The note's run. Commit and push.

## Report

The note's items, with the figures above.
