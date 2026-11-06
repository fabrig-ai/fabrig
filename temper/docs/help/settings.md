# Settings

Kind: concept

How work runs in a deployment: its charters, templates, policies and rules, which its owners change while it runs, one checked decision at a time.

## What it is

Settings cover charters, templates, policies, rules, landing rules, forge
charters and project ways and questions. Name one by kind and number,
or project for ways and questions; rules have no number: `charter 2`, `template 1`, `policy 1`, `rules`,
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

A change can name a project's policy as `project-policy P`: set whole,
alone and at most once per file. It is no setting: `settings show` omits
it; `project show PROJECT` reads its roles and spends.

The binary holds defaults; the store holds overrides. A setting without
an override follows its default. Settings never live in the configuration
document.

The stock has charters 1, 2, 3, 4, 6, 9 and 10, and templates 1 for a
chat, 2 for a goal and 3 for the configuring chat. Charter 10 is a model's
trial and is never offered. Its rules offer six charter executors
(1, 2, 3, 4, 6 and 9) and procedures 2, 3 and 4.

A setting's whole value is JSON, as `settings show` prints it. For members,
types and bounds, read `temper help settings-fields`.

The configuring chat's `settings` pages fit the host answer allowance;
`next: OFFSET` continues at that byte offset. File replies share this bound;
`[file truncated]` marks cuts, including those for metadata room.

## How it changes

A change holds at most one item per setting. Each item:

- sets a whole value, replacing the previous value;
- removes it from the offer, its default included;
- resets it, erasing its change to follow its default again.

The rules are set or reset, never removed.

A project's way is checked against the templates the whole decision would
leave. One change or accepted proposal can install a template and select
it as the project's way. Removing it in that decision leaves it unavailable;
a reset follows its binary default, or leaves it unavailable if it has none.
A refusal names the item, the project's ways and the rejected way number.

A change names the newest revision its maker read. It makes one revision
(`temper help revision`), or is refused whole.
The deployment's items need its owners; a project's policy needs that
project's owners; a change of both needs one person who owns both. A
party with a deployment or project role proposes one for those owners
to decide (`temper help proposal`).

A start with a new binary default makes a revision if the setting follows
its default. A stored change still wins over a changed default.

## What temper guarantees

- Before commit, temper checks each value's shape and bounds, each kind's
  count, and references such as a template's charter or a charter's models.
- A refused change commits nothing. Its refusal names the item, its
  setting and its fault. A charter document refusal also names the member,
  the document index where applicable, and its rule. A document count past
  the allowance says, for example, `documents: supplied 3; maximum
  tasks.charter_documents 2`. A path or heading refusal names its byte bound;
  a name refusal names the authority segment bound; a connector refusal says
  that the connector must be registered. Change the named member and submit
  the proposal again.
- An effort its model doesn't offer is refused before anything is sent,
  naming the item, the model and what it offers, where `me` shows that
  model; the engine refuses any other by the model's name.
- A change against any other revision is refused as stale, naming the
  newest. Read again, and decide on what you read.
- A change that alters no setting is refused as unchanged.
- An accepted change takes effect without a restart.
- Repeating a change's key gives its first outcome, with no second revision.
- Each kind is read when it is used: a charter's at each run, from its
  task's revision (`temper help revision`); a template's when a task is
  made from it; a policy's when a project is made from it; the rules at
  every decision.

## Acting on it

- Tell the configuring agent how you work (`temper help chat`). It reads
  settings and proposes what temper enforces and agents follow; an owner decides.
- `temper ctl`: `settings show` reads the inventory: newest revision,
  settings and their value sources, held revisions and pending proposals.
  Given a kind and number, it reads the whole value. `settings apply`
  changes settings; `settings propose` proposes a change.
  `settings show landing-rule NUMBER` reads a deployment landing rule;
  add `--project PROJECT` for a project's.
  `settings show forge-charter NUMBER` reads one of the forge's charters.
  `settings show project-ways PROJECT` and `settings show project-questions
  PROJECT` read by project number or name, marking the stock when none is
  stored. Both take `--revision` for an earlier value and print state and
  value; `--json` gives the form a settings file's `set` takes.
  `proposal show PROPOSAL` includes proposed values, removals and resets, and a project's current
  and proposed policy, each whole. Each verb's help gives its arguments:
  `temper ctl help settings apply`.
- A settings file holds the items: `temper help settings-fields` gives its form.
- A run needs the settings family in its charter and authority to read
  with `settings` and propose with `propose_settings`: stock charter 3 alone
  (`temper help charter`). It reads and proposes `charter`, `template`,
  `policy`, `rules`, `landing-rule`, `forge-charter` and
  `project-policy`, `project-ways` and `project-questions`, in the settings
  file's form. Ways and questions require `project` and take no `number`. A landing rule names
  its project, or none for the deployment; a project's policy names its
  project and is set alone. The tool reads a named project's policy,
  though the inventory omits it.

### Recovering from a check reserve refusal

A worker's check timeout must fit every applicable modifying charter's
`reserve_time`, including versions held by tasks. A timeout of 60,000
milliseconds needs at least 60 seconds of reserve; equality is sufficient.
An accepted setting may still be unusable by a worker: proposal and
acceptance do not check connected workers' bounds.

When `task show TASK` names a charter's `reserve_time` in its next-run
refusal:

1. Find the held revision in the task's `settings:` line and `settings
   show`. Read it with `settings show charter 2 --revision 13`, using
   your deployment's charter and revision.
2. Read the current charter with `settings show charter 2 --json`. Set
   `reserve_time` to at least the worker's check timeout, in nanoseconds
   in the settings file: 60 seconds is `60000000000`. Keep `time` greater
   than the reserve. Apply the whole value with `settings apply FILE
   --against REVISION --key KEY`, or propose it for an owner to accept.
3. Held charter versions remain in force.
   Use `revision advance 13 --key KEY` for each affected held revision.
   Read its moved and stayed counts and the tasks again. A live run keeps
   its version until it ends, and a task that cannot move still holds
   its revision (`temper help revision`).

The worker is admitted on a later hello once all applicable versions fit.
Changing the current reserve alone leaves incompatible held versions.

## Limits

For members needing a restart, read `temper help configuration`.

- temper does not patch a member: an item sets a whole value.
- The person's client has no page to change settings.
- ctl sends values within `temper help settings-fields` bounds; the
  deployment may refuse them past its own limits.
- Providers, models, listeners, workers and limits are configured at start;
  changes need a restart. Limits must hold the stock's settings or start
  refuses. Another model or new stock can change the limits needed.
- Stock floors are seven charters, three templates and nine executor kinds
  (six charters and three procedures); your settings may need more.
