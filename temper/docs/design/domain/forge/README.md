# The forge connector

Provisional, 2026-10-04, revised 2026-11-02. How temper drives a forge,
Forgejo or GitHub: the forge as a connector on jig's contract (jig's
`domain/connectors.md`), with its resources, effects, topics, procedures,
requirements and reads; a forge connected or not, and removed; adopting
repositories, and what the picker offers; keeping up; a change's way to
landing and the requirements its merge meets; landing queues, a forge's own merge
queue among them; updates and conflicts; issues; following another
project's branch (`follows.md`); depending on another project
(`dependencies.md`); a goal's reds, the failures its branch carries
(`reds.md`); the outbox's room, and the line of writes waiting for it
(`room.md`); catching up after a restart, while the engine serves
(`catching-up.md`); and the ways temper takes part in what
others own. Its protocol layer, the component that
makes each call to a forge and hears its deliveries, is
`../../protocol/forge/README.md`. What is still open is listed in section 11.

## 1. In one page

- **What the forge holds:** git; pull requests as the unit of a change;
  CI; branch protection, as a second net; and a record people can read.
  From what temper owns there, only CI, people's approvals and drift
  come back: no comment on it is an input, and no label marks what
  temper tracks.
- **Live work comes from the store.** The connector holds a working set
  of what the forge owns of the resources live tasks name (pull
  requests' heads and states, CI on those heads, landing branches'
  tips), kept up by listing what changed, sooner on a webhook's hint.
- **Quiet on the forge.** temper reads what a reader needs, when it can
  have moved (`keeping-up.md`, 2 and 3): a branch nothing has pushed
  waits on its push, with `fallback` as its only poll; a hint wakes only
  the changes it can concern; a gather reads each commit's CI once and
  shares a fresh read its fellows ask with it; the repository's own
  facts are read once and kept; pages hold as many real rows as their
  bytes allow; and a read of something unchanged costs a `304`.
- **The change procedure** takes a change from produced to landed:
  produce, open, CI, gates, its turn in its landing branch's queue, an
  update from its base when it lacks the base's tip, and a merge at
  exactly its head; CI and the update only where its landing branch's
  rules ask them (`change/gates.md`, 2). Agents are woken for what needs judgement: producing,
  repairs, reviews, conflicts.
- **Landing rules are requirements.** A project's rules say what a merge
  into a branch needs: CI passed, the head up to date with the branch,
  gates, people's approvals. The forge judges each from its own facts at
  the merge's exact head, which the forge's merge is conditional on (`change/gates.md`, 2).
- **By queue or by turn.** A project lands by the queue below, its
  default, or by turn: landings across repositories, prepared, proved by
  receipts and fast-forwarded in the project's turn (`../landing.md`),
  which replaces steps 3 and 5 to 7 of `change/README.md`, 2, and `queues.md` and `updates.md`, section 1, for
  its changes.
- **Landing queues.** Changes ready to land in a branch wait in line, by
  their goal's priority then by when each became ready; only the first
  is brought up to date and merged. Where the forge keeps a merge queue
  of its own on the branch, temper's queue decides only what enters it
  and when; the forge lands it, or ejects it, which is a repair (`queues.md`, 1).
- **Connected or not.** A forge is connected while temper runs, or not:
  the root knows which from the routing's events, rebuilt at each start.
  A forge not connected answers every call `NotConnected` at once, and
  adopts nothing. Its writer, given when it connects, is the forge
  client's own record, restored at start (`adoption-connection.md`, 1).
- **A forge can be removed,** whole, by the deployment's owners: ended
  at once, its live work cancelled, its secrets and records erased, its
  box group stopped and its directory removed, each act kept until the
  removal is whole; nothing on the forge is undone but what temper ran
  (`adoption-connection.md`, 1).
- **Adoption picks from what temper can reach:** a GitHub App's
  installations, what a connected Forgejo's API token can push to, or
  any repository of a managed or owned Forgejo, where adoption first
  gives temper's user its access (`adoption.md`, 6, 7). What it reads of the landing
  branch's rules, and the merge style, it says in words (`adoption.md`, 8).
- **Updates are lazy and mechanical.** A change takes its base when it
  is first in line, about to get a run, or at its check with its head
  already behind (`change/README.md`, 2, step 3), through the forge's own update; an LLM
  only resolves a conflict, in a workspace where the merge is in progress.
- **Every landing is news** for every goal working in that repository;
  it wakes a goal's coordinator only when it overlaps the files of that
  goal's changes.
- **Issues stay, one per goal,** as a projection people can read: the
  goal, its plan as a task list, and its milestones. Pull requests refer
  to it.
- **Landed drafts:** a draft in a repository, which its writer says is
  ready to start a goal, read once at the landing branch's head and kept
  with its commit, its chat and its goal once started (`drafts.md`).
- **Plugged in as it is.** temper adopts repositories with whatever
  access it is given; webhooks, labels, protection and CI are all
  optional. Nothing is forced: pushes are fast-forwards, and merges are
  made at exactly the head decided on.

Documents:

- `adoption.md`: adopting repositories, their rules, workflows and landing branch.
  - `adoption-connection.md`: a forge's connection, access state and removal.
  - `adoption-person.md`: a box's person on its forges.
- `keeping-up.md`: the working set, fresh reads and request budget.
- `effects.md`: effects and recovery across restarts.
- `topics.md`: topics and landings as news.
- `updates.md`: lazy, mechanical updates.
- `change/README.md`: the change procedure and its documents.
- `queues.md`: landing queues.
- `branches.md`: task branches and held resources.
- `issues.md`: goals' issues on the forge.
- `attention.md`: what reaches a person.
- `follows.md`: following another project's branch.
- `dependencies.md`: dependencies between projects.
- `drafts.md`: landed drafts.
- `reds.md`: a goal's reds.
- `room.md`: the outbox's room.
- `waits.md`: a procedure's waits, as the core's records, and their bounds.
- `walkthrough.md`: a change through the forge connector.

## 2. What it gives

- **Resources,** named `forge` `<host>` `<owner>` `<repo>`, then:
  - `branch` and the branch's own segments: `… branch temper 7 c42`;
  - `pull` and a number; `issue` and a number;
  - nothing beyond, for the repository itself.

  The host, the owner and the name are those the repository was adopted
  with (`adoption.md`).
- **Holds** (jig's `domain/connectors.md`, 3.3): a branch under temper's
  prefix takes an exclusive hold, held by the task it is named after: a
  change, a task's runs, a landing, or a goal for its own branch (`branches.md`, 1).
  A hold already taken makes its task wait (jig's `domain/tasks/holds.md`,
  2), but a goal's own branch's, which refuses, naming the goal: no
  two goals share one. Landing branches, pull requests, issues and the
  repository itself are shared and take none: merging into a landing
  branch is ordered by its queue (`queues.md`). A goal's own branch is a
  landing branch too, held and ordered by its queue at once (`branches.md`, 1).
- **Symbols** (jig's `domain/connectors.md`, 3.1): 1, *its own goal
  branch*, offered on a tracked task only (`branches.md`, 1); a batch naming it for
  an untracked task is refused, naming the symbol.
- **Effects:** open, edit, update, merge, close and reopen a pull
  request; post a review, a comment or a commit status; create, edit,
  close and reopen an issue; set reviewers; create and delete a branch
  (`effects.md`, section 1). A run's push is the worker's, over git, under its task's
  hold (`../worker/checkouts.md`).
- **Kinds,** for authority (jig's `domain/authority.md`, section 4),
  numbered: 1, reads; 2, a run's push; 3 to 15, and 17, the effects, one
  per row of `effects.md`, section 1's table (16 and 17 are also a landing's,
  `../landing.md`, 11). No kind implies another: each write a task means
  to make needs a grant of its own kind. A change is given kind 17 on its
  branch wherever it is given kind 2 there, since a push of the same head
  would run the same CI (`change/README.md`, 2, step 3). Landing rules govern kind 4,
  the merge (`change/gates.md`, 2). None forces a push (`change/gates.md`, 2).
- **Topics:** landings on a branch; CI on a head; a pull request's
  state; the comments and reviews on a participating object (`topics.md`, section 1).
- **Procedures:** the change (`change/README.md`), and the follow, whose
  task makes a follow change for each landing it keeps (`follows.md`).
- **Requirements:** CI passed, up to date, a gate, an approval, which
  landing rules name as judges of the merge (`change/gates.md`, 2).
- **Reads an agent may call:** each of the forge client's reads but
  locating a repository: a page of the issues and pull requests changed
  since a time; an issue or a pull request and its comments; a pull
  request, or the one for a head and a base, its reviews, a review's
  remarks, and its files; a branch's head, and the branches; the
  statuses and checks on a commit, and a failed job's log; a comparison
  of two commits; a file at a head; a repository's protection, settings
  and collaborators, and a user's permission. Each answer is cut to its
  limit. The host tools that serve them, and those for effects and
  topics, are `../agent/tools.md`.
- **Effects offered to agents** (jig's `domain/connectors-effects.md`, 1.5):
  creating an issue, posting a comment on an issue or a pull request,
  posting a commit status. What the change procedure owns (opening,
  updating, merging, reviewing as a gate) is not offered: a run that
  wants a change landed delegates it.
- **Brief sections,** of three kinds, each at the change's head: the pull
  request, its files and what differs from its base, or, for a
  resolution and a semantic repair, what landed in the base, with the
  files in conflict for a resolution; CI's failures, each with its
  failed step and the end of that step's output; reviews' remarks. And
  a fourth for any run, a document its charter names, at its branch's
  head (`../../protocol/configuration/settings.md`, 6).
- **Workspace items:** for each repository a run works in (../root-translation.md,
  2.1), a checkout: where it starts (a landing branch or the
  repository's default, a branch, saved work, or a merge in progress),
  whether it may be written and the branch it pushes, and the git
  identity it is fetched and pushed with (`../worker/hosted-runs.md`, 1.1;
  `../../protocol/credentials-git-identities.md`, section 1).
- **Documents and parts** (jig's `domain/connectors-facts.md`, 1.5): what
  temper's client shows of the forge, made from the connector's records
  and working set: a change's way to landing, its result and its hold,
  the queues, the repositories and their landing rules, goals' issues,
  findings and the forge itself (`../../client/forge/served.md`).
- **What temper owns there** (jig's `domain/model.md`, 8.1): what it
  creates, recorded in the store by key. On the forge, ownership shows
  as authorship, by temper's own user (on GitHub, its App's bot user),
  and as temper's branch prefix, which branch protection can reserve to
  it.

## 3. Structure

```
temper-engine-domain-forge              the connector's top: resources and holds, adoption, topics, judges, briefs, routing; the effects and their identities
├── temper-engine-domain-forge-vocabulary  what crosses the top's edge: its events, requests and documents, records and keys, limits, settings, and the values its views show
├── skein-forge-client                  the working set, keeping up, fresh reads, the request budget, effects' keys, write lanes and recovery (skein's forge-client.md)
├── skein-forge                         the vocabulary the top and the client share: calls, answers, errors, hints (skein's forge/vocabulary.md, section 1)
├── temper-engine-domain-forge-change   the change procedure and the landing queues: decisions over facts
├── temper-engine-domain-forge-landing  the landing by turn: decisions over a landing's state (landing.md)
├── temper-engine-domain-forge-issues   the goals' issues: what each projects, when to write it
└── temper-engine-domain-forge-tiers    the tiers' verdicts: decisions over a tier's record, its run's report and receipts (after-landing.md)
```

- **The top** keeps the forge's resources as the project knows them
  (roles, what temper may do there), the holds on branches, the topics
  and their subscribers, the changes' procedure states, the issues'
  projection states, and the landing rules' gates and judges' parameters.
  It routes between its children and translates to and from the root.
  It names the client's live resources, owns the effects and their
  identities (`effects.md`, section 1), and hands the client their bytes and its
  namespace, the deployment's id, before any restored entry or live
  resource, by which time the client has restored each forge's writer
  from its own record (`effects.md`, section 2). It gives the client a forge's
  writer when the forge connects, and keeps each forge's connection as
  the root tells it (`adoption-connection.md`, 1).
- **`skein-forge-client` is a capability:** it knows the forge's API as
  `skein-forge`'s vocabulary states it, the working set, the request
  budget, and how a write is made once. Its owner, how it is driven and
  what it never knows are skein's `forge/client.md`, sections 2 and 9.
- **`change`, `issues` and `tiers` are policy:** given a change's state
  and the facts, a goal's state, or a tier's record and what its run
  reported, they say what is due. They keep nothing between calls; their
  states are the top's, committed in the store.

The top's vocabulary is a crate of its own, below the top
(`temper/docs/design/draft/archive/crate-graph.md`, 5):

- **It holds what crosses the top's edge:** every item a crate other
  than the top names (the events, requests and documents of its
  boundary with the root; the stored records and keys; the limits and
  configuration; the settings; the values of its views, findings,
  follows, panels, drafts and statuses; the effects' descriptions and
  words), with every item that their definitions, their inherent
  methods and their trait impls reach. None of it names `Domain`, its
  state, or a type only `Domain` keeps. A file goes whole where the
  rest of it reaches nothing but the vocabulary and itself, as
  `boundary.rs`, `settings.rs` and `limits.rs` do. Otherwise its
  vocabulary goes and the rest stays, so a view's values go and the
  `Domain` methods that build them stay.
- **`temper-engine-domain-forge` keeps `Domain`:** its state, its
  construction, `step`, `resume`, `fire` and `writer`, every method and
  function that reads or changes it, the helpers only they use, and the
  step tests that drive it. `Domain` isn't cut, since 35 of the top's
  files add methods to it (`temper/docs/design/draft/archive/crate-graph.md`,
  5.2). So the top stays over the size bound
  (`tools/crates/design.md`, 3.1) until its state is grouped into owned
  parts (`temper/docs/development/debt/D013-forge-one-object.md`).
- **The two share one module tree:** an item keeps its module path, so
  `temper_engine_domain_forge_vocabulary::settings::Value` is
  `temper_engine_domain_forge::settings::Value`. Each of the top's
  modules re-exports its namesake in the vocabulary whole, and the
  top's root re-exports the vocabulary's root, so a caller builds
  through the same paths in either crate. An item the top reaches
  across the cut is public in the vocabulary.
- **Each crate names the one it needs:** a library whose own code
  needs only the vocabulary, and none of whose other normal
  dependencies reach the top, depends on
  `temper-engine-domain-forge-vocabulary`: the store's
  (`temper-protocol-store`), `temper-engine-domain-ways` and
  `temper-fake` among them. Its tests may still use the top. Every
  other crate names `temper-engine-domain-forge`. So a change to how
  the top decides recompiles the top and what names it, not the
  store, and a build from scratch compiles the top beside the store.
- **Its tests** are the step tests of its items that build without
  `Domain`. A test file that mixes them with `Domain`'s stays with the
  top whole, and the tests that drive `Domain` reach the items through
  the top's re-exports.
- **It has no features** (`skein/docs/foundation/cli.md`, 3): what the
  top's tests need of it is plain public code.

## 4. Effects

Moved to [`effects.md`](effects.md), section 1.

## 5. Topics, and landings as news

Moved to [`topics.md`](topics.md), section 1.

## 6. Updates: lazy and mechanical

Moved to [`updates.md`](updates.md), section 1.

## 7. Participating objects

Not needed first, and kept possible with one mechanism (jig's
`domain/model.md`, 8.2), for three cases: outside contributions to a
project temper works on; a repository where temper may not merge; and
temper's own contributions upstream (section 8).

- **What people write on what temper owns is no input.** A comment on
  temper's pull request or issue reaches no task; the pull request's
  body says where to reach temper instead. The one fact temper reads
  there is a review's verdict by a person a landing rule's approval
  counts (`change/gates.md`, 2); where that person asks for changes, the reviews at that
  head and their remarks reach the repair the request starts, as quoted
  words (`change/gates.md`, 1).
- **Outside contributions** to a project temper works on: their issues
  and pull requests, which a task may triage and review. A task given
  one subscribes to its comments and reviews; what others write reaches
  its inbox as news, in order, from a position the connector keeps.
  Outside contributors are not the project's parties unless adoption
  seeded them (`adoption.md`) or they sign in: what they write is a
  participant's words.
- **A repository where temper may not merge:** a change there is ready
  when its CI and gates hold, and then waits for someone else's merge,
  which is a landing like any other. People's reviews on its pull request
  are news for the change; a request for changes is a repair, given its
  remarks.
- **What temper writes** there (a reply, a review) is a participant's:
  drafted by an agent's task, and, where the project's policy says so,
  proposed to a person before it is posted.

## 8. Upstream contributions

Not needed first, and kept possible:

- **A fork,** on the upstream's forge where it can be, so that upstream's
  CI runs on it unchanged and the pull request is native there. Its main
  follows upstream's, fast-forward only, and temper's own changes land
  into a work branch in the fork: a goal's own branch (`branches.md`, 1).
- **The submission is a task:** brought up to date with upstream, its
  commits shaped to upstream's conventions, described and signed off,
  and accepted by a person before anything is posted, often under that
  person's own identity.
- **Once posted,** its pull request is a participating object
  (section 7): a task reads the maintainers' reviews, updates the
  branch, and drafts replies a person approves.

## 9. Restart

Moved to [`effects.md`](effects.md), section 2.

## 10. The world

The connector's world, `tests/engine/forge`, runs its subtree with the
fake forge below it (`../../testing/fakes.md`, 2) and the root scripted above it;
the tiers that run it through bytes and on a real Forgejo are
`../../testing/tiers.md`, section 1. Its stories: a change produced, opened,
checked, gated, queued and landed; CI failing, repaired; a gate asking
for changes, repaired, its verdict given again; a gate added while
queued; two changes into one branch, the second updated cleanly and its
review carried over; an approval carried over a clean update and asked
again after a repair; a change that leaves the queue for a conflict and
comes back at its place; a low-priority change aging past the window
ahead of later ones; a conflict resolved from a merge in progress; a
conflict settled by the merge job alone, and one it leaves to a
resolution, under rules asking an update and under rules asking none; a
catch-up into a goal's branch, merged by the merge job, and one already
contained, ended with no change; a
semantic conflict repaired; a CI repair ended by CI passing at its head,
due, retrying or held, and one released after it with no fresh repair; a
run of one left to finish; CI passed on an earlier head, changing nothing;
a review's repair untouched; the branch's own CI broken by a landing
temper did not make, the queue paused and repaired; its pull request
closed by someone else, held, released and reopened; a landing that
overlaps a goal's change, and one that does not; a goal's issue written,
revised and closed; a merge whose answer is lost, found merged after a
restart; an update whose answer is lost, found, or made again and landed,
under a queue and as a turn's prepare; one lost for good, decided again
until its count holds it, told why, and released; one whose base went on
while it was lost; a copy of it landing after the change moved on, held;
a goal's landing whose own update lands, going on to CI on its new head
and merging, and one whose goal's close-out merged into its branch just
before, doing the same; a push from outside during an update, held with
both heads named; a stalled update held after the stall, its words
naming the step; two changes into one branch, the second, better placed,
staying queued while the first updates, is checked at its new head and
merges, then taking its turn unprompted; a change's CI failed at its
set-up step, run again and passing, its gates kept; one run again that
fails again, then repaired, its brief saying so; a workflow step's
failure repaired with no re-run; a re-run whose answer is lost, found made;
a mergeability unsettled within the window, then settled; a
forge preloaded with ten times the history, idle at the same cost. Of
the rules as settings (`change/gates.md`, 3): a landing rule changed between two merges,
the second judged by the new rule; a gate a rule adds asked for on a
queued change, which leaves the queue, and a gate a rule drops no longer
waited on; a project's rule adding an approval; a rule refused in its
change's decision; a change kept on the charter it was made with, and
one held when that charter is no longer offered; a restart that hands
the values back before the records.

Its stories of access and of GitHub's ways, the root and routing
scripted: a forge connected while temper runs, an adoption refused as
not connected before it and made after it, the writer given then and
restored at the next start; an owned forge set up from the client,
each state's row and `SetUp`'s answer checked together, with and
without the box: asked when offered and none sets up, "busy" while
any sets up, "not offered" connecting, connected, stopped, not owned
or asked, or not kept, "not under the box" without the box and
after `BoxClosed`, each state the routing tells kept as told and none
restored; a forge stopped while a change waits on it,
nothing narrowed, its entry committed again once it connects; a managed
Forgejo's adoption whose access is refused, refused and named, and one
given, then read; the picker's listing on each kind of forge, a managed
one's admin's view read while another run writes, one not listed, an
installation's repositories changed and an adoption marked,
then unmarked when it is listed again; an adoption on a branch whose
rules want a merge queue and signatures, said in words with the one
setting; a merge queued, landed at the group's commit, and another
ejected and repaired; an update GitHub makes after it answers, found by
the head read afresh; no commit status written on GitHub, and nothing
held for it. The forge below is skein's fake, its GitHub layer
included: no example calls github.com or uses a real credential.

Its stories of removal: a forge removed with a change on it, its
watches retired and nothing read of it after. With the root's, in the
engine's world (`tests/engine/domain`): a removal, and a project's
deletion (jig's `domain/people/acts.md`, 2), each cut by a crash after
each kept act and resumed at the next start to the same end, a chat
that worked on the forge reading its repository's name, open or ended,
until the chat is erased; a removed forge's number never given again;
and nothing of the forge or the project read afterwards but what `adoption-connection.md`, 1
and jig's `domain/people/acts.md`, 2 keep.

Its referee, beyond jig's for every connector (jig's
`domain/connectors.md`, section 15): nothing lands without every
requirement its branch's enforced rules name, guarded ones met at the
exact head merged and up to date met on the reads of the decision that
merged it; merges land exactly the heads decided; temper never writes to
what it does not own, and its runs never force a push (the forge's prepare
rebase is `../landing-prepare.md`, 1.2); a held change makes no effect;
a landing wakes a goal exactly when it overlaps; no change waits in a
queue past the aging window behind changes that became ready after it;
temper's own pushes while it holds the writer slot are never taken for
drift, but a late update copy moving the head after its change moved on
is drift (`updates.md`, section 1); the calls made while idle are bounded by the
limits; nothing is narrowed, and no change held,
for a forge not connected; a change whose merge was answered queued is
not merged again until the forge has landed or ejected it, and lands at
the group's commit; nothing is written to, read of or listed for a
forge once its removal's decision has committed, and a removal ends
whole.

## 11. Open questions

- A workflow that runs on no pull request: `adoption.md`, section 14.

- A closer's stops on a goal's path: `change/branch-path.md`, section 8.

- A close-out's own charter: `change/branch-path.md`, section 8.

- Project-scoped rules on a real forge: `change/gates.md`, section 4.

- **Forge-read recovery with real agents** (`keeping-up.md`, 3): the existing ChatGPT
  and Claude Code owned-forge landing stories are the after-landing
  check under real traffic. A fake-forge owner-collision regression
  establishes the routing mechanism alone, not either real-agent
  outcome or resumption after `serve` restarts. Default: keep those live
  stories; the ownership contract requires no change to them. Forge
  bounds and room figures remain separate work, as below.


- Saved branches outside the task's saved set: `branches.md`, section 4.
- A clean update's other words: `change/repairs-later.md`.
- "The brief on Home" in Settings: `attention.md`, section 7.
- A status run's kept row: `attention.md`, section 7.
- A stock project's room, shared: `attention.md`, section 7.

- What lands on a goal's branch and then on main: `attention.md`, section 7.
- What a refused copy leaves: `adoption.md`, section 14.
- A queue's throughput: `queues.md`, section 2.

- **Checks without an LLM:** a repository's checks run as agents' tasks
  at the exact head (`change/gates.md`, 1); a checks-only run on a worker, running the
  project's commands with no model, would cost no turns (`../worker/README.md`,
  section 10).
- **A person's approval as a rule:** temper takes it two ways, a person
  gate answered in the client (`change/gates.md`, 1) and a review the person posts on the
  forge, which a landing rule counts (`change/gates.md`, 2). Whether an approval given in
  the client should also be posted on the forge as the person, so that
  the forge's protection requires it independently, needs a forge
  credential of the person's (jig's `domain/people/README.md`, section 11).
- A gate's live run past the stall: `change/stalls.md`, section 6.
- A green head no wake reaches: `change/repairs-later.md`.
- A retry before the read: `change/repairs-later.md`.
- A hold at gating at a green head: `change/repairs-later.md`.
- A repair of the branch's own CI at a tip since passed: `queues.md`, section 2.
- A repair held for its brief: `change/repairs-later.md`.
- A forge out of reach or not connected: `change/repairs-later.md`.
- A failing check's repair brief: `change/repairs-later.md`.
- **A failure that is the runner's, not the change's:** a job whose
  set-up step failed, or whose runner was lost, is a repair like any
  other under the queue, where landing by turn retries it, never
  repaired (`../landing-proof.md`, 2.4). Running the job again first, through the
  API, is `../../draft/ci-on-forgejo-17.md`, with the stalls' reasons and
  the agents' log tools.

- **Intake:** issues others open as a source of goals.
- A resource refused as forbidden: `keeping-up.md`, section 6.
- A merge style per repository,: `adoption.md`, section 14.
- A change whose forge is not connected,: `adoption.md`, section 14.
- `Forbidden` and `Missing` taken as access lacking: `adoption.md`, section 14.
- Several changes in a forge's merge queue: `queues.md`, section 2.
- The update before entering a forge's merge queue: `queues.md`, section 2.
- What failed on a queue's group: `change/repairs-later.md`.
- Taking a change out of a forge's merge queue: `queues.md`, section 2.
- Signed commits required: `adoption.md`, section 14.
- Connecting a forge as another user: `adoption.md`, section 14.
- Which checks are CI on GitHub, without required checks: `keeping-up.md`, section 6.
- What ends the Away switch: `attention.md`, section 7.
- The classes per person: `attention.md`, section 7.
- What code decides, listed: `attention.md`, section 7.
- A refused `message` keeps no record: `change/questions.md`, section 3.
- A passed-up question whose asker ends: `change/questions.md`, section 3.
- A forge's run on a head replaced: `change/records.md`, section 1.
- An early update against the bound: `change/records.md`, section 1.
- A report's head past a merge: `change/questions.md`, section 3.
- **A person's request on a goal landing's pull request** (`change/gates.md`, 1; `../landing.md`): a
  landing is not a change, and its merge's refusal keeps `../landing.md`'s own rule,
  so such a request starts no repair. Whether a landing should repair it as a
  change does, from the remarks of a person reviewing a whole goal, is undecided.
  Default: so, until a request asks.
- **CI asked again at an unchanged head** (`change/repairs.md`; `../landing.md`): a change
  whose CI the runner failed has it run again (`change/README.md`, 2, step 3), but one
  that failed for another infrastructure fault inside a workflow's step (a network
  outage, a flaky test) cannot have its CI run again until a new commit or a cancel;
  a person's ask would rerun a change's failed jobs. It needs a client request and
  stored retry state, so the restart work of 2026-11-10 left it for a plan of its
  own. Default: a new commit or a cancel. A landing's failed receipt is superseded
  by a repair's flake verdict and its release (`../landing-regate.md`, 3), and a
  red of known flakes is rerun before its receipt (`../landing-regate.md`, 2).
