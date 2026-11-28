# Briefs and notes

## 1. Briefs

The brief child domain builds every run's brief from sections, each
within a budget, the brief within its own: sections follow why the run
runs, its reasons (`executors.md`, 1, step 2; ../tasks/messages.md, 6), those it runs for are
required, the rest optional; what does not
fit is cut in its kind's order and says how much; sections are gathered
within one deadline, while the task is preparing, and a required one
missing fails the brief, which pauses the task or counts a failure by
why (1.1), no attempt spent (../tasks/lifecycle.md, 2).

Its sections come from two places:

- **The core's,** which it holds typed: the task (its number and project,
  which a run names in a note's scope, its requester, its spec and, unless
  its requester ends it (../tasks/lifecycle-endings.md, 2) or it stands
  (../tasks/standing.md, section 2), its contract,
  and its lineage: its requesters' specs, cut); its dependencies' and
  inputs' results in jig's terms, and, for a run after its task was kept
(../tasks/lifecycle-endings.md, 1), what it waited for: each dependency added, when and by
whom; its delegates, their states, and the
  revisions a decision about them names, each one's spec's and a held
  one's hold's (`executors-tools.md`, 1), and the results that came since (the plan, for a
  coordinator); its earlier
  attempts and why they failed; the calls its task committed since its
  last turn, when it does not resume; what waits for it, the waiting
  section: the questions asked of it still open (1.2), the words it
  keeps unrecorded (1.3), and the proposals
  and escalations waiting for its decision; the index of its notes; for a run that
  explains to its person, that person's preferences, whole (2.1); for a
  standing task, its Now, whole (../tasks/standing.md, section 6); a
  transcript's tail,
  when it does not resume, but for a standing task's session's first run
  (../tasks/standing.md, 5.4), the window's last turns as text rendered by the
  host's protocol layer when loaded, a turn too large to load as its
  fixed marker (`executors-transcripts.md`, 1), within the tail's own budget; and,
  for a run whose tools hold `delegate`,
  the catalogue: each model its authority allows (../authority.md, section
  3), with its LLM provider, its window and output, its efforts, its
  prices, and what it is good at, a description its configuration gives
  (`../../protocol/configuration-engine.md`, section 1), and whether an account
  that serves the run's payer is usable now, limited and until when, or
  needs sign-in (../accounts-sources.md, 2), so that a planning agent gives each
  task it delegates the LLM choice its work needs, by capability and
  cost (../tasks/README.md, section 4).
- **Its charter's documents' (`settings.md`, 9),** planned and gathered by the
  connector that owns each document's resource as it is when the run is
  prepared, each optional, titled by the document, after the core's
  sections and before the task's resources', for every run whatever its
  charter's parts.
- **Its connectors',** which the engine asks each connector that serves
  sections to plan, then places by priority and budget but never holds
  (../model.md, section 5; 1.4): for what the task names of its system,
  the connector plans its sections, gathers each, keeps it, and says its
  size; the brief child decides what fits, asks for cuts by size, and,
  once the run is prepared, the root takes each section from its
  connector to put the assignment together (../connectors.md, section 9).

A section is loaded or read as it is cut, asking for no more than it
keeps. The host's protocol layer renders the whole as text, in order.

**The task's lines.** The task's section closes with a line naming the
task, `[This task: T, in project P]`, and one naming its requester:
`[Requested by task N]`, `[Requested by deployment for project P]`, or,
for a person, `[Requested by person N, NAME]`, NAME their member name in
the task's project (`../people/README.md`, section 4). Each control byte
of the name, below 0x20 and 0x7F, is written as a space, so that the
line stays one line. A person with no member name there, or an empty
one, is `[Requested by person N]`. The name is for the run's words
alone: a run still addresses its requester by number. Both lines are
cut as the section's other text is.

**Within the deadline.** The brief's one deadline covers its gathering
and its cuts, and every section that can be cut is cut in time, never
late:

- **Its owner learns the time it has.** The engine gives each connector
  the brief's deadline with each gather it asks for (1.4). A connector section that reads in several reads (pages, one log
  after another) ends ahead of that deadline, at a cutoff of its owner's
  choosing that leaves the brief time to cut: it completes with what it
  has gathered, its read still unanswered left and its late answer
  dropped, and its marker says what it left and how to read it. That
  marker says the section was cut at the brief's deadline, apart from
  any cut for room (`cut at the brief's deadline: N of M files`, in the
  section's own terms).
- **Late only with nothing usable.** A section ends late only when it has
  gathered nothing usable by its cutoff, such as the pull request it is
  about read at all; it then waits on to the deadline, and a required
  one still gathering there fails the brief (1.1).
- **The settle point.** The brief child stops waiting for optional
  sections seven eighths of the way from its plan to its deadline. One
  still gathering then is dropped and marked late, as one past the
  deadline is, so that the sections it has get the last eighth for
  their cuts, and a required section ready in time is never failed by an
  optional one that is not. A required section still gathering at the
  settle point is waited for to the deadline. An owner's cutoff comes
  before the settle point.

A brief has no inbox section. The messages waiting in the task's inbox
that the run is offered go with the assignment as its start's messages,
and the run reads them as messages, whole (`executors.md`, 1; ../tasks/messages.md, 2 and 6).
Nor has it a section for its charter's parts: they are its instructions
(`executors.md`, 1, step 4).

### 1.1 A brief that fails

A brief fails whole: a required section is missing, the brief child
refuses its plan, a required section a connector planned has no slot
(1.4), or the core cannot make its parts. Every such failure carries a **failure**: how it counts, and
why, as a section's name and fixed words. Both name and words are
fixed in the code, so a reason cannot carry a run's words or a
secret; the one thing a reason may carry beside them is a task's
number, where a `results` failure names the task it read (below). A
connector chooses its own, and names no more than the kind of
its section and of the error its system gave.

How it counts decides what becomes of the task:

- **Capacity:** nothing is wrong but room, which frees by itself. The
  task pauses and is due again after a backoff, no try spent, with no
  bound on how many times (../tasks/lifecycle-holds.md, 1).
- **Transient**, **infrastructure** or **permanent:** a failure of that
  class (../tasks/lifecycle-holds.md, 1). The task backs off within that class's
  allowance and, past it, is held for `Failures` of that class, its hold
  naming the reason. A section whose system failed for now, or that its
  system did not answer by the brief's deadline, is the infrastructure's:
  its allowance rides out a system down for minutes.

A brief that fails, however it counts, gives its brief slot back at once,
its sections still gathering dropped, before its task pauses or backs
off, so another task's brief takes the slot while it waits.

Which is which, by what `executors.md`, 1 prepares:

| The failure | Counts as | Section and words |
|---|---|---|
| no brief slot: the brief child's table is full, or this brief is still gathering | capacity | `brief: no brief slot is free` |
| a connector says it has no room, or that its system admits nothing yet | capacity | the connector's |
| the core's result reads, its notes or its proofs have no room | capacity | `results: no read flight is free`, `notes: no room for the index`, `run: no room for its proof` |
| a claim refused (a host's or a writer's room; ../tasks/lifecycle-holds.md, 1) | capacity | `claim: refused` |
| a required section with nothing gathered by the brief's deadline | infrastructure | `SECTION: not gathered by its deadline` |
| a required section whose cut is unanswered at the brief's deadline | infrastructure | `SECTION: not cut by its deadline` |
| a connector says its system failed for now | infrastructure | the connector's |
| a connector says its section is to be gathered again, what it reads having moved meanwhile | transient | the connector's |
| a dependency's or input's result whose load failed: the store could not serve it now | infrastructure | `results: task N's result could not be read: the store answered busy` |
| a dependency's or input's result the store answered with no row | transient | `results: task N's result is missing: the store holds no ended record of it` |
| a dependency's or input's result the store answered with another task's row, or with a row of another project | transient | `results: task N's result is missing: the store answered task M's record`, or `results: task N's result is missing: the store's record of it is another project's` |
| a dependency's or input's result the store answered with its row not ended | transient | `results: task N's result is missing: the store's record of it has not ended` |
| a transcript's window that does not load | transient | `transcript: its window could not be loaded`, or `transcript: its window could not be read` for an open window unread, which closes it (`executors-transcripts.md`, 1) |
| a required section that cannot fit, whole or cut | permanent | `SECTION: does not fit the brief's budget`, or `brief: its required sections do not fit its budget` |
| a section a connector answered past its bound | permanent | `SECTION: answered past its bound` |
| a plan the brief child refuses: more sections than a brief holds, a budget past the limit, a core text past it, a section planned twice, a deadline passed | permanent | `brief: ...`, naming which; for a core text, `SECTION: its text is past the brief's limit` |
| a connector says its system answered wrongly, or will not answer | permanent | the connector's |
| no slot for the required sections the connectors planned, with no optional core section left to displace (1.4) | permanent | `brief: no room for its required sections` |
| a core section the core cannot read within its bounds | permanent | `task: ...`, `results: ...`, `transcript: ...` |
| a brief's notes index refused for any reason but room | permanent | `notes: the index could not be read` |
| no attempt number left | permanent | `run: no attempt number is left` |

**A dependency's result.** The core reads each of a run's dependencies'
and inputs' results from the store, one task at a time (`executors.md`,
1), and a failure names the task it was reading, `N` its number in
decimal, and what the read found:

- **Busy:** the load failed; the store could not serve it.
  No load free yet is no failure: the engine keeps the
  load until one frees (`decisions.md`, 1.3), so no preparation fails for
  room. A busy answer counts as infrastructure, so a store that cannot
  serve reads for minutes holds no task: the task backs off within that
  class's spaced allowance, some 20 to 40 minutes at the defaults
  (../tasks/lifecycle-holds.md, 1), and is held only past it.
- **No row, another task's or project's row, or a row not ended:** the
  store answered, without task `N`'s ended record in the preparing
  task's project; `M` is the number of the task whose row it gave. It
  counts as transient, as the engine's own reads missing do
  (../tasks/lifecycle-holds.md, 1): a result that is truly gone holds past
  that allowance, the hold's words naming the task and what was read.

Each is at most 110 bytes with both numbers at their longest, inside
`REASON_BYTES`, so no hold's words are cut.

A stale preparation and a wait for an account (`executors.md`, 1) are neither: the task
returns to due without a timer, as it does.

**Where the reason goes.** The tasks child keeps it, cut to 128 bytes at a
character's edge (`REASON_BYTES`): on the retry notice, so a task's document
and its chip say it beside the count, and on the hold, whose words say it
(11). It is durable with the record. The core also tells the root each
failure as it happens, with the task, for the application's log: only there
does a failure that then resolves itself leave a trace once the notice moves
on.

### 1.2 The open questions in the waiting section

A question a delegate asks stays open until it is answered or ends
(../tasks/lifecycle-endings.md, 1), but its message is taken from the inbox
by the turn that reads it (../tasks/messages.md, 2). A run that reads
a question and ends without answering it would leave the next run
nothing to go on, so every brief shows the open questions:

- **Which.** Each question asked of the task that is still open: its
  asker holds the question's credit (../tasks/lifecycle-endings.md, 1), with
  the task as its answerer, and the task has heard no answer to it.
  A question this run's start carries as a message is left out, since
  the run reads it whole as a message.
- **Which runs.** Every run that starts from a brief. A run that resumes
  has no brief; its window already holds the question it read
  (`executors-transcripts.md`, 1).
- **Its form.** The waiting section, core kind `Waiting`, opens with
  the line `Questions asked of you, still open, oldest first:` and
  lists them oldest first, each as one line and then its words whole,
  as its asker sent them:

  ```
  Task 12 asks (question 40); answer with message target=12, form=answer, question=40:
  Which of the two error forms does 3.2 mean?
  ```

  The number is the one `message`'s answer form takes: the number of
  the message the task heard, never a number the asker was itself
  asked by.
- **Its place.** Optional, as every section but those a run runs for,
  and kept before every other optional section; past its budget it is
  cut from its end, as a core section's text is, and says how much.
- **Its words.** Read from what the core keeps of each open question
  for its asker's credit, with no store read where it keeps them. If
  the core keeps a question's words nowhere, the brief loads them from
  the question's activity line, as other core sections load what they
  show.

The same open questions are among every run's reasons
(../tasks/messages.md, 6), and one left open wakes the task again
(../tasks/lifecycle.md, 2).

### 1.3 The unrecorded words in the waiting section

A words message a task keeps is taken from its inbox by the turn that
reads it, and stays among the task's unrecorded words until a run
records it (../tasks/messages.md, 2). So that a run that read it and
ended, parked or stopped, without recording it leaves the next run
something to go on, every brief shows them:

- **Which.** Each words message the task keeps unrecorded, oldest
  first. A message still in the inbox is none of them: the run's start
  carries it whole, as a message.
- **Which runs.** Every run that starts from a brief, the run after one
  that was stopped among them. A task whose runs resume keeps no words.
- **Its form.** The waiting section holds them after the open
  questions (1.2), or alone where none is open. They open with the line
  `Words sent to you, not yet recorded, oldest first:` and are each one
  line and then the words whole, as their sender sent them:

  ```
  Person 42 wrote (message 77); once it is in a spec or a note, mark_read messages=[77], or name 77 in that note's records:
  Also add a /ready endpoint beside /health.
  ```

  The sender is `Person N` or `Task N`, its number as the activity
  names it; the number is the message's, the one `mark_read` and a
  note's `records` take (`executors-tools.md`, 1). Where open questions come
  first, a blank line parts the two.
- **Its place.** The waiting section's, as 1.2 says: optional, kept
  before every other optional section, cut from its end past its budget,
  saying how much. Words it cut stay unrecorded, shown again to the
  next run, and listed in the task's document as every unrecorded
  message is.
- **Its words.** The kept `Word`'s, read from the task's record with no
  store read.

### 1.4 Planned with its connectors

The engine plans a run's brief with the connectors that serve
sections, places what they would give within the brief's slots, then
gathers and cuts what it placed, through the connectors' port
(`vocabulary.md`, 2.5; `temper/docs/design/draft/jig-domain-api.md`, 6.6). No root
takes part until the run is prepared and its assignment put together
(../root.md, section 7).

```rust
// jig-domain-contract's parts, beside a document's, a read's, lines' and a public document's asks
pub struct Preparation {
    pub task: u64,
    pub project: u32,
    pub requester: ops::Party,        // a task requester is the task's parent
    pub root: u64,                    // the root of the task's tree
    pub documents: Box<[Document]>,   // its charter's documents with a slot, in the charter's order
    pub core: Box<[Claim]>,           // the core's own sections, in the brief's order
    pub slots: u32,                   // sections left beside the core's and the documents'
    pub budget: u32,                  // the brief's bytes
}
pub struct Claim { pub size: u32, pub limit: u32, pub priority: u16, pub required: bool }
pub struct Document { pub connector: u16, pub name: effects::Name,      // settings.md, 9
                      pub path: Box<[u8]>, pub heading: Option<Box<[u8]>> }
pub struct Section {
    pub token: Token,                 // its connector's own, unique among the sections it holds
    pub kind: u16,                    // its connector's own numbering, carried to the assignment
    pub name: &'static [u8],          // its fixed name in a failure (1.1)
    pub size: u32,                    // what it expects to gather, 0 when it can't tell
    pub limit: u32,                   // the most it may keep after cuts
    pub priority: u16,                // a lower one keeps room first
    pub required: bool,
    pub document: Option<u16>,        // the document it reads, by its place in `documents`
}
pub struct Failure { pub fate: Fate, pub section: &'static [u8], pub words: Box<[u8]> }   // 1.1
pub enum Fate { Capacity, Transient, Infrastructure, Permanent }
pub const DOCUMENT_PRIORITY: u16 = 5;

pub enum Ask {
    Plan(Box<Preparation>),                                             // ▷ Up::Planned
    Abandon { task: u64 },                                              // · forget a plan still being made
    Gather { task: u64, section: Token, budget: u32, deadline: Time },  // ▷ Up::Sized or Up::Failed
    Cut { task: u64, section: Token, size: u32 },                       // ▷ Up::Sized or Up::Failed
    Drop { task: u64, section: Token },                                 // · release it, planned or held
}
pub enum Up {
    Planned { task: u64, sections: Box<[Section]> },
    Sized { task: u64, section: Token, size: u32 },     // gathered or cut, and held
    Failed { task: u64, section: Token, failure: Failure },
}
```

`Document`, `Failure` and `Fate` move down from the core's vocabulary and the brief
child, each still named along its old path, and `DOCUMENT_PRIORITY` with them.

- **Asked of each connector that serves sections.** A connector says it
  serves them in its declaration (`sections`, ../connectors.md, section 2). Once the
  core's own sections are planned and its notes index read (1), the
  engine asks each such connector, in its number's order, `Plan` with
  the run's preparation context. A brief is named by its task's number,
  since a task prepares one run at a time. What a task names of a
  connector's system (its project's home, its parent's change, a
  document of its charter) is the connector's to know (../connectors.md,
  section 9), so each connector judges it, and answers with no sections
  where there is none.
- **The context** is what the brief stands on, in values: the task, its
  project, its requester, its tree's root, the documents of its
  charter's revision whose connectors serve sections, while a slot is
  left for them (`settings.md`, 9), the core's own
  sections' claims in the brief's order, the slots left after them and
  the documents, and the brief's bytes (`brief.brief_bytes`). A
  connector reads its own records beside it, never the core's.
- **A plan may end later.** A connector answers `Up::Planned` in the
  step it is asked or a later one, once what it waits on is taken
  (temper's forge waits for a goal's workspace). The engine waits for
  every connector it asked, with no deadline of its own: the task stays
  preparing, as it does for a load. Its wait is kept with the
  preparation's context, bounded by the live tasks as the wait for notes
  is (2.2), and takes no brief slot.
- **Placed by the engine,** once every plan is in, in this order: the
  core's sections, in its order; each document's section, in the
  charter's order; then every other section, by connector number and in
  each plan's order. The core's sections and the documents planned have
  their slots of the brief's `brief.sections`. A required section takes a
  slot left, or else displaces the last optional core section, then the
  one before it, and a displaced section is left out with no marker.
  With none left to displace, the brief fails, permanently, `brief: no
  room for its required sections` (1.1), and every section planned is
  dropped. An optional section takes a slot left after the required
  ones, or is dropped. A document no connector planned is not in the
  brief.
- **A plan out of bounds** is taken as no plan, and each of its
  sections dropped: more sections than a brief holds, a token named
  twice, a document that is not its own in the context or is planned
  twice, or a document's section planned required. The engine tells the
  connector nothing more (3).
- **Gathered and cut by the brief child** (1): the placed sections, the
  brief's bytes and its deadline, `brief.gather` after placing. Each
  connector section is asked `Gather`, with the read bound
  (`brief.read_bytes`) and the deadline, and answers `Sized` or
  `Failed`; once all are in or the settle point passes, the child allots
  sizes by priority and asks `Cut` to a smaller size, answered the same,
  or `Drop` of an optional section without room. A section is named by
  its connector and its token together: two connectors' tokens may be
  equal. *Within the deadline* (1) and 1.1 hold as they are.
- **Kept until the run is prepared.** A brief complete goes on to the
  run's workspace in the same decision, and the engine keeps its placed
  sections, each connector's still held by it, until the run is
  prepared: they go to the root with it (`Now::RunPrepared`), and the
  root takes each from its connector as it puts the assignment together
  (../root.md, section 7). A section its connector no longer holds then goes in
  as missing, its why `Failed`.
- **Abandoned** when the preparation ends before its run is prepared:
  stale, failed, its claim refused or its task ended. The engine asks
  `Abandon` of each connector whose plan it still awaits, and `Drop` of
  each section planned, gathering or held, as a brief that fails drops
  its sections (1.1). While the engine stops, a brief completes but its
  preparation goes no further. Nothing of a brief outlives a restart: a
  preparation there begins again.
- **Answers that name nothing awaited** are taken as the core takes
  them (`vocabulary.md`, 2.5): a plan from a connector the engine awaits
  no plan from for that task has each of its sections dropped; a size or
  a failure for a section no brief awaits is dropped.
- **Why the engine plans:** the roots planned briefs three ways, two of
  them with fixed answers (../root.md, section 14), and what fits a brief is the
  brief child's to judge. A connector knows what its system holds for a
  task; the engine knows the brief's slots and budget. The context
  carries the second to the first, and the plan the first to the second.
- **Why a plan ends later,** where the draft's capabilities' table
  makes it a judgement: the same draft has each connector answer once
  the holds it waits on are taken, and temper's forge plans only once a
  goal's workspace is ready.
- **Why every connector that serves sections is asked,** not only those
  the task names: the core does not know what a task names of a
  system, its project's home or its parent's change, and a connector
  with nothing for it says so in one empty answer.
- **Why a cut is answered,** where the draft's table has a notification:
  the size a connector's own rules cut to is what the brief holds, and a
  cut unanswered by the deadline keeps failing as 1.1 says.

## 2. Notes

Notes are what agents learn and pass on: a service that is slow to warm
up, why an approach was dropped, which check is flaky, what a person
prefers. The notes child domain knows scopes, entries and indexes:

- **An entry** has a one-line description, a body, a scope, references to
  tasks, and who wrote it: a party, or a task.
- **Scopes:** the deployment; a project; a goal; a resource pattern of a
  connector's (a service, a repository), within a project; and a party's,
  a person's own, in no project (2.1).
- **A chat's own scope** is the goal scope named by a chat's own number. A
  root conversation, a task flagged conversation with no tracked ancestor
  (`goal_of` none), writes and reads that scope as a goal's tree does its
  goal's, where its authority's note scopes hold a goal; another task's run
  never does, nor any task of another project. The store and the wire keep
  one scope: a goal's `project` and number. What lives there is the
  application's to say: an application's chats may keep a document in it. A
  brief's index lists this scope first, as it lists a goal's first (section
  1). A standing task's own scope is the same: the goal scope named by its
  own number, which holds its Now (../tasks/standing.md, section 6).
- **In the store,** where parties read, correct and delete them through a
  client. Notes are hints: where a note and the facts disagree, the facts
  are right.
- **Written by a tool,** checked against the task's note scopes
  (../authority.md, section 3), and committed like any decision. A revision
  names the revision its run recalled, and is refused as moved if the
  entry changed since.
- **Whole, or by a line range.** A call writes a body whole, or, naming an
  entry and the revision it read, replaces a range of that body's lines:
  `first`, 1-based, and `remove`, a count of lines, 0 to insert, its body the
  text that takes their place. An append is `remove` 0 and `first` past the
  body's last line: the body's lines + 1, or any line beyond it, which writes
  at the end as the body's lines + 1 does, and is answered as that write is,
  its `first` the line it changed. Lines count as the answer's do, below. A newline is added only
  where two lines would otherwise join, and nothing else of the body or the text
  changes, so a body with no final newline keeps none unless a line follows it.
  With `join: true`, the replacement continues the line before `first` without
  a newline: that line's ending newline, if present, is removed, and none is
  added before the replacement. This lets a run build one long line in parts,
  under the same revision and body bounds.
  There is this one form and no named section, since a body needn't be Markdown.
  It writes the description it names, and keeps the entry's references unless it
  names some.
- **Stored whole.** A revision stores the resulting body, so that a reader, a
  client and `recall` see no parts, and a revision is a body as any other.
- **Refusals say what to do next,** and each leaves the note as it was: moved
  says the entry's revision; a `first` of 0, or a range that removes lines
  past the end, says the body's lines; a
  result past `notes.body_bytes` says its bytes and the bound.
- **Its answer says what changed.** A write answered carries, beside the
  note's name and revision, `lines`, the new body's lines, and, from the
  body before it and the body written with their common leading and
  trailing lines trimmed, `added` and `removed`, what remains of the new and
  of the old, and `first`, the new body's 1-based line where they first
  differ. A first write has `added` equal to `lines`, `removed` 0 and `first`
  1. A body that ends in a newline has no further empty line; an empty body
  has 0 lines. A write that removes only trailing lines, or changes nothing,
  has `first` the new body's last line (1 when it has none). It is a pure
  function of the two bodies, bounded by `notes.body_bytes`, and no shortest
  edit: a person's reader wants a count and a place, not a diff. A range's
  answer is the same function of the same two bodies, so its `first` is where
  they first differ, which the range's own `first` may precede.
- **Read in two steps.** A brief carries the index of the notes in its
  scope, a line per entry, saying how many more there are when they do
  not all fit; a run asks for the entries it wants with `recall`, by name
  or by a search of their descriptions, answered a page at a time.
  Indexes are kept per scope in use, and evicted least recently used;
  entries per scope are bounded.
- **A body's bound,** `notes.body_bytes`, is 65,536 bytes in `standard` and
  32,768 in `small`, room for a document a person reads, and what carries a body
  holds one whole: a tool call's input and a host's event, twice the bound and
  4,096 for its JSON's escapes (`core.call_answer_bytes`, `hub.event_bytes`), a
  load (`core.load_bytes`), and a client's read, cut at `notes.body_bytes` rather
  than `documents.text_bytes`. A model's output bounds one call's payload all the
  same (smith's `protocol/limits.md`, 13), and not a body: a body past what one
  call holds is built in parts, by line ranges, up to the bound, and a run reads
  it back whole with `recall`, which costs input, not output. A party's
  correction is bounded by `people.amendment_bytes`.
- **Knowledge that belongs to a system** belongs there, such as a
  repository's guide: a note that matures may be proposed as an effect
  on it.

### 2.1 A party's scope

A person's preferences: a few short lines kept for them across the
deployment and read by the runs that explain to them, so that what they asked
to be remembered holds in their next conversation (`../../draft/notes-memory.md`,
section 3, taken up as far as an application's asking runs need it; temper's
Ask temper is the first, its `temper/docs/design/draft/ask-temper.md`, section 7).

- **The scope,** `Party { party }`, a person's number: one per person, in no
  project, so that no project's reads list it and a project's deletion leaves
  it. The store keys it as it keys every scope, its index lines and entries
  beside the others' (`../../protocol/store/keys.md`, section 1).
- **Its entries are preferences,** the kind its scope gives them: an entry's
  description is the line and its body is empty, at most `notes.party_bytes`
  for the description, and at most `notes.party_entries` entries in one
  party's scope. A write past the bytes is refused naming them and the bound,
  a body is refused, and a new entry past the count is refused as full, as a
  scope past `notes.entries_per_scope` is. The other kinds of
  `../../draft/notes-memory.md`, section 2, are not built: no other scope has one.
- **Written for its party by the runs that explain to them:** a run whose
  task's requester is the person, `Party::Person(party)`, and whose
  authority holds the asking family (../authority.md, section 3), through its
  application's named actions: `Note`, a new entry or a revision at the
  revision it read, and `NoteDelete`, an entry dropped at the revision it
  read, which no other scope admits from a run. The note family decides and
  records each as it does a note (`executors-tools.md`, 1); for this scope its check is that
  requester and that family, in place of the authority's note scopes, and any
  other run's write there is refused beyond authority, naming the scope. A
  run's own `note` and `recall` tools name no party's scope; a requester two
  tasks up, which a delegate works for, is later (`README.md`, section 10).
- **Read by those runs, whole in their brief:** the core section
  `preferences` (section 1), present where the run's task's requester is a
  person and the asking family is one its run's charter names and its
  authority holds, as a host family is offered (`executors.md`, 1), so that
  a run whose charter names no asking family reads no person's preferences,
  whatever its authority holds; a charter whose record carries no mask
  names every family, the asking one among them, as it decodes
  (`RunPolicy::EVERY_FAMILY`). The section holds the scope's index lines,
  every one, in name order, the oldest first, each `N. DESCRIPTION`, `N` its
  place from 1, or the one line `None kept.`. It is required, and its budget
  holds `notes.party_entries` lines of `notes.party_bytes` numbered, so it is
  never cut; a scope whose index cannot be read fails the brief as the notes
  index does (1.1). The notes index (section 2) never lists the scope.
- **Read and kept by its party alone:** the party reads the scope through a
  client (`../../client/domain/contract/reads.md`, section 1) and corrects or deletes
  its entries whatever their roles, project 0 named as a deployment's note
  names it. No other party reads it, an owner of the deployment included: a
  read is answered unknown, a correction or a deletion refused as unknown.
- **Its runs' parts are its party's:** a turn of a task whose requester is a
  person shows its brief's `preferences` section and the calls of the
  application's private tools to that person alone. The application names
  its private tools in the core's settings, temper's `remember` among them,
  and those tools and the section's title as its runs' briefs render it in
  the client component's, which projects. Each turn the core answers to any
  other party, on its own (`Turn`) or in an activity page, and every turn a
  run reads as its person (`executors-tools.md`, 1), is marked *withheld*, and the protocol
  layer that carries it projects it before it leaves: in the turn's system
  text and each text block of its messages, the section's heading line and
  the lines after it that are empty, `None kept.` or a number then `. `
  become the heading and one line, `Withheld: shown to its
  person alone.`, and each private tool's call keeps its tool and position
  with its input `{}` and its result that line. A run's live document
  (`../../client/domain/contract/watches.md`, 1.3) shows a private tool's call and the
  input being written to the task's requester alone, and to every other
  follower the call's tool alone. The stored turn stays whole, so that the
  run resumes from it and its person reads it as it was.
- **Bounded and erased with what holds them:** the scope's index is kept as
  every scope's is, evicted least recently used (section 2), and parties are
  never erased (../people/README.md, section 3), so neither is their scope; a party's
  erasure is later, with its scope.

### 2.2 A brief's wait for notes

The notes child serves one operation at a time: a write, a read or a
project's deletion. A cold index holds it busy while it loads each
scope it lacks, page by page, until every scope is cached or a load
fails. A preparation whose brief needs an index (its notes, section 1,
or its person's preferences, 2.1) while notes is busy waits for it, and
does not fail:

- **The wait table.** The core keeps one entry per waiting task, keyed
  by the task: either it waits for notes with its preparation kept, or,
  refused busy while notes is idle (an index whose load found no load
  free, `decisions.md`, 1.3), it gives up its preparation and its next activation
  returns to due (`executors.md`, 1, step 2). When notes is idle again
  the core resumes one waiting preparation, and the next as each
  operation ends, so that none is left waiting while notes is idle. A
  task's ending removes its entry, and the table starts empty.
- **Its bound is the live tasks' (`tasks.tasks`),** the bound of the
  preparation contexts, and not the briefs gathering (`brief.briefs`):
  every entry is one live task's, and every preparing task may wait at
  once. A preparation is bounded only by live tasks, and the brief
  child's table counts briefs gathering, a later step whose briefs are
  the costly part. After a restart every due task activates together,
  and the first cold index holds notes busy while the rest arrive: an
  application that gathers two briefs at once (temper's root does) runs
  many more preparations than that. An entry is a task number and its
  mark, so the table costs a few bytes per live task, nothing that grows
  with the store.
- **A full table pauses, never aborts.** The bound leaves no room for a
  full table while the tasks child keeps its own, but the core does not
  rely on it: a task whose wait finds the table full fails its
  preparation as capacity, `notes: no room for the index` (1.1), and is
  due again after a backoff with no try spent. Its activation's reasons
  and its inbox stay with the tasks child, so nothing is lost.

## 3. Later

- **The rest of the preparation context** the draft gives
  (`temper/docs/design/draft/jig-domain-api.md`, 6.6): the run's attempt,
  its spec, its saved resources and what it may modify, with a run's
  workspace items planned in the same plan, with the milestone of a
  run's workspace and claim
  (`temper/docs/design/draft/jig-domain-api.execution.md`). Until then a
  connector reads what it needs of them as it does today.
- **A plan's own deadline.** A connector that never answers `Plan`
  keeps its task preparing until the task moves or ends.
- **Section kinds declared** (`temper/docs/design/draft/jig-domain-api.md`, 6.2), each
  with its fixed name, in place of the declaration's `sections`; and one
  way to name a part a connector keeps (`PartRef`, 6.6 there) in place
  of a section's connector and token.
- **A plan refused with why** (`temper/docs/design/draft/jig-domain-api.md`, 6.4),
  told to the connector whose plan was out of bounds.
