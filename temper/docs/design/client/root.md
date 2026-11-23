# The client root

The client root's state, routing and requests are split from [the client's overview](README.md).

## 1. The client root

`temper-client-domain` is temper's client root, in jig's one shape
(jig's `client/domain/root.md`, section 3): `new`, `step`, `fire`,
`next_deadline` and `take`, and `shown`, a count of what is shown,
which the view builds from. It holds `jig_client::Client` as a
component and calls it; the client never calls it back, and the root
routes nothing of jig's (jig's `client/domain/client/README.md`, section 4).

### 1.1 What it holds

- **`jig-client`,** given its limits and a seed at construction.
- **Two children, each an owner** (jig's `client/domain/root.md`,
  section 15): temper's pages, owner `Pages`, which ask the client for
  jig's operations, reads and subjects only; and the forge's client
  side, owner `Forge` (`forge/family.md`); numbered in 4.6. Each is the asker
  of every handle and key it gives `jig-client`, and draws its keys'
  numbers from the seed the shell gives the root.
- **`me`,** watched from start to sign-out under a handle of its own:
  the signed-in party, their projects with their roles and pools, what
  waits for them counted, and each project's merged templates (jig's
  `client/domain/contract/documents.md`, 1). Every page shows the link's state,
  `jig-client`'s `Link`.
- **The page at its address** (`pages/README.md`): its handles, its windows, the
  composer's words, and the open confirmation (1.6), at most one.
- **The project the person is in,** chosen among `me`'s projects; the
  first at start.
- **The pages' acts in flight,** each by its key: which page's card or
  composer sent it, so its answer reaches what shows it (1.3).

Retained state is bounded by the domain's limits: the handles a page
opens, the rows of each window, the composer's bytes, the acts in flight.

- **Internal hand-offs are uncapped.** The pages, the forge's side and
  `jig-client` hand off through queues that grow as they fill. Each
  step settles them; each event's fan-out is bounded by admitted state.
  No client has a watermark: new work enters through capped edges,
  the engine link's exchanges, the kernel's queues and terminal or
  browser input (skein/docs/foundation/programming-model.md, 7.1). The terminal, browser and
  CLI shells use uncapped internal hand-offs too.

### 1.2 Its modules

```
temper-client-domain/src/
├── lib.rs          the tree, re-exports
├── boundary.rs     events in (the start, a place gone to, the person's acts, jig-client's events), requests out (jig-client's, a place shown, signing out, kept work dropped, facts)
├── owner.rs        temper's two owners, Pages and Forge, as askers (3.3)
├── domain.rs       Domain: new, step, fire, next_deadline, take, shown; jig-client held
├── route.rs        each event to its child; what jig-client tells, to the child its asker names (3.3)
├── address.rs      temper's addresses and the inbox's filters (3.5)
├── pages.rs        the pages: their handles, the project the person is in, navigation (3.1)
├── pages/          one module per page: what it watches and reads, its state, its acts
├── acts.rs         acts as jig's operations, through jig's helpers; the confirmation (3.6)
├── amend.rs        the amendment editor: a draft of a task's controls, to jig's Amendment (3.7)
├── forge.rs        the forge's client side (section 4), with its modules below forge/
├── limits.rs       jig-client's limits, the pages', the forge side's, and their sum
└── tests.rs
```

### 1.3 Routing

As jig's client root routes (jig's `client/domain/root.md`, section 5),
with temper's two children:

- **Out and in:** `jig-client`'s requests to the transport leave the
  domain as they are, for the shell to carry; the transport's events
  reach `jig-client` as they are.
- **By asker:** what `jig-client` says of a handle or of a key goes to
  `Pages` or `Forge`, matched whole; an asker that names neither is a
  fact (`UnknownAsker`), and reaches no child. The pages hand it to the
  page that opened the handle, or the card or composer that sent the
  key, and drop one for a handle they closed; the forge's side to its
  form, its picker or the forges' page that sent it.
- **The link** goes to both children, and changes what every page
  shows; a page reopens nothing of its own, since `jig-client` keeps the
  watches. Kept work dropped goes to the shell (1.4).
- **A place gone to** goes to the pages, and to the forge's side when
  it is `Adopt { project }`; any other tells the forge's side it left.
  The forge's side watches the deployment's forges for the session
  (`forge/records.md`), whatever page is open.

### 1.4 At start

The shell starts the domain with the small store's records and the
inbox's address, then signs in (`sign-in.md`, section 1), holding back neither. The
domain's children ask their watches from their start, `me` among them,
and `jig-client` opens none on the engine until it is signed in (jig's
`client/domain/client/README.md`, section 9), so no stream leaves without the
credential.

- **`me` comes with the stream** (jig's `client/domain/client/watches.md`,
  2): the stream's hello brings it, and the link is live with it, so
  what waits for `me` (the templates' labels, the party, the projects
  it lists) is ready when the stream is, and is asked in the step that
  hears the hello.
- **What names no party waits for no `me`.** The party's own and the
  deployment's watches and reads (jig's
  `client/domain/contract/README.md`, 5.5): the forge side's
  `dismissed`, `away`, `trials`, `approvals`, `devices` and `forges`,
  jig's `settings`, and the `deployment`, `playground` and program
  environment's reads, are asked once the link is live, never after a
  value of `me`'s, so they leave in the step that hears the hello,
  beside the page's. A watch or read that needs what `me` says (a
  deployment role, a template's label, the party's number) still asks
  in that step, from the hello's `me`.
- **Nothing of the frame's or a page's before the hello.** Their reads,
  as their watches, are asked once the link is live, in the step that
  hears the hello, never at start before it: the rail's `Read::Project`
  of Team's (`pages/team.md`, 1, *Its reads*) and the `accounts` watch
  the banner draws from (`pages/accounts-words.md`, 1) among them. A watch
  asked before the hello would leave as its own exchange ahead of a
  reload's kept keyed requests, which `jig-client` releases only once
  `me` confirms their party, and the link keeps the order exchanges
  were asked in (jig's `client/domain/client/README.md`, 4.1, *The
  settling*), so the kept requests would part it from the batch and it
  would cost a request of its own. So a cold
  open sends the stream alone first, and everything else after the
  hello: one batch, and on a reload the kept keyed requests on their
  own routes beside it, once the hello's `me` confirms their party
  (jig's `client/domain/client/README.md`, section 8; `pages/work.md`,
  1, *Its budget*).
- **Another party's kept work** stays parked and is not shown (jig's
  `client/domain/client/README.md`, section 8); the terminal says so in a line
  of its own (`line.md`, 6).
- **Pending reads at sign-in** end visibly in jig's existing forms,
  on successful installation even before the first `me`, and on
  sign-out (jig's `client/domain/client/README.md`, sections 6 and 9).
  Whoever asked must ask again; no read is replayed automatically.
- **Kept work dropped:** the domain hands `jig-client`'s `Dropped {
  count }` to the shell (jig's `client/domain/root.md`, section 8), and
  the terminal says so in a line of its own (`line.md`, 6).
- **temper's pages keep no record of their own:** a client closed on
  any page opens at the inbox, and a composer's words not sent, or a
  form's values, are not kept across a restart (`README.md`, section 5). A browser
  opens at its address (`web/addresses.md`, 1), and after the printed
  link's sign-in at `Welcome` (`web/sign-in.md`, 1;
  `../draft/archive/first-run.md`, 2.3), which shows its first step while the
  store is empty, opens at the step left undone while it is
  unfinished, and moves on to the chats page once it is whole (`pages/welcome.md`, 1).
  On a deployment with no welcome it moves on to the
  inbox, with nothing of the welcome drawn first (`pages/welcome.md`, 1).
  The terminal, which opens at the inbox, reads the same fact for the
  pages it changes.

### 1.6 Requests and confirmations

- **One act, one request,** keyed by the domain and sent through
  `jig-client`, which keeps it until its answer is durable (jig's
  `client/domain/client/requests.md`): a double press joins the
  pending one, and nothing is shown done before its answer.
- **jig's operations, through jig's helpers** (jig's
  `client/domain/client/helpers.md`; jig's
  `client/domain/contract/requests.md`, 2 and 4): starting a chat is
  `shapes::chat`; setting a goal is `shapes::set_goal`, or from a template
  `shapes::set_goal_from` (`pages/goal.md`, 3), a proposal where the person's authority
  does not cover it; closing a chat is
  `shapes::close_chat`, one compound request, with what becomes of each
  live delegate; amending and releasing a held task is one compound
  request; a goal's priority changes by its amendment (1.7).
  Every other act is one operation.
- **Said before it is sent.** An act that cannot be undone (cancelling,
  closing a chat, accepting, rejecting, releasing, removing a forge,
  deleting a project) says what it will do before it is sent, from the facts the document gives (a decision's
  facts, as `jig-client`'s `facts` reads them): what accepting grants
  and costs and the pool after it, what cancelling ends, what releasing
  does (jig's `client/domain/objects.md`, section 1; jig's
  `client/domain/contract/documents.md`, 4). Closing a chat says only what closing
  changes (*The sheet,* below). Removing a forge says what goes and what stays on the
  forge (`forge/deployment.md`); deleting a project names it and says what
  goes with it (*The sheet,* below; `pages/settings.md`, 1). Pause and Resume, on a
  chat, a goal or an agent's task, are undone by each other and sent at once. A
  goal's Close says that what landed on its branch stays there (`pages/task.md`, 4). Where the document offers an act
  as proposable (jig's `client/domain/contract/documents.md`, 3), the card says
  so, and to whom; where it offers it as refused, the card says why and
  sends nothing.
- **The sheet.** Every confirmation of `confirm.rs` is one sheet, from the
  look request `close-chat`: the mock's `dialog.sheet`, narrower, centred over the page. It asks in a
  sentence, says only what will change, and names its acts: what keeps things
  as they are first, then the act itself, primary. There is no "Confirm" and no
  "Dangerous action": the sheet is how temper marks an act that can't be undone
  (jig's `client/domain/client/helpers.md`).

  | Confirms | Title | Says | Keeps | Does |
  |---|---|---|---|---|
  | closing a chat | "Close this chat?" | "You can still read it, but it can’t be reopened.", and what it changes | "Keep it" | "Close chat" |
  | "Start over" | "Start over?" | "A new Ask temper chat starts. This one stays in Chats.", and what it changes | "Keep going" | "Start over" |
  | closing a goal | "Close this goal?" | "You can still read it, but it can’t be reopened.", then "What landed on its branch stays there." | "Keep it" | "Close goal" |
  | closing an agent's task | "Close this task?" | "You can still read it, but it can’t be reopened.", and what it changes | "Keep it" | "Close task" |
  | accepting a proposal | "Accept this proposal?" | its facts | "Not yet" | "Accept" |
  | rejecting a proposal | "Reject this proposal?" | its facts and reason | "Keep it" | "Reject" |
  | releasing a task | "Release this task?" | what releasing does; an uncertain effect's two choices | "Keep it held" | "Release" |
  | deleting a project | "Delete NAME?", NAME the project's name as `me` gives it | "NAME leaves every list, and its chats, goals and notes are deleted. Its repositories stay on their forges. This can’t be undone." | "Keep it" | "Delete project" |

  - *What it changes,* under a hairline, each line present only where it holds,
    and no count at zero:
    the run, while one of this task's is live: "The run under way gets N seconds
    to finish, then stops.", the template's grace, or "The run under way stops
    now." with none (a chat at rest closes at once, jig's `domain/tasks/lifecycle-endings.md`,
    2); each live delegate, by its number and title and its fate, "T4 Fix the
    parser is cancelled." or "T5 Review the parser moves to you.", with the other
    fate as "Take it instead" or "Cancel it instead"; the tasks below it that are
    not delegates, "N tasks below it are cancelled.", the document's waiting,
    running, held and closing counts summed, less the delegates listed; "N
    proposals are withdrawn."; and, while any of those holds, "Changes not yet
    begun are withdrawn. Those under way settle.". A chat at rest with nothing
    below it has no hairline and no such line: closing it changes nothing an
    effect is in flight for, and the connector's panel names one that is (jig's
    `client/domain/objects.md`, 11). What closing doesn't change (completed,
    failed and cancelled tasks) is not said.
  - *It stays until it is answered.* The press sends the ask and the sheet stays,
    saying the ask's progress above its acts. Answered done, it goes. Any other
    outcome leaves it open, saying why above its acts, where a press sends again;
    a press that sends nothing closes it, but for a rejection with no reason, which
    keeps it open and says nothing. The act that keeps things closes it at any time,
    Escape pressing it, and an ask in flight goes on, its outcome on the page as before. While the sheet is
    open, the outcome and progress of its own ask are the sheet's alone.
  - *A press stays under the finger* (`web/README.md`, *A press a render can lose*):
    the acts are a group of their own, keyed, and each delegate's row keyed by its
    task, so a line arriving above them, or the page updating behind, keeps each
    button's element.
  - *The terminal* prints the same lines as text, and the acts by their names.
- **Closed ends on one line.** A chat its requester closed ends on one quiet line,
  "Closed at 21:05" (*The Chats page,* its ended line), between two hairlines; its
  bar's status says "Closed", with no time. "Applied" isn't said once it is
  closed, nor "Decision made" for the closing itself: that line says it; nor
  after a chat's Pause or Resume: its status says it (`pages/chat/README.md`, 1, *A person's pause*).
- **It names what the person saw.** Each deciding act names the
  revision of the document the card showed; when the engine refuses it
  as moved, the card asks the person to look again (jig's
  `client/domain/contract/documents.md`, 2). jig's check holds it in temper's
  worlds (`testing.md`).
- **Answered:** the card shows the outcome, or the refusal in jig's
  words with what fits (look again, change what was asked); unknown,
  past the engine's retention, it says so. A goal's page says it in the toast (`pages/task.md`, 4).
- **A key is pasted once, never kept.** Its field is a secret (skein's
  `ui/tree.md`, 1.2 and 1.4), and adding or replacing a key is a request
  carrying a value (jig's `client/domain/client/requests.md`; jig's
  `protocol/client/small-store.md`, section 1): sent once under a name, never in
  the small store, never parked, never sent again by the client. The
  field is emptied as the request leaves, and the key is shown nowhere,
  then or after. Answered unsent or lost, the card says so and asks for
  the key again; the person's second send keeps the first's name, so
  that a request the engine made answers with its first answer, and no
  second account is made. A key empty or past 4 KiB (the bound of
  `ctl/bounds.md`, section 1) is refused before it is sent; one the
  engine refuses is answered with its refusal.
- **A forge's pasted value is sent the same way.** A GitHub App's
  private key (with no webhook secret: GitHub has none of temper's),
  a Forgejo bot's tokens and their webhook secret, a site
  admin's token, and a replacement of any of them are secret fields,
  and the request is a request carrying a value (jig's
  `protocol/client/README.md`, section 3): the value is sent once, never in
  the small store, never parked, never sent again by the client, and
  the fields are emptied as the request leaves. The request names no
  number for it and the client draws none: jig numbers the value as it
  keeps it (`../protocol/credentials.md`, pasted values). Answered
  unsent or lost, the page asks for the value again, and the second
  send keeps the first's key. A value empty, or a bundle past 4 KiB,
  jig's bound on a carried value (the bound of `ctl/bounds.md`, section 1),
  is refused before it is sent, in the words of `forge/deployment.md`; one
  the engine refuses, past the
  deployment's `records.value_bytes` among them, shows its refusal
  (`engine.md`). The client reads no configuration, so its bound is jig's.

### 1.7 The amendment editor

One editor drafts a person's amendment of a task, for every page and
command that amends one: the task's pages (`pages/goal.md`, 2, `pages/task.md`, 2) and `temper ctl
task amend` (`ctl/areas.md`). It is `temper-client-domain`'s `amend`
module, beside `acts.rs`, and pure: it reads a document and builds
jig's `Amendment`; it sends nothing, judges no bounds and keeps no
clock. Each caller lays out its own fields, compares the change with
the task's offered `amend` act by jig's admission (`c::admission::
amendment`), and sends it, as `Amend` or `Propose`, as 1.6 says.

- **A draft is made from the task's document** as read,
  `amend::Draft::of(&c::Task)`: its base is the document's
  `spec_revision`, and the controls the amendment names, as the
  document shows them, the same as jig's `OwnerAmendmentState` holds
  them, with the tracking and the LLM choice: its tracking (`priority`,
  tracked when supplied), its spec (`SpecificationDocument::whole()`),
  its authority (`OwnerAuthorityDocument::whole()`), its instructions,
  its dependencies (its links' numbers, in order) and its `llm`.
- **An edit sets one control,** whole, and leaves the rest as the
  base holds them:

  | Edit | Sets in the `Amendment` |
  |---|---|
  | `track(Tracking)` | `tracked`: `Tracked { priority }` or `Untracked` |
  | `deadline(Option<Wall>)` | `authority`: the base's, with its `deadline` replaced |
  | `instructions(Text)` | `instructions`: whole; empty is none, explicitly |
  | `words(Text)` | `spec`: the base's, with its `words` replaced |
  | `dependencies(List<u64>)` | `dependencies`: the whole list |
  | `llm(LlmChoice)` | `llm`: the whole choice |
  | `address(AddressChange)` | `addressed`: the person task's new address and the person revision it names |

  An edit equal to the base is no change, but for an address, which is
  always one: re-addressed to where it waits now, a person task is let
  go by a party who took it (jig's `domain/tasks/changes.md`).
  `amend::address(&c::Task, c::PersonAddress) -> Option<c::AddressChange>`
  makes the change a read allows: the address at the document's
  `person_revision`, and none for a document without one, which is no
  person task; every caller that sends an address, a draft's or one
  named whole, makes it so. A deadline needs the base's
  whole authority and words its whole spec: where the document cut it,
  the edit answers `Cut::Authority` or `Cut::Spec`, and the draft is
  unchanged, so a page shows the field read only and `ctl` says so.
- **Its revision.** `Draft::revision()` is the base's. `Draft::moved
  (&c::Task)` says whether a document read since has another
  `spec_revision`: a page's draft is then stale, and says so before
  sending; `ctl` compares it with its `--rev REV`.
- **The change,** `Draft::amendment() -> Option<Drafted>`: none
  without a change; else `Drafted { revision: Option<Revision>,
  change: c::Amendment }`, the revision absent exactly when the change
  is a tracked task's priority alone (jig's `domain/tasks/changes.md`
  ), and the base's otherwise, an address among the changes that name
  it.
- **What is whose.** The editor holds the rules of a draft (what
  each edit keeps, what is cut, when a revision is named). The words
  and fields are the caller's: a page's form, `ctl`'s flags and its
  lines on stderr.

## 2. Open questions

- **A moved delegate's own tasks, on a close's sheet** (1.6, *The sheet*): jig's
  impact counts every task below the chat, with no count per delegate, so "N tasks
  below it are cancelled." takes off the delegates listed and still counts a moved
  delegate's own tasks, which closing doesn't cancel. A count per delegate is a new
  fact of a protocol, the user's. Default: they stay in N.
- **What a page keeps across a restart:** where the client was, the
  composers' words and the form's values, as records of temper's own,
  which the terminal keeps beside `jig-client`'s small store, never in
  it (jig's `protocol/client/small-store.md`, section 1). Default: none; the client
  opens at the inbox.
