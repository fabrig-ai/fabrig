# The engine's settings: an application's settings

## 1. An application's settings

An application's own settings ride the same mechanism: defaulted,
changed, proposed and decided as jig's are, each change in a revision.
Their values are the application's, kept by the connector that owns
their kind, in its own family: jig keeps what it decides, whether a
setting follows its default, which revision changed it and the size of
its value, never the value. No trait and no generic: a kind is a
number, a value crosses jig as bytes it never reads, and its connector
judges and keeps it within the change's decision.

- **Declared at the start.** For each of its kinds the application
  gives the core its number, in a space of its own beside jig's kinds;
  its scope, the deployment or each project; the connector that owns
  it, by number (../model.md, 3.5), which the root may route to any child
  of its own; and its bounds, a value's largest bytes and the values
  per scope, which the core's worst case prices for what it keeps of
  them, and the connector's for the values. Its defaults, each a number
  and bytes, it gives that connector, never the core.
- **The connector keeps the values,** in its own family, written in the
  core's decisions as every child's records are (../model.md, section 4):
  its kinds' defaults as the application last gave them, each setting's
  current change, and the items of each pending proposal of settings
  that name its kinds. A value is a record of the connector's own, so
  that its registered layout converts at read like any of the
  application's records (`../../protocol/store/layouts.md`, 4). A
  change no plan derives brings the owner's written job
  (`../../protocol/store/jobs.md`, 4), and jig's families hold no bytes
  of another owner's.
- **The core keeps the rest,** as it keeps its own kinds' (7): for
  each setting of an application's kind, whether it follows its
  default, is changed and by which revision, or is removed, and its
  value's size; in each revision's record, each item's edit and the
  size of what it set; in a pending proposal's set, its items' edits
  and sizes. jig's documents carry no value of an application's
  (`../../client/domain/contract/documents.md`, 1): they say the state and the
  size, and the application's own reads serve the value.
- **Judged in the change's decision.** A change naming items of a kind
  hands them, each with the value it would leave (a reset as the
  default it returns to, a removal as none) and the revision they would
  make, to that kind's connector through the root, within the
  decision's step (`vocabulary.md`, 1.3), as a requirement's verdict is asked
  (../connectors.md, section 7): the values a party's change carried, or,
  for an accepted proposal, those the connector kept. The connector
  accepts, refuses with its reason, which the change's refusal carries,
  or answers that an item leaves its setting as it is, which the core
  counts toward *empty* (2). A set commits only when every connector
  it names accepts.
  During `JudgeSettings`, the root may read the templates the active
  decision would offer a project through the core's
  `settings_project_templates` query, named by the judge's owner token.
  This applies the same set, removal and reset selection as 1's check,
  including the project's policy overrides the change would leave, without
  publishing or mutating the current settings.
- **Kept in the same decision.** Once a change commits, each connector
  saves its items' values as their settings' changes, a removal as a
  change with no value, so that a removed setting is never taken for
  one following its default, and erases the changes of those reset;
  it acts on them from then on, its
  state running ahead of the commit as every child's does (`decisions.md`, 1.1). A
  proposal of settings naming its kinds hands the connector its items'
  values in the decision that makes the proposal, which the connector
  checks against its kinds' bounds alone and keeps; the decision that
  withdraws or rejects it erases them, and the one that accepts it
  turns them into the changes, with the same edits a party's change
  makes.
- **At each start,** before its records are restored (`restart.md`, step
  2), each connector is given its kinds' defaults by the application,
  compares them with those it stored, and tells the core which of its
  settings' defaults differ and their sizes, so that a changed default
  no change is over joins the release's revision (7), the decision
  in which the connector saves the new ones. It also names, by kind,
  scope and number, each default it stored that the application no
  longer gives: a setting that follows one of those is removed in that
  revision, its state erased. A setting that follows a default its
  connector neither reports nor names, as one a reset returned to a
  default the application gives its connector for no one scope, or
  gives none of, keeps its state, its size and its slot (below), and
  the release's revision does not name it. A stored value it cannot
  read refuses the start, naming it, as a record it cannot read does; a
  default past its kind's bounds refuses it, naming the kind; and a
  stored setting, the core's or the connector's, of a kind the
  application no longer declares, or past its kind's values per scope,
  refuses it, naming the kind, scope and number. A release that drops a
  kind brings jobs at open to erase both owners' records of it
  (`../../protocol/store/jobs.md`, 4). Nothing is judged
  again: a stored value was judged when its change was made, and the
  defaults are the application's own.
- **Charters a value names.** A kind may declare that its values name
  charters, at most a number per value it declares. The connector says
  which a value names when it judges an item and when it reports a
  default, and the core keeps those numbers with the setting's state and
  holds them in its check of references (1, *reference*), as it
  holds a template's charter: a change that would leave such a charter
  unoffered is refused, naming the setting that names it, and a start
  whose defaults would is refused the same way.
- **Read by the application.** The root, and through it the
  application's own children and clients, read a value from the
  connector that owns its kind, by kind, scope and number: the newest,
  whatever revision a task was made in. Never pinned.
- **History** keeps, for an application's kind, each revision's edits
  and sizes, and the state and size of what each item replaced; the
  connector keeps the current values, the defaults, and, beside each
  change it saves, the value the setting held before, by the revision
  that made the change, for as long as the history keeps that
  revision's record (10), so that a restore proposes them again:
  every past value a restore reaches is one a later change replaced,
  and no other is kept.
- **The root routes, never judges** (../root.md, section 11): the judging
  and the keeping are the connector's, in its own terms.

The values-per-scope bound counts every setting state the core keeps in that
scope: changed, removed and following its default after a reset. Removing or
resetting a number does not free its slot. A set naming a new number past
that bound is refused as count before judging; editing a number already kept
adds no slot (decided 2026-11-03).
