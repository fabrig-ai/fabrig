# jig-fuzzy-entry: the plan

From `inputs/request.md`: `tools/tiers/budgets.toml` gains `[gate.jig.fuzzy]` at 120, with one comment line, and nothing else changes. This was judged from reading the code on 2026-11-07, with the edit tried on a copy outside the repository: 4 lines added, none removed, and `budgets()` accepts the result.

## Parts and tasks

| Part | Task | Needs | Effort | Minutes |
|---|---|---|---|---|
| `fuzzy-entry` | `fuzzy-entry` | | medium | 5 |

One task, and **nothing is dropped.** No question is open.

**Critical path:** `fuzzy-entry`, about 5 implementer-minutes.

## What was decided

- **"No parts" means no split.** A spec has to name a part, so the one task has one part, in one slice, with `walk = false`.
- **"Low effort" is `medium`.** A spec's effort is `high` or `medium`, and `medium` is the one for mechanical work.
- **No design is written.** The request limits the change to the one file. The part cites `tools/tiers/design.md`, 7.3, which already describes the table.
- **The task's check also reads the real file.** The request names `tools/tiers/test_cpu.py` as the check, but that suite writes its own fixtures and never reads `tools/tiers/budgets.toml`. A passing run says the validator works, not that this file is valid, so the spec also calls `budgets()` on the real file. That runs nothing but the validator, within the request's "nothing is measured".
- **The acceptance's diff is against the trunk.** The plan's own files are on the trunk and not on main, so a diff against main would name them too.
- **The new lines follow Skein's.** The file's comments and gate tables run hearth, smith, skein, in the order they were added, so jig's comes last in each.

## Readers

No reader needs an edit:

- `budgets()` in `tools/tiers/cpu.py` is the only reader in this repository, and it already accepts any `[gate.<tree>.<profile>]` table (`tools/tiers/test_cpu.py`, `test_gate_budget_validation`).
- `tools/tiers/run` calls `budgets()` for its pairs, and a `gate` table is no pair (`tools/tiers/design.md`, 7.3).
- The comment `Live pairs and other default/fuzzy suites have no entry yet` stays true of the others, jig's focused suite among them.
- No Markdown read by tests names the file (`.config/gate/read-by-tests.txt`).

## Left as they are

The request limits the change to one file, so these stay:

- `tools/tiers/design.md`, 7.3 and 8, and `tools/tiers/design.status.md` name Smith's and Skein's entries and their open figures, not jig's. What they say stays true, and the new comment alone records that jig's figure is open. Whoever settles these figures will want to add jig there.
- Nothing in this repository measures a gate entry (`tools/tiers/design.md`, 8), so the entry makes no run red here on landing. Whatever measures a tree's share reads it from outside.
- The request's reading of jig's suite as well under 120 comes from the queue's record, which is outside this repository. This plan neither checks nor measures it.

## Overlaps

None could be avoided, and none is expected. On 2026-11-07 the trunks of `limits-fuzzy`, `model-catalogue-3`, `account-email`, `ui-forge-setup`, `skein-limits-rows` and `ui-agent-keys` held the same `tools/tiers/budgets.toml` as main, and no draft beside this one lists the file. `reference-walks`, `v0.1.0-ladder` and `configuring-walks` are queued and list no files yet. The spec says what to do if one of them changes the file first.
