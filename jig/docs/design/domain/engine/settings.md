# The engine's settings

A deployment's charters, templates, the policies its projects are made
from, its rules, its LLM models, and an application's settings of its own are its
**settings**: the binary's defaults, with the changes the store keeps
over them. A change is one decision; each makes a revision; a task's
runs read their charter from the revision the task was made in. This
section is the mechanics; the model is ../model.md, section 13; who asks
is ../people/acts.md, 1; what crosses to a connector is ../connectors.md,
section 17; the records are `../../protocol/store/keys.md`, 1.4.

## 1. What a setting is

- **A setting is named by its kind and its number,** and, for an
  application's kind kept per project, its project:

  | Kind | Number | Value | Read |
  |---|---|---|---|
  | charter | the charter's, 1 to 64, the defaults' 1 to 32 (below) | a charter whole: its instructions as text, its documents (9), its models, tool families, hosts, waiting, resume and default budget (`../../protocol/configuration-engine.md`, section 1); or, over a default, an overlay: the fields it replaces and the text it adds after the default's (`overlays.md`) | at each run's preparation, from the run's revision (3), an overlay laid over its default |
  | template | the template's, 1 to 64, the defaults' 1 to 32 (below) | a template whole (`../../protocol/configuration-templates.md`, 1) | when `me` is served (`views.md`, 1): the newest |
  | policy | the policy's, 1 to 64, the defaults' 1 to 32 (below) | a whole policy in a seed's format, a project made from it copying it (../model.md, 6.1) | at each `MakeProject`: the newest |
  | rules | none: one setting | the deployment's rules, whole (../authority.md, section 6) | at every decision: the newest |
  | limits | none: one setting | overrides of jig's limits, at most 256, each a limit's dotted name, a value and a reason of at most 120 bytes, ascending by name; none is the application's stock (`../../protocol/limits.md`, limits as settings) | by the application, at its start, never by the core |
  | model | the model's, 1 to 64, the defaults' 1 to 32 (below), never given to another name (11) | a model whole: its name, shown name, LLM provider, endpoint, window, output, efforts and default effort, prices, speeds and default speed (`settings-catalogue.md`, 1), description, whether a picker shows it, whether it is withdrawn, where it came from (the binary, a provider's list or a person), whether its prices are estimated and when a list first gave it | at each decision and when `me` is served: the newest; a run reads it from its charter (11) |
  | worker | the worker's, 1 to the fleet's configured workers, never given to another name (`workers.md`, 2) | a worker whole: its name, its places, its secret's digest, whether it is withdrawn (`workers.md`, 1) | at each opening and each hello, and when a deployment document is built: the newest (`workers.md`, 4) |
  | provider | the LLM provider's, 1 to 16, never given to another name (`llm-providers.md`, 3) | a provider whole: its name, its sources, its list's patterns, its terms and its endpoints, whether it is withdrawn (`llm-providers.md`, 1 and 2) | by the check and the keeping ask: the newest; by what dials, signs in and lists: the providers the start ran, so a change applies at the next start (`llm-providers.md`, 5) |
  | an application's kind | the application's | bytes jig never reads, kept by the connector that owns the kind (5) | by the application, from that connector: the newest |

- **The defaults are the binary's.** The application passes them as
  values when it builds the core (`Config`'s defaults): its models, charters,
  policies and rules, its templates, jig's chat and goal among them
  where it wants them, which jig gives as values made for the charter
  they run (`../../protocol/configuration-templates.md`, 1); its own kinds'
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
- **Two ranges of numbers.** A charter's, a template's, a policy's and a
  model's number is 1 to 64, `NUMBERS`: 1 to 32, `STOCK_NUMBERS`, are
  the defaults', and 33 to 64 a deployment's own, both constants of
  `jig-domain-people`'s settings vocabulary. So a release's new default
  never lands on a number a deployment's new item takes; one landing on
  a kept item's (below) refuses its start.
  - *A deployment's own item* is a setting of those kinds that no
    default has and that the settings offer. One below 33 that an edit
    offers anew is refused. So
    is any offered setting of those kinds past 64, a default's too, and a
    default past 32: a start refuses defaults that break the line as it
    refuses any default out of shape.
  - *Kept where a store holds it:* a deployment's own item below 33 that
    a stored change sets, made before the line was drawn, stays at its
    number, and so does every reference to it: a template's executor, a
    task's charter and revision, a goal's template. An edit may set it
    again or remove it, and its being there refuses no other edit; a
    start warns of each (7). Nothing moves it: moving it as the store is
    read would rewrite every family that names it (tasks, revisions'
    items, held charter versions, pending proposals, connectors' kinds
    numbered by a charter or a template), where keeping it moves no
    stored format. An owner moves one in one change: its value set at 33
    to 64, its references pointed there, the old number removed.
  - *A change over a default* stays at the default's number, set,
    removed or reset as any change is, whatever its number.
  - *A removal or a reset* at a number no default has offers nothing,
    and is never refused; nor is a withdrawn model, which keeps its
    number for the tasks that hold it and offers nothing (11), so that
    an owner can always clear what is out of range.
  - *The kinds' counts stay counts:* `tasks.charters`, `templates` and
    the policies' limit cap how many a kind holds, never its numbers.
  - *Judged by the one check* below, as its fault *range*: at a set, at a
    proposal's entrance and its decision (2 and 6), each judging what its
    items set and nothing they leave alone, and at a start (7), which
    names every default out of its range and every change past 64, and
    warns of the kept items. An application's own
    kinds numbered by a charter or a template bound their numbers in
    their connectors (5), as their owner designs.
- **Names.** A charter, a template and a policy has a *name* beside its
  number, or none: 1 to 32 bytes, `NAME_BYTES`, of lowercase ASCII
  letters, digits and `-`, starting with a letter, so that a name is
  never a number; `NAME_BYTES` is a constant of `jig-domain-people`'s
  settings vocabulary beside `NUMBERS`. A template's words for a person are its
  *shown name* (`../../protocol/configuration-templates.md`, 1), as a
  model has a name and a shown name (11). The number stays the
  setting's key: requests, records, tasks and the references among
  settings keep numbers, and whoever takes a name from a person
  resolves it to its number from what it is served: `me`'s templates
  and charters, and the settings' inventory
  (`../../client/domain/contract/documents.md`, 1).
  - *A default's name is the default's:* a default may carry one, and
    the defaults' names are unique within their kind; a start refuses
    defaults that break it, as it refuses any default out of shape.
    Within a kind, a name a default carries is reserved to it, whether
    the settings offer that default or remove it.
  - *At a default's number* the name is the default's: a change that
    sets the setting carries that name or none, and none reads as the
    default's, so a change stored before names reads with it; an
    overlay keeps its default's. Another name there is refused.
  - *A deployment's own item* is set with a name or with none. One with
    none, set without one or stored before names, is reached by its
    number alone until a change names it. Its name is refused where a
    default of its kind has it, or another item of its kind the
    settings offer. A change setting it with another name renames it:
    records keep numbers, so a rename moves nothing that ran, and a
    name given up, by a rename or a removal, may be taken by another
    item of the same kind.
  - *A newer build's default* may carry a name a stored own item has. The
    start offers that item with no name and warns of it, as of a kept
    item, so the name never refuses a start; an owner names it again.
  - *Judged by the one check* below: a name out of its form is *shape*;
    one taken, by a default or by another item of its kind, and another
    name than the default's at a default's number, are its fault *name*,
    judged as *range* is, at a set, at a proposal's entrance and at its
    decision, each judging what its items set and nothing they leave
    alone, the change's later item refused where two of its items take
    one name. A client words *name* from the setting's kind and number
    alone, the number saying which: "template 2 keeps the stock's name",
    or "template 33's name is taken, by a stock template or another of
    your own".
  - *Where they travel:* the values of the three kinds carry their name,
    in the store and on the client's wire, a value stored at an earlier
    layout reading with none; the engine–worker wire carries none,
    since a run is given its charter's text and fields, never its name.
- **Checked whole, by one check.** The core checks settings as a start
  has always checked the document (`../../protocol/configuration-engine.md`, section 1, and `../../protocol/configuration-templates.md`, section 1 and `../../protocol/limits.md`
  ): each value in its shape and within its bound
  (a charter's instructions within the journal's run bytes, its budgets
  above its reserve and holding one of its default model's costliest
  completions, at the prices of the speed it runs that model at
  (`executors.md`, 1), its models configured; a template's charter offered and
  its grace within the tasks child's span; a model in 11's shape; a policy within the rules,
  its roles well formed, its overrides' templates offered, its owner
  role able to be a root's requester), each number within its kind's
  range (above), the counts within their limits
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
  requester), *name* (a charter's, template's or policy's name taken, by
a default or another item of its kind: Names, above), *range* (a
charter, template, policy or model numbered
  out of its range, above: each client's words, and an application's
  for its agents' tools, say which line it crossed from the setting's
  kind and number alone, naming the kind and both ranges:
  "template 9 is among the stock's numbers, 1 to 32; your own templates
  take 33 to 64", or "template 70 is past the templates' numbers: the
  stock's take 1 to 32, your own 33 to 64"). Within the check the range
  is judged first, so that a set out of its range is refused as *range*
  whatever else it breaks.
- **The limits are a setting the core keeps and never applies:** they
  size the core's tables at the start, so the core checks the limits
  setting's shape alone (each name's bytes, the order, each reason,
  their count; fault *shape*), keeps it, shows it, and the application
  reads it at its next start (`../../protocol/limits.md`, limits as
  settings).
- **What is not a setting:** the hosts but the workers (`workers.md`), the connectors, the
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
  (../authority-budgets.md, section 1), and every later decision under it. It makes
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
  and a template removed leaves its number unused (`../../protocol/configuration-templates.md`,
  1). A task made after the change reads its
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
  own (../tasks/lifecycle-endings.md, 2), moves as each of its runs begins to be prepared,
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

Moved to [`settings-application.md`](settings-application.md), section 1.

## 6. A proposal of settings

Moved to [`settings-proposals.md`](settings-proposals.md), section 1.

## 7. In the store, and at a start

Moved to [`settings-store.md`](settings-store.md), section 1.

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
  beyond those held, within the store's fill (`../../protocol/store/growth.md`,
  1.2). The core holds none of it.
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
  at a time as `revisions` reads them (`../../client/domain/contract/reads.md`,
  section 1), at most the history's limit of records; the decision is
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
  (`../../client/domain/contract/reads.md`, section 1), and an application's
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

Moved to [`settings-catalogue.md`](settings-catalogue.md), section 1.

## 12. A provider's list

Moved to [`settings-catalogue.md`](settings-catalogue.md), section 2.
