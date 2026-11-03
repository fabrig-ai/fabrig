# Charter

Kind: concept

How an agent's run works: the instructions its agent reads, the models it may use, its tools in the workspace, the hosts that may run it, and its limits.

## What it is

A charter is a setting, `charter N`, numbered from 1. An agent task runs
under one charter, which its template or its creator names. Each run of
the task reads its charter, whole:

- its instructions, which the run's agent reads before its task's brief;
- its models: `model`, the default, then `alternatives`, in order;
- its tools in the workspace: to inspect, to modify (write, edit and remove
  files), to run a shell, to open sub-agents; a run modifies files only
  where it can deliver them, a change's run: a chat's, a goal's or an
  advisor's run reads alone, whatever its charter says;
- the hosts that may run it: the deployment's workers, the engine, or
  both;
- a run's limits: its turns and time, a reserve of each for finishing,
  how long it waits idle, and the timeout of its calls to temper's
  tools.

The forge's procedures run a change's agent work under three charters,
which the change keeps from its making to its end: its producing and
repairs under the forge's change charter, kind 3 number 1, whose stock
value is charter 2; its reviews under kind 3 number 2, whose stock value
is charter 4, which reads the
change and gives its verdict, and edits, pushes and runs nothing;
its foremen run under `charter 6`, reading the change and design to settle
its producer's question, and edit, push and run nothing.

Each member, with its type and bound, is in
`temper help settings-fields`.

## How it changes

A change of a charter reaches the tasks made after it. A task made
before it keeps reading the charter of its own revision until it moves
to the newest (`temper help revision`). A run already going keeps the
instructions it started with.

A removed charter is no longer offered: no new task may name it, and a
change that leaves a template or a delegation naming it is refused.
Tasks made before keep it, on their revision.

## What temper guarantees

- A task names no model a charter does not offer. With none named, a
  task takes, when it is made, the first of the charter's models with an
  account that can pay for it, else `model`; its runs keep the model it
  took.
- A charter's models are the deployment's: each must equal a configured
  model in every member but its effort, and its effort must be one the
  model takes.
- A run has more turns and time than its reserve, and keeps the reserve
  to finish and report.
- A charter's tools in the workspace are its own: a sub-agent has no
  more than its run. What a run may do beyond its workspace (its
  effects, its reads of the forge, its delegates) is its task's
  authority, within the project's policy and the rules
  (`temper help policy`).

## Acting on it

- `temper ctl`: `settings show` reads one, as the newest revision or a
  held one has it; `settings apply` and `settings propose` change it.
- A run reads charters with the host tool `settings`.

## Limits

- temper does not give a charter's `spend` to its runs: a run is given
  what its task has left, at most the rules' `maximum_run_spend`. A
  charter's spend must still hold its reserve.
- The forge's charter numbers are settings, kind 3: number 1 for changes,
  number 2 for reviews. Read them with `temper ctl settings show
  forge-charter 1` or `forge-charter 2`, and change them with `temper ctl
  settings apply`. A change keeps the charter numbers it was made with.
- temper does not check, when a charter changes, that a connected host
  can run it.
- A charter's instructions and models are within the deployment's bytes
  for a run; past them, the change is refused.
