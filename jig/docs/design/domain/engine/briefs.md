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
  which a run names in a note's scope, its spec and, unless
  its requester ends it (../tasks/lifecycle.md, 9) or it stands
  (../tasks/standing.md, section 2), its contract,
  and its lineage: its requesters' specs, cut); its dependencies' and
  inputs' results in jig's terms, and, for a run after its task was kept
(../tasks/lifecycle.md, 8), what it waited for: each dependency added, when and by
whom; its delegates, their states, and the
  revisions a decision about them names, each one's spec's and a held
  one's hold's (`executors.md`, 3), and the results that came since (the plan, for a
  coordinator); its earlier
  attempts and why they failed; the calls its task committed since its
  last turn, when it does not resume; what waits for it, the waiting
  section: the questions asked of it still open (1.2) and the proposals
  and escalations waiting for its decision; the index of its notes; for a run that
  explains to its person, that person's preferences, whole (2.1); for a
  standing task, its Now, whole (../tasks/standing.md, section 6); a
  transcript's tail,
  when it does not resume, but for a standing task's session's first run
  (../tasks/standing.md, 5.4), the window's last turns as text rendered by the
  host's protocol layer when loaded, a turn too large to load as its
  fixed marker (`executors.md`, 2), within the tail's own budget; and,
  for a run whose tools hold `delegate`,
  the catalogue: each model its authority allows (../authority.md, section
  3), with its LLM provider, its window and output, its efforts, its
  prices, and what it is good at, a description its configuration gives
  (`../../protocol/configuration.md`, section 3), and whether an account
  that serves the run's payer is usable now, limited and until when, or
  needs sign-in (../accounts.md, 3.5), so that a planning agent gives each
  task it delegates the LLM choice its work needs, by capability and
  cost (../tasks/README.md, section 4).
- **Its charter's documents' (`settings.md`, 9),** gathered by the connector that
  owns each document's resource as it is when the run is prepared,
  each optional, titled by the document, after the core's sections and
  before the task's resources', for every run whatever its charter's
  parts.
- **Its connectors',** which it plans by kind, priority and budget but
  never holds (../model.md, section 5): for each resource the task names, and
  for each dependency or input whose result is a connector's, the
  connector gathers the section, keeps it, and says its size; the brief
  child decides what fits, asks for cuts by size, and, when the brief is
  complete, the root takes each section from its connector to put the
  assignment together (../connectors.md, section 9).

A section is loaded or read as it is cut, asking for no more than it
keeps. The host's protocol layer renders the whole as text, in order.

A brief has no inbox section. The messages waiting in the task's inbox
that the run is offered go with the assignment as its start's messages,
and the run reads them as messages, whole (`executors.md`, 1; ../tasks/messages.md, 2 and 6).
Nor has it a section for its charter's parts: they are its instructions
(`executors.md`, 1, step 4).

### 1.1 A brief that fails

A brief fails whole: a required section is missing, the brief child
refuses its plan, or the core or the root cannot make or put together
its parts. Every such failure carries a **failure**: how it counts, and
why, as a section's name and fixed words. Both name and words are
`'static` in the code, so a reason cannot carry a run's words or a
secret. A connector chooses its own, and names no more than the kind of
its section and of the error its system gave.

How it counts decides what becomes of the task:

- **Capacity:** nothing is wrong but room, which frees by itself. The
  task pauses and is due again after a backoff, no try spent, with no
  bound on how many times (../tasks/lifecycle.md, 6).
- **Transient** or **permanent:** a failure of that class (../tasks/lifecycle.md,
  6). The task backs off within that class's allowance and, past it,
  is held for `Failures` of that class, its hold naming the reason.

A brief that fails, however it counts, gives its brief slot back at once,
its sections still gathering dropped, before its task pauses or backs
off, so another task's brief takes the slot while it waits.

Which is which, by what `executors.md`, 1 prepares:

| The failure | Counts as | Section and words |
|---|---|---|
| no brief slot: the brief child's table is full, or this brief is still gathering | capacity | `brief: no brief slot is free` |
| a connector says it has no room, or that its system admits nothing yet | capacity | the connector's |
| the core's result reads, its notes or its proofs have no room | capacity | `results: no read flight is free`, `notes: no room for the index`, `run: no room for its proof` |
| a claim refused (a host's or a writer's room; ../tasks/lifecycle.md, 6) | capacity | `claim: refused` |
| a required section past the brief's deadline | transient | `SECTION: not gathered by its deadline` |
| a connector says its system failed for now | transient | the connector's |
| a dependency's result the store no longer answers with | transient | `results: a dependency's result could not be read` |
| a transcript's window that does not load | transient | `transcript: its window could not be loaded`, or `transcript: its window could not be read` for an open window unread, which closes it (`executors.md`, 2) |
| a required section that cannot fit, whole or cut | permanent | `SECTION: does not fit the brief's budget`, or `brief: its required sections do not fit its budget` |
| a section a connector answered past its bound | permanent | `SECTION: answered past its bound` |
| a plan the brief child refuses: more sections than a brief holds, a budget past the limit, a core text past it, a section planned twice, a deadline passed | permanent | `brief: ...`, naming which; for a core text, `SECTION: its text is past the brief's limit` |
| a connector says its system answered wrongly, or will not answer | permanent | the connector's |
| the root cannot put the brief together: a section's owner lost it, or no room for a required section | permanent | the application's |
| a core section the core cannot read within its bounds | permanent | `task: ...`, `results: ...`, `transcript: ...` |
| a brief's notes index refused for any reason but room | permanent | `notes: the index could not be read` |
| no attempt number left | permanent | `run: no attempt number is left` |

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
(../tasks/lifecycle.md, 8), but its message is taken from the inbox
by the turn that reads it (../tasks/messages.md, 2). A run that reads
a question and ends without answering it would leave the next run
nothing to go on, so every brief shows the open questions:

- **Which.** Each question asked of the task that is still open: its
  asker holds the question's credit (../tasks/lifecycle.md, 8), with
  the task as its answerer, and the task has heard no answer to it.
  A question this run's start carries as a message is left out, since
  the run reads it whole as a message.
- **Which runs.** Every run that starts from a brief. A run that resumes
  has no brief; its window already holds the question it read
  (`executors.md`, 2).
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
  beside the others' (`../../protocol/store/README.md`, section 4).
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
  records each as it does a note (`executors.md`, 3); for this scope its check is that
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
  client (`../../client/domain/contract/README.md`, section 4) and corrects or deletes
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
  run reads as its person (`executors.md`, 3), is marked *withheld*, and the protocol
  layer that carries it projects it before it leaves: in the turn's system
  text and each text block of its messages, the section's heading line and
  the lines after it that are empty, `None kept.` or a number then `. `
  become the heading and one line, `Withheld: shown to its
  person alone.`, and each private tool's call keeps its tool and position
  with its input `{}` and its result that line. A run's live document
  (`../../client/domain/contract/README.md`, 5.3) shows a private tool's call and the
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
  refused busy while notes is idle (an index whose load found no room
  in the root), it gives up its preparation and its next activation
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
