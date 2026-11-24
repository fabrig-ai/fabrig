# The engine's settings: proposals

## 1. A proposal of settings

- **Proposed by a member or a run.** A party with a role in the
  deployment or in a project proposes a set with `Propose` (../people/acts.md,
  1); a run with its `propose` call, when its charter gives it the
  settings family (`executors-tools.md`, 1), which reads the settings and proposes a set;
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
