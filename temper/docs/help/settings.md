# Settings

Kind: concept

How work runs in a deployment: its charters, templates, policies and rules, which its owners change while it runs, one checked decision at a time.

## What it is

Settings cover charters, templates, policies, rules, landing rules, forge
charters, goal templates' work, charter words, and project ways, questions, classes and words. Name one
by kind and number, or project for project settings; rules have no number: `charter 2`, `template 1`, `policy 1`, `rules`,
`landing-rule 1`, `landing-rule 1 --project 2`, `forge-charter 1`, `template-goal 4`,
`project-ways 2`, `project-questions 2`, `project-classes 2`, `charter-words 2`, `project-words 2`.

- A project's ways say whether chats can write drafts, whether Work's Goals
  and Changes filters show, and which offered template with
  `tracked` set a goal starts by; 0 takes the first. The stock offers drafts, hides the
  two filters and uses 0. Goals and Changes hide no rail entry;
  work under a hidden filter stays under Work's All filter.
- A project's questions give a change's question route when its maker
  names none: `none` or `up`. The stock uses `none`.
- A project's classes say which questions agents decide and tell you about,
  ask you, or ask you even when away (`temper help project`).
- A charter's words (`charter-words`) are the line How work runs says the
  charter by. A project's words (`project-words`) are How work runs' headline
  and lede when the project runs a way of its own. Ask temper writes both
  with its proposals.
- A landing rule governs changes landing on matching branches, for the
  deployment or one project: `temper help landing-rule`.
- The forge's charter 1 selects the change's charter, and 2 its reviews'.
  Its value is a charter's number.
- A goal template's work, `template-goal N`, says whether its coordinator
  designs first (`design`) and makes a close-out (`close_out`), both bools.
  With none kept, it plans and makes no close-out. The stock sets both false
  for template 4.
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

A charter overlay replaces only named default fields and appends text, taking
later defaults. A file item uses `"overlay"` instead of `"set"`: literal
`instructions` and `parts`, or `"charters":["."]` for directories relative to
the file (`-` requires absolute paths). See `temper help charter` for assembly.
`settings apply FILE...` joins overlays in order, later named fields winning;
all other duplicate settings, and set/remove/reset beside an overlay, are refused.
`settings propose` takes one file. Show marks replacements and additions;
`--json` exports the literal `overlay` beside `value`, naming no directory.

The stock has charters 1, 2, 4, 5, 9, 10, 11 and 12, and templates 1
for a chat, 2 for a goal, 4 for a draft goal, 5 for Ask temper and 6 for a
design session. Charter 5 coordinates a draft goal. Charter 10 is a
model's trial and is never offered. Charter 12 is the drafter: it writes a
draft with you as a design session's document, reads code through the
forge, and starts nothing. Its rules offer seven charter executors
(1, 2, 4, 5, 9, 11 and 12), procedures 2, 3 and 4, merge job 3 and
roles 0, 1 and 2.

Values are JSON, as `settings show` prints. Members, types and bounds:
`temper help settings-fields`.

Ask temper reads settings with `see settings`. Its answers fit the host
answer allowance in parts of at most 6,144 body bytes; `more: TOKEN`
continues with `see more TOKEN`. Its file reads use the same parts.

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

- On the rail, **How work runs** opens the project's process and its
  Repositories tab. On a phone, choose **How work runs for PROJECT** in
  the project switcher.
- Tell Ask temper how you work (`temper help chat`). It reads
  settings and proposes what temper enforces and agents follow; an owner decides.
- `temper ctl`: `settings show` reads the inventory: newest revision,
  settings and their value sources, held revisions and pending proposals.
  Given a kind and number, it reads the whole value. `settings apply`
  changes settings; `settings propose` proposes a change.
  `settings show landing-rule NUMBER` reads a deployment landing rule;
  add `--project PROJECT` for a project's.
  `settings show forge-charter NUMBER` reads one of the forge's charters;
  `settings show template-goal NUMBER` reads a goal template's work.
  Both take `--revision` for an earlier value and `--json` for a settings
  file's `set` form.
  `settings show project-ways PROJECT` and `settings show project-questions
  PROJECT` and `settings show project-classes PROJECT` read by project number or name, marking the stock when none is
  stored. `settings show charter-words NUMBER` reads a charter's words,
  marking the stock's with no stored value, or saying none is kept.
  `settings show project-words PROJECT` reads a project's headline and lede,
  or says none is kept. All take `--revision` for an earlier value and print
  state and value; `--json` gives the form a settings file's `set` takes.
  `proposal show PROPOSAL` includes proposed values, removals and resets, and a project's current
  and proposed policy, each whole. Each verb's help gives its arguments:
  `temper ctl help settings apply`.
- A settings file holds the items: `temper help settings-fields` gives its form.
- A run needs the settings family in its charter and authority to read settings
  and propose with `propose_settings`: Ask temper, charter 11, uses `see`
  and the asking form of `propose_settings`
  (`temper help charter`). It reads and proposes `charter`, `template`,
  `policy`, `rules`, `landing-rule`, `forge-charter`, `template-goal` and
  `project-policy`, `project-ways`, `project-questions`, `project-classes`, `charter-words` and
  `project-words`, in the settings file's form. Ways, questions, classes and project
  words require `project` and take no `number`; charter words take the charter's
  `number` and no `project`. A landing rule names
  its project, or none for the deployment; a project's policy names its
  project and is set alone. The tool reads a named project's policy,
  though the inventory omits it.

### Models

In Deployment, at the rail's foot, open Models. temper reads each provider's
list where it can: as an account becomes usable, once a day and when you
choose **Read again**.
A new model starts off the picker, marked “new” for a week. Its prices are
estimated until Ask temper sets them. A failed read says so on
Models and changes nothing.

A Claude subscription has no list to read: use **Add a model by name** for
a new model there, or any model a provider's list doesn't show. Choose its
provider and enter its name; the first run shows whether it is served.
An owner can turn models on or off in the picker. **Ask temper** in the
project rail opens its newest open chat, or a new chat where you can ask
about temper or tell it how you work. Use it to change prices, a default
or a role's models, or other settings.

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

- An item replaces a whole value or overlay; it never patches a stored one.
- ctl sends values within `temper help settings-fields` bounds; the
  deployment may refuse them past its own limits.
- Providers, listeners, workers and limits are configured at start;
  changes need a restart. Limits must hold the stock's settings or start
  refuses. Another model or new stock can change the limits needed.
- Stock floors are eight charters, five templates and fourteen executor kinds
  (seven charters, three procedures, one job and three roles); your settings
  may need more.
