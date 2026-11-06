+++
id = "figures-before"
part = "figures"
kind = "docs"
needs = []
files = ["temper/docs/plan/limits-worlds/figures/figures-before.md"]
effort = "medium"
minutes = 60
+++

# Where the worlds' stories spend their instructions

## Goal

The plan's *before*: a callgrind profile of the five stories below on the trunk as your slot has it, and one ranked list of causes, naming which task's crates each sits in. `engine-causes`, `real-loop-causes` and `public-causes` start from your list. These worlds share an engine, a store, a fake forge and a fake LLM, so the list's job is to show a cost that appears in several of them. This task changes no code.

| Package | Test (the `suite` binary's name) |
|---|---|
| `temper-real-loop-world` | `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` |
| `temper-real-loop-world` | `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` |
| `temper-real-loop-world` | `github_land::a_chats_change_lands_on_connected_github` (focused) |
| `temper-conformance-world` | `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` |
| `temper-one-process-world` | `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` (focused) |

Acceptance:

- `temper/docs/plan/limits-worlds/figures/figures-before.md` holds, in `notes/measuring.md`'s form, each story's instructions and native CPU seconds, its largest functions by inclusive and by self instructions, and for each real-loop story both runs.
- It holds a **self-instructions by crate** table, one row per crate holding at least 1% of any story, one column per story, by the symbol's first path segment.
- It holds **Causes, largest first**: each a cause you can name from the tree, not only a function (a repeated encoding, a scan, setup rebuilt per restart, a poll where an event would do, git run where a fixture would do, or what you find), where it is, its share of each story, and whose it is by `notes/common.md`'s table. A cause under 2% of every story isn't listed.
- `git diff` against your base holds that one file and nothing else.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`, `temper/docs/plan/limits-worlds/notes/measuring.md`.
- `temper/docs/plan/limits-worlds/inputs/request.md`, its second section.
- `skein/docs/foundation/testing-strategy.md`, 8 ("A slow test is understood, never hidden") and 8.1.

## Owns and may change

Owns the figures file. Everything else stays: the raised patience and the valgrind copy never reach a commit.

Options, each decided:

- **A story fails on the trunk, natively:** write it down with its log's path, profile the others, and put it under `## Left`.
- **A story can't finish under callgrind:** as `notes/measuring.md` says, keep the partial figure, marked, and go on.
- **The fork in `cuts-fork` lands while you run:** your slot's base stays as it is. Measure on that.
- **You see a cause and its fix looks one line:** don't make it. List it.

## Steps

1. Get valgrind, as `notes/measuring.md` says.
2. Run each story natively once for CPU seconds, then under callgrind: the real-loop stories twice each, with the raised patience.
3. Read each with `callgrind_annotate`, inclusive and self. Open the code behind the top functions until you can name the cause and not just the function.
4. Write the figures, restore `PATIENCE`, delete `target/limits-worlds/`, commit and push.

## Report

The stories' instructions and CPU seconds, the three largest causes, what ran under callgrind and how much slower, and your implementation and check minutes apart.
