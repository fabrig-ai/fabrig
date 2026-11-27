# The core: decisions and the store

## 1. Decisions and the store

### 1.1 Commits

- **One decision, one commit.** What a decision changes in every child
  and connector it touched is one commit: records written or removed, as
  one.
- **Whole across families, through a kill.** That commit is one
  skein-kv commit (`../../protocol/store/commits.md`, 1), so a process killed at
  any point of its write, after any record, mid-frame or between
  families, starts again from what was durable: the last whole
  decision, never part of one. No family's record is durable without
  the records of its decision that it links to.
- **Nothing links ahead of what it names.** A decision saves a record
  that names a party, task, project or revision only when the core
  holds what it names, made in that decision or an earlier one. A
  person named by an input from outside the core (a procedure's or a
  connector's batch, an agent's ask or tool call, an application's
  request) that the people child does not hold is refused where the
  input is admitted, saying which, and never written, so a start finds
  every link it makes (`restart.md`).
- **Numbered and in order.** The journal numbers commits and the store
  applies them in that order. Its answer is that a commit is durable,
  which makes every earlier one durable too.
- **The state runs ahead.** A child's state changes as the decision is
  made, not when it commits, so the next decision sees it.
- **New work is paced by the root's watermark.** The backlog counts the
  journal's commits and outputs, the root's work and its queued store
  answers (../root.md, sections 3, 4 and 10). What finishes admitted work,
  including the store's answers, is always taken. New external work
  past the watermark is handed back before any child changes; its
  sender keeps it and offers it again. No route reserves journal room.
  The core's `Limits::routes` bounds each drain within a step, with
  its predicate asserted empty afterward (../root.md, section 10). A
  commit's write limit remains the store's edge
  (skein/docs/foundation/programming-model.md, 7.1).
- **A failed commit stops the engine** (../model.md, section 4). Nothing
  waiting for that commit, or any later one, goes out. A restart starts
  from the last durable commit.

### 1.2 What waits for a commit

Every output that someone outside the engine could act on waits for the
commit it follows from:

- **effects:** an outbox entry is handed to its connector to make
  (../connectors-effects.md, section 1);
- **assignments:** a run is assigned only once its claim is durable, so
  an attempt never runs that a restart would not know of;
- **acknowledgements:** a host's answer, and a run's turn, are
  acknowledged once committed; until then the host keeps them;
- **answers to tool calls** that changed state, and to parties'
  requests;
- **messages relayed to a live run,** so a run never reads what a
  restart would not have;
- **loads,** so a load sees every commit before it;
- **what parties read and watch:** a read's answer and a watch's
  delivery decide nothing, but are made from state that may run ahead of
  the store, so each waits for the last commit made before it, as an
  output of a decision with no commit of its own (`views.md`);
- **grants:** a generation sent to an attempt (`README.md`, section 6) decides
  nothing, but goes to the attempts the fleet keeps, which run ahead of
  the store: one whose assignment waits for its commit is an attempt a
  restart would not know of, and its host does not know it yet either.
  So a grant waits for the last commit made before it, and leaves after
  its attempt's assignment, since held outputs leave in order
  (`../../rails.md`, 1.2).

What decides nothing and shows nothing durable waits for nothing: a
run's text streamed to watchers as it is written (`views.md`), and the
answers to a run's reads.

### 1.3 Loads

- **What is loaded, not held:** ended tasks and their results;
  transcripts; history; notes' entries; anything a client pages through.
- **A load is a request** with a token and one terminal event, rows in
  bounded pages (skein/docs/foundation/programming-model.md, 4.4); a
  window's pages are its answers in order, the last its terminal
  (`vocabulary.md`, 2.4). What waits on it is in a
  state that says so: a brief gathering, a run being prepared to resume,
  a party's page, an amendment adding an ended task (../tasks/changes.md
  ). That amendment's request stays unanswered while the rows load,
  one at a time: each ended task it adds, then the ended tasks above it
  that its reach is read through, at most one per level of the tree. Once
  they are in, it is admitted again from the start, on the rows and on
  live state as it then is: the pass that found them missing answered
  nothing and committed nothing. If what waits goes first (its run ends,
  its party leaves, the engine restarts), its load goes with it, and its
  request is left as any unanswered one is.
- **What a decision loads stays as loaded until it is decided.** Its
  loads are held (`vocabulary.md`, 1.2): each is sent once every commit made before it is
  durable, so it sees them. While they are in flight, a request whose
  decision would change what they load is answered busy (../people/README.md,
  section 5, a mark on an ended conversation). Otherwise two decisions
  that each loaded one record before either changed it would both decide
  on the old record, and the later commit would undo the earlier.
- **Bounded:** the loads in flight, and the bytes a load brings, are
  limits; a load past its bytes is cut, and says how much.
- **The engine paces its loads** (`temper/docs/design/draft/jig-domain-api.md`,
  4.4), and no root answers for room. At most `Limits::load_slots` of
  its loads are in flight, a window's later pages keeping the load its
  first page took. Parties' and runs' reads are admitted apart, under
  the same figure, by the reads they hold, as their own room: neither
  waits on the other's. With no load free:
  - **an agent's activation** that would load its window stays due, with
    no try spent, and runs once a load frees; a window begun loads
    whole, whatever else holds the loads (`briefs.md`, 1.1);
  - **an ended task's load** (a run's dependencies' and inputs' results,
    and the ended task an amendment, a batch or a decision reads) waits
    in the engine, in the order asked, and is sent once a load frees,
    before any load asked after it, held behind every commit made before
    it is sent, so behind those made before the core asked. Its waiting
    loads are bounded by the core's read flights, which the core keeps
    within its own limit (`briefs.md`, 1.1, `results: no read flight is
    free`). So a run's preparation never fails for a slot, however many
    tasks a restart makes due together (`briefs.md`, 1.1, *A dependency's
    result*);
  - **any other record or proposal** a party's request or a run's call
    needs (an ended conversation and its read position, a decided
    escalation or proposal, a batch's inputs, a person's ended task) is
    answered busy at once: a party's request `Again(Busy)`, a run's call
    unavailable. A notes operation's page fails at once, and the notes
    child answers busy (`briefs.md`, 2.2);
  - **a project deletion's notes page** is the one that waits: the notes
    child retries that page on its failure until the project's notes are
    dropped (`briefs.md`, 2), so failing it at once would hand the child
    its own retry in the same decision, again and again. The engine keeps
    the one page the retry asks for, since notes serves one operation at
    a time, and sends it once a load frees, after the ended tasks' loads
    waiting then; a stopping engine drops it, and the deletion's kept
    step takes the page up again after the restart.
- **Busy is a load that failed.** A load of an ended task is answered
  with its row, with none, or busy: the store could not serve it, its
  answer `Failed` or `Busy` (`vocabulary.md`, 2.4). So is any other load
  of a record or a proposal, busy reaching whoever waited on it once; a
  window's failure is the window's (`executors-transcripts.md`, 1), and a
  read's is the read's.
- **Why the engine:** it knows what each load is for and what waits on
  it, so the root keeps no table of the engine's loads and answers no
  flag for them. The figure is the one temper's root bounded these loads
  by, its `root.loads.loads`, which it gives the core as `load_slots`,
  and reads keep their own room, so temper's room doesn't change with the
  move; `ops`'s root and the test root, which bounded them by nothing,
  gain the bound.
- **Parties' reads** are documents and pages the client's contract lists
  (`../../client/domain/contract/reads.md`, section 1): from live state where the
  engine holds it, loaded otherwise, and answered once the last commit
  made is durable (1.2).
- **Ended tasks are read through the store's indexes** (`views.md`, 2): an ended
  task by its key, a list of them through one ended index, each row
  joined to its task's record, and an ended conversation's read position,
  by point reads in the same step (`../../protocol/store/loads.md`, 3).

### 1.4 What the store keeps of the core

| Record | Written by | Loaded |
|---|---|---|
| the deployment: its id, made at its first start, and its numbers | the core | at start |
| settings: the defaults the newest revision was made with, the changes over them, the held revisions' records, the charter versions they read and pending proposals' sets (`settings-store.md`, 1) | the core | at start, after the deployment, before every other record |
| settings' history: the records of revisions no longer held, within the history limit (`settings-store.md`, 1) | the core | on demand |
| projects: resources and roles, homes, policy, parties' roles | people | at start |
| parties: identities, sign-ins | people | those with a live sign-in or a role, at start; others as they sign in |
| parties' requests' keys, with their answers, for a retention | people | at start, those within it |
| tasks: every field of ../tasks/README.md, section 3 (its label, flags, last activity, LLM choice, kept account and revision among them), holds and their queues, writer slots, the last turn committed for a live attempt, and the revision a claim was prepared under (`settings.md`, 3) | tasks | live ones at start; ended ones on demand |
| projection feeds: each tracked goal's hot header and words, its whole subtree's hot task rows and words, pending milestones and connector settlements; a nested tracked task remains in enclosing plans, with every save updating its phase and words there (`../connectors.md`, 8) | the core | at start; derived stale plan links repaired before the switch (`restart.md`) |
| LLM accounts: each account's holder, LLM provider, subject and display name, uses, and its sources in their order, each with its kind, a program's name, its state and a limit's reset; never a program's login on a host, which hosts report (../accounts.md, 3.1, and ../accounts-sources.md, 2) | accounts | at start |
| the indexes of ended tasks: by requester, every ended task a party requested; by project, the ended tasks flagged listed; each saved in the decision that ends its task (`views.md`, 2; `../../protocol/store/loads.md`, 4) | tasks | on demand |
| parties' read positions over conversations: a live one's; an ended one's, moved by the decision that ends the task and erased with it (`views.md`, 4) | people | live ones at start; ended ones on demand |
| calls committed since each live task's last turn, by attempt and name, with the core's part of their answers | the core | with their task |
| each live agent task's conversation: its first attempt, where its current window starts, whether an upgrade closed it, and whether an upgrade crossed its claim (`executors-transcripts.md`, 1) | the core | with their task |
| funders' numbers: each party's pool and each project's spend, per period; a period's beginning, and when the next begins if its source said | tasks | at start |
| a task's history: made, amended, held, released, cancelled, by whom, why; and its activity: words written to it, runs begun and ended, what it heard (`executors-transcripts.md`, 1) | tasks | on demand |
| inboxes: messages not yet taken | tasks | with their task |
| proposals: pending, and decided ones' outcome | tasks | pending ones at start |
| transcripts: turns, with their spend and whether each opens a window | the core | to resume, from the current window's start; on demand |
| notes | notes | indexes per scope in use; entries on demand |
| secrets: minted credentials' digests, by their sign-in's number, and services' credentials' digests; LLM accounts' records and keys, by their account and source, each kept before it is lent (`../../protocol/store/secrets.md`; `../../protocol/llm.md`, sections 5 and 6) | the protocol layer, never the domain, which names them by account, source and generation, or by a sign-in's or a service's number | at start, by the protocol layer |

Each connector keeps its own records beside these (../connectors.md,
section 2): its outbox, procedures' states and parameters, what it made
by key, its projections' states, and its own state. The application's
root wraps both in its store vocabulary (../root.md, section 6).

**Times** the store keeps are wall times (`env.wall`), so a record means
the same after a restart; a wait the core arms is a deadline on its
monotonic clock (`env.now`), computed from them (skein/docs/foundation/programming-model.md,
section 9). Accounts' source deadlines retain both clocks where the
limit or retry is created (`../accounts-sources.md`, 2): timers use `Deadline.at`,
and every projection and stored reset uses its source-owned
`Deadline.wall`, never a conversion at read or save time. Restore keeps
the exact wall and maps it to the new monotonic clock once. Fleet grace
and adrift times are the fleet's own (`fleet.md`).

### 1.5 What the core asks of the store

The store is assumed (../model.md, section 2); this is what the core relies
on, and all it relies on:

- a commit of several records is applied whole or not at all;
- commits are applied in the order sent, and durable once answered;
- a load sees every commit answered before it was sent;
- records are keyed by the domain's own keys, and listed by key ranges,
  in bounded pages; what is live (live tasks, pending proposals, the
  outboxes, live resources) is kept in key ranges of its own, so a
  restart reads it without reading history;
- what one engine wrote, the next reads: records carry the version of
  their shape;
- nothing else writes it.

### 1.6 How a child's records reach a commit

Each child that keeps durable state, and each connector, defines its own
records and ordered keys in its own vocabulary. Neither a sibling nor
the store chooses their shapes. The parent translates their keys and
carries their records in its own store vocabulary; the store's protocol
layer encodes each shape with a version (1.5).

- **`Save { record }`** keeps the record under its own key, and
  **`Erase { key }`** removes that key. These requests are part of the
  decision the parent is making. They have no terminal event and do not
  ask the child to wait: its state changes as it decides (1.1).
- **`Restore { record }`** supplies one record at restart, and
  **`Restored`** says every record in the child's live ranges has been
  supplied. The child then decides from that restored state.
- **`Load { owner, range }`** asks for one bounded page of the child's
  own key range. Its one terminal event is **`Loaded { owner, rows,
  more }`**. The token is echoed, and `more` says another page remains,
  which the child asks for with a load of its own.

A child never waits for a commit and never holds an output back for one.
Its outputs leave it as it decides, each saying which of the three of
`vocabulary.md`, 1.2 it is, and its parent passes them up. A child's world plays its
parent and the store: it keeps saved records, applies erases, commits at
the end of each step, and restarts by feeding the durable records as
`Restore` followed by `Restored`. Restart is therefore tested before the
root is built.
