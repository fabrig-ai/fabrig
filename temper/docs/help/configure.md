# Change how work runs

Kind: walk

Change a deployment's settings while it runs: read them, write a settings file, apply or propose it, see that it took, move running work to it, and restore what was there.

## Before you start

You can also describe how you work to the configuring chat: temper help chat.

- **A running deployment** and `DIR` signed in to it (temper help box,
  steps 1 to 3). An owner of the deployment applies a change; a party
  with another role proposes it, and an owner decides (step 3).
- **jq** on `PATH`, to read and write the settings' JSON.
- **What to change:** this walk changes charter 1's instructions, the
  chat's (temper help charter). Every setting changes the same way:
  its kind's page names its fields (temper help settings).
- `NEWEST`: the newest revision, as the last `settings show` printed
  it. Every change names it, and each accepted change makes the next.
- `FILE`: the settings file you write, in a directory of yours.

### Deployment capacity

Deployment limits are startup configuration in `temper.json`, under
`engine.limits.overrides`. Stop and restart the deployment after editing
them (temper help deployment). Stored settings and task authority do
not raise these limits.

| Member under `engine.limits.overrides` | Bounds |
|---|---|
| `tasks.tasks` | Deployment-wide live tasks |
| `tasks.project_tasks` | Live tasks per project |
| `tasks.tree_tasks` | Lifetime tasks per tree, including its root |
| `tasks.depth` | Depth below a root at depth zero |
| `tasks.delegates` | Live direct delegates per task |
| `tasks.batch` | Tasks created atomically |
| `tasks.project_goals` | Live tracked goals per project |
| `documents.lines` | Lines per charter document read |
| `people.requests` | Retained deciding answers across the deployment |

Size live concurrency separately from lifetime tree capacity; ended
tasks do not return tree room. `people.inbox_tasks` must be at least
2 * `tasks.tasks`; `people.operations` must be at least
max(`tasks.delegates` + 1, `tasks.project_goals`) and at most
`tasks.tasks`. For example, `tasks.tasks: 64` needs
`people.inbox_tasks: 128`, while `tasks.delegates: 8` needs
`people.operations: 9` and enough live task capacity to hold it.

`people.requests` holds a day's deciding answers: 2,000 on small,
8,000 on standard, for 4 or 16 people × 20 daily conversations ×
(creation + three messages + decision) × fivefold headroom. Other
settings and deciding acts share it; pending decisions need room too.
Unexpired answers stay for retries under the same key. When full, ctl
reports the count and oldest expiry. Size it for more work or retention.

A refused start names the dependency and minimum. A scalar override
cannot raise a schema ceiling or profile store ceiling. Batch bytes,
`tasks.batch_bytes`, and inbox messages grow the stored task record;
reduce its capacity or spec, message, result and authority sizes if it
exceeds that ceiling. Check from a fresh configuration: an earlier
refusal can persist. For goal sizing, read temper help budget;
it promises no hundred-change profile.

### Coordinator tool families

A coordinator needs delegation (1), messages (2), control (4) and
reads (512). Control offers `change`'s `landing`, `producer`, `after`,
`llm` and `questions`; without it, words and repository use the default
landing and producer. S needs `landing: goal` and `producer: 7`.
The reference also uses decisions (8), subscriptions (32), notes (128)
and recall (256). Stock tools 1,543 include settings (1024) for the
configuring chat. Preserve the other family bits when adding control
with bitwise OR 4.

Check `ceiling.tools` in `rules`, and `ceiling.tools` plus
`roles[0].authority.tools` in `policy 1` for role 0. Another role uses
its own entry. Changing policy 1 affects only new projects;
existing projects need `project-policy` with those same policy fields
(step 2). Read and apply each value whole before starting the goal.

## 1. Read the settings

```
temper ctl DIR settings show
```

Expect: `newest revision NEWEST`, each setting's source, such as
`charter 1: default, last changed in revision 1`, held revisions and
pending proposals.

If not: exit 1 is ctl's side, such as `DIR` signed out (temper help
ctl).

## 2. Write a settings file from the value

Read a value whole, edit it, and set it whole.

```
temper ctl DIR settings show charter 1 --json | jq '{items: [{setting: .setting, set: (.value | .instructions = "Answer in one short paragraph.\n")}]}' > FILE
```

Expect: no output; `FILE` holds `{"items": [{"setting": {"kind":
"charter", "number": 1}, "set": {...}}]}`, the value whole with its
instructions changed.

If not: an empty `FILE` and stderr say why the read failed; read again.
A file holds at most 1024 items, one per setting: `set`, or `"edit":
"removed"` or `"edit": "reset"`.

Read temper's settings the same way:

```
temper ctl DIR settings show landing-rule 1 --json
temper ctl DIR settings show landing-rule 1 --project PROJECT --json
temper ctl DIR settings show forge-charter 1 --json
```

This file sets deployment rule 1 and charter 2 for new changes.
Choose an offered charter; keep the rule's other fields as read:

```json
{"items": [
  {"setting": {"kind": "landing-rule", "number": 1},
   "set": {"pattern": {"segments": ["666f726765"], "last": {"open": ""}},
           "enforce": true, "ci": true, "up_to_date": true,
           "gates": [{"number": 1, "blocking": true, "freshness": "clean"}],
           "approvals": []}},
  {"setting": {"kind": "forge-charter", "number": 1}, "set": 2}
]}
```

For a project's own policy, start from policy 1, the stock's starting
point, and set the whole policy for `PROJECT`:

```
temper ctl DIR settings show policy 1 --json | jq --argjson project PROJECT '{items: [{setting: {kind: "project-policy", project: $project}, set: .value}]}' > FILE
```

`project-policy` accepts only `set`, at most once per file, and may share a file with other settings;
no `number`, removed or reset edit. Edit its fields before sending it.
It immediately resizes the open period and parties' pools to its spends;
policy 1 changes only what new projects start with.

For example, accept project-policy `period_spend` 8200000 beside rules
`period_spend` 8300000 in one file, keeping every other field as read.
Then read `project show PROJECT`: the open period's budget is 8200000.

For a project's landing rule, add `"project": PROJECT` to its `setting`.
A proposal's temper items, with their whole values or their removed or
reset edits, are included in `proposal show PROPOSAL`.

## 3. Apply it, or propose it

The deployment's owners decide its items, a project's owners its
policy and landing rules. A file touching both needs one person who
owns both; its proposal waits for them. An owner applies it, naming the
newest revision read:

```
temper ctl DIR settings apply FILE --against NEWEST --key KEY
```

A party that is no owner proposes it, with a reason as `TEXT`, and
reads its proposal:

```
temper ctl DIR settings propose FILE TEXT --against NEWEST --key KEY
temper ctl DIR proposal show PROPOSAL
```

An owner of every scope the file touches then decides it, from the inbox:

```
temper ctl DIR inbox
temper ctl DIR proposal decide PROPOSAL accept --running stays --key KEY
```

Expect: `revision` and the new revision's number, which is `NEWEST`
from now on; or `proposed: PROPOSAL, to the deployment's owners` for this
charter. A project's items name `project PROJECT's owners`; a file touching
several scopes names their owners and says `one person must own every
affected scope`. Then expect
`proposal PROPOSAL · settings · against revision NEWEST, newest
NEWEST` and a line per item, `charter 1 · set · now {...} (default) ·
would be {...}`, each value whole; the owner's decision prints `done:
decide`, and prints it again when asked again under its key. A project's
policy line is `project-policy PROJECT · set · now {...} · would be
{...}`, each policy whole.

If not:

- `settings apply: stale; newest revision NEWEST; read settings show
  again` (exit 2): a change came first; go back to step 1 and decide
  again on what you read.
- `settings apply: settings unchanged` (exit 2): the file sets what is
  there already.
- `settings apply: out of shape; item 0; charter 1: budget` (exit 2):
  a value breaks a bound or a reference; the line names the item,
  counted from 0, its setting and its fault. Mend `FILE`, then a new
  key.
- `temper ctl: settings file item 1: item: exactly one of set or edit
  required` (exit 1): the file is out of form; the line names the item,
  counted from 1, and its rule.
- A pending proposal is withdrawn as stale once another change makes a
  new revision: read the settings again and propose again (temper help
  proposal).
- `decided first: by party 2, accepted, 14:02` (exit 0): the proposal
  was decided already, by the party it names (`you` when it was you),
  how and when; this decision changes nothing.

## 4. See that it took

```
temper ctl DIR settings show
temper ctl DIR revision show NEWEST
```

Expect: `newest revision NEWEST`, with `charter 1: changed in revision
NEWEST`; the revision's record then names who decided it and its item,
`charter 1: set`, with its value.

If not: `charter 1` still follows its default: the change waits as a
proposal for an owner; read `inbox`.

## 5. Move running work to it

A live task keeps its revision until it moves: `task show TASK` prints
`settings: revision R · newest NEWEST`; `settings show` lists held
revisions and their tasks. Move `REVISION`'s tasks at their next run:

```
temper ctl DIR revision advance REVISION --key KEY
temper ctl DIR task show TASK
```

Expect: `moved 1 task to revision NEWEST`, the count of tasks moved.
In `task show`'s output, the task's `settings:` line then reads
`settings: revision NEWEST`.

If not: a task whose charter the newest revision no longer offers with
its models stays on its revision, and the answer counts it among those
that could not move; it finishes there. A chat moves by itself at its
next run. To move running work as you apply, give `--running moves` in
step 3.

## 6. Restore what was there

To its default, reset the setting:

```
printf '{"items": [{"setting": {"kind": "charter", "number": 1}, "edit": "reset"}]}' > FILE
temper ctl DIR settings apply FILE --against NEWEST --key KEY
```

Expect: `revision` and the new revision's number. After a reset,
`settings show` lists `charter 1: default, last changed in revision
NEWEST`.

To restore earlier settings, page through history, newest first.
Each page ends with `revision list --before REVISION` to read on:

```
temper ctl DIR revision list
temper ctl DIR revision show REVISION
temper ctl DIR revision restore REVISION TEXT --key KEY
temper ctl DIR proposal show PROPOSAL
temper ctl DIR proposal decide PROPOSAL accept --running stays --key KEY
```

Use a new KEY for each act.

Expect: `proposed restoring revision REVISION as proposal PROPOSAL to
role K`, then `left out: SETTING` for
each deleted project's setting or policy left out. Read the proposal's
items before accepting it. The acceptance makes a new revision; running
work stays on its revision.

If not:

- `temper ctl: revision R is forgotten; the oldest kept is revision O`
  (exit 2): choose a kept revision; a forgotten one cannot be restored.
- `revision restore: too large; revision R's settings pass a proposal's
  bound; restore a later revision` (exit 2): choose a later revision.
- `revision restore: nothing to restore; the settings are as revision R
  left them` (exit 2): no change is needed.
- `revision restore: no revision R; the newest is revision N` (exit 2):
  choose a revision from the history, numbered from 1.
- `revision restore: stale; newest revision N; read settings show again`
  (exit 2): read again, then restore with a new key.

## Clean up

- Remove `FILE`; the settings stay as the newest revision left them.
- A held revision is released once its last task ends or moves.

## Next

- temper help settings: what a change is, and what temper checks.
- temper help revision: the history, and what running work reads.
- temper help proposal: deciding a proposal of settings.
- temper help charter: a charter's fields.
