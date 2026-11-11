# Change how work runs

Kind: walk

Change a deployment's settings while it runs: read them, write a settings file, apply or propose it, see that it took, move running work to it, and restore what was there.

## Before you start

Or describe how you work to Ask temper: temper help chat.

- A running deployment and `DIR` signed in (temper help box, steps 1 to 3).
  Owners apply settings; other parties propose them (step 3).
- jq on `PATH` for JSON.
- This walk changes charter 1's chat instructions (temper help charter).
- `NEWEST`: the last `settings show` revision; every accepted edit advances it.
- `FILE`: your settings file.

### Deployment capacity

Edit startup limits in `temper.json` under `engine.limits.overrides`, then
restart (temper help deployment). Settings and authority cannot raise them.

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

`people.requests`: 2,000 on small, 8,000 on standard, for 4 or 16 people,
20 daily conversations (creation, three messages, decision), fivefold
headroom. Settings and pending decisions share capacity. Answers replay
until expiry; full ctl reports count and oldest expiry. Size for workload
and retention.

A refused start names its dependency and minimum. A schema ceiling or
profile store ceiling cannot be raised with scalar overrides.
`tasks.batch_bytes` and inbox messages grow the stored record: reduce
capacity or spec, message, result and authority sizes if it exceeds the
ceiling. Check edits from a fresh configuration; earlier refusals can
remain. Goal sizing: temper help budget.

## 1. Read the settings

```
temper ctl DIR settings show
```

Expect: `newest revision NEWEST`, each setting's source and last revision,
held revisions and pending proposals. Exit 1 means a ctl error, such as
`DIR` signed out (temper help ctl).

## 2. Write a settings file from the value

Read, edit and set the whole value.

```
temper ctl DIR settings show charter 1 --json | jq '{items: [{setting: .setting, set: (.value | .instructions = "Answer in one short paragraph.\n")}]}' > FILE
```

Expect: no output; `FILE` holds charter 1's whole value with new instructions
in `items` (at most 1024, one per setting). If empty, read stderr and retry.
To remove or reset, replace `set` with `"edit": "removed"` or `"edit": "reset"`.

Keep other fields. This selects charter 2, keeping project 1's stock landing requirements:

```json
{"items":[
 {"setting":{"kind":"landing-rule","project":1,"number":1},
  "set":{"pattern":{"segments":["6272616e6368"],"last":{"open":""}},
         "enforce":true,"ci":true,"up_to_date":true,
         "gates":[{"number":1,"blocking":true,"freshness":"clean"}],"approvals":[]}},
 {"setting":{"kind":"forge-charter","number":1},"set":2}
]}
```

A project-policy item accepts only `set`, no `number`, at most once per
file; it may share a file with other settings. Edit `set` before sending. Its spends
resize the project's open period and parties' pools immediately; policy 1
sets only future projects' starting values.

Set project-policy `period_spend` 8300000 beside rules `period_spend` 8400000;
`project show PROJECT` confirms the open period's budget is 8300000.

For a project's landing rule, add `"project": PROJECT` to its `setting`.

### Human approvals

On a box you need none of these steps: temper knows your account on the box's
own Forgejo, and on GitHub when temper made the App from your own account.

With several members (`temper-box serve`), a managed Forgejo or a pasted App:
Add the human collaborator on the forge. On Forgejo, a repository admin must give temper's
bot admin: collaborator reads can require it even with write. Stock maps the
human's admin to maintainer (1), write to member (2), read to observer (3).
Edit the project's whole project-policy (step 2); apply by step 3. Re-adopt with
`repository adopt`, a new key and the same options; verify the role with
`project show`. Propose at the newest revision. Policy 1 affects future
projects; re-adoption keeps roles. temper's writer never counts.
See temper help landing-rule.

### Authority for new charters

Installing charters alone grants no task authority. Widen the rules ceiling,
policy 1's ceiling and the creator-role authority before work uses them: each
tool family its runs call and each note scope, a bit each, and each charter,
procedure, job or role it delegates to (temper help settings-fields). Preserve
budgets, grants, other fields and granted bits. Size tasks, depth and spend
(temper help budget).

Policy 1 affects future projects. Existing projects need project-policy: edit
each one's saved whole current `project-policy` settings item the same way.

Apply through step 3 as owner of both scopes. Tools need their bits at task,
project and deployment; notes also need scope bits. Existing tasks do not gain
widened authority automatically; moving revisions only changes charters.
Instead, amend or allot their authority explicitly, or start anew.

For the reference process's settings, see `temper/docs/examples/reference/README.md`.

## 3. Apply it, or propose it

Deployment owners decide deployment items; project owners, their policy and
landing rules. Both scopes need one owner of both. Apply against the newest revision:

```
temper ctl DIR settings apply FILE --against NEWEST --key KEY
```

Others propose with reason `TEXT`:

```
temper ctl DIR settings propose FILE TEXT --against NEWEST --key KEY
temper ctl DIR proposal show PROPOSAL
```

An owner of both decides:

```
temper ctl DIR inbox
temper ctl DIR proposal decide PROPOSAL accept --running stays --key KEY
```

Expect: `revision` and its new number, now `NEWEST`; or
`proposed: PROPOSAL, to the deployment's owners`. Project items name
`project PROJECT's owners`; several scopes require one owner of all.
`proposal show` prints the revision and each whole current and proposed value,
including `project-policy PROJECT · set · now {...} · would be {...}`.
Acceptance prints `done: decide`, repeated for the same key.

If not:

- `settings apply: stale` (exit 2): read the newest revision and decide again.
- `settings apply: settings unchanged` (exit 2): already set.
- `settings apply: out of shape` (exit 2): the line names the item
  (from 0), setting and broken bound or reference. Mend `FILE`; use a new key.
- `temper ctl: settings file item 1: item: exactly one of set or edit required`
  (exit 1): invalid file; diagnostics count items from 1 and name the rule.
- Another accepted edit withdraws pending proposals as stale; read and
  propose again (temper help proposal).
- `decided first` (exit 0): names the first decider (`you` if yours),
  decision and time; nothing changes.

## 4. See that it took

```
temper ctl DIR settings show
temper ctl DIR revision show NEWEST
```

Expect: `newest revision NEWEST`, `charter 1: changed in revision NEWEST`,
the decider, items and whole values. If charter 1 is still default,
read `inbox` for a proposal awaiting an owner.

`temper ctl DIR chat show TASK` reads a run's brief, charter document sections
and recorded commits; follow `--before CURSOR` for older pages. `task text` reads
specification or instructions, not received document content (temper help charter).

## 5. Move running work to it

Tasks keep their creation revision (`task show TASK`'s
`settings: revision R · newest NEWEST`). `settings show` lists held revisions
and live tasks. Move `REVISION` at their next run:

```
temper ctl DIR revision advance REVISION --key KEY
temper ctl DIR task show TASK
```

Expect: `moved 1 task to revision NEWEST` (the moved count), and
`task show`'s `settings: revision NEWEST`.

If not: an unoffered charter or model keeps the task on its revision,
counted as unable to move. A chat moves at its next run.
Use `--running moves` in step 3 to move tasks as you apply.

## 6. Restore what was there

Reset to its default:

```
printf '{"items": [{"setting": {"kind": "charter", "number": 1}, "edit": "reset"}]}' > FILE
temper ctl DIR settings apply FILE --against NEWEST --key KEY
```

Expect: `revision` and the new revision's number. After a reset,
`settings show` lists `charter 1: default, last changed in revision
NEWEST`.

Find an earlier revision newest first; continue with
`revision list --before REVISION`.

```
temper ctl DIR revision list
temper ctl DIR revision show REVISION
temper ctl DIR revision restore REVISION TEXT --key KEY
temper ctl DIR proposal show PROPOSAL
temper ctl DIR proposal decide PROPOSAL accept --running stays --key KEY
```

Use a new KEY per act.

Expect: `proposed restoring revision REVISION as proposal PROPOSAL to role K`;
`left out: SETTING` names omitted deleted-project items. Inspect before
accepting. Acceptance makes a new revision; running work stays put.

If not (exit 2): forgotten revisions cannot be restored; choose a kept
revision. `too large` needs a later revision within the proposal bound;
`nothing to restore` needs no action; `no revision` needs a listed revision
(from 1); `stale` needs a fresh read and key.

## Clean up

- Remove `FILE`; keep policies needed later. Applied settings stay.
- Revisions are released when their last task ends or moves.

## Next

- temper help settings: what a change is, and what temper checks.
- temper help revision: the history, and what running work reads.
- temper help proposal: deciding a proposal of settings.
- temper help charter: a charter's fields.
