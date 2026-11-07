# smith-gate-comment: smith's gate entries carry the user's word

The request: `inputs/request.md`. `tools/tiers/budgets.toml`'s comment on `[gate.smith.default]` (30) and `[gate.smith.fuzzy]` (120) says whether those are their figures stays open; the user confirmed on 11-07 that they are, as CPU seconds. One comment line changes, and nothing else.

## Parts and tasks

| Part | Task | Needs | What it makes run | Effort, minutes |
|---|---|---|---|---:|
| `comment` | `tasks/comment.md` | | `budgets.toml` line 5 is the request's new line; `test_cpu.py` passes and the real file validates | medium, 10 |

**Critical path:** `comment`, about 10 minutes. No build.

## Design

None written: the request allows no doc change. The task cites `tools/tiers/design.md`, 7.3, for context.

## What was decided

- **One part, one task.** The request says "no parts"; fab needs every task in a part, and every part in a slice, so the plan has one of each. Nothing is split.
- **Effort `medium`.** The request asks for low effort. fab takes only `high` or `medium` (`fab status` refuses `low`), so the task uses `medium`, the lower one.
- **The check is `test_cpu.py`**, as the request says, plus `cpu.py`'s `budgets()` run on the real file: `test_cpu.py` reads only fixture copies it writes itself, so the one-liner is what shows the real file still parses.
- **The other trees' comments stay** (Skein's, Hearth's and Jig's still say "stays open"). The user's word was given for Smith's entries alone.
- **The design stays behind the file.** `tools/tiers/design.md`, 7.3 and 8 and `tools/tiers/design.status.md` still say Smith's CPU figures are open. The request forbids any doc change here, so this plan leaves them. The task tells the implementer to leave them too and not to list them as left. A later plan closes them.

## Overlaps

None. No draft beside this one names `tools/tiers/budgets.toml`. `bundle-list/list` (merged) touched `tools/tiers/run`, `test_run.py` and `design.status.md`, none of which this plan changes.
