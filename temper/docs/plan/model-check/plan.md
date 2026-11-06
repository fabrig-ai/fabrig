# model-check: the plan

From `inputs/request.md` and `temper/docs/design/draft/model-check.md`: once an account is usable, one real turn through
the path a chat takes, in the background, shown in the side pane. The trunk is `model-check/integration`.

## What the cut decided

- **The trial is a task in project 0:** no project exists on the welcome's Model step. jig gains "a person's task in no
  project" (`jig/docs/design/domain/authority.md`, 6.1): the person's pool pays, so it selects as a chat of theirs does;
  its creator is the deployment, so no list holds it.
- **Its state is the engine's, in memory:** a child, `temper-engine-domain-trials`, a row per account
  (`temper/docs/design/domain/trial.md`, 3). No store record: the draft says a trial's end is no fact of the account.
- **The client starts it:** on seeing an account newly usable on the welcome or `ConnectModel`, and on "Try again". The
  engine sees no "became usable", and only a person's act may spend.
- **A restart spends nothing:** the root cancels live trials before a run can be claimed, and nothing starts another. The
  cost, named in the design's Later: a failed trial's line is lost at a restart.
- **A committed turn is the first committed text:** the run has no tools, and the root reads no turn's bytes.
- **A failed run ends the trial at its first ending,** cancelling the task, so jig's retries spend nothing.

## Parts and tasks so far

| Part | Task | Tree | Needs | Effort, minutes |
|---|---|---|---|---|
| `trial-charter` | `trial-charter` | temper | | medium, 45 |
| `trial-machine` | `trial-machine` | temper | | high, 75 |
| `trial-wire` | `trial-wire` | temper | | medium, 50 |
| `person-task` | `person-task` | jig | | high, 120 |

More follow: the root's part, the client's facts, the pane, the accounts row, Home, the worlds and the walk.

## Critical path, as it stands

`person-task`, then `trial-root` (to come).

## Overlaps that couldn't be avoided

- `temper-client-protocol-wire/schema/**`, its goldens and `temper-service/src/family.rs` (`trial-wire`):
  `sign-in-new-tab/return-goes`, merged on its trunk, removes a request. Hand-merged schema, regenerated goldens.
- `temper/docs/design/draft/model-check.status.md` (the design's push): `look-round-12` rewrites its 3 and 4; this plan
  changes its 2 alone.
- `temper/docs/design/STATUS.md`: `chat-reading-c/reading-close`, a line in its list.
