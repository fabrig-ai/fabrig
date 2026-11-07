# smith-gate-entries: the plan

From `inputs/request.md`: `tools/tiers/budgets.toml` gains `[gate.smith.default]` at 30 and `[gate.smith.fuzzy]` at 120, with one comment line, and nothing else changes. This was judged from reading the code on 2026-11-07. A cut builds nothing, so no test ran; `budgets()` was run on a temporary copy of the file with the two additions, and it accepted them.

## Parts and tasks

| Part | Task | Needs | Effort | Minutes |
|---|---|---|---|---|
| `gate-entries` | `gate-entries` | | medium | 5 |

One task, and **nothing is dropped.** No question is open.

**Critical path:** `gate-entries`, about 5 implementer-minutes.

## What was decided

- **"No parts" means no split.** A spec has to name a part, so the one task has one part, in one slice, with `walk = false`.
- **"Low effort" is `medium`.** A spec's effort is `high` or `medium`, and `medium` is the one for mechanical work.
- **No design is written.** The request leaves `smith/docs/design/testing.md` and the strategy as they are. The part cites `tools/tiers/design.md`, 7.3, which already describes this table.
- **The task's check also reads the real file.** The request says `tools/tiers/test_cpu.py` validates the file's shape, but it writes its own fixtures, and no test reads `tools/tiers/budgets.toml`. A passing run says the validator works, not that this file is valid. The spec adds one command that runs the same `budgets()` on the real file and compares what it prints. `tools/tiers/run --verify-all` would also read it, but that builds and is outside a plan's runs.
- **The diff is compared with the trunk.** The trunk holds this plan's files and main doesn't, so a diff against main would name them too.

## Readers

No reader needs an edit:

- `budgets()` in `tools/tiers/cpu.py` is the only reader in this repository, and it already accepts any `[gate.<tree>.<profile>]` table (`tools/tiers/test_cpu.py`, `test_gate_budget_validation`).
- `tools/tiers/design.md`, 7.3 names the table's form, with `[gate.hearth.default]` as an example, so it stays true.
- The comment `Live pairs and other default/fuzzy suites have no entry yet` stays true of the others.
- No Markdown read by tests names the file (`.config/gate/read-by-tests.txt`).

## Where the request and the repository differ

The first two change nothing that the request asks for. The spec follows the request's shape and doesn't repeat its reason.

- **`wall_hard_s`.** The request says these tables don't take it. `tools/tiers/design.md`, 7.3 gives them "the same three keys and no `auto_reap`", and `budgets()` accepts all three. The entries still hold `cpu_suite_s` alone, as asked.
- **`test_cpu.py`** doesn't read the real file, as above.
- **Smith's figures against the queue's record: not confirmed.** The request says the record already reads smith's suites under 30 and 120. The cut could not check this: `fab report landing` gives whole `default` and `fuzzy` steps with no tree's share, and this repository's tools read nothing of the table. The request measures nothing, so this plan doesn't either. `tools/tiers/design.md`, 7.3 says whatever measures a share records a run over it rather than refusing it.

## Left as they are

The request limits the change to one file, so these stay. Whoever settles the open question will want them:

- `tools/tiers/design.md`, 7.3 says the file's tables "record the user's approved limits", and the file's first comment opens `The user's budgets`. The new comment says the figures stay open, which qualifies them for these entries but doesn't change their words.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1 still give 30 s and 120 s as idle-machine figures that stand until a tier's CPU budget is set, while the entries' key is `cpu_suite_s`.
- `smith/docs/design/testing.md` doesn't record the question: 6.1 only points at the strategy, and 8 doesn't list it. Hearth's has a section for it (`hearth/docs/design/domain/testing.md`, 11), so for smith the new comment is the only place that says it's open.

## Overlaps

None could be avoided, and none is expected. On 2026-11-07 no integration trunk had changed `tools/tiers/budgets.toml`, and no other plan's request names it. No draft beside this one lists the file, including `limits-worlds` and `skein-limits-heavy`. The spec says what to do if another tree's entries land first.

`model-catalogue-2/smith-limits` and `limits-worlds/engine-causes` change smith's code and tests on their trunks. They share no file with this task, though they can move what these figures will later measure.
