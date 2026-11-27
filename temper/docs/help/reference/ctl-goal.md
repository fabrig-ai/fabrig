# temper ctl goal

Kind: reference

Goals: start one from a project template, or read one and its reds after landing.

## goal start

```
goal start --project PROJECT [--template TEMPLATE] TEXT [--charter CHARTER] [--priority PRIORITY] [--budget AMOUNT] [--as-allowed] [--deadline TIME] [--model MODEL] [--effort EFFORT] [--sub-agent MODEL[:EFFORT]]... [--milestone TASK] [--after TASK]... [--no-deadline] --key KEY
  --project PROJECT: a project's name or number; required. The project
  --template TEMPLATE: a name or number from 1 to 4294967295; optional. An offered template by name or number; stock goal template 2 when absent
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The goal's words; the first line, at most 60 characters, is its title
  --charter CHARTER: a name or number from 1 to 4294967295; optional. A charter by name or number, required when the template lets its creator choose; omit when it fixes a charter
  --priority PRIORITY: a number from 0 to 4294967295; optional. Override the template's tracked priority
  --budget AMOUNT: an amount with at most two decimals, as 130 or 1.25; optional. The spend, as task show prints budgets; required for a given budget unless --as-allowed
  --as-allowed: a switch. Take all you may give from your pool, or with --milestone all the milestone has left, instead of a named budget: none is left for the next start while this goal is live
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
Template sets charter, contract, tracking, label, flags, models
and symbolic resources. TEXT's first line is the goal's title, as Home
and Work show it: at most 60 characters naming the work for a person,
as "Store shape: history in task logs"; the rest of TEXT follows
on the lines below.
Given budget: --budget or --as-allowed; given deadline: --deadline
or --no-deadline. Defaults otherwise.
A goal waiting on tasks counts its deadline from when it was made.
With --milestone, --as-allowed takes all the milestone has left, so
a second --as-allowed goal in it is refused for its budget: give each
goal in a milestone --budget AMOUNT.
--effort needs --model, defaults if absent. --model wins each run
and replaces the later choice; only --sub-agent replaces sub-agents.
Without --model the choice holds whole.
Refused model flags name value and bound; nothing made.
Within bounds: created; past but proposable: proposed (exit 0);
not offered or past ceilings, nothing sent (exit 1).
task show TASK --until replied.
```

## goal show

```
goal show TASK
  TASK (task): a number; required. The goal's task number
Read a goal and its reds after landing
Prints: The task's document, then reds after landing, uncleared first, and where each stands
--json: the record goal_show.
Key: none.
Reads the task, then its project's findings blamed on it, once.
A task that is not a goal has no reds. A cut is marked.
--since and --until are not taken. --json prints task, reds with stands, and cut.
```
