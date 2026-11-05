# Settings

Kind: concept

How work runs in a deployment: its charters, templates, policies and rules, which its owners change while it runs, one checked decision at a time.

## What it is

A deployment's settings are its charters, templates, policies, rules,
landing rules, forge charters and each project's ways and questions.
A setting is named by kind and number, or project for ways and
questions; the rules have no number: `charter 2`, `template 1`, `policy 1`, `rules`,
`landing-rule 1`, `landing-rule 1 --project 2`, `forge-charter 1`,
`project-ways 2`, `project-questions 2`.

- A project's ways say whether chats can write drafts and whether Goals,
  Changes and the Away switch show, and which offered template with
  `tracked` set a goal starts by; 0 takes the first. The stock hides all
  four and uses 0.
- A project's questions give a change's question route when its maker
  names none: `none`, `foreman` or `up`. The stock uses `none`.
- A landing rule governs changes landing on matching branches, for the
  deployment or one project: `temper help landing-rule`.
- The forge's charter 1 selects the change's charter, and 2 its reviews'.
  Its value is a charter's number.
- A charter is how an agent's run works: `temper help charter`.
- A template is how a new task is filled: `temper help template`.
- A policy is what a project made from it starts with, and the rules are
  the ceilings of every project: `temper help policy`.

A project's policy is no setting, but a change can name it as
`project-policy P`: set whole, set alone, at most once per file.
`settings show` omits it; `project show PROJECT` reads its roles and spends.

The binary holds defaults; the store holds changes over them.
A setting without a stored change follows its default. Settings never
live in the configuration document.

The stock has five charters (1, 2, 3, 4 and 6) and three templates:
1 for a chat, 2 for a goal, 3 for the configuring chat. Its rules offer
those five charter executors and procedures 2, 3 and 4.

A setting's whole value is JSON, as printed by `settings show`.
Its members, types and bounds are in
`temper help settings-fields`.

The configuring chat's `settings` pages fill the host answer allowance;
`next: OFFSET` continues at that byte offset. File replies also fit that
allowance: `[file truncated]` marks a cut, including cuts for metadata room.

## How it changes

A change holds at most one item per setting. Each item:

- sets a whole replacement value;
- removes the setting from those offered, its default included;
- resets it, erasing its change to follow its default again.

The rules are set or reset, never removed.

A change names the newest revision its maker read. It makes one revision
(`temper help revision`), or is refused whole, changing nothing.
The deployment's items need its owners; a project's policy needs that
project's owners; a change of both needs one person who owns both. A
party with a role in the deployment or in a project proposes one
instead, for those owners to decide
(`temper help proposal`).

A start with a new default for a setting following its default makes
a revision. Stored changes still win over changed defaults.

## What temper guarantees

- Before committing, temper checks the whole change: values' shapes
  and bounds, counts per kind and references between settings, such as
  a template's charter or a charter's models.
- A refusal commits nothing and names the item, setting and fault.
- A change against an older revision is refused as stale, naming the
  newest. Read again and decide on what you read.
- A change leaving every setting as it was is refused as unchanged.
- An accepted change takes effect without a restart.
- Repeating a change's key gives its first outcome, with no new revision.
- Each kind is read when it is used: a charter's at each run, from its
  task's revision (`temper help revision`); a template's when a task is
  made from it; a policy's when a project is made from it; the rules at
  every decision.

## Acting on it

- Describe how you work to the configuring agent (`temper help chat`).
  It reads the settings and proposes one change, with what temper
  enforces and what your agents follow; an owner decides it.
- `temper ctl`: `settings show` reads the inventory (the newest
  revision, each setting and where its value comes from, the held
  revisions and the pending proposals) and, given a setting's kind and
  number, its whole value; `settings apply` makes a change; `settings
  propose` proposes one. `settings show landing-rule NUMBER` reads a
  deployment landing rule; add `--project PROJECT` for a project's.
  `settings show forge-charter NUMBER` reads one of the forge's charters.
  `settings show project-ways PROJECT` and `settings show project-questions
  PROJECT` read a project's, by number or name, marking the stock with none
  stored. They take `--revision` for an earlier value.
  Both print the setting's state and its value; `--json` prints the value
  in the form a settings file's `set` takes. `proposal show PROPOSAL`
  includes temper's proposed values, removals and resets, and a project's
  policy now and as it would be, each whole.
  Each verb's arguments are its help: `temper
  ctl help settings apply`.
- A settings file holds a change's items: its form is
  `temper help settings-fields`.
- A run whose charter names the settings family and whose authority
  holds it reads with the host tool `settings` and proposes with
  `propose_settings`: charter 3 alone in the stock
  (`temper help charter`). It reads and proposes `charter`, `template`,
  `policy`, `rules`, `landing-rule`, `forge-charter` and
  `project-policy`, `project-ways` and `project-questions`, in the settings
  file's form. Ways and questions require `project` and take no `number`. A landing rule names
  its project, or none for the deployment; a project's policy names its
  project and is set alone. The inventory lists no project's policy;
  the tool reads it when named.

### Recovering from a check reserve refusal

A worker's check timeout must fit every applicable modifying charter's
`reserve_time`, including versions held by tasks. A timeout of 60,000
milliseconds needs at least 60 seconds of reserve; equality is sufficient.
An accepted setting can still leave a worker unable to run it: proposal
and acceptance do not validate against connected workers' bounds.

When `task show TASK` names a charter's `reserve_time` in its next-run
refusal:

1. Read the task's `settings:` line and `settings show` to find its held
   revision. Read that value with `settings show charter 2 --revision 13`
   (substitute the charter and revision named in your deployment).
2. Read the current charter with `settings show charter 2 --json`. Set
   `reserve_time` to at least the worker's check timeout, in nanoseconds
   in the settings file: 60 seconds is `60000000000`. Keep `time` greater
   than the reserve. Apply the whole value with `settings apply FILE
   --against REVISION --key KEY`, or propose it for an owner to accept.
3. Updating the current setting leaves held charter versions in force.
   Use `revision advance 13 --key KEY` for each affected held revision.
   Read its moved and stayed counts and the tasks again. A live run keeps
   its version until it ends, and a task that cannot move still holds
   its revision (`temper help revision`).

The worker is admitted on a later hello once all applicable versions fit.
Changing the current reserve alone does not release an incompatible held
version.

## Limits

For configuration needing a restart, read `temper help configuration`.

- temper does not patch a member: an item sets a whole value.
- The person's client has no page to change settings.
- A value's bound in `temper help settings-fields` is what ctl sends;
  the deployment may refuse a value within it, past one of its own
  limits.
- The providers, models, listeners, workers and limits are not
  settings: they are the configuration document's, read at a start, and
  a change to them needs a restart. Its limits must hold the stock's
  settings: a start refuses one that does not, and another model or a
  release with new stock can move what they need.
- Stock floors are five charters, three templates and eight executor
  kinds (five charters, three procedures); your settings may need more.
