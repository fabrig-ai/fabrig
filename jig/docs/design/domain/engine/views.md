# Views

The views child domain keeps parties' watches, and paces what each is
delivered; the core makes the documents they deliver, and answers
parties' reads, from what its children hold. What a watch shows, and
what a client reads, is the client's contract
(`../../client/domain/contract/documents.md` and `../../client/domain/contract/README.md`, sections 4 to 5); what the core
computes for them is said below (1 to 5).

- **Watches:** jig's subjects (`../../client/domain/contract/README.md`, 5.1):
  `me`, the signed-in party's own document (1); a task; a task's
  tree; chips, a bounded set of tasks; the party's inbox, across
  projects; a task query's live lines (2); a task's live run; the
  party's LLM accounts (../accounts.md, section 5); the deployment; and an application's own subjects, through the root to the
  connector that owns them. The other objects a client shows (a
  proposal, an escalation, a question, a person task, a result, a task's
  activity, a task query's ended part, a project, a note) are read, and
  read again when a watch that names them moves.
- **An application's subjects name their scope,** as its reads do, since
  the core cannot read a connector's subject: a project, the deployment,
  or the party's own, what is the signed-in party's across projects. The
  core admits a watch or a read of one on the party's sign-in and their
  role in that scope, the party's own on the sign-in alone, and on the
  views' limits, as it admits jig's; whether the subject is in that
  scope, and what the party may see of it, is the connector's to judge
  (../connectors.md, 5.5). Its watches and reads carry a window, as jig's
  do (`../../client/domain/contract/README.md`, 5.5). Only the document, or why
  there is none, crosses back; the root only routes.
- **Public reads** (../people/public-reading.md) are the public party's,
  with no sign-in: the core admits one only on its public index
  (../people/public-reading.md, 9), the project first, then the task, before any other
  work, and answers every other unknown alike. It makes jig's public
  documents (projects, a board, a card) by their projections from the
  index alone, within the board's bounds, never scanning the live tasks
  or loading from the store; an application's it asks of the connector
  that declared the kind, handing it the project's public tasks, and
  checks the tasks its answer names again in the step that completes
  the read (../people/public-reading.md, 10). It keeps the index with every decision
  that changes what is public (../people/public-reading.md, 7), building a project's
  entry from its live tasks at start and when it is made public.
  No watch is opened, nothing is marked, nothing is committed.
- **Level-triggered,** as facts are (../model.md, section 4). A delivery is
  the subject's whole current document, within the watch's window, made
  by the subject's owner when the delivery is made: the core, or a
  connector through the root. A watch has one delivery in flight; a
  change meanwhile marks it, and the next delivery, once the watcher has
  taken the last, is made from the state as it is then. Changes in
  between are coalesced, never queued. A watch ended with a delivery in
  flight ends once that delivery is taken or dropped
  (`../../protocol/client/README.md`, section 5). An unwatch whose watcher the
  core does not know receives an unwatched ending behind the commit
  barrier, so closing its stream does not wait on an absent watcher.
- **Finishing belongs to the core** (../root.md, section 3). Gathered subject
  changes, retained writing's proof reconciliation and unreported watch
  endings remain owed after a projection pass defers them. A closing watch
  awaiting its delivery or stream acknowledgement waits on its child;
  repeating its ending is not finishing work. Sign-in expiry reaches the
  core through the people's timer (../people/README.md, section 3); stopping still
  completes watch endings and the drain notification (`restart.md`, section 1).
- **An ended task or tree opens as a finished read**
  (`../../client/domain/contract/README.md`, 5.2). Construction uses a bounded
  task load (`README.md`, 5.3), held behind preceding commits. The core
  checks the sign-in and retained row's project on completion, makes the
  final document without restoring the row, and closes the watch after
  its delivery terminal. A close during the load drains construction
  without delivering. Live watches are paced as every watch is.
- **Watches are never stored.** A restart (`restart.md`) starts with none,
  and is told nothing of the deliveries made before it; each client
  opens its watches again (`../../protocol/client/wire.md`, section 2).
- **A slow watcher never holds the engine up,** and is never told it
  missed anything: it sees fewer deliveries. What the views child keeps
  per watch is its subject, its window, and two marks: a delivery in
  flight, and a change since. A task query's subject is its keys and its
  order, of a fixed size. Nothing grows.
- **Marked by commits:** each decision's commit names the tasks, projects
  and parties it touched, and the views mark the watches they concern:
  - *a task touched:* its own, the trees along its lineage, chips
    naming it, and every task query it matches before the decision or
    after it (2);
  - *what a task waits on moving:* the same watches of the tasks along
    its lineage (3);
  - *a party touched,* their entries, a turn of a conversation they
    requested, their read positions, their roles or pools, or a policy's
    templates in a project of theirs: their `me` and their inbox.
  - *an LLM account touched,* its source, state, order or uses, a
    program's login reported on a host, or a sign-in or a check begun or
    ended: the `accounts` of the parties it concerns, and the
    deployment's (../accounts.md, section 5).

  A connector marks its own subjects (../connectors.md, 5.5).
- **Durable before shown:** a delivery waits for the last commit made
  before it, as every read's answer does (`README.md`, 5.2).
- **A run's text as it is written** is the one stream: kept per live run
  being watched, within a bound, named by the turn and the retry the host
  reports it for (../hosts/hub.md, 5), and delivered from the offset each
  watcher has; never stored, and marked provisional. The core keeps
  `text` and `input`: the root routes them as `RunText` and `RunInput`.
  Reasoning is dropped and counted at that boundary. So is a piece whose
  offset is past the 32 bits the views count, where a host counts
  smith's 64 (`jig/docs/design/domain/hosts/hub.md`, 5). The protocol layer carries every kind
  and offset as it is (`../../protocol/channel/README.md`, 5.4), and the same root
  route serves the engine's own hub and its workers'. A later retry
  replaces an earlier one's text; text that starts past what the views
  hold of its retry, because facts were dropped between, marks that
  turn's text cut, as text past its bound does. It decides nothing and
  leaves at once (`README.md`, 5.2). A committed turn reaches a client through the
  task's activity, read (`executors.md`, 2).
  Input retains the call being written in the current-call slot, within
  the same id + tool + input allowance. A later call replaces it; gaps and
  trimming the oldest bytes mark it cut. A begun call replaces it, so the
  call being written and the call being made are never retained together.
  A new turn or retry clears it, and committed or fenced turns cannot
  change it. Deliveries carry the current input suffix whole as `writing`.
- **History is the store's.** What a client shows of the past (a task's
  results, its activity, a plan's revisions and why, the ended tasks a
  query finds) is read (`README.md`, 5.3), paged, never held.

Nothing the core decides depends on a view arriving.

## 1. `me`

- **The signed-in party's own document,** about whoever the credential
  belongs to; no party watches or reads another's. The core makes it
  from:
  - the people child: the party, their kind, login and name; their
    projects and their role in each; what waits for them, counted
    exactly, and their updates unread, counted up to a limit; their read
    position over results; their role in the deployment, if any;
  - authority: their pool in each project, and what they may create,
    the bounds a draft is compared with, their allotment left among
    them;
  - the templates: each of their projects' merged templates.

  What it carries, field by field, is the client's contract's
  (`../../client/domain/contract/documents.md`, 1).
- **Templates are served, never used.** The core keeps the
  current templates, the newest revision's (`settings.md`;
  `../../protocol/configuration.md`, 3.1), and
  each project's policy's overrides of them (../authority.md, section 6)
  only to merge them, per project, into `me`. No decision reads one: the
  client fills a template into an ordinary request, which the core
  checks as one written by hand, so a template grants nothing, and a task
  keeps the values it was made with, never which template gave them.
- **A resolved default beside intent.** Each effective template in `me`
  carries `resolved_default: Option<LlmChoice>` beside its configured
  LLM choice. An explicit template choice previews itself. For an omitted
  choice, the core takes what a run's check would take now for a task
  keeping no account (../accounts.md, 4.3): the first model in charter order
  with an account usable now, at its default effort, under the effective
  creating authority and with the
  current receiving party as payer. No usable model, or an executor still
  *chosen*, gives `None`. This is an observation, never admission or
  an account reservation. The client's shapes keep the configured
  intent, never copy the preview into a request. A task made with an
  omitted choice keeps none, and each of its runs' checks takes its
  models (../tasks/README.md, section 3; ../accounts.md, 4.3);
  the preview changes no task or core store layout.
- **The offer** (`../../client/domain/contract/documents.md`, 1), per project: each
  model of configuration's order that the party's authority there allows and
  an account usable now for the party as payer can run (../accounts.md, 4.3) and
  allows (../accounts.md, 3.7), with those accounts. A direct source's account needs no host, so that a
  worker's restart takes no model away and what `account list` calls usable
  is offered; a program's source needs a connected host that has reported
  its login usable. A
  provider with no such account gives no model, whatever is configured of
  it. An application's agent that proposes settings, as temper's Ask temper,
  may name the same list for its task's payer, through the same function. It is an observation, as the preview is;
  a run's own check decides.
- **Marked** as any of it moves. A pool's amount left moves with spend;
  account and source changes, a program's login reported on a host, a host
  connecting or leaving, policy, membership and effective-executor changes
  invalidate the affected offers and previews.
  Its marks are coalesced, as every watch's are.

## 2. Task queries

- **One query for every list of tasks.** A task query names keys, an
  order and a window: its keys among project, requester (a party or a
  task), root, parent, label, phase class, executor kind and tracked;
  its order one of number, last change, last activity, priority and
  ended at; its window by cursor (`../../client/domain/contract/README.md`, 4.1).
  Its rows are task lines, the one record every list of tasks is made of
  (`../../client/domain/contract/documents.md`, 1): its live tasks first, in its
  order, then its ended ones, each group with its own cursor. Its live
  lines are watched; its ended ones are read, a page at a time. The core
  admits it on the party's role in the project it names, or, naming
  none, for tasks whose requester the party is
  (`../../client/domain/contract/README.md`, 5.1).
- **Live tasks are scanned.** The tasks child keeps no secondary index,
  and its live tasks are bounded by its limits (../tasks/README.md, section 6), so
  a query's live part is one pass over them, whatever its keys, keeping
  the window's rows in order. The fixed set of keys bounds what marking
  compares and what a query carries, never the filtering.
- **Ended tasks are found through indexes** in the store, which the tasks
  child saves in the decision that ends a task (`../../protocol/store/loads.md`,
  3 and 4):
  - *by requester:* every ended task whose requester is a party, by that
    party and the label;
  - *the ended delegates',* for a query naming a requester task or a
    parent: that task's ended delegates;
  - *by project:* only the ended tasks flagged *listed*, by their project
    and the label. The flag is the task's, set when it was made, so the
    core reads no template to index.

  A query's keys choose one index, in that order: a requester party; a
  requester task or a parent; a project (`../../protocol/store/loads.md`, 3).
  Its other keys, the root among them, filter that index's pages, which
  may come back cut, and the next page goes on from the last row
  examined. A query whose keys choose none has no ended part, and its
  answer says so. The decision that forgets a task forgotten once read
  erases its index rows (4); retention erases the rest with their
  tasks (`../../protocol/store/README.md`, 6.1).
- **What a query never finds:** words in a text (`../../protocol/store/README.md`,
  section 11); ended tasks by outcome, resource or spend; tasks by a
  procedure's parameters, which belong in the application's own records
  naming the tasks; ended tracked tasks across projects.
- **Each line** comes from its live task, or from an ended task's index
  row joined to its record, with what only the core can give: what the
  task waits on (3), and, for a conversation, its unread count
  (4).
- **Marked by a task it matches,** before the decision or after it, so a
  task entering a query, leaving it, moving within its order or ending
  marks it. Matching compares the task's fields, and its lineage's root
  and parent, with the query's keys: a bounded comparison per watch. The
  core gives the views each touched task's keys as they were and as they
  are.

## 3. What a task waits on, and its last activity

Two values each task line and each task's document carry (../tasks/README.md,
section 3):

- **What it waits on is computed,** since only the core holds a task's
  tree and the entries waiting in it together. It is never stored, so no
  restart finds it stale. It is the first of:
  - *a party,* when an entry in the task's live subtree, itself
    included, waits for a party: a question put to one, a proposal or an
    escalation waiting at parties, a person task addressed to a party or
    a role. It names whom the oldest such entry waits for, a party or a
    role, and how many wait;
  - *nothing,* when it is working: its run live and not waiting;
  - *its dependencies,* when it is kept (../tasks/lifecycle.md, 8): the pending
    ones, the oldest first, and how many;
  - *its delegates,* when it has live delegates;
  - *news,* when it is idle, its run parked or waiting, or its procedure
    waiting on its facts: what moves it next is a message its wake
    policy lets through, or a fact its connector reads (../tasks/messages.md, 3;
    ../connectors.md, 6.2);
  - *nothing,* otherwise: its phase says what it waits for (its
    dependencies, its holds, its closing), or it has ended.

  The core computes it as it makes a line or a document, by a walk of
  the task's live subtree, within tasks per tree. An entry for a party
  made or ended, and a delegate made or ended, touch every task along
  its lineage, within the depth limit, so their watches are marked
  (`views.md`).
- **Its last activity is kept** on every task: a wall time, set by the
  decision that commits its cause, words the task heard, a turn its run
  committed, or a delegate's end (../tasks/README.md, section 3). It orders lists,
  moves no revision, and stays on the ended task, so a query's ended
  rows show it.

## 4. Conversations

- **Known by its flag,** never by its charter, its label, or the
  template that made it (../tasks/README.md, section 3). Only an agent's task
  carries it, so its turns are its transcript's (`executors.md`, 2).
- **Its turns are entries for its requester,** while that requester is a
  party: a turn committed is an update for them, and marks their `me`,
  their inbox, and the queries the task matches.
- **Read positions are the people child's** (../people/README.md, section 5),
  moved only by the party's `Seen`. A line's unread count is the task's
  turns past its requester's position. There is one per task
  flagged conversation, its requester's, so they number at most the
  tasks flagged conversation:
  - *a live task's,* in the people child, restored at start (`restart.md`);
  - *an ended task's,* moved to the store by the decision that ends it,
    never held, and joined by a point read where a party reads the
    conversation or a query's ended row (`README.md`, 5.3), so an ended conversation
    reads with what its party had read.
- **A task's retention class** says what becomes of it once ended
  (../people/README.md, section 5; ../tasks/README.md, section 3 defines the flag); the
  decision that forgets a task forgotten once read erases its read
  position and its index rows with it.

## 5. What a decision would do

- **Offers are the core's judgement.** Each object's document says which
  acts the party may make on it now, and for each whether it would be
  made, proposed and to whom, or refused and why, with its bounds. The
  core judges them from the authority, policies and pools only it holds,
  and judges again when the act comes (`../../client/domain/contract/documents.md`,
  3).
- **A decision's effect is data,** made from the core's state as the
  document is made (`../../client/domain/contract/documents.md`, 4): what accepting
  a proposal grants and costs, and the pool after; what a release does;
  what a cancel ends, its tasks by phase, its live runs and its pending
  proposals, never its connectors' effects; the reservation a move
  takes. Confirming a decision is the application's;
  the core gives the facts it shows, and takes the decision naming the
  revision its party was shown (`../../client/domain/contract/documents.md`, 2).
