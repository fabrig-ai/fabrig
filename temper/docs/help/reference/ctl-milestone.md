# temper ctl milestone

Kind: reference

Milestones: make one, say whole, list and show its goals.

## milestone new

```
milestone new --project PROJECT NAME [--words TEXT] [--priority PRIORITY] [--budget AMOUNT] [--as-allowed] [--deadline TIME] --key KEY
  --project PROJECT: a project's name or number; required. The project
  NAME (name): text, at most 1048576 bytes; required. Its name, 1 to 60 characters on one line, no control character
  --words TEXT: text, at most 1048576 bytes, or - to read stdin; optional. Its words; - reads stdin
  --priority PRIORITY: a number from 0 to 4294967295; optional. Override the template's tracked priority
  --budget AMOUNT: a number; optional. Spend in the deployment's unit; required for a given budget unless --as-allowed
  --as-allowed: a switch. Take as much as allowed instead of a named budget
  --deadline TIME: text, at most 1048576 bytes; optional. UTC to the minute, as 2026-10-24T17:00Z
Make a milestone from project template 8
Prints: `done: task N, milestone`, or `proposed: PROPOSAL, to RECIPIENT`
--json: the record milestone_create.
Key: --key KEY, required.
NAME is the spec's first line; --words follows a blank line.
The executor is jig's gathering; no --charter. Budget and deadline
controls replace template defaults; absent controls keep them.
Within bounds, created; past them but proposable, proposed (exit 0);
not offered or past ceilings, nothing sent (exit 1).
Fill it with goal start --milestone TASK, then milestone whole TASK.
```

## milestone whole

```
milestone whole TASK --key KEY
  TASK (task): a number; required. The milestone's task number
Say every goal is in the milestone
Prints: `done: task N, whole`
--json: the record milestone_whole.
Key: --key KEY, required.
Whole cannot be undone. A second call keeps the first conclusion.
A task that is not a gathering, or has ended, is refused (exit 2).
Pause, resume and close are task's, as for any task.
```

## milestone list

```
milestone list --project PROJECT [--ended] [--after CURSOR]
  --project PROJECT: a project's name or number; required. The project
  --ended: a switch. Read the ended group instead of the live
  --after CURSOR: a cursor, as a read prints it; optional. Read on from the printed cursor
Read one group of the project's milestones
Prints: Each milestone's number, name, D / N done and state; then read on with: or no more
--json: the record milestone_list.
Key: none.
Reads tracked tasks by template 8's label. State is live, or
complete, ended when some goal failed, or closed when cancelled.
Cancelled goals are left out of both counts.
```

## milestone show

```
milestone show TASK
  TASK (task): a number; required. The milestone's task number
Read a milestone's facts, totals and goals
Prints: Its name, whole time, words, maker, counts, spend and last landing or end; then its goals in task order
--json: the record milestone_show.
Key: none.
Goals merge live and ended delegates, each with its number, title
and state: live, done, failed or closed. A list past one page prints
read on with: task list. Where forge totals are gone, totals: none.
```
