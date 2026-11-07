# hearth-fuzzy-entry: the plan

From `inputs/request.md`: `tools/tiers/budgets.toml` gains `[gate.hearth.fuzzy]` at 120, with one comment line, and nothing else changes. This was judged from reading the code on 2026-11-06. A cut builds nothing, so no test ran; `budgets()` was run on a temporary copy of the file with the two additions, and it accepted them.

## Parts and tasks

| Part | Task | Needs | Effort | Minutes |
|---|---|---|---|---|
| `fuzzy-entry` | `fuzzy-entry` | | medium | 5 |

One task, and **nothing is dropped.** No question is open.

**Critical path:** `fuzzy-entry`, about 5 implementer-minutes.

## What was decided

- **"No parts" means no split.** A spec has to name a part, so the one task has one part, in one slice, with `walk = false`.
- **"Low effort" is `medium`.** `fab status` refuses `low`: a spec's effort is `high` or `medium`, and `medium` is the one for mechanical work.
- **No design is written.** The request leaves `hearth/docs/design/domain/testing.md`, 9.1 and 11, and `testing.status.md` as they are. The part cites `tools/tiers/design.md`, 7.3, which already describes this table.
- **The task's check also reads the real file.** `tools/tiers/test_cpu.py` writes its own fixtures, and no test reads `tools/tiers/budgets.toml`, so a passing run says the validator works, not that this file is valid. The spec adds one command that runs the same `budgets()` on the real file and compares what it prints. `tools/tiers/run --verify-all` would also read it, but that builds and is outside a plan's runs.

## Readers

No reader needs an edit:

- `budgets()` in `tools/tiers/cpu.py` is the only reader in this repository, and it already accepts any `[gate.<tree>.<profile>]` table (`tools/tiers/test_cpu.py`, `test_gate_budget_validation`).
- `tools/tiers/design.md`, 7.3 names the table's form, with `[gate.hearth.default]` as an example, so it stays true.
- The comment `Live pairs and other default/fuzzy suites have no entry yet` stays true of the others.
- No Markdown read by tests names the file (`.config/gate/read-by-tests.txt`).

## Left as they are

The request limits the change to one file, so these stay. Whoever settles the open question in `hearth/docs/design/domain/testing.md`, 11 will want them:

- `tools/tiers/design.md`, 7.3 says the file's tables "record the user's approved limits", and the file's first comment opens `The user's budgets`. The new comment says the figure stays open, which qualifies them for this entry but doesn't change their words.
- `hearth/docs/design/domain/testing.md`, 9.1 still gives the fuzzy suite's 120 as wall time, while the entry's key is `cpu_suite_s`. The new comment cites section 11 for this.

## Overlaps

None could be avoided, and none is expected. On 2026-11-06 no integration trunk of `limits-worlds`, `limits-fixtures` or `limits-heavy` had changed `tools/tiers/budgets.toml`, and none of their requests names it. No other draft beside this one lists the file. The spec says what to do if one lands first.
