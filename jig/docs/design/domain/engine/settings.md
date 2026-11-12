# The engine's settings

A deployment's charters, templates, the policies its projects are made
from, its rules, its LLM models, and an application's settings of its own are its
**settings**: the binary's defaults, with the changes the store keeps
over them. A change is one decision; each makes a revision; a task's
runs read their charter from the revision the task was made in. This
section is the mechanics; the model is ../model.md, section 13; who asks
is ../people/acts.md, 1; what crosses to a connector is ../connectors.md,
section 17; the records are `../../protocol/store/README.md`, 4.4.

## 1. What a setting is

- **A setting is named by its kind and its number,** and, for an
  application's kind kept per project, its project:

  | Kind | Number | Value | Read |
  |---|---|---|---|
  | charter | the charter's, from 1 | a charter whole: its instructions as text, its documents (9), its models, tool families, hosts, waiting, resume and default budget (`../../protocol/configuration.md`, section 3); or, over a default, an overlay: the fields it replaces and the text it adds after the default's (`overlays.md`) | at each run's preparation, from the run's revision (3), an overlay laid over its default |
  | template | the template's, from 1 | a template whole (`../../protocol/configuration.md`, 3.1) | when `me` is served (`views.md`, 1): the newest |
  | policy | the policy's, from 1 | a whole policy in a seed's format, a project made from it copying it (../model.md, 6.1) | at each `MakeProject`: the newest |
  | rules | none: one setting | the deployment's rules, whole (../authority.md, section 6) | at every decision: the newest |
  | model | the model's, 1 to 64, never given to another name (11) | a model whole: its name, shown name, LLM provider, endpoint, window, output, efforts and default effort, prices, description, whether a picker shows it, whether it is withdrawn, where it came from (the binary, a provider's list or a person), whether its prices are estimated and when a list first gave it | at each decision and when `me` is served: the newest; a run reads it from its charter (11) |
  | an application's kind | the application's | bytes jig never reads, kept by the connector that owns the kind (5) | by the application, from that connector: the newest |

- **The defaults are the binary's.** The application passes them as
  values when it builds the core (`Config`'s defaults): its models, charters,
  policies and rules, its templates, jig's chat and goal among them
  where it wants them, which jig gives as values made for the charter
  they run (`../../protocol/configuration.md`, 3.1); its own kinds'
  defaults it gives the connectors that own them, never the core
  (5). An application that passes none has `jig-protocol-config`
  read them from the document (7).
- **The current settings are the defaults with the stored changes over
  them.** A change sets a setting's value whole, or removes a default,
  so that it is no longer offered; a charter's change may instead set
  an overlay, kept over its default, which each start lays over the
  default it is given (`overlays.md`). A setting no change names follows its
  default, whatever the binary makes it. A reset erases a setting's
  change: it follows its default again. A setting no default has is
  added by a change that sets it, and its removal, like its reset,
  erases that change. The rules are set or reset, never removed.
- **Checked whole, by one check.** The core checks settings as a start
  has always checked the document (`../../protocol/configuration.md`, sections 3 and 3.1 and `../../protocol/limits.md`
  ): each value in its shape and within its bound
  (a charter's instructions within the journal's run bytes, its budgets
  above its reserve and holding one of its default model's costliest
  completions, its models configured; a template's charter offered and
  its grace within the tasks child's span; a model in 11's shape; a policy within the rules,
  its roles well formed, its overrides' templates offered, its owner
  role able to be a root's requester), the counts within their limits
  (charters within `tasks.charters`, templates within `templates`,
  policies within theirs), and the references among them, across the
  set. One function of the core's runs it: at each start over the
  defaults and the current settings, and for a change over the settings
  it would leave. Its refusal names a setting by kind and number, which
  the parser maps to the document's path when the document gave it,
  and its fault, one of a closed list a client words without checking
  anything itself: *shape* (a member out of its form or past its
  bound, except a rules ceiling's delegation-kind count), *count* (its
  kind past its limit, or a rules ceiling's delegation kinds past
  `authority.executors`, also when restoring a stored value), *reference*
  (it names a charter, template, model or host the set does not offer), *rules* (a
  policy above them), *budget* (a charter's below its reserve or short
  of one costliest completion), *grace* (a template's past the tasks
  child's span), *owner* (a policy's owner role unable to be a root's
  requester).
- **What is not a setting:** the limits, which size memory at the
  start; the hosts, the LLM providers and their endpoints, the connectors, the
  clients and the first owners, the document's; and each project's own
  policy, which its owners change (../people/acts.md, 2).

## 2. A change

- **A set, against a revision.** An owner's `ChangeSettings` (../people/acts.md,
  1) names the revision it was made against and its items, at most
  one per setting, each a setting and what becomes of it: *set* to a
  value, whole; *removed*; or *reset* to its default. It says, too,
  whether running work *stays* on its revision or *moves* (4).
- **A project's own policy** is no setting (1), and is an item of a
  change too, so that one decision sets a project up whole: its policy
  beside the charters, templates, rules and application's settings its
  work runs under. The item names a live project, at most one per
  change, and is *set* alone, to a policy whole in a seed's format, as
  a policy projects are made from is given (1); *removed* or *reset*
  is out of shape, since a project's policy has no default to return
  to, and a project not live is refused unknown, as every request
  naming it is (../people/acts.md, 2). It is the project's item, decided by
  the project's owners: beside the deployment's items it needs one
  party who owns both (*not theirs*, below). It is checked against the
  settings the change leaves, as 1 checks a policy projects are made
  from (within the rules, its roles well formed, its overrides'
  templates offered, its owner role able to be a root's requester), and
  as its owners' own edits of it are (../people/acts.md, 2), so a change may
  add the template the policy overrides; a refusal names the item and
  what fails, and a policy equal to the project's leaves it as it is,
  counted toward *empty*. Committed, it replaces the project's policy
  whole in the change's decision, as its owners' edits apply (../people/acts.md,
  2): the open period and the parties' pools resized to its spends
  (../authority.md, section 7), and every later decision under it. It makes
  no setting and is never pinned (4): the revision's record keeps
  the item as any other, its value whole and the project's policy it
  replaced (10). A change, and a pending proposal (6), naming one
  is priced at one policy beyond the whole set 8 prices.
- **Refused whole,** nothing of it committed, when it is:
  - *stale:* another revision is the newest, which the refusal names;
  - *not theirs:* an item of the deployment's (jig's kinds, and an
    application's kept for the deployment) from a party who is not an
    owner in the deployment (../people/README.md, section 3), or an item of a
    project's from one who is not that project's owner (../people/README.md,
    section 4). A set touching both needs one party who is both;
  - *out of shape:* the settings it would leave fail 1's check,
    which names the item and what fails. The refusal's `at` is that
    item's position among the change's items as the request gave
    them, counted from 0 as a batch's members are: the last item
    naming the failing setting, so that a repeated setting names its
    repeat, and 0 when no item names it (a setting the change leaves
    as it was), which the fault's setting still names;
  - *judged:* a connector refuses an item of its kinds (5), saying
    why;
  - *full:* the revisions or the charter versions held once the
    decision is done, its moves included, would pass their limits
    (8). The refusal names the oldest revision held and its
    live tasks, within the limit on the tasks a refusal names (../tasks/README.md,
    section 6), so the owner may move them (4) and send it again;
  - *empty:* it changes nothing of the current settings.
- **One decision.** Its commit holds the changes saved and erased, the
  revision it makes with its record (3 and 7), the charter
  versions it makes and those no held revision reads any longer, the
  answer under the request's key, naming the new revision, and what each
  part hears of it, in the same decision: the tasks child and the fleet
  the charter versions added and dropped; authority the rules; the
  people child a project's policy it sets (above); the
  views a mark on every `me`, when a template changed, and on every
  watch of the settings; each connector the word to keep its kinds' items' values (5); and, when
  it moves running work, the tasks moved (4). After a crash the old
  revision holds or the new one does, never a mix (../model.md, section 4).
- **What follows it.** A narrowed rule reaches every later decision,
  tasks already made included, and a widened one widens no existing
  task (../authority.md, section 6); a live project's policy is not checked
  against new rules, only the policies projects are made from are, and
  a project's policy the change itself sets. The
  next `MakeProject` copies the policy as changed; a project made before
  keeps its own copy (../people/acts.md, 2). Clients re-read `me`'s templates,
  and a template removed leaves its number unused (`../../protocol/configuration.md`,
  3.1). A task made after the change reads its
  charter from the new revision; running work, 17.4.
- **Asking is deciding.** A change is an owner's direct request,
  decided as it is admitted. One proposed by someone who may not make it
  is a proposal of settings (6), and accepting it makes this same
  decision.

## 3. Revisions

- **A revision is the settings as a change left them,** numbered from
  1, the first start's, and each change makes the next. A start whose
  defaults change the current settings makes one too, recorded as the
  release's (7). Everything made reads the newest.
- **A task records its revision:** the newest, when it is made, by
  whatever makes it (a party's batch, a run's delegation, a procedure's
  step, a recurring task, an accepted proposal), whatever revision its
  parent runs under. It keeps it, live and ended, until it moves
  (4). A delegation to a charter the newest does not offer is refused
  as not offered, as any create naming one is, before its authority is
  checked, so the run reads the true cause, and the agent adapts. A
  procedure's or a person's task reads nothing of a charter: its
  revision is its history alone, and it holds none.
- **A run records the revision it is prepared under:** its task's, as
  its preparation begins (`executors.md`, 1, after a conversation's move, 4),
  committed with its claim. What the run is given of its charter is
  that revision's: its instructions, models, tool families, hosts,
  waiting, resume and budget; and so is what the core decides from the
  charter while the run lives: its calls' checks, its placement, its
  finish. A change made during its preparation or its run changes
  neither.
- **Charter versions.** A charter's version is its value as one
  revision set it, named by the charter's number and the revision that
  made it, and kept once, however many revisions read it. A revision
  reads, for each charter, the latest version made at or before it; a
  removal is a version too, from which on the charter is not offered.
  So a revision costs only the charters it changed. Only charters are
  kept by version: every other kind is read at the newest (1), and of
  an older revision only its record says what it set of them.
- **Two lookups.** `charter_of(task)` is the version of the task's live
  run, or, with none live, of the task's revision; `charter(number)` is
  the newest's, for what is about to be made. Every site of the core
  that reads a charter reads it through one of them. An application
  that reads a task's charter through `charter_of` reads the task's own
  version with no change of its own.
  An amendment of a task's LLM choice is checked against its own
  version's models, a delegate's against the newest's.
- **Held revisions.** The newest is held, and so is every revision a
  live agent task records or a run in flight was prepared under (its
  preparation, its claim, its placement). A revision nothing holds,
  other than the newest, leaves memory in the decision that ends its
  last holder: the charter versions no held revision reads go with it,
  from the core, the children and the store, and its record stays as
  history (7).
- **The children keep versions, not revisions.** The core tells the
  tasks child and the fleet each charter version a held revision reads
  as it is made (to the tasks child, the models its tasks may choose; to
  the fleet, the host needs of its runs), and each one dropped. The
  tasks child finds a task's version by its charter and its revision,
  the latest at or before it, from its own table; the fleet is given the
  version with each run it places. No other child reads a charter.
- **What documents show.** A document that shows a task's charter (its
  tree, its waits, its LLM choice) reads the version its revision reads
  while that revision is held; once it is not, as for a task ended long
  since, it names the charter and the revision, and nothing read from
  the charter.

## 4. Moving running work

- **A move** sets a live agent task's revision to the newest. A task moves
  only when the newest offers its charter and allows its LLM choice
  (../tasks/README.md, section 3); one that cannot stays on its revision and
  finishes there. A move takes effect at the task's next run: a live
  run goes on with the version it was prepared under, until it ends or
  an owner cancels it, and holds that revision until then.
- **A conversation moves at its next run.** A task flagged
  conversation, which its requester ends and which never ends on its
  own (../tasks/lifecycle.md, 9), moves as each of its runs begins to be prepared,
  when it may. So a fix to its charter reaches its next turn.
- **A change may move running work:** committed with *moves*, it moves,
  in its decision, every live agent task on an older revision that may
  move.
- **An owner moves running work later** with `MoveToNewest` (../people/acts.md,
  1), naming a revision: every live agent task on it that may move moves, in
  one decision, and the answer counts those moved and those that could
  not. It is what
  a change refused as full offers.
- **What a task may do is never pinned.** Its authority is the rules',
  its project's policy's and its chain's as they stand (../authority.md,
  section 6); only what its runs are given is its charter version's.

## 5. An application's settings

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
  that the application's patches reach it as they reach any of its
  records (`../../protocol/store/opening.md`, 3), and jig's families hold no
  bytes of another owner's.
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
  decision's step (`README.md`, 4.3), as a requirement's verdict is asked
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
  state running ahead of the commit as every child's does (`README.md`, 5.1). A
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
  kind, from the baseline on, erases both owners' records of it by its
  patches (`../../protocol/store/opening.md`). Nothing is judged
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

## 6. A proposal of settings

- **Proposed by a member or a run.** A party with a role in the
  deployment or in a project proposes a set with `Propose` (../people/acts.md,
  1); a run with its `propose` call, when its charter gives it the
  settings family (`executors.md`, 3), which reads the settings and proposes a set;
  never a procedure. A proposal names its revision, its items and its
  reason, as a change does, and leaves to its accepter whether running
  work moves. The reason is bounded by `people.words` bytes for both a
  party and a run. A run past that bound is refused with the words limit
  and may shorten its reason and propose again. The pending review shows
  every admitted reason whole, independently of the document snippet bound.
- **In the deployment's scope.** No task's authority covers settings,
  so a proposal of settings waits for parties from the start: the
  deployment's owners for the deployment's items, and a project's owners
  for that project's; a set touching both waits for parties holding all
  of them. One no party may accept is refused at its entrance, as every
  proposal nobody may accept is (../tasks/proposals.md).
- **It belongs to the projects its items touch.** An application's item of
  a project's scope and a project's own policy touch that project; a
  proposal's `projects` are those, ascending and distinct, at most 32, worked
  out by the core from its items, which never change once it is made, and never
  by a client. One whose items touch no project, or that has no items, is the
  deployment's, and has none. A set of both a project's items and the
  deployment's belongs to that project. It stays in the deployment's scope, as
  above: its `project` and its holder are the deployment's and its decision is
  made there. A reader's inbox row and project-filtered reads show it in each of
  its `projects`, and one of the deployment's in each project that reader owns,
  at most 32, the lowest numbers first (`../../client/domain/contract/documents.md`, 1).
  A row counts once across projects, and once in each project it shows in.
- **The owner keeps the value.** The tasks child keeps the proposal as
  it keeps every proposal: its proposer, reason, where it waits and its
  state. The core keeps its set, by the proposal's number, in memory and
  in the store, restored with it, but for the values of its items of an
  application's kinds, which their connectors keep (5).
- **A project's own policy among its items** (2) is proposed as a
  change names it, its value kept by the core with the set. The
  proposal waits for that project's owners, and, beside the
  deployment's items, for the parties who own both: one decision by one
  party, refused at its entrance where no party owns both. Its owners'
  own edits of the policy while it waits make no revision and withdraw
  nothing: the review shows the policy as it is then. A project deleted
  while it waits withdraws it as no longer applying, in the deletion's
  decision, and its proposer hears so.
- **Accepting it is the change** of 2, with the accepter's choice of
  whether running work moves, checked again then. An acceptance refused
  (out of shape, judged, full) leaves the proposal pending, and the
  accepter is told why. Its revision records the proposal and its
  accepter.
- **Its proposer hears the outcome** as every proposer does, a message
  in its inbox or the party's updates (../tasks/proposals.md): accepted,
  naming the revision it made; rejected, with the accepter's reason;
  withdrawn as stale, naming the newest. A run that proposed reads it
  as any message, and may propose again.
- **Stale is withdrawn.** The decision that makes a revision withdraws
  each pending proposal of settings made against an older one, as no
  longer applying (../tasks/proposals.md), and its proposer hears so; a
  fresh proposal names the newest.
- **A run's fresh one replaces its own.** A run whose task's proposal of
  settings waits may propose settings again: admitted, it withdraws the
  one that waits in the same commit, whose set goes as any withdrawn
  set does (../tasks/proposals.md). Refused, by a stale revision, a bad
  item or a full bound, it changes nothing. A party's proposal is not
  replaced.
- **Reviewed against its own revision.** A pending proposal is always
  against the newest, the stale ones withdrawn, so the core serves its
  review from the current settings: for each item, the setting, what
  becomes of it (set, removed, reset), what it is now (its value, and
  whether it follows its default or the change of which revision, or
  that it is not offered) and what it would be (the value set, none, or
  its default's), an application's items with their sizes alone, the
  application serving their values (5), and a project's policy as
  it is now and as it would be, each whole. A reviewer never compares against a later revision
  than the proposal's; once decided or withdrawn, a proposal carries its
  items alone, and an accepted one names the revision it made
  (`../../client/domain/contract/documents.md`, 1).
- **Bounded:** pending proposals of settings in the deployment are a
  limit of their own, each priced at a whole set (8), beside the
  tasks child's limits on pending proposals.

Each settings proposal naming an application's items receives a
deployment-unique `u64` key when it is made. Connector asks to keep, judge,
accept or drop its values use this key, since proposal numbers are only
unique within a proposer. The settings review document exposes the key with
the set so the client can name the application's proposal-value read
(2026-11-04).

A review of an application's item carries its current state, size and
charter references. A set gives the size it would set; a removal gives none;
a reset says it returns to its default without giving that default's size.
The core keeps no separate default size for this review, makes no connector
ask during it, and supplies neither zero nor an unknown-size marker. The
application's own reads serve its default and size (decided 2026-11-03).

Deleting a project withdraws pending settings proposals naming its policy
before the project state changes. Those proposals are a subset of the
deployment's pending settings sets, at most `settings.proposals`, so their
writes fit `settings::withdrawal_writes`, the allowance already included in
the settings write bound checked against `journal.writes`. Message-number
capacity is checked before any withdrawal or deletion mutation; this
requires no additional limit or root-to-core allowance (decided 2026-11-03).

## 7. In the store, and at a start

- **What the store keeps** (the core's settings family,
  `../../protocol/store/README.md`, 4.4): the defaults the newest revision was
  made with, setting by setting; the changes, each a setting's value whole or its
  removal, with the revision that made it, and of an application's
  kind its state and size alone, its value the connector's (5); a
  record per revision, with
  its number, the revision it was made against, when, who decided it (a
  party, the release, the first start, a provider's list), the proposal it came from,
  whether it moved running work, and its items with the values they
  set and what each replaced (10), an application's items with
  their sizes alone; each charter
  version a held revision reads and the newest does
  not; and pending proposals' sets. The tasks family keeps each
  task's revision, live and ended, and each claim's.
- **History** is the revision records, kept for every revision held
  and for the latest ones within the history limit (8); the decision
  that makes a revision, or that ends the last holder of one, erases
  those that fall out. A version is restored by a proposal of the
  settings as it left them (10). History is read from
  the store, a revision by its number and the records a page at a
  time, newest first, each with what it changed; a revision erased is
  answered as forgotten, naming the oldest kept
  (`../../client/domain/contract/README.md`, section 4). The store keeps the
  history's extent, the oldest revision it holds, among the live
  settings. Each commit that makes a revision, and a start whose history
  limit was lowered, erases from the oldest up to the limit's cutoff,
  within the settings commit's write budget, and records the new
  oldest. A revision below the cutoff is answered as forgotten even while
  a row that commit had no room for remains; later revisions' commits
  erase the rest.
- **Settings writes** are checked at configuration against the store's
  commit edge (`../../protocol/store/records.md`, 4).
  `jig_domain_core::settings::writes_bound` bounds the first start's or
  a revision's writes; it must fit `JournalSizes.writes`, which supplies
  the store's `commits.writes`. An overflow or a bound past that edge
  refuses the configuration.
- **A start** reads the settings right after the deployment's
  numbers, before any other record of the core's (`restart.md`, step 1):
  the stored defaults, the changes, the held revisions' records and the
  charter versions. It then compares the
  defaults it is given with those stored:
  - *a first start* makes revision 1 from its defaults, recorded as the
    first start's;
  - *defaults that differ* are stored, and when they change the current
    settings (a setting no change is over), they make a revision
    recorded as the release's, with no proposal, once the core's
    records are restored and every connector has said which of its
    kinds' defaults differ (`restart.md`, steps 1 and 2; 5), with the
    connectors' new defaults saved in it; like any revision, it
    withdraws stale proposals of settings (6); new work takes it,
    running work keeps its own. A model a change is over stays as that
    change left it, so one a person turned off stays off, and a release's
    new model comes in at its default. An overlay masks nothing: a
    charter's default that differs under its overlay changes the
    current settings, the overlay laid over the new default
    (`overlays.md`, 4);
  - *the current settings are checked whole* (1): a start whose
    settings fail is refused, naming the setting, the change over it if
    any, and what fails, as a document failing its check always was;
  - *past limits:* held revisions or charter versions past limits
    lowered since, or a release's revision that would pass them, refuse
    the start, saying which, as a live set past lowered limits does. A
    task or a claim naming a revision the store holds no record of is a
    link that cannot be made, and refuses the start (`restart.md`).

  A saved project policy above the current policy byte budget refuses the
  start naming its project and policy; the named restore failure is retained
  until the shell reports it.

  All of it, the release's revision included, is committed before the
  core admits anything.

  The release's withdrawals are in its decision (6), so a crash
  leaves the revision and its withdrawals both or neither: a start after
  the commit finds the revision made and the proposals withdrawn, and
  makes no second one; a start after a commit that was lost makes them
  both. They run at `Phase::Link`, with every link made. The decision
  holds their records: the tasks child's, the core's set erased, and a
  connector's dropping of an application's values, a settings ask like
  the release's others. The people child's projections of them follow
  in memory: it takes a projection event once its census is whole, at
  its link and again once its start has ended, never while loading or
  sweeping, and a restart rebuilds them from the records. What acts on
  them waits, by `restart.md`: a proposer left due is activated when the
  tasks child is told it is restored, and a word left
  for a live run is relayed after that run's next turn, as any restored
  task's are. Nothing is activated or relayed from the decision itself,
  so a first start and a second after a crash do the same.
- **The document's members as defaults.** An application that passes
  no defaults gives the document's `llm` models, `charters`, `templates`, `policies`
  and `rules` (`../../protocol/configuration.md`, section 3), and the
  store's changes lie over them as over any defaults. A deployment that
  restarts with the same document and no change behaves as it did; a
  document edited between starts gives the release's new defaults
  (above); a stored change wins over the document's member; a reset
  returns the setting to the document's value.

## 8. Bounds

- **Two limits of the core's, sized at the start**
  (`../../protocol/limits.md`): the *revisions held*
  besides the newest, and the *charter versions held* besides the
  newest's. A held revision costs its record's head, never its items'
  values; a charter version costs a charter at its bound (the journal's
  run bytes and the core's part of a charter), the tasks child's models
  for it and the fleet's needs. The core's worst case prices both, and
  the tasks child's and the fleet's tables are sized for the charters
  and the versions together: the tasks child's for twice the charters
  and every version. A release's decision adds two journal writes per
  held revision, one per charter version and one more. The settings
  restore's `SettingsCommit { room }` carries its write budget, not a
  per-event journal reservation; the commit stays within the store's
  write limit.
- **The current settings** cost the defaults and the changes, 64 models
  among them, each within its kind's bounds (1): at most two whole sets, priced so;
  of an application's kinds, the core prices only what it keeps of
  them, and their connectors the values (5).
  The fleet keeps, for each connected host, what its hello declared for
  its endpoints and its agent's bounds, its reply bound among them,
  priced by the endpoints' bound, so that it can judge the host against
  changed needs and name its runs' reply bound (`fleet.md`).
- **History** is a third number, of revision records the store keeps
  beyond those held, within the store's fill (`../../protocol/store/README.md`,
  6.2). The core holds none of it.
- **Proposals of settings** bring a fourth limit, of pending ones, each
  priced at a whole set (6).
- **A change fits one request:** its items within the clients'
  component's message bytes (`../../protocol/client/README.md`, section 10). A
  set larger is several changes, each against the revision the last
  made.
- **A full bound names its holders:** a change refused as full names
  the oldest held revision's live tasks, and moving them is the way on
  (4).

## 9. A charter's documents

A rule that only changes how an agent behaves stays text, where the
person wrote it, and reaches the runs that should follow it, those
with no workspace among them, which read no repository's own files: a
charter may name documents, which every run of the charter reads in
its brief, as they are when the run is prepared. A rule written in no
document goes in the charter's instructions.

- **A document** is a resource by its name, a branch of a connector's
  (../authority.md, section 4): the connector's number and the branch's
  segments within the authority's pattern bounds; a path within it of
  at most 256 bytes; and, optionally, a heading of at most 80 bytes,
  the part of the file to read. A charter names at most
  `tasks.charter_documents` of them (2 in Small). jig never reads one:
  it checks only that the connector is registered and the bounds hold
  (1, *shape*), and keeps the documents with the charter, versioned
  and pinned with it (3), so a run reads the documents its
  revision's charter names.
- **Planned with the brief:** the core binds each document to a
  section's token in the plan it hands the root, optional, after its own
  sections and before the task's resources' (`briefs.md`, section 1), and the root
  gives each document to its connector before the gather. A start whose
  brief's sections cannot hold the core's own most and a charter's
  documents is refused, naming both limits.
- **Gathered for each run** as a brief section of the connector that
  owns the resource (`briefs.md`, section 1; ../connectors.md, section 9): the file at
  the branch's head as the connector finds it when it gathers the
  section, one commit for the section's whole life, its cuts included,
  so a branch that moves during the preparation changes nothing of
  this run's; or its part under the heading; within the section's
  budget, which is the connector's (its brief budgets). A rule that lands reaches the
  next run, with no change of settings. A document the connector cannot
  read (no such resource, file or heading, or a read that failed) is a
  section saying so, never a run held: the documents are optional.
- **No authority** is asked of the run: the charter's owners chose its
  documents, as they chose its instructions.

A document's path is 1 to 256 bytes; its heading, when present, is 1 to 80
bytes. An empty path or a present empty heading is out of shape
(2026-11-04).

A document refusal retains the *shape* fault, the item and the setting,
and carries bounded facts naming the member, the document index (and name
segment index where applicable), the failed rule and its scalar values.
A count overflow names `documents`, its supplied count and
`tasks.charter_documents` with its configured value. Name failures name
`authority.segments` or `authority.segment_bytes`; path and heading failures
name their minimum or maximum byte bound; a connector failure names the
unregistered connector. A charter whose stored policy, its parts with it,
passes `run_bytes` (1) is refused with the same fault and facts naming
`stored_policy`, a member that is no document's: the policy's stored bytes
as supplied, the largest `u32` where they pass it, and `run_bytes` as its
maximum. These facts cross proposal answers, stored answers
and the client wire unchanged; a renderer words them without validating
the charter again.

The configuration and service start check that brief sections hold the
core's own most and `tasks.charter_documents`, naming both limits on
refusal. The core's in-code validity check does not require this relation:
fixtures may supply fewer sections. The core then plans documents in charter
order only while room remains after its own sections, omitting a document
from both the section plan and the root's document list when no room remains
(2026-11-04).

## 10. Restoring a version

Restoring a version makes a new proposal of the settings as a revision
left them, never a silent rollback: what was decided since stays in
the history, and going back is a change like any other, reviewed and
decided by the owners of what it touches.

- **Each item records what it replaced:** in a revision's record, beside
  what the item set, the setting as it was before: its value whole, that
  it followed its default, that it was removed, or that it was not
  offered. A release's revision records each of its items as having
  followed its default, which is all a restore needs of it (below).
  An application's items record, in the core's record, the state and
  size of what they replaced, and their connector keeps the value
  (below); a project's policy item records the project's policy it
  replaced, whole (2).
- **A connector keeps what its items replaced.** As a change commits
  items of its kinds, the connector keeps, beside each change it saves,
  the value the setting held before, by the revision that made the
  change, in its own family, for as long as the history keeps that
  revision's record (7), and erases them in the decision that erases
  the record; a release's revision asks it to keep none, its items
  having followed their defaults. Its worst case prices them: for each record the history may keep, a
  whole set of its kinds' values.
- **The settings as revision R left them** differ from the newest only
  in the settings a later revision changed, each as the earliest of
  those revisions found it: its record says what each item replaced,
  and a project's policy likewise. So a restore of R reads every record
  after R, which the history keeps without a gap from its oldest kept
  to the newest (7): R may be any revision from the history's oldest
  on, and an older one, even one still held, is refused as forgotten.
- **A setting that followed its default at R follows it again,** today's
  default, which a release may have changed since; the review says so.
  What a person restores is what they had chosen, and a default was
  the application's choice, which a release improves for every
  deployment that kept it (1).
- **`Restore { revision, reason }`,** a party's request in the
  deployment's scope (../people/acts.md, 1), proposes, against the newest, one
  item per setting a later revision changed: *set* to the value it
  replaced, *reset* where it followed its default or was not offered,
  *removed* where it was removed; and, for each project whose policy a
  later revision set, that project's policy *set* to the one it
  replaced. It is a proposal of settings like any (6), reviewed
  against the newest and decided by the owners of its items; the owner
  who asked for it accepts it like any other. Its reason says "restores
  revision R". An application's items take the values their connectors
  kept: in the restore's decision, each connector is given the
  proposal and its items of its kinds, each with the revision whose
  record replaced it, keeps the values it kept as replaced as the
  proposal's items, and answers their sizes and the charters they name,
  checked against its kinds' bounds alone, as a proposal's items are
  (5). Its items leave out the settings and the policy of a project
  deleted since, which went with it (../people/acts.md, 2), and the answer
  names each one left out.
- **Its decision loads the records after R** before it decides, a page
  at a time as `revisions` reads them (`../../client/domain/contract/README.md`,
  section 4), at most the history's limit of records; the decision is
  made against the newest it loaded, and a revision made meanwhile
  refuses it as stale, naming the newest, as any change against an
  older one is. A restore of a revision erased is refused as forgotten,
  naming the oldest kept, and one of the newest as empty.
- **Bounded** as any proposal of settings: one item per setting, at most
  a whole set (8), and at most one project's policy (2); one whose
  items would pass it, settings added and removed since R among them,
  is refused as too large, saying so, and a later revision can be
  restored instead.
- **Reviewed as any proposal,** each value whole within the document's
  bound or with its size; a value past it is read whole by
  `setting{kind, scope, number, proposal}`, a pending proposal's item
  (`../../client/domain/contract/README.md`, section 4), and an application's
  items from the application's own read of a proposal's items (5).

The restore's size is counted before any deleted project's items are left
out. Every item found after R counts toward a whole set and at most one
project's policy, including one that will be omitted, so the answer's
omitted items are bounded by that same set plus one policy (2026-11-04).

Restoring revision 0 or a revision above the newest is refused as stale,
naming the newest revision. This differs from an erased revision, which is
forgotten and names the oldest kept (2026-11-04).

The proposal's reason is `restores revision R: ` followed by the party's
reason. The combined text is cut to the reason's byte bound at a character
boundary (2026-11-04).

## 11. The catalogue

The current settings of kind `model`, in number order, are the
deployment's **catalogue**: what a task's LLM choice, an authority's
models and a charter name.

- **A number is a position.** Model `i` of a task's choice and bit `i` of
  an authority's models are the setting numbered `i + 1`
  (`../tasks/README.md`, section 3; `../authority.md`, section 3), so the store's and
  the wire's indices hold across a change and a release. The bound is
  `MODELS`, 64, for the deployment's life; a number no setting has is a
  hole, which nothing offers or resolves. The binary's defaults take low
  numbers and keep them; a model a person or a provider's list adds takes
  the highest free number, so a release's new default never meets one.
- **A number keeps its name.** A set naming another name than its
  default's or its stored change's is out of shape; so are a dialect or
  program its endpoint doesn't have, a name out of `valid_name`, and a
  zero window, output or price unit. An unknown provider or endpoint is a
  reference fault. Nonzero window and output values no agent's bounds
  hold make a run wait, as a charter's does (`fleet.md`).
- **A model stays for the tasks that hold it.** A *removed* item keeps the
  setting, *withdrawn*: out of the offer (`views.md`, 1) and of every charter's
  resolution, with its number and name. A *reset* returns one with a
  default to it, and one with no default is out of shape. A run reads the
  charter version of its revision (3), whose records are the models as
  they were, so a task keeps the model it has, and no change of a model
  moves one.
- **A charter's models are names, resolved.** A charter names its first
  choices, in order, and whether **the rest** follow
  (`../../protocol/configuration.md`, section 3). Its resolution is each
  name's record in the catalogue, at the effort the charter selects for
  it where it selects one (`../tasks/README.md`, section 3), withdrawn ones dropped, the main the
  first left; then, with the rest, every other model in number order,
  while it stays within `settings.alternatives` and `run_bytes`. A
  resolution with no model is a *reference* fault. What a run reads, what
  the fleet needs and what the tasks child checks a choice against is the
  resolution; a setting's value, a read and a proposal keep the charter as
  set, so that a new model needs no charter's change.
- **A change of a resolution is a change of its charter.** A decision or
  a release whose models change a charter's resolution makes that charter's
  next version at its revision, and holds the old one (3 and 8),
  though no item names the charter and its stored value stays. A change of
  a model's `picker` alone changes no resolution, and so no version.
- **What the charter left out:** a model past a charter's bounds stays
  in the catalogue and off that charter. `me` says how many, for a
  project's chats (`../../client/domain/contract/documents.md`, 1).
- **Who hears.** A decision or a start's restore that changes the
  catalogue asks the root to tell the clients' component and the store,
  before its commit (`Ask::Catalogue`; `../../protocol/store/records.md`);
  the tasks child hears the efforts with the charters (2), the fleet its
  needs, and the views mark every `me`.
- **An owner's act.** `ChangeModel` (`../../client/domain/contract/requests.md`, 2)
  turns one model's picker on or off, or adds a model by name, a provider
  and a name, as one change at the newest revision, running work staying,
  decided as 2's. Adding checks the name's shape alone (`valid_name`),
  since only a first run shows whether the provider serves it. It takes the
  highest free number and the provider's dearest stock model's endpoint,
  window, output and prices, marked estimated, so that a budget stops
  early rather than late, with no efforts and on the picker. A name the
  catalogue holds is out of shape, unless withdrawn, which it brings back.
  A provider with no stock model takes its `endpoint` and the catalogue's
  dearest stock model's figures.
- **A person's removal of a list's model** makes it a person's: the item keeps
  the setting withdrawn, with `source` person, so that a list naming it again
  doesn't bring it back (12). Adding it by name does.

## 12. A provider's list

A read of an account's list (../accounts.md, 3.7) comes to the core as an answer.
The core alone turns it into settings, as the one decider that's neither a party,
the release nor a first start. It moves no running work.

- **Placing a read.** The core gives the fleet the account's grant and the
  endpoint its provider names (`../../protocol/configuration.md`, section 3). The
  fleet holds it until a connected worker's bounds hold that endpoint, sends it to
  the one with the most free slots, one read in flight per worker, and holds it
  again if that worker's channel is lost before it answers. A renewal reaches a
  read still held. Host rule is not asked (../accounts.md, 3.4): no tool runs on a
  read. With no worker connected, a read waits and nothing says it failed.
- **What it keeps.** Of the rows, in the provider's order: a name that a `keep`
  pattern matches and no `drop` pattern does, and that `valid_name` holds. The rest
  never reach the store.
- **A name new to the catalogue** takes `new_model` (11), `source` list, off the
  picker, and the highest free number, in the provider's order, each seeing the
  last. Its `first_read` is this read's time and never changes, so it's "new" once.
  Its figures are those of `new_model`, and where the list says them, its shown
  name, its window and output, no larger than `new_model`'s, since the binary's
  defaults bound what a run is given, whatever a provider allows, and its
  efforts, its default effort being `high` where it lists that, else its first.
  Its prices stay estimated.
- **The catalogue's room.** Past `MODELS`, a name takes no number, and the
  account's `left_out` counts it, so the provider's first are kept.
- **A name gone from every list:** a model of this provider with `source` list
  that no account's last good read holds, this one's included, is withdrawn: out of
  the offer and every charter's resolution, its number and name kept for the tasks
  that hold it. A withdrawn one with `source` list that a read names again comes
  back. A read never touches a model of the binary's or a person's, nor another
  provider's, nor a kept row's picker, prices or figures; the accounts' names narrow
  those (../accounts.md, 3.7).
- **What a read decides.** The new, gone and returned models are one change at
  the newest revision, `Running::Stays`, decided as 2's, by the provider's list
  (`Decider::List`, naming the provider, in the revision's record). A read that
  changes nothing makes no revision. Like any revision it withdraws stale proposals
  (6) and makes the versions of charters whose resolution it changes (11).
  Refused, for held versions past their limits or a whole set out of shape, it makes
  none, and the attempt fails as the catalogue's room.
- **One commit.** The account's record, the revision and its versions commit
  together, so that a crash leaves both or neither; `settings::writes_bound` prices
  the record's one write.
- **What it marks.** A read marks the `accounts` document of whoever holds the
  account, and the deployment's, since its times moved. `me` is marked only when
  what it says moved: the decision changed the current settings, or the account's
  `names` changed, which narrows its offer, or its displayed `left_out` changed
  (`../../client/domain/contract/documents.md`, 1). A day's alarm alone marks nothing, nor
  does a failed read, a repeat, a refused act or a read still held.
- **A failed read** changes no setting and no name (../accounts.md, 3.7).
