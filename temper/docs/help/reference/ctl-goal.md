# temper ctl goal

Kind: reference

Goals: start one from a project template.

## goal start

```
goal start --project PROJECT [--template TEMPLATE] TEXT [--charter CHARTER] [--priority PRIORITY] [--budget AMOUNT] [--as-allowed] [--deadline TIME] [--model MODEL] [--effort EFFORT] [--sub-agent MODEL[:EFFORT]]... [--milestone TASK] [--after TASK]... [--no-deadline] --key KEY
  --project PROJECT: a project's name or number; required. The project
  --template TEMPLATE: a number from 1 to 4294967295; optional. An offered template; stock goal template 2 when absent
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The goal's words; the first line, at most 60 characters, is its title
  --charter CHARTER: a number from 1 to 4294967295; optional. Required when the template lets its creator choose; omit when it fixes a charter
  --priority PRIORITY: a number from 0 to 4294967295; optional. Override the template's tracked priority
  --budget AMOUNT: a number from 0 to 18446744073709551615; optional. The spend in the deployment's unit; required for a given budget unless --as-allowed
  --as-allowed: a switch. Take as much as allowed instead of a named budget
  --deadline TIME: text, at most 1048576 bytes; optional. UTC to the minute, as 2026-10-24T17:00Z; or --no-deadline for a given deadline
  --model MODEL: text, at most 1048576 bytes, or - to read stdin; optional. Configured model name
  --effort EFFORT: text, at most 1048576 bytes, or - to read stdin; optional. Effort offered by that model
  --sub-agent MODEL[:EFFORT]: text, at most 1048576 bytes, or - to read stdin; repeated, at most 64. Replace the template's sub-agents; needs --model; each model once, its effort optional as --model's
  --milestone TASK: a number; optional. Start beneath this live milestone of the project
  --after TASK: a number; repeated, at most 64. Wait on this task; repeatable, each task once, in order
  --no-deadline: a switch. Give the goal no deadline; the alternative to --deadline
Start a goal from a project's template
Prints: `done: task N, goal`, or `proposed: PROPOSAL, to RECIPIENT` when another party must agree
--json: the record goal_create.
Key: --key KEY, required.
The template fills charter, contract, tracking, label, flags, models
and symbolic resources. TEXT's first line is the goal's title, as Home
and Work show it: at most 60 characters naming the work for a person,
as "Store shape S7: history in task logs"; the rest of TEXT follows
on the lines below. --charter is required only for a chosen executor.
Given budget needs --budget or --as-allowed (alternatives); given
deadline needs --deadline or --no-deadline. Absent controls keep defaults.
A goal waiting on tasks counts its deadline from when it was made.
--effort needs --model; absent, its default is taken. --model wins
each run, replacing the later choice too; sub-agents stay unless
--sub-agent replaces them. Without --model the choice holds whole.
Refused model flags name value and bound; nothing made.
Within bounds, created; past them but proposable, proposed (exit 0);
not offered or past ceilings, nothing sent (exit 1).
Then task show TASK --until replied.
```
