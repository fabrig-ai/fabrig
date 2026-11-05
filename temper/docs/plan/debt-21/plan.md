# debt-21: the twenty-first debt round

Six independent parts (`inputs/request.md`): smith's shell-cwd grader; jig's D004 record; skein D009, D020; jig D017; hearth D009. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| request | `shell-cwd-grader` | `grader-worst-case` | | high | 60 | `worst_case` back; a bundle's proof, or "not validated" in the status |
| request | `d004-record` | `d004-record` | | medium | 5 | one line of a debt record |

Still to come: skein D009, skein D020, jig D017, hearth D009.

## What was decided

- **`shell-cwd` stays at version 3.** `smith/docs/design/benchmarks.md`, 5.2 changes a task's version with its grader, so that results of different versions are never compared. No result exists at 3: version 3 came on 2026-11-04 and no bundle has run it. A bump would say something changed since a run that never happened.
- **The rebuild is timed.** The frozen source is in neither this repository nor the old mirror, so a digest match is a search among rewritten commits and may not exist. `grader-worst-case` has 40 minutes, then records "not validated".

## Critical path

So far, `grader-worst-case`, 60 minutes.
