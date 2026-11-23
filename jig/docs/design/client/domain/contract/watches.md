# The client and the engine: watches

## 1. Watches

### 1.1 Subjects

| Subject | Its document | Who may watch |
|---|---|---|
| `me` | `me` | the party, always; every stream watches it from its opening, its first value in the hello (`README.md`, section 6) |
| `task{n}` | the task's document | a party who may see it |
| `tree{root}` | a task's live subtree, within a window | likewise |
| `chips` | a bounded set of tasks the client names, as chips | likewise, each |
| `inbox` | the party's own, across projects, within a window | the party |
| `tasks{query}` | the live task lines a query selects (`reads.md`, 1.1), in its order, within the watch's window, saying whether there are more | a party with a role in the project it names; a query naming no project, only the party, for tasks whose requester they are |
| `run{n}` | a task's live run: its attempt, its provisional text and current call (1.3) | a party who may see the task |
| `deployment` | hosts, accounts, capacity | a party with a deployment role |
| `settings` | the settings (`documents.md`, 1) | a party with a deployment role; a project's items, a party with a role there |
| `accounts` | the party's accounts (`documents.md`, 1): their own LLM accounts, their sources and states, their sign-ins and checks under way, and whether an account serving their work is *usable* and everything they asked is *settled* | the party, always |
| `terminal{n}` | a terminal the client lends for a program's login, `n` its own number for it: the login asked of it, while one is under way; the watch is the offer (`../../../domain/terminal-login.md`, 2) | the party, always |
| an application's subject, naming its owner, its scope and its window | the application's document | a party the scope admits (1.5); what of it they may see, as its owner, the application's connector, says |

The objects a client shows apart from these (a proposal, an escalation,
a question, a person task, a result, a task's activity, a project, a
note) are read (`reads.md`, section 1), and read again when a watch that names them
says they moved. A turn is a read and not a subject: a committed turn
does not change, so it is read once (`reads.md`, section 1).

Account source/uses and fleet changes invalidate accounts watches; membership
changes invalidate them too, including another member joining a sole-holder
project. The catalogue is immutable startup configuration and arrives on every
new `me` watch, including after restart.

### 1.2 Level-triggered

- **A delivery is a whole document,** the subject's current state within
  the watch's window, computed when the delivery is made, not when the
  change happened, and held until every commit it reflects is durable,
  as a read's answer is (`reads.md`, section 1).
- **One delivery in flight per watch.** A change while one is in flight
  marks the watch changed; when the client has taken that delivery, the
  next is computed from the state as it is then. Changes in between are
  coalesced, never queued.
- **Nothing is missed:** the next delivery has every change. A slow
  client sees fewer deliveries, a fast one more; neither holds the engine
  up, and what the engine keeps per watch is a mark and a window, never a
  backlog.
- **The first delivery** is the subject's document when the watch is
  admitted. For a task or tree whose root has ended, the core loads its
  retained task facts through the bounded durable read path, checks the
  current sign-in and visibility, and delivers once. A tree still means
  the live subtree (`documents.md`, 1): its retained ended root has no live descendants,
  folded counts or continuation, whether it was a root or a delegate.
  The watch ends with the subject-gone reason after that delivery
  is taken or dropped. Missing or hidden retained facts end it without a
  delivery; retained facts too linked to load within the read's join
  bound end it without a delivery as too linked, naming the bound, as
  the task's read is refused (`reads.md`, section 1; `../../../protocol/store/loads.md`,
  3), never as the subject gone; an ending alone is never terminal
  success. Other ended
  subjects are refused `Unknown`. No retired task is restored to serve it.
- **Ended,** each saying why: the client left; its subject ended and left
  the engine; the party may no longer see it; the sign-in ended; the
  engine is stopping.
- **Admitted** against the party's sign-in and what they may see, and the
  watches the engine allows per party, per stream
  (`../../../protocol/client/README.md`, section 10) and in all; past them, refused
  as busy. A watch holds its place from its admission until the engine has
  ended it: while its closing is in flight, and while its stream's end
  or its last delivery is (`../../../domain/engine/views.md`). A
  deployment allows each party, and each stream, at least the engine's
  watches one client opens at its limit, `me`'s included (../client/README.md,
  section 11), so that a party's one client within its limit never finds
  its room full. Busy is the room full for now, and the client opens the watch
  again after a backoff (../client/watches.md, 1), as a request refused busy is
  sent again (`answers.md`, section 1).
- **A window past the engine's bounds is clamped,** at admission, never
  refused: the watch keeps the window the engine's rows admit, and each
  delivery is its document within it, saying there is more as its
  subject's document does (a tree's folded counts and its cursor; a task
  query's and an inbox's cursor; `documents.md`, 3). An application's window is
  clamped likewise before its connector is asked, and so is a live
  read's window. So busy is never a window's answer.
- **A document the engine cannot make within its bounds** for any window
  (a whole value past them, which its bounds are derived to rule out,
  `documents.md`, 3) is refused as unknown at admission, never as busy, since opening
  again would not mend it: the client ends the handle (../client/watches.md, 1).
  A delivery that cannot be made ends the watch, as its subject gone.
  An answer of its own is open (`README.md`, section 9).
- **An application's subject names its owner, its scope and its
  window,** as an application's read does: its owner by its number, the
  scope a project, the deployment, or the party's own (1.5). The core
  cannot read a connector's subject, so it admits on what it holds: the
  party's sign-in, their role in that scope, and the views' limits, as
  for jig's subjects. The connector, asked for the document, judges that
  the subject is in that scope and what the party may see of it, and
  answers unknown or not allowed otherwise. The engine's root only
  routes.
- **What marks a watch changed:** each decision's commit names the
  tasks, projects and parties it touched. A task touched marks the
  watches of that task, of every tree along its lineage, of chips naming
  it, and of every task query whose keys it matched before the decision
  or matches after it, so a task that leaves a query's set changes it
  too; a party's entries, role or pool touched mark their inbox and
  their `me`; a project's name or policy changed marks the `me` of every
  party with a role in it; an account's source, state, order or uses
  touched, a program's login reported on a host, a host connecting or
  leaving, or a sign-in or a check begun or ended, marks the `accounts` of
  the parties it concerns and the `deployment`'s, and their `me`, whose
  offer and previews read the same facts. Membership and effective-executor
  changes
  also invalidate affected `resolved_default` previews (`documents.md`, 1). Matching a
  task against the open queries is bounded by the watches the engine
  allows, each a fixed set of keys. A connector marks its own subjects,
  through the root, as its state changes.

### 1.3 A run's live text

The one stream: what a run writes before its turn commits is not state
the engine keeps (`../../../domain/engine/views.md`), but a person
reads it as it comes. Its deliveries decide nothing and follow no
commit, so they leave through the journal's door (`../../../rails.md`,
1.3), marked provisional.

- **The run's document** is its attempt, the turn being written, named by
  its attempt, its number within it and its completion's retry as the
  host reports them, that retry's text so far, the call it is making, and the call being
  written (`writing`), with its provider id, tool, bounded input suffix,
  byte offsets and cut mark,
  within a bound on bytes. A call of the application's private tools
  carries its input, made or being written, to the task's requester alone;
  every other follower is sent its tool and no input
  (`../../../domain/engine/briefs.md`, 2.1).
- **Read from an offset:** each delivery carries the turn's name, the text
  after the offset the client has, and the offset it ends at; the client
  takes it and is sent the rest when there is more. A new turn's text,
  or a new retry's, starts at zero under its own name, so it never resets
  another turn's; a later retry replaces the earlier retries of its turn.
- **Committed turns are not in it.** A turn's committed form reaches the
  client only through its task's activity, read once the task's document
  says the activity has moved, and held until durable (`reads.md`, section 1).
  `jig_client::thread` keeps a turn's provisional text until then
  (../client/conversations.md); the client's watch holds only the turn being
  written (../client/watches.md, 3).
- **Past its bound,** the text is cut from its start, at a character's
  boundary, and the delivery says so; the turn, once committed, has it
  all.
- **The watch follows the task,** not one attempt: a new run's text
  follows the last, and its beginning is in the document.
- **Only from a host that gives it.** A run's text and the call it is
  making come from hosts that hand them up (`../../../domain/hosts/hub.md`,
  5): an agent process gives text and the call being written; it gives
  no call being made. Its durable turns arrive as they commit.

### 1.4 An application's streams

An application's data that its people follow as it comes (a log
followed, a metric's latest points) is a stream of the application's:
a subject naming its owner, its scope and its window, read from an
offset, as a run's text is (1.3). Its deliveries decide nothing and are
never a document of the engine's committed state: they leave through
the journal's door (`../../../rails.md`, 1.3), marked provisional, and an
application shows them marked so. It adds a kind of subject to the
application's own and a use of the journal's door, with no change to
1.1's subjects or 1.2's rules, and crosses on the client's one stream as
any watch's deliveries do (`../../../protocol/client/README.md`, section 5).

### 1.5 An application's scopes and windows

- **Scopes.** An application's request, read or subject names one:
  - *a project,* admitted on the party's role in it;
  - *the deployment,* admitted on their role in the deployment
    (`../../../domain/people/README.md`, section 3), as jig's `MakeProject` is
    (`requests.md`, 2);
  - *the party's own,* what is the signed-in party's across projects (an
    application's settings of theirs, the incidents they are paged
    for), admitted on the credential alone, and naming no party: the
    core names the party from the sign-in, as for `me`.

  The connector judges what any scope shows (1.2).
- **Windows.** An application's watches and reads carry a window, as
  jig's subjects do: bounded rows and bytes, at the cursor its owner
  gave, so that what the views keep per watch is a window and a mark,
  never a backlog, and every document fits the client's limits.
- **Streams** are 1.4's; **versions** are per owner (`README.md`, section 6).
