# Charter

Kind: concept

How an agent's run works: the instructions its agent reads, the models it may use, its tools in the workspace, the hosts that may run it, and its limits.

## What it is

A charter is a setting, `charter N`, numbered from 1. An agent task runs
under one charter, which its template or its creator names. Each run of
the task reads its charter's shared instructions whole, then each part
whose `on` meets why it runs. A charter holds:

- its instructions, which the run's agent reads before its task's brief;
- its `parts`: instructions for some reasons a run runs, each with its
  `on`: `first`, the task's first activation, or the kinds of message
  that woke it;
- its `documents`: files from an adopted repository, read as brief sections
  of their own;
- its models: `model`, the default, then `alternatives`, in order;
- its `tools`: the host tool families it names; temper uses this list
  to offer the settings family, and offers the other host tools by the
  task's authority. A charter naming only asking and optionally settings
  is declared `see`, `document`, `remember` and `propose_settings` alone, as its
  families and authority permit;
- its tools in the workspace: to inspect, to modify (write, edit and remove
  files), to run a shell, to open sub-agents; a run modifies files only
  where it can deliver them, a change's run: a chat's, a goal's or an
  advisor's run reads alone, whatever its charter says;
- the hosts that may run it: the deployment's workers, the engine, or
  both;
- a run's limits: its turns and time, a reserve of each for finishing,
  how long it waits idle, and the timeout of its calls to temper's
  tools.

The stock holds eight charters: 1 for chats, 2 for coding, 4 for reviews,
5 for the goal's agent, 9 for the status agent, 10 for a model's trial,
11 for Ask temper and 12 for design sessions. Ask temper explains temper and turns how a person describes
their work into a proposal of settings (`temper help chat`).
The status agent writes the few lines at the top
of a project's Home from the facts in its task, with no tools.
Charter 10 runs one short turn,
with no tools, paid by the person. It is never offered as an executor;
the stock's rules offer seven charter executors, procedures 2, 3 and 4,
merge job 3 and roles 0, 1 and 2.

Charters 5 and 2 are built from the stock's set of charter files in
`temper/docs/charters/stock/`: `common`, `goal`, `designer`, `coordinator`
and `implementer`. Charter 5 joins `common` and `goal`, with `designer`
and `coordinator`'s sections as parts; charter 2 joins `common` and `implementer`,
with no parts. The goal's agent designs a goal's draft, cuts it into
changes, and answers what they ask.

The forge's procedures run a change's agent work under two charters,
which the change keeps from its making to its end: its producing and
repairs under the forge's change charter, kind 3 number 1, whose stock
value is charter 2; its reviews under kind 3 number 2, whose stock value
is charter 4, which reads the
change and gives its verdict, and edits, pushes and runs nothing.

Each member, with its type and bound, is in
`temper help settings-fields`.

### Documents from a repository

Put rules in a charter's `documents` when its runs should read them from
an adopted repository. In Ask temper, call `see` with address `settings`:
`project repositories` lists readable adopted repositories in the
chat's project, with each canonical host and OWNER/NAME. Copy that host
into the document's `forge` and OWNER/NAME into `repository`.

People can read `temper ctl DIR forge list` for the forge's `host`;
`temper ctl DIR forge list --json` prints the same `host` in each forge row. Copy that host unchanged into `forge`, including its port.
It is the host in repository resource names, not the forge number or
the forge's web address.

For example, add this `documents` member to the charter's value in a
settings file, keeping its other members (`temper help settings-fields`):

```json
{
  "documents": [
    {
      "forge": "git.example:3000",
      "repository": "work/hello",
      "branch": "main",
      "path": "AGENTS.md",
      "heading": "1. In one page"
    }
  ]
}
```

Use your printed host and adopted repository's OWNER/NAME, branch and
file path. `heading` is optional: omit it or use null to read the whole
file. With a heading, the run reads from the first Markdown heading
whose text matches exactly to the next heading of its level or above.
A missing repository, branch, file or heading gives a brief section
saying so.

Each new run reads the documents at their branch heads when its brief
is gathered, with the commit named in each section. A rule landed there
reaches the next run; a run already going keeps the sections it started
with. Changing the charter's document list follows the charter's
revision rule below.

Read what a run received with `temper ctl DIR chat show TASK`: its gathered
brief includes the document sections and their recorded commits. Follow
`chat show TASK --before CURSOR` when the printed continuation leads to
older transcript pages. To find a turn, read `task activity TASK` for its
`A.T`, then `task turn TASK A.T`; follow its `--part PART --from OFFSET`
continuation for a cut part. `task text` reads the task's specification or
instructions, which do not establish the document content supplied to a run.

### Over a default

An overlay keeps optional field replacements and added text over a charter's
binary default. Unlike a whole `set`, it takes later defaults: omitted fields
follow them. Named fields replace their whole, including an empty list.
Instructions append with a blank line; parts append in order, never merge.
Number and wake policy cannot be replaced. Only a charter with a default takes
an overlay; applying another replaces the stored overlay, not adds to it.

In a settings file, use `{"setting":{"kind":"charter","number":2},
"overlay":{"shell":true,"instructions":"Our rules"}}`. Literal
`instructions` and `parts` are optional, empty when absent. Or replace both
with `"charters":["."]`: directories relative to that settings file, in order.
Standard input (`-`) requires absolute directories. Each directory's Markdown
files are read, not its subdirectories; other extensions are ignored.
The optional five names and assembly are the stock's above. `designer` is on
`first`; each coordinator section begins `## part, on:` and comma-separated
reasons, its heading excluded. Texts retain every byte but opening and closing
line feeds. Unknown Markdown names, text before the first coordinator section,
and empty, repeated or unknown reasons are refused by file and line, even in
files the selected charter does not take. Sets are only for charters 5 and 2.
Markdown counts toward the settings file's input bound.

`settings apply FILE... --against REVISION` stacks files in order: overlays
of one charter join their text and later named fields win; every other setting
may occur in only one file. A whole set, removal or reset beside an overlay of
that charter is refused, naming both files and items. `settings propose` takes
one file. No refusal sends a change.

`settings show charter N` prints the current charter and marks what the overlay
replaces and adds; inventory origin is `overlay`. With `--json`, `value` is the
charter and `overlay` the literal overlay, with no directories. Wrap that member
as a settings-file item to apply it back over later defaults anywhere.

## How it changes

A change of a charter reaches the tasks made after it. A task made
before it keeps reading the charter of its own revision until it moves
to the newest (`temper help revision`). A run already going keeps the
instructions it started with.

A removed charter is no longer offered: no new task may name it, and a
change that leaves a template or a delegation naming it is refused.
Tasks made before keep it, on their revision.

## What temper guarantees

- A large gathered brief is shortened to fit before a run starts. Cuts
  are marked, and each marked section says where the agent can read the
  rest. Instructions, tools and the result contract stay whole.
- A task names no model a charter does not offer. With none named, a
  task takes, when it is made, the first of the charter's models with an
  account that can pay for it, else `model`; its runs keep the model it
  took.
- A charter's models are the deployment's: each must equal a catalogue
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
- Ask temper reads charters with `see settings charter 1`, using the
  charter number it needs, and proposes with `propose_settings`.
  A charter of a deployment's own that names the settings family and
  not asking reads with `settings` when its task's authority holds that
  family. The family also offers `propose_settings`; it changes nothing
  until an owner accepts.

## Limits

For the configuration members that need a restart, read `temper help configuration`.

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
