# Documents

## 1. What they are

- **A document** is a typed value about one subject, from the engine's
  committed state, as the party may see it. Its owner is jig, or the
  application: jig's documents carry jig's values only.
- **jig's documents:**
  - *`me`:* the signed-in party's own document, always about whoever the
    credential belongs to: the party (number, kind, login and name);
    their projects, each with its number and its name, which every
    project has and no other project of the deployment shares
    (`../../../domain/model.md`, 6.1), its public identifier while it is
    public, none while private (`../../../domain/people/public-reading.md`, 2 and
    8), their role there and their pool;
    what waits for them, counted exactly (questions, proposals,
    escalations, person tasks), and their updates unread, counted up to
    a limit; their read position over results; each project's merged
    templates (`requests.md`, 6), and for each with one a `resolved_default: LlmChoice`
    beside them (`defaults`, by the template's number), separate from its
    configured LLM intent. The preview is what a run's check of a task
    given none and keeping no account would take now
    (`../../../domain/accounts.md`, 4.3): the first model in charter order with
    an account usable now, at its default
    effort, under the effective creating authority and the current receiving
    party as payer (`../../../domain/engine/views.md`, 1), so that it is one of
    the project's offer (below) where that authority's models are. An
    explicit choice previews itself; an omitted choice with no usable model
    or an executor still *chosen* gives `None`. It grants and reserves
    nothing. Shapes retain configured intent, never the preview, and
    a clipped catalogue gives no order from which to guess it. Relevant
    account/source, policy, membership and effective-executor changes
    invalidate it (`README.md`, 5.2). The document also gives what they may create
    in each project, as the bounds a draft is compared with (3);
    their role in the deployment, if
    any; the **offer** in each project, what they can pick there now, one
    list that every client draws from and works nothing out, so that a page
    can't show what it leaves out: the models of the brief's catalogue
    (`../../../domain/engine/briefs.md`, section 1), in the catalogue's order, that
    their authority there allows and an account can run, an account of the
    model's provider being usable now and serving them
    (`../../../domain/accounts.md`, 4.3). A provider with no such account gives
    no model, whatever configuration declares of it. Each model has its LLM
    provider, window and output, the efforts it takes, low to high, the
    default among them that a task given none takes (`effort`), its prices
    (amounts per `unit` tokens in the deployment's spend unit; cache writes
    are priced at input when no separate price is given), its optional
    description and shown name, its setting's `number`
    (`../../../domain/engine/settings.md`, 11), where it came from (`source`: the
    binary's, a provider's list or a person's), whether its prices are
    `estimated`, the wall time until which it is new (`new_until`, a week past
    when a list first gave it, none for one no list brought; the client compares
    it with its clock, as it says "2 hours ago"), the accounts that can run it for them, the one a run would
    take first first, and whether a picker shows it (`picker`), so that a
    task's LLM choice (`requests.md`, 2) is drafted from it; how many models the
    catalogue holds past the bounds of the charter a project's chats run
    under (`left_out`, `../../../domain/engine/settings.md`, 11); the most bytes a key may have,
    which bounds a request carrying one (`requests.md`, 1), and the most a pasted
    sign-in address may have, the driver's redirect bound; the most bytes
    a party's words may have (a task's spec, a message, an answer) and an
    amendment's, so that a client says the bound before it sends; and the retention of the
    engine's answers (`requests.md`, 1). It carries no version: the stream's hello does
    (`README.md`, section 6);
  - *a task:* its header, overview, budget and authority; its fields
    (`requests.md`, 2): tracked and priority, label, flags, when it was made
    (`made`, its record's `created_at`), last activity, and what it
    waits on; the LLM choice its next run takes, what it was given or, given
    none, what its check would take now (`../../../domain/accounts.md`, 4.3):
    its `later`, when present, as the choice's main and sub-agents with
    `later` none, once the task's reasons hold no `first` and its charter's
    runs do not resume; otherwise the first choice with its `later`
    beside it (`../../../domain/tasks/README.md`, section 3, and `../../../domain/tasks/messages.md`, section 6),
    and none for a task given none that no account serving its payer can
    run, so that a client names no model it wasn't given, with the creator's
    optional account preference,
    the separate account kept by the last successful claim, and its next-run
    account/source wait with the source's typed state and optional wall reset
    (unknown stays unknown), or a credential or host wait. `Task.run_wait`
    also carries `Refused(TaskWaitRefused)`: `Hello { kind, at, why,
    figures }`, the fleet's latest observed allowed-kind refusal,
    suppressed by any admitted connected non-draining allowed-kind host;
    or, while such hosts exist and every one of them falls short of a
    bound of the run's charter version, `Bounds { kind, why, figures }`
    (`../../../domain/engine/fleet.md`), `why` an `AgentShortfall`, none
    where each falls short of another bound. `figures`, `Option<Figures>`,
    `Figures { required: u64, offered: u64 }`, is what the bound needs and
    what is offered, in the bound's own unit, a duration in milliseconds:
    for `Hello` the refused host's, for `Bounds` the most any of those
    hosts offers (the least, for the checks timeout); none for a cause
    without amounts. An `AgentShortfall` names its charter on every
    charter bound, `Model`, `Window` and `Output { charter, model }`,
    `Turns`, `Time`, `Waiting` and `Checks { charter }`; `Spend`,
    `Message`, `Inbox` and `Unacknowledged` are the deployment's, with
    none. An Open task runs
    on a program source as any task does, so no run waits for its program
    (`../../../domain/hosts/charters.md`, 1). The client uses its own `HostKind` and
    `HelloRefusal` vocabulary; the canonical deployment translation keeps
    the shortfall context limits. These are derived document facts, never
    a new hold or stored state. A task line carries no run wait. Its hold
    and its escalation if held; its dependencies, each a task's number
    and phase, pending or done, with when and by whom each was added
    after the task was made, and whether they keep it
    (`../../../domain/tasks/lifecycle.md`, 8); the proposals waiting in its subtree; how
    far its activity has committed; for an agent task, `Task.settings:
    Option<TaskSettings>`, `TaskSettings { task: u32, run: Option<u32>,
    newest: u32 }`: the settings revision it records, its live run's
    while that run was prepared under another, and the newest's number,
    so that a party sees which charter version its runs read and what a
    move would change, when the task may move
    (`../../../domain/engine/settings.md`, 3 and 4); none for a procedure's or
    a person's task; and the acts the party may make on it, with what
    each would do (3, 4);
  - *a task line:* one record for every list of tasks (`README.md`, 4.1): the task's
    chip; its project and its requester; tracked and priority, label
    and flags; when it last changed, its last activity, and when it
    ended, if it has; its deadline; what it waits on; its budget spent
    and left; its live subtree counted by phase class, with how many are
    held below; its direct delegates counted, those done and all of them,
    cancelled ones left out, true however many they are and whichever
    page shows them (`../../../domain/tasks/README.md`, section 3); and, for a task
    with a conversation, its requester's
    read position over its turns and their unread count;
  - *a chip:* a task's number, title, executor's kind, phase; a closing
    phase says what the task waits for: its delegates still closing,
    counted, and the connectors it waits on to settle, by number. Their
    effects are the connectors' own, and jig's documents never count
    them;
  - *a tree:* a task's live subtree, as chips, with the proposals waiting
    on each, folded beyond the watch's window into counts by phase, with
    counts of what is held below. Each unshown descendant folds under
    its nearest shown ancestor, including the root. There are at most
    as many folds as window rows; overflow joins the root's fold,
    retaining every phase and held count;
  - *an inbox:* the party's entries, each naming its object and its
    project, in its groups, within a window, with the party's read
    positions: one over results, one per conversation over its turns. An
    entry's `projects` is empty but on a pending proposal of settings: there, its
    proposal's `projects`, or, for one of the deployment's, each project the
    party owns, at most 32, the lowest numbers first
    (`../../../domain/engine/settings.md`, 6). `shows_in(project)` holds where `project` or
    `projects` names it; the `inbox` read's project filter is the same rule;
  - *a proposal, an escalation, a question, a person task, a result:* each
    object whole, a proposal of settings naming its `projects`, empty for one of
    the deployment's (`../../../domain/engine/settings.md`, 6), with the acts the party may make on it and what each
    would do; a question's words within the document's text bound, cut
    at a character's boundary and saying so (3), their whole in the
    asking task's activity, where the turn that asked has it all, so a
    question is never refused, nor answered busy, for its length; a pending proposal of settings with its review
    (`../../../domain/engine/settings.md`, 6): the revision it was made against,
    the newest, and for each item the setting, what becomes of it, what
    it is now (its value, following its default or the change of which
    revision, or not offered) and what it would be, each value within
    the document's bound or with its size, as a revision's items are
    (`README.md`, section 4), an item of an application's kinds with its size alone,
    the application serving its values, and an item of a project's own
    policy with that project's policy now and as it would be, each whole
    within the document's bound or with its size (`../../../domain/engine/settings.md`,
    2). A retained task-origin proposal keeps its original reason, at the
    live reason's bound, and its decision state; one of settings also keeps
    its items and `projects`, so a client can draw it after a reload while
    its record is retained;
  - *an activity line:* one of a task's words, run boundaries, turns,
    results heard, news heard, decisions, in commit order; a line of a change
    to a proposal of the task's names that proposal's number and says whether
    the change was its making, so a client can tell when a proposal was
    made and a decision from the making; a turn as its
    bytes, within the page's byte bound, which the client's link
    decodes, with no read of its own, into the client's turn or into a
    turn that does not decode, with why (../client/conversations.md): a turn
    that alone passes the page's bound comes with its size instead, and
    does not decode, as too large; one the engine could not read whole,
    as malformed;
  - *a project's,* its policy (its templates' overrides among it), roles
    and spend, a note; a project's document carries its recurring
    tasks' lines, each saying when it next runs: when the next period
    begins, where the period's source said so; otherwise at the next
    period, with the current period and when it began
    (`../../../domain/authority.md`, section 7);
  - *the deployment:* hosts, each with its program sources' logins;
    the deployment's LLM accounts, as the party's own are (below), with
    the runs using each and the tasks waiting on each; capacity;
  - *the party's accounts:* the party's own LLM accounts, each with its
    LLM provider, display name and holder; what its provider says of it,
    its facts (below); its sources in order, each
    with its state: usable; limited, until when where the provider said;
    refused, and why; needs sign-in, and for a program on which hosts;
    misconfigured; unloaded, where and why; a program source's state on
    each host; and its uses. With them, the facts the providers' terms
    ask an application to show: whether each is a plan, its LLM
    provider's name and manage-usage address, and a plan's first use;
    the party's sign-ins under way, and how the last ended: made,
    renewed, or refused for a subject another holder holds; the checks
    they asked of a program's login that a host has yet to answer, with
    those hosts; whether a terminal of theirs can be lent, their
    program's login under way in it, and how the last ended
    (`../../../domain/terminal-login.md`, 5); and two facts a client waits on: *usable*, that an
    account which serves the party's work (their own, those whose uses
    name them, and the deployment's: `../../../domain/accounts.md`, 4.3)
    has a source usable on a host, and *settled*, that no sign-in or
    check the party asked is in progress
    (`../../../domain/accounts.md`, sections 3 and 5). It holds no value,
    and the words and screens are the application's.
  - *the settings:* the newest revision's number; each setting, by kind,
    scope and number, with its name where it has one, whether it
    follows its default or a change, and the revision that last changed
    it, a reset included, which leaves it following its default from
    that revision; each revision held, with who decided it, when, the proposal it
    came from, and how many live tasks hold it; the pending proposals of
    settings, by number (`../../../domain/engine/settings.md`). It
    carries no value: one setting is read whole on its own, from the
    newest or a held revision (`README.md`, section 4), and one of an application's
    kinds from the application, whose connector keeps its value (17.5),
    jig's read giving its state and size alone. Seen by a party with a
    deployment role, and a project's items by a party with a role there.
Program host facts use each worker's whole configured name. Inventory still awaited
is unknown, including after a reconnect; a disconnected worker's earlier inventory
is not a current login fact. The deployment lists all issued names, including
retired names awaiting cleanup; a party's accounts watch lists only that holder's
names. Each row distinguishes the login observation from the last sign-out's
result and the desired cleanup state. Bounded diagnostics say when cut.

- **What a task waits on** is the engine's to compute, since only it
  holds the tree and the inbox together: a party (who), for a question,
  a proposal, an escalation or a person task in its tree; its
  dependencies, for a kept task: the pending ones, the oldest first, and
  how many; its delegates; news; or nothing (`../../../domain/tasks/README.md`,
  section 3).
- **Its last activity** is when it last heard words, replied, or had a
  delegate end: a wall time, kept on every task, by which lists order.

An account row in the party's `accounts` or owners' deployment document
also carries its provider's configured name and optional manage-usage
address. The name uses the document's cut-aware text bound; the address
is whole, at most 2,048 bytes. The client link rejects addresses outside
`valid_manage_usage`'s ASCII HTTPS subset before publishing either account
document form. This subset is safe for the view's external-link builder;
configuration applies stricter host-label and port checks.
Provider and account numbers remain whole.
The account's plan flag follows its current sources and configured
signed-in-plan flag; program sources remain plans. These facts require no
read of another holder's accounts. Deployment capacity history labels
account sign-in, sign-out and change without their provider, holder,
subject, source, key or digest payloads, and includes the accounts',
holder's accounts', sources' and sign-ins' room limits under the same
keyed retention and bounded entrance-history rules as other actions.

An account row with a list (`../../../domain/accounts.md`, 3.7) also carries it
(`list`): when its last good read was (`read`), when its last attempt ended
(`tried`), why that failed (`failure`), and how many names took no number
(`left_out`). None says it has no list, so the client offers no reading.
Reads move these and not `me`, unless they move the offer
(`../../../domain/engine/settings.md`, 12). `left_out` in `me` also counts the largest of
the accounts' `left_out` its party may see, so that the page's one count holds both.

An account row in either document also carries its facts
(`../../../domain/accounts.md`, 3.7), at most 24, each with its kind, label,
value, when it was said, the source it came through and its origin. A row
carries only those whose kind, origin and window state this build registers, and
applies each kind's audience to its reader: a holder's kinds to the account's
holder alone, an account's to whoever sees the account. A party's `accounts`
holds the party's own accounts, so it carries every registered kind. In the
deployment's, whoever may decide on the deployment's accounts reads for its holder
(`../../../domain/accounts.md`, 3.2), and any other reader with a deployment role
gets the account's kinds alone. A fact is whole, at its
own bounds: neither the document's text bound nor a line's `cut` touches it.
The wire and the client's contract carry the registry's numbers, and the
contract names those a client reads. The client link refuses a share past 10000.

The public provider catalogue in `me` carries the configured provider number,
cut-aware name, sign-in and key offers, whole program names (at most 32 bytes
each), the no-credential offer, signed-in-plan flag and whole manage-usage
address. Each offer carries the actual key ceiling; `me` also carries that shared
ceiling for a common key input. It is the smaller of startup's accounts token
limit and the client wire's carried-value limit. Offers describe configuration,
not a running backend. Provider and program lists use the document line bound;
each has its own cut flag. Absence from a cut catalogue means nothing.

`accounts` also carries volatile readiness rows keyed by provider and payer:
the party itself, then every project it may read, including deployment-role
membership where it is a readable project. Rows expose no serving holder,
account or source identity. Coverage is the same current people/uses join as
run selection. A row is usable when a serving source can run on an eligible
connected host; unavailable when the complete known evidence rules that out;
unknown when program login evidence is still missing. Slot and budget
admission, task charter restrictions and account pins remain the concrete run's
check. Settled is optional: false for this party's pending sign-in at that
provider, unknown when program checks cannot yet be known, true otherwise.
The readiness list has its own cut flag; it never makes a negative statement
about an omitted provider/payer. Decisions use full domain state, independently
of displayed account/source/host prefixes. The document keeps the aggregate
usable and settled fields beside the rows.

## 2. Revisions

- **One for each thing a party decides about,** which moves only when
  that thing changes, never with a priority, spend, a delegate's phase
  or a run's progress:
  - *a hold's,* on a held task: what holds it, and why. It moves when
    the task is held anew or its reason changes, never when its
    escalation passes up. A release and a decision about its escalation
    name it;
  - *a spec's,* on a task: its words, parameters, wake policy,
    dependencies, authority, instructions and whether it is tracked; an
    amendment names it;
  - *a person task's:* its question, options and the results it shows;
    an answer names it, and an amendment of its address;
  - *a note's,* as the notes child keeps it; a correction or a deletion
    names it.

  A proposal never changes once made, so deciding it names its number;
  where it waits may move, and a party it no longer waits for is refused
  as no longer the one it waits for. Stopping, cancelling, moving and a
  change of priority alone name no revision.
- **Where an escalation waits is checked apart from its hold.** Deciding
  an escalation (releasing the task from it, leaving it held, passing it
  up) needs the escalation to wait for the party, and is refused as not
  waiting for them otherwise. Releasing, amending or cancelling the held
  task needs the task in the party's tree, or their role in the project
  to allow it, wherever its escalation waits
  (`../../../domain/people/acts.md`, 1): a release that names the hold's
  revision from before a pass up is made, not refused as moved.
- **The revision is the promise below the presentation.** Whatever
  showed the person a decision, the request names the revision of a
  document the engine gave, and is refused if it moved, for every
  client and every presentation alike.

## 3. What else they carry

- **Acts offered.** Each object's document says which acts the party may
  make on it now, as the engine judges it, and what each would be:
  *made*; *proposable*, and to whom, where it is beyond the party and a
  proposal of it could be accepted; or *refused*, and why (role,
  authority, nobody who could accept a proposal of it). An act whose
  answer depends on what the party writes is offered *bounded* (below).
  A presentation offers what the document says, and the engine judges
  again when the request comes.
- **Offered with bounds.** Creating and amending are made, proposable or
  refused by what the party writes: a budget, a deadline, a grant. A
  document cannot say which for every draft, so it gives the engine's
  facts a draft is compared with, in two bounds, and what happens beyond
  the first:
  - *creating,* in `me`, per project: the executors the party may create
    with, and their pool left; within, the largest budget and the latest
    deadline they may give a task alone, and the authority they may give
    it, from their role and their pool: whole, their role's, one level
    shallower in delegation and its tasks within a tree's; and, past
    that, the largest
    budget and the latest deadline any root task of the project may
    have, from its spend per period, its policy's ceiling and the
    deployment's rules;
  - *amending,* in a task's document: within, the whole authority the
    party may give alone (their role's, its budget no more than their
    pool has left, its tasks no more than the task has left), the budget
    they may add to the task, and the grants they hold, listed; and,
    past that, the policy's ceiling, its budget no more than their pool
    has left;
  - *beyond the first bound:* proposable, to the roles the project's
    policy names for that kind of proposal, in their order; or, where it
    names none, refused as lacking authority, saying that no proposal
    could be accepted.

  A presentation, or the helper `shapes` (../client/helpers.md),
  compares the draft with those facts before the request is sent, and
  says which it will be. A task is within the first bound when its
  executor is listed, its budget no larger and its deadline no later
  than the bound's (a task with no deadline being beyond any bound that
  has one), and its authority within what the party may give; a budget
  `AsAllowed` is within it, and its answer says what it gave; a
  deadline given as a span (`requests.md`, 2) is within both bounds, since no
  presentation reads a clock: the engine judges it at the decision,
  as the wall it then makes (`../../../domain/authority.md`, section 7),
  and answers a `Create` it puts past the first bound as beyond
  authority, saying whether a proposal of it could be accepted. Past the
  second bound, or with an executor not listed, it is refused as not
  offered, and nothing is sent. An amendment that widens no authority is
  made. One that widens anything, even while it narrows something else,
  is judged whole: the authority it leaves the task must fit the first
  bound, the budget it adds be no more than the party may add, and each
  grant it adds be among those listed; past the ceiling it is refused as
  not offered, and nothing is sent. Between the two bounds, a
  presentation sends `Propose` where the offer says proposable, or says
  the refusal in place, sending nothing. That is the engine's word read,
  not authority computed: the engine judges again when the request
  comes, by the rule that made the offer. No read assesses a draft.
  Creating's charter and procedure lists are complete facts, each received
  within `documents.lines`. If either list does not fit, the creating
  offer is refused at the authority limit; a shortened list never stands
  for the executors the party may name.
- **Reordering,** in a project's document: whether the party may change
  its tracked tasks' priorities, in one compound request of
  priority-only amendments (`requests.md`, 4). It is judged by the party's role in
  the project alone, as a change of priority is ("maintainer, for
  tracked or priority", `../../../domain/people/acts.md`, 1), and the policy's
  word on that role's amending: made or refused, never proposable, and
  never bounded, since a priority asks no authority and names no
  revision (2). Amending's offer in a task's document bounds what the
  party may write of the task's authority and budget, not its priority,
  so a client that moves tasks reads the project's offer, not each
  task's: a party who may amend a task of their tree may still not
  reorder it.
- **Parts.** Where a jig document names an application's value, it
  carries the part's reference: its kind, its owner, and jig's key (a
  resource's path, a kind's number, a task's or a proposal's number).
  Its owner is the connector whose value it is: every part jig's
  documents name is a connector's, so the number a part carries is its
  owner number (../root.md, section 15). The value is in the application's
  own documents.
- **Times are wall times,** which the store keeps, so a document means the
  same after a restart. Accounts' source limits and retries carry the
  wall fact retained when they were created (`../../../domain/accounts.md`,
  3.5): a task's wait, the account's and the deployment's projections
  all use that exact fact, as the store does, never convert again at
  read time. Their timers remain monotonic.
- **Bounded:** each document within its limits, in bytes and windows,
  never in rows sized for one client's screen. A path and an authority
  are bounded by the engine's names and authorities, not by a window's
  rows (below).
- **A path crosses whole.** A resource's path and a pattern are names,
  compared segment by segment (`../../../domain/authority.md`, section 4),
  and the client acts on them as they are (a filter by a resource, a
  draft's pattern). A document carries each whole, with as many
  segments as the engine admits in a name (the most of its authority's
  and its tasks' bounds on a name's segments): the engine never cuts
  one, nor refuses a document for one. A document's bound on its short
  lists is not a path's.
- **An authority crosses whole.** A draft is compared with an authority,
  and an amendment's authority (`requests.md`, 2, `Amend`) is drafted from the one a
  document carries, so the client holds each as the engine does: a
  task's authority, and each authority an offer's
  bounds, a proposal or an amendment carries, with its grants, its
  executors and its notes' scopes; and an amending offer's grants
  listed and the implications it names. A document carries each list
  whole, with as many entries as the engine admits there: grants, a
  grants listing and notes' scopes, the most of its authority's and its
  tasks' bounds on an authority's grants; executors, the most of its
  authority's bound on executors and its tasks' on executor kinds;
  implications, its authority's bound on them. The engine never cuts
  one, nor refuses a document or an offer for one, nor marks one cut. A
  document's bound on its short lists is not an authority's.
- **Texts are UTF-8,** words for showing. A source's text is UTF-8 where
  it enters the engine: a party's words, refused at their request's
  entrance when they are not; a task's spec, its words and its text
  parameters, refused with a spec's other faults wherever the tasks
  child admits a spec, a batch's, an amendment's or a proposal's,
  whoever made it: a party, an agent's tool, a procedure's step, a
  recurring task (`../../../domain/tasks/README.md`, sections 3 and 4); a task's
  result's texts, such as its words or a failure's reason, refused as a
  result its contract does not admit wherever the tasks child admits a
  result: a run's end, a procedure's step, a restore
  (`../../../domain/tasks/README.md`, section 3); an identity's login and name,
  refused where they are minted; a run's output; an application's words
  through its own documents. A value that is not text, such as a
  commit's id, is a task's parameter or a result's words only as text
  its application spells, such as its hexadecimal digits. A text cut to
  its bound is cut at a character's boundary, never inside a character,
  and says it was cut. So every text a document carries is UTF-8, as the
  wire's text fields check (`../../../protocol/client/wire.md`, section 1), and
  the engine never makes a document its wire refuses. Connector hold accounts
  carry their complete retained words, bounded by the tasks child's 512-byte
  account bound independently of `documents.text_bytes`; their task and chip
  document bounds include that room.

## 4. A decision's effect

- **What a decision will do is data,** in the documents that offer it,
  from the engine's state as the document was made:
  - *accepting a proposal:* what it grants and what it costs, and the
    pool that funds it, after;
  - *releasing a held task:* what lifting its hold does: the task judged
    afresh, a run started again, or an uncertain effect made again;
  - *cancelling:* what ends: the live tasks below the task, counted by
    phase; and, of the task and those below it, the live runs (each
    attempt claimed whose answer its task has not heard) and the pending
    proposals (each a task made that no holder has decided). Its effects
    are its connectors' (1): each withdraws the entries not yet sent and
    settles those in flight, and the document counts none of them;
  - *moving a task:* the reservation that moves to the party's pool.
- **The confirmation is the application's.** jig holds the revision,
  the "once" and admission below the API; whether and how a person
  confirms a deciding act, and which of these facts it shows, is the
  application's choice, which it makes wherever it confirms. jig's
  reference UI keeps jig's confirmation (../view-state.md, 5); jig's check of
  it is ../testing.md's, 6.2.

## 5. Configured-bound holds

- **A typed category.** `HoldCategory::Exceeded(Exceeded)` carries the
  client's own `Exceeded { bound: Bound, value: u64 }` and the nine
  `Bound` tags of `../../../domain/hosts/hub.md`, 3. The payload is `Copy` and
  inline. `value` is the configured allowance in the bound's unit,
  never a measured size. `History` does not identify the ceiling's unit;
  its value is still carried and stored. People's `HoldReason::Exceeded(Exceeded)`
  carries the same cause in revision refusals, with people's own types.
  Translations between the domains' copies are exhaustive, without a
  wildcard.
- **Words name the allowance.** Jig's hold phrases, including the
  reference view's, take `N` as a Number parameter except for `History`,
  whose phrase takes no number because its unit is not known:

| Bound | Phrase |
|---|---|
| `Reasoning` | Held: the model's reasoning went past its configured allowance of N bytes |
| `Metadata` | Held: the model's metadata went past its configured allowance of N bytes |
| `Items` | Held: the model's answer went past its configured allowance of N items |
| `Answer` | Held: the model's answer went past its configured allowance of N bytes |
| `Tools` | Held: the run's tools went past their configured allowance of N tools |
| `HistoryItems` | Held: the run's request went past its configured allowance of N messages and blocks |
| `History` | Held: the run's history no longer fits its configured allowance, even compacted |
| `Window` | Held: the run no longer fits the model's window of N tokens |
| `Output` | Held: the request went past the model's output ceiling of N tokens |

- **Release keeps its authority.** The hold is lifted by an authorized
  `Release`, not by words. The task's immediate hold and release rules
  are `../../../domain/tasks/lifecycle.md`, 6 and `../../../domain/tasks/holds.md`.

## 6. Placed holds

Placed holds are `../../../domain/tasks/lifecycle.md`, 10's.

- **A task held by a placed hold** shows `HoldCategory::Placed { hold
  }`, and its document names the hold: its owner (a party or a task),
  its mode (`Stop` or `Drain`), its scope (`Subtree(task)` or
  `Project`), its reason, what lifts it (an event, a task's end, a
  time, or nothing), when it was placed, and its revision; and whether
  the reader may lift it, or only propose its release.
- **A project's placed holds** are a read of their own, `holds{project}`,
  each as above with the tasks it holds counted, and the tasks it
  exempts.
- **A party's inbox** shows a proposal to release a task from a hold of
  theirs as any proposal waiting for them, naming the hold.

## 7. Job tasks

Job tasks are `../../../domain/tasks/lifecycle.md`, 3.1's. A job task's
document shows its executor's kind, `Job { kind }`, its spec and result
as its application's part describes them (3), its host and its
attempts. No party creates one: `Create` naming a job is refused as an
executor not offered.

The job executor's visible name is `job KIND`, KIND its numeric application
kind, in the shared client and the application's command-line task reads
(2026-11-04).

A job's result, heard by its requester, is an activity line like any
delegate's result (1): its word's result kind is `Job`, with no payload,
and its words are empty. The job's result bytes are its procedure's to
read, not words, and no client is sent them
(`../../../protocol/client/wire.md`, 1). The shared client shows the line
as a result card of kind job with no words; the application's
command-line activity prints its kind as `job`, as it prints `report`.
