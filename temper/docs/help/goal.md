# Goals

Kind: concept

A goal is a tracked task: work given a numeric priority, which orders the goals' listing and the work waiting for resources, and whose tree is its plan.

## What it is

- **A goal is a task that is tracked,** at a priority, a number;
  any task can be one, a delegate among them. `task show` prints its
  tracking as `priority P`, and an untracked task's as `untracked`.
- **Its plan** is its tree of delegates: `task show` prints it in one line,
  `plan:` and how many live delegates run, wait or are held below.
- **The goal template,** template 2 in the stock settings, makes a
  tracked task at priority 1.

## How it changes

- **Made a goal:** an agent's run delegates a task tracked at a
  priority, with the host tool `delegate`, or with `goal`, which
  takes the work's words alone and fills in the rest from the run's
  own authority, last among the project's goals; or a party accepts a
  proposal that makes one; or an amendment tracks a task. An advisor
  is a goal's alone, and the run goes on after either refusal: a goal's
  own `goal` without `advisor`, when the work is not marked to be agreed,
  is refused, `a goal starts an advisor
  alone: set advisor, or make the change with change`; any other run's
  `goal` with `advisor` set, a chat's among them, is refused, `to change
  a repository, call change; a goal is for work they want to agree first
  or a survey, and its goal asks for an advisor`. A person who wants a
  survey alone gets a goal that asks its advisor and reports, changing
  nothing.
  `agree`, set by the party's chat when the party wants the work agreed
  first, makes the goal owe its requester a question: its brief says so, and
  before that question is answered its `change` is refused, `wait for the
  answer to question N`; a goal that finishes before asking is woken to ask.
  For an agreed goal, calling `goal` with `agree` or without `advisor`
  is refused, `this work is to be agreed: ask your requester your plan
  with message, as a question, and wait`. It can still ask an advisor
  without `agree`. Otherwise `agree` with `advisor` set is refused,
  `an advisor agrees nothing: set agree or advisor, not both`.
  A goal waiting for your answer can park when its run reaches its turn
  limit. Its question stays in your inbox. Your answer wakes a new run with
  a fresh turn allowance; spending still counts against the goal's remaining
  budget.
- **Reprioritised:** an amendment of its priority alone needs no
  spec revision; tracking an untracked task, or untracking a goal,
  names the spec revision the amender saw.
- **Ended** as any task ends (temper help task).

## What temper guarantees

- **The listing's order:** `task list` ordered by priority lists the lower
  number first, then by task number.
- **Waiting work in the same order:** among tasks waiting for the same
  write hold, or for a slot to run in, temper starts the one under the
  lower priority number first, each task taking its nearest goal's
  priority; a task under no goal waits behind every goal's, then the
  earlier wait goes first.
- **No interruption:** a new priority does not stop or restart a run
  in progress.

## Acting on it

- **ctl:** `goal start --project PROJECT --template TEMPLATE TEXT --key KEY`
  starts one from an offered template, defaulting to template 2 without
  `--template`. The template supplies its charter, contract, tracking,
  label, flags, models and symbolic resources. For a creator-chosen
  executor, name `--charter CHARTER`; for a fixed charter, omit it.
  A template with budget given needs `--budget AMOUNT` or `--as-allowed`;
  one with deadline given needs `--deadline TIME`, UTC to the minute,
  as `2026-10-24T17:00Z`. `--priority` overrides its tracked priority.
  `--model MODEL --effort EFFORT` changes its main choice and keeps
  its sub-agents; explicit `--sub-agent MODEL[:EFFORT]` replaces them
  and requires `--model`. Read `temper help ctl-goal` for the controls.
  Within your creating bounds the goal is made; beyond them, when
  offered, a proposal asks the named recipient first. An executor not
  offered or a goal past the project's ceilings sends nothing.
  `task list` lists a project's goals, tracked alone and
  ordered by priority; `task show` reads one; `task amend` changes its
  priority or its tracking; `proposal decide` accepts a proposal that
  makes one.
- **temper's runs** make goals with `goal` or `delegate` and
  reprioritise their delegates with `amend`; `goal` with `advisor`
  makes an untracked delegate instead, which reads and reports and
  can change nothing: the only `goal` a goal may call, and one no
  other run may.

## Limits

- ctl goal creation offers charter executors; procedures have no control here.
- temper has no board to reorder goals: each changes by its own
  amendment.
