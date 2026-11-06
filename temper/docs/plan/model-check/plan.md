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

## Spend

One trial is one short turn, at the person's cost as for a chat: on a plan its included usage; on a key one short turn at
the key's price. It runs when an account first becomes usable in a session on the welcome or `ConnectModel`, and at each
"Try again". Nothing else runs one: not a restart, a reload, a page, `temper ctl` or a chat. A person's next way through a
page after a restart may run one again: they've signed in again, and that's the cause. A try is cut off at a minute, or at
its first ending, whichever is first, so jig's retries spend nothing.

## Parts and tasks

| Part | Task | Tree | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| `person-task` | `person-task` | jig | | high, 120 | a person's task in no project |
| `trial-charter` | `trial-charter` | temper | | medium, 45 | charter 10 |
| `trial-machine` | `trial-machine` | temper | | high, 75 | the child, a row per account |
| `trial-wire` | `trial-wire` | temper | | medium, 50 | the request, the document |
| `trial-root` | `trial-root` | temper | those four | high, 150 | the root runs one |
| `trial-facts` | `trial-facts` | temper | `trial-wire` | high, 90 | the client keeps the rows |
| `pane-feed` | `pane-feed` | temper | `trial-facts` | high, 110 | Model's line reads them |
| `account-row` | `account-row` | temper | `pane-feed` | medium, 50 | the account's row |
| `home-line` | `home-line` | temper | `pane-feed` | medium, 55 | Home's line |
| `trial-start` | `trial-start` | temper | `trial-root`, `trial-facts`, `pane-feed` | high, 120 | the meeting point, and its fallout |
| `trial-worlds` | `trial-worlds` | temper | `trial-start`, `account-row`, `home-line` | high, 110 | the request's stories, joined |
| `trial-close` | `trial-close` | temper | `trial-worlds` | medium, 40 | the design says what stands |

Slice 1 has no walk: its parts run no story a person can see. Slice 2 walks on a real box, once, with a good account.

## Critical path

`person-task`, `trial-root`, `trial-start`, `trial-worlds`, `trial-close`: ~540 implementer-minutes, then the trunk's end check.
Beside it, `trial-wire`, `trial-facts`, `pane-feed`: ~250, to `trial-start`.

## Taking `look-round-12`

Its pane lands first, by catch-up: its trunk holds `ready-pane` and `pane-proof`, and it's `priority`. `pane-feed` and the
tasks after it build on it. Each starts by checking its base, and if it isn't there merges `look-round-12/integration` as its
first commit, pre-approved by `notes/client.md`: no slot waits on another plan's landing. Reading the `model_check` function
it left is the join, and no rule or node order of the look moves here.

## Overlaps that couldn't be avoided

- `temper-client-protocol-wire/schema/**`, its goldens and `temper-service/src/family.rs` (`trial-wire`):
  `sign-in-new-tab/return-goes`, merged on its trunk, removes a request. Hand-merged schema, regenerated goldens.
- `temper/docs/design/draft/model-check.status.md` (the design's push): `look-round-12` rewrites its 3 and 4; this plan
  changes its 2 alone.
- `temper/docs/design/STATUS.md`: `chat-reading-c/reading-close`, a line in its list.
- `temper-client-view/src/pages/model_check.rs`, `pages/home.rs`, `pages/accounts/rows.rs`, `words.rs`, `limits.rs`, `accounts.rs`,
  `choice.rs` (`pane-feed`, `account-row`, `home-line`): `look-round-12`, `ui-walk-3/fix-flow`, `chat-reading-c`. Small hunks, by
  `notes/common.md`.
- `temper-client-domain/src/forge.rs`, `route.rs` (`trial-facts`, `trial-start`): `sign-in-new-tab`, `look-round-12`. A line each.
- `temper/tests/client/**`, `temper/crates/temper/tests/**` (`trial-start`): `sign-in-new-tab`, `test-speed/walk-waits`. Expectations
  and a helper's call only.
- The gallery's `names.txt`, `samples.rs` and `tests.rs` (`account-row`, `home-line`): `look-round-12`, `ui-walk-3/fix-look`,
  `chat-reading-c/reading-close`. Regenerated on a merge.
- `temper/docs/design/client/README.md`, `browser.md`, `mockups/STATUS.md` (`trial-close`, last): `chat-reading-c/reading-close`.

## Decided against the draft

Named again in its status by `trial-close`:

- **"First committed text" is the first committed turn:** the root reads no turn's bytes, and a trial's run has no tools.
- **A restart cancels a live trial** rather than letting it run again: "Leaving" keeps it, a restart doesn't. The request's test holds.
- **A second account of a later model** may not be what answers: the trial names no model, so it takes what a new chat takes. The row
  says which it took; that account's row holds the line, and the new one's holds nothing.
- **A deployment's accounts aren't tried:** the document holding them is the accounts page's alone, and the trigger reads the
  person's. Named in `temper/docs/design/domain/trial.md`, 8.
- **Who starts it:** the client, not the engine: the engine sees no "became usable", and `temper ctl` must not spend.

## Dropped

Nothing of the request.
