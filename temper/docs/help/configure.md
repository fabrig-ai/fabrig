# Change how work runs

Kind: walk

Change a deployment's settings while it runs: read them, write a settings file, apply or propose it, see that it took, move running work to it, and restore what was there.

## Before you start

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

Size live concurrency separately from lifetime tree capacity; ended
tasks do not return tree room. `people.inbox_tasks` must be at least
2 * `tasks.tasks`; `people.operations` must be at least
max(`tasks.delegates` + 1, `tasks.project_goals`) and at most
`tasks.tasks`. For example, `tasks.tasks: 64` needs
`people.inbox_tasks: 128`, while `tasks.delegates: 8` needs
`people.operations: 9` and enough live task capacity to hold it.

A refused start names the failed dependency and required minimum.
A fixed schema ceiling or profile store ceiling cannot be raised with
a scalar override. `tasks.batch` grows the stored task record; reduce
the named capacity or its spec/result/authority sizes when it exceeds
that ceiling. Check each edit from a fresh configuration: a refusal of
an earlier edit can still be present. For a goal's sizing example,
read temper help budget; it does not promise a hundred-change profile.

## 1. Read the settings

```
temper ctl DIR settings show
```

Expect: `newest revision NEWEST`, then each setting and where its value
comes from, such as `charter 1: default, last changed in revision 1`,
then the held revisions and the pending proposals.

If not: exit 1 is ctl's side, such as `DIR` signed out (temper help
ctl).

## 2. Write a settings file from the value

A value is whole: read it, change what you mean to change, and set it
whole.

```
temper ctl DIR settings show charter 1 --json | jq '{items: [{setting: .setting, set: (.value | .instructions = "Answer in one short paragraph.\n")}]}' > FILE
```

Expect: no output; `FILE` holds `{"items": [{"setting": {"kind":
"charter", "number": 1}, "set": {...}}]}`, the value whole with its
instructions changed.

If not: an empty `FILE` and a line on stderr: the read failed, and its
line says why; read the settings again. A file holds at most 1024
items, one per setting; an item sets a value, or names `"edit":
"removed"` or `"edit": "reset"` in place of `set`.

For temper's settings, read the value in the same form:

```
temper ctl DIR settings show landing-rule 1 --json
temper ctl DIR settings show landing-rule 1 --project PROJECT --json
temper ctl DIR settings show forge-charter 1 --json
```

A landing rule item and a forge charter item may share a file. This
example sets the deployment's first rule and selects charter 2 for new
changes. Choose a charter the deployment offers, and keep the rule's
other fields as the read gave them:

```json
{"items": [
  {"setting": {"kind": "landing-rule", "number": 1},
   "set": {"pattern": {"segments": ["666f726765"], "last": {"open": ""}},
           "enforce": true, "ci": true, "up_to_date": true,
           "gates": [], "approvals": []}},
  {"setting": {"kind": "forge-charter", "number": 1}, "set": 2}
]}
```

For a project's landing rule, add `"project": PROJECT` to its `setting`.
A proposal's temper items, with their whole values or their removed or
reset edits, are included in `proposal show PROPOSAL`.

## 3. Apply it, or propose it

An owner applies it, naming the newest revision read:

```
temper ctl DIR settings apply FILE --against NEWEST --key KEY
```

A party that is no owner proposes it, with a reason as `TEXT`, and
reads its proposal:

```
temper ctl DIR settings propose FILE TEXT --against NEWEST --key KEY
temper ctl DIR proposal show PROPOSAL
```

An owner then decides it, from the inbox:

```
temper ctl DIR inbox
temper ctl DIR proposal decide PROPOSAL accept --running stays --key KEY
```

Expect: `revision` and the new revision's number, which is `NEWEST`
from now on; or `proposed: PROPOSAL, to ` and who decides, then
`proposal PROPOSAL · settings · against revision NEWEST, newest
NEWEST` and a line per item, `charter 1 · set · now {...} (default) ·
would be {...}`, each value whole; the owner's decision prints `done:
decide`, and prints it again when asked again under its key.

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

A live agent task, such as a chat started before the change (temper
help chat), keeps the revision it was made in until it moves: `task
show TASK` prints it as `settings: revision R · newest NEWEST`, and
`settings show` lists each held revision with its live tasks. Move the
tasks of a held revision, `REVISION`, at their next run:

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

To what an earlier revision set, find it in the history, newest first,
a page at a time: a page's last line is the call that reads on,
`revision list --before REVISION`. Its items are a settings file:

```
temper ctl DIR revision list
temper ctl DIR revision show REVISION --json | jq '{items: .items}' > FILE
temper ctl DIR settings apply FILE --against NEWEST --key KEY
```

Expect: `revision` and the new revision's number. After a reset,
`settings show` lists `charter 1: default, last changed in revision
NEWEST`.

If not: an item that prints `too_large` in place of `set` is too large
for the record: read its value with `settings show`, as `revision
show` names it. `revision R is forgotten; the oldest kept is revision
O`: temper keeps a bounded history, and a forgotten revision cannot be
read. `settings show` of a charter at an older revision answers
`unknown` (exit 2) once that revision is no longer held: read it from
`revision show`.

## Clean up

- Remove `FILE`; the settings stay as the newest revision left them.
- A held revision is released once its last task ends or moves.

## Next

- temper help settings: what a change is, and what temper checks.
- temper help revision: the history, and what running work reads.
- temper help proposal: deciding a proposal of settings.
- temper help charter: a charter's fields.
