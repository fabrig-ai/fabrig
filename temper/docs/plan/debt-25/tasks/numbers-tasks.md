+++
id = "numbers-tasks"
part = "numbers"
kind = "feature"
needs = []
files = ["jig/tests/tasks/**", "jig/testing/jig-tasks-kit/**"]
effort = "medium"
minutes = 40
+++

# The tasks world names its party once

## Goal

Jig D035's tasks row: 1 of the Measure's 37, but the widest repeat. `PERSON` is written eleven times, with values 9, 1 and 7. `Party::Person(9)` appears 142 times in 21 files. `response` and `done` have four copies each.

Acceptance:

- The kit holds `PERSON: u64 = 9`, `OWNER` and `MEMBER` if their copies agree, and a `person()` or a `const` for `Party::Person(PERSON)`. The files whose `PERSON` is 9 read it, and `Party::Person(9)` reads the kit's name throughout.
- **The other values stay,** under their own names: `tests/open/resume.rs` and `tests/suite/llm.rs` (1), and `tests/suite/open.rs` (7). `placed.rs`'s `PERSON` is an `Executor`, which isn't this number's copy: give it a name that says so.
- `rg -c 'Party::Person\(9\)' jig/tests/tasks` reads 0.
- **`close_project.rs`:** `RUN` replaces 10, also in `[10]`. The spends are `MEMBER_SPEND`, `DIRECT_SPEND` and `RUN_SPEND`, and the assertion of 35 reads as their sum.
- **Acts:** `response` and `done` move to the kit, or a suite support module, where their bodies are the same. `refused` and `step` follow if theirs are.
- The world's Measure lines read 0. The test lines fall by about 40. Expected values don't change.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`
- `jig/docs/design/domain/tasks.md`, for the verbs' names

## Owns and may change

- **Owns:** the files in the header.
- **`jig/testing/jig-tasks-kit`'s other readers** read a name added to it and nothing else changes. Keep existing names as they are.

## Steps

1. The kit's names and acts, then the sweep, one file group at a time, then `close_project.rs`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-tasks-kit -p jig-tasks-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-tasks-world -p jig-tasks-kit`. Commit and push.

## Report

The common note's items; net test lines; each `PERSON` left, and its name. These are renames and moves: no rule to break by hand.
