# Budgets

Kind: concept

A budget is what a task may spend, in units its models' prices make of their usage, and by when; each run draws a share of it, and a task without enough is held.

## What it is

- **Spend** is counted in whole units. A model's prices turn its
  usage, fresh input, output, cache reads and cache writes, into units;
  reasoning is output and counts once. A unit has no currency.
- **Two scales in ctl:** `me` prints units as they are, `budget 40`;
  a task prints them divided by 100, `spent 0.01 of 0.40`.
- **A task's budget** is the spend its authority allows, and its
  deadline, a wall time, where it has one.
- **A project's period** funds all of its work: temper may spend up
  to N units in it, N the project's period spend. Its first period
  opens at the project's making, and each period lasts a week: at its
  next start the next opens, full, at the period spend its policy holds
  then. `project fund PROJECT AMOUNT` sets N, and resizes the open
  period with it at once.
- **Funding** comes from a person's pool, carved from the project's
  period up to their role's period spend, from an ancestor task, or
  from a recurring task.
- **A run** draws what its task has left, up to the deployment's run
  cap; its charter (temper help charter) sets its turns, its time and
  a reserve to finish in. The stock sizes its spends at each start
  from C, one costliest completion of the first configured model at
  its prices: its rules' floor is 2C and their run cap K, 4C or four
  units for each of the chat's run turns (256 for its 64), whichever
  is more; a chat or goal may spend 24K; the ceilings and owner's
  pool hold 80 × 24K for eighty at once, the project's period one
  chat more for temper's work. On `temper-box init`'s box C is 1,057 units, so a
  run draws 2,114 to 4,228. The stock chat runs 64 turns and an hour;
  the coding agent, its reviewers and the goal's agent 800 turns and eight
  hours, each with a reserve of 1 turn and 60 seconds; a run is held
  at its spend long before its turns are spent, and its person funds
  more. `settings show` reads the current ones.

## How it changes

- **At the start:** a new task's template sets its budget rule: an
  amount, as much as is allowed, or the amount its maker gives.
- **Delegated:** a delegate's budget is reserved from its funder's.
  When the delegate settles, its spend counts below the funder, the
  reservation goes, and what it did not spend returns. A change task
  starts with half of what its parent has left.
- **What is left** is the budget, less the task's own spend, less
  what its delegates settled, less what is reserved for them; never
  below zero.
- **Spent out:** a run that runs out of its turns, spend or time
  fails as a run fails: retried with a growing pause, then held, out
  of tries. A retry of a run whose charter resumes its transcript
  starts from the files its last attempt left, saved when it ended; a
  run that starts afresh from its brief starts from a clean checkout. A task without enough for its next run is held for its
  budget, one past its deadline for its deadline, which comes first
  when both hold; `task show` prints `Held`, and its record names the
  reason and what lets it run: for its budget, more budget, then a
  release; a release alone holds it again.
- **The period spent:** nothing more in the project is funded until
  the next period opens, or until `project fund PROJECT AMOUNT` raises it. A person's chats
  are funded from their pool, which their role's period spend sizes and
  which reserves from the period, so raise the role's next: `project
  fund PROJECT AMOUNT --role ROLE`. A cut keeps each budget at what it
  has used: nothing is taken back. A project made after a change of
  policy 1 starts with the new period spend.

## What temper guarantees

- **No run without its floor:** a run starts only with more than the
  floor to spend, never with more than the run cap, and only with its
  reserve: enough for its reserve turns and one more at its model's
  costliest.
- **Reservations count:** what is reserved for a delegate is not
  spent elsewhere while it runs.
- **A deadline is checked when a run starts,** never as a timer that
  ends a run in progress; a run starting at its deadline is admitted.
- **Release grants nothing:** a released task faces the same checks
  at its next run.

## Acting on it

- **ctl:** `task show` shows a task's spend, its budget and its
  funding; `me` prints what is left of your pool in each project;
  `project show` prints a project's open period and each pool in it,
  with what each has left; `settings show` reads the rules' run cap and floor, a charter's
  turns, and policy 1's period spend, which a new project takes;
  `settings apply` changes policy 1; `proposal decide` accepts a
  proposal that widens a task's authority; `task amend` changes a
  deadline; `task release` lifts a budget's or a deadline's hold once
  its cause is mended.
- **temper's runs** give a delegate its budget with `delegate`, raise
  it with `amend` within their own, and ask for more with `propose`.

For a goal of up to a hundred changes, count the whole task tree. For
example, assume 100 change procedures, 100 producers, 200 reviews and
100 repairs, plus 12 coordinator tasks including the goal:
512 tasks in the tree, 511 delegated tasks. If each agent task needs
one run at cap K and procedures spend no model units, allow 412K units
(400 production/review/repair runs plus 12 coordinator runs), then add
headroom for retries, extra reviews and revisions. This is an estimate,
not guaranteed capacity; reservations and each change's half-budget
allocation can require more headroom even before that total is spent.

Reserving five delegation levels
below the goal requires role depth at least 6. For this estimate, give
the goal at least 511 delegation tasks and permit the executor kinds
for its change procedures, producers, reviewers, repairs and
coordinators. The deployment's rules, the project's policy ceiling and
your role authority must all permit the spend and delegation, with
enough period funding and pool left. Use `settings show` and `me` to
check them; policy 1 changes affect projects created afterward, so
apply them before creating the project.

Before forge setup, size live concurrency and lifetime tree capacity
under `engine.limits.overrides` in `temper.json`:

| Member | Bounds |
|---|---|
| `tasks.tasks` | Deployment live tasks |
| `tasks.project_tasks` | Project live tasks |
| `tasks.tree_tasks` | Lifetime tasks including root |
| `tasks.depth` | Depth below root at zero |
| `tasks.delegates` | Live direct delegates |
| `tasks.batch` | Atomic creation |
| `tasks.batch_bytes` | A batch's bytes in all |
| `tasks.project_goals` | Project live goals |
| `documents.lines` | Document read lines |

`people.inbox_tasks` >= 2 * `tasks.tasks`: 64 needs 128.
`people.operations` >= max(`tasks.delegates` + 1, `tasks.project_goals`),
and <= `tasks.tasks`. At a profile store ceiling, reduce `tasks.batch_bytes`, `tasks.inbox_messages` or the spec,
message, result and authority sizes.
Raising policy authority alone does not raise deployment limits
(temper help configure).
The stock hundred-change profile remains a documented gap (N21);
stock capacity is not a promise that this estimate will fit.

`me` shows what a new root task can receive, with delegation tasks
calculated as min(role tasks, deployment tree tasks) - 1 and depth as
role depth - 1: the root consumes one task and one depth level. Thus
1024/6 and a tree limit of 32 yield 31/5.

## Limits

`temper help configuration`: what needs a restart.

- More task funds need an accepted proposal or an amendment within
  its funder's authority; none are given on demand.
- Settings changes never refill a task; a new run cap applies to
  later runs.
- Periods last a week from the project's first; this is not a setting.
  An unspent period is not carried over, and what an old one
  reserved stays its own. Restarting past the next start opens one
  period from then: `temper help weekly-budget`.
- ctl changes a project's own policy in its spends alone (`project
  fund`).
