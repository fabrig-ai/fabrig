# skein-gate-entries: the plan

From `inputs/request.md`: `tools/tiers/budgets.toml` gains `[gate.skein.default]` at 30 and `[gate.skein.fuzzy]` at 120, with one comment line, and nothing else changes. This was judged from reading the code on 2026-11-07. A cut builds nothing, so no test ran; `budgets()` was run on a temporary copy of the file with the two additions, and it accepted them.

## Parts and tasks

| Part | Task | Needs | Effort | Minutes |
|---|---|---|---|---|
| `gate-entries` | `gate-entries` | | medium | 5 |

One task, and **nothing is dropped.** No question is open.

**Critical path:** `gate-entries`, about 5 implementer-minutes.

## What was decided

- **"No parts" means no split.** A spec has to name a part, so the one task has one part, in one slice, with `walk = false`.
- **"Low effort" is `medium`.** A spec's effort is `high` or `medium`, and `medium` is the one for mechanical work.
- **No design is written.** The request limits the change to `tools/tiers/budgets.toml` and leaves the strategy as it is. The part cites `tools/tiers/design.md`, 7.3, which already describes this table.
- **The task's check also reads the real file.** The request says `tools/tiers/test_cpu.py` validates the file's shape, but it writes its own fixtures, and no test reads `tools/tiers/budgets.toml`. A passing run says the validator works, not that this file is valid. The spec adds one command that runs the same `budgets()` on the real file and compares what it prints. `tools/tiers/run --verify-all` would also read it, but that builds and is outside a plan's runs.
- **The diff is compared with the trunk.** The trunk holds this plan's files and main doesn't, so a diff against main would name them too.

## Readers

No reader needs an edit:

- `budgets()` in `tools/tiers/cpu.py` is the only reader in this repository, and it already accepts any `[gate.<tree>.<profile>]` table (`tools/tiers/test_cpu.py`, `test_gate_budget_validation`).
- `tools/tiers/design.md`, 7.3 names the table's form, with `[gate.hearth.default]` as an example, and its words on comments that distinguish provisional figures fit the new one.
- The comment `Live pairs and other default/fuzzy suites have no entry yet` stays true of jig's and temper's.
- `skein/docs/design/testing.md`, 7 says the 30 s and 120 s idle-machine figures stand until the user sets the suites' CPU budgets. The entries set no such budget, so it stays true.
- No Markdown read by tests names the file (`.config/gate/read-by-tests.txt`).

## Where the request and the repository differ

The first changes nothing that the request asks for; the spec only adds a check to it.

- **`test_cpu.py`** doesn't read the real file, as above.
- **Skein's figures against the queue's record: not confirmed.** The request says the record reads skein's suites under 30 and 120. The cut could not check this: `fab report landing` gives whole `default` and `fuzzy` steps with no tree's share, and this repository's tools read nothing of the table. The request measures nothing, so this plan doesn't either. `tools/tiers/design.md`, 7.3 says whatever measures a share records a run over it rather than refusing it.

The figures themselves are as the request cites them: `skein/docs/foundation/testing-strategy.md`, 8 gives 30 seconds and 2 minutes, and 8.1 calls them the focused and fuzzy suites' 30 s and 120 s idle-machine budgets.

## Left as they are

The request limits the change to one file, so these stay. Whoever settles the open question or closes this plan will want them:

- `tools/tiers/design.md`, 7.3 and 8, and `tools/tiers/design.status.md` name smith's gate entries and their open question, and not skein's. Commit `ac6159c6f6` wrote those lines when smith's plan closed. Nothing they say becomes false, but until the same is done for skein, the new comment is the only place that says skein's figures are open.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1 still give 30 s and 120 s as idle-machine figures that stand until a tier's CPU budget is set, while the entries' key is `cpu_suite_s`. The new comment names that gap and leaves it open.

## Overlaps

None could be avoided, and none is expected. On 2026-11-07 no integration trunk had changed `tools/tiers/budgets.toml` or `tools/tiers/design.md`, and no draft beside this one lists either file. The spec says what to do if another tree's entries land first.
