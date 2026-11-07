# Instructions per wake

Draft, 2026-11-07, from a session with the user that day. It gives a first and
a later LLM choice in place of the choice per kind of wake that
`reference-code-production.md`, section 16 leaves for later, and ends the
amendment by hand of `goal-as-designer.md`, section 8. It revises
`reference-code-production.md`, sections 13 and 14, where a message reaching a
live run is always relayed to it and the goal's charter is one text. It wins
where they differ until its plans fold it in.

The pipeline we develop with answers a question in a fresh call, with a prompt
for answering alone. temper's goal answers in a fresh run too, but every run
reads its one charter whole: the cut's instructions, the answers', every
wake's, and rules for work it never does. The charter the comparisons run
(`dogfood-hacks.md`, 2.10) is about 31 KB, read whole at each of the goal's
runs, and in the first comparison on it each later run spent between a
twentieth and a sixth of what the first run spent. The user asked "would it be
possible to have the same model in temper?", picked instructions chosen by why
the goal woke over a task added to answer each question, and asked for "the
full story in place: per-wake instruction and focused prompts ... cut and answer
are separate prompts" before the next comparison. Reasons: `notes.md`,
section 8.

## 1. In one page

- **A charter may have parts.** Beside the instructions every run reads, a
  charter may hold parts, each for some kinds of message. A run reads the shared
  instructions and the parts for why it runs (section 3).
- **Why a run runs:** its task's first activation, or the kinds of message that
  woke it, from jig's stored vocabulary. A run that continues one kept by its
  own question runs for what that one ran for (section 2).
- **A run is offered what its parts cover.** It also always gets replies to its
  own acts, its entries, an amendment, a request to finish and a person's
  words. Any other message stays in the inbox for a run that has its part
  (section 4).
- **A later choice.** A task's LLM choice may hold a second choice, for every
  run after its first activation's. The goal's template takes Opus high there,
  so nobody amends the goal by hand after its first run (section 5).
- **The goal's charter in four parts:** the cut, questions, results and holds,
  and news. An answer reads the shared instructions and the questions part
  (section 6).
- **Focused charters:** each charter holds the rules its role acts on, and each
  rule once (section 7).
- **A charter that can run:** a charter too large to store says how large
  (section 8).

## 2. Why a run runs

When a task is due, the core gathers its brief "for why it runs: its first
activation, or what woke it" (jig's `domain/engine.md`, 7.1, step 2). This
draft names those reasons so that a charter can choose instructions by them,
and keeps them on the task's claim.

- **`first`:** the task's first activation. It lasts until a run of it parks
  idle or finishes. A run that parks kept, by its own question, a question it
  owes or a dependency (jig's `domain/tasks.md`, 5.7), leaves the run that
  continues it `first` too. A first run's reasons are `first` plus the kinds
  that woke it, such as a person's words waiting at the goal's start.
- **The kinds that woke it:** each kind of message in its inbox, or entry
  waiting for its decision, that its wake policy let wake it (jig's
  `domain/tasks.md`, 8.3), named as jig's store names them (jig's
  `protocol/store.md`). A kind that rode along without waking it is no reason. So
  a non-last result under `last_or_failure`, or news a pace held, chooses no
  part.
- **A continuation:** a run that continues one parked kept runs for that run's
  reasons, plus the kinds that woke it now.
- **A retry:** a retry of a failed attempt is due by its backoff, not by a
  message. It runs for that attempt's reasons, kept on the task across attempts,
  plus the kinds at its start that its policy would let wake it. A retry's start
  carries what is waiting then, not what the failed attempt read (jig's
  `domain/engine.md`, 7.2).

**Always offered.** Some kinds go to every run, whatever its parts (section 4),
and choose none: `answer`, `proposal_decision`, `settings_decision`,
`effect_not_made` and `met`, each a reply to the task's own act; and
`amendment` and `finish_requested`, which change or end what the run is
doing. Every other kind chooses parts: `question`,
`question_answered`, `question_ended`, `question_owed`, `escalation`,
`proposal`, `person_stalled`, `result`, `notice`, `nothing_awaited`, `news`,
`timer` and `words`. A person's `words` also go to every run.

## 3. A charter's parts

- **Shape.** A charter keeps its `instructions`, read by every run, and may add
  `parts`, in order, each with `on`, a nonempty set of `first` and the kinds that
  choose parts (section 2), and its own `instructions`. How many parts a
  charter may hold is a limit, `charters.parts`, 8 in every profile.
- **A run's instructions** are the charter's instructions, then each part whose
  `on` meets the run's reasons, in the charter's order, each part once. A run
  that no part is for reads the shared instructions alone. A charter with no
  parts gives every run its instructions and every message, as today.
- **Fresh runs only.** A resumed run keeps the instructions its conversation
  began with, and changing them would lose the conversation's cache and leave
  its earlier turns read under other words. So a charter with parts whose runs
  resume (`resume: true`) is refused as out of shape.
- **Documents** (jig's `domain/engine.md`, 17.9) stay for every run. The
  reference's goal charter names two documents' "In one page" sections, which
  every part leans on.
- **Bounds.** A charter's stored policy, its parts with it, stays within the
  root's `root.journal.run_bytes`, as its instructions do today. So parts make a
  run read less, not the charter store less.
- **The settings form** (jig's `protocol/configuration.md`, section 3, the
  charters member) carries `"parts": [{"on": ["question"], "instructions":
  "..."}]`. An unknown kind, an always-offered kind, an empty `on` or a part
  past the limit refuses the change at its path. temper's settings form and `temper ctl
  settings show` and `apply` carry them (`../client/tools.md`, 5.3).

## 4. What a run is offered

A run's start carries the messages waiting in its task's inbox, as many as the
run's inbox takes, and the rest are relayed as it reads (jig's
`domain/tasks.md`, 8.2; jig's `domain/engine.md`, 7.1 and 7.4). For a charter
with parts, both offer only these:

- **The kinds its parts are for:** every kind named in the `on` of a part this
  run reads.
- **Always:** the always-offered kinds of section 2, so a run that asked or
  waited hears its answer and one being amended or ended hears it; a person's
  `words`, which no batch holds (jig's `domain/tasks.md`, 8.3); and proposals and
  escalations waiting for its decision, which are relayed to a live run whatever
  its parts (jig's `domain/tasks.md`, section 9). The shared instructions say how
  to take each, and to leave an entry no part of the run covers undecided.

**What isn't offered keeps its place.** A message of any other kind isn't
offered to the run, at its start or while it runs. It keeps its place in the
inbox, and when the run ends it is judged again as if it arrived then, as a
message refused or left unread is (jig's `domain/tasks.md`, 8.2 and 8.3). This
changes how a turn takes messages: today a turn's fence takes every message
before it (jig's `domain/hosts.md`, 6.4; jig's `domain/engine.md`, 7.2), so a
message kept back would go with the next one read. A turn takes only the offered
messages up to its fence instead. A message a pace keeps back from a live run
needs the same rule (jig's `domain/tasks.md`, 8.3). A message that arrives
between a run's preparation and its claim is offered or kept the same way,
against the parts prepared.

**An entry left undecided wakes the next run.** Entries aren't inbox messages,
so nothing re-judges one when a run ends. So when a run ends with an entry
still waiting for the task's decision, the task is woken again for it, its
reasons that entry's kind, as an unread message would wake it.

So a question that reaches the goal during its cut waits for the run after it,
which reads the questions part, as the pipeline's cut never answers one. A
non-last result waits for the last, whose run reads the results part and every
result. News a pace held wakes a run of its own once the pace allows. An entry
reaching the cut is offered, left undecided and wakes the run after the cut. A
cut longer than a pending escalation's stall (jig's `domain/tasks.md`, section
11) passes such an entry up to its person, as it would today.

## 5. A later choice

- **What it is.** A task's LLM choice (jig's `domain/tasks.md`, section 3) may
  hold `later`: a main model with its effort, and the sub-agents' models with
  theirs. Every run whose reasons aren't `first` takes it. Without `later`,
  every run takes the one choice, as today.
- **Who gives it:** the task's creator, or a template (jig's
  `protocol/configuration.md`, 3.1). It takes no account of its own: a task's
  account preference covers both choices. An amendment changes it as it changes
  the rest.
- **Checked as the first choice is:** within the task's authority at its batch
  or amendment, and within its charter's models and their declared efforts at
  each run's check, under the policy's overrides, and again on a restore.
- **Its account.** A `later` main on another LLM provider than the first's
  chooses the task's kept account again at its first later run, as any change
  of that provider does (jig's `domain/tasks.md`, section 3).
- **Fresh runs only,** as parts are: a task whose charter resumes takes no
  `later`. A change's `llm` (`../domain/agent.md`, 4.3.1) carries none, since a
  change's producers resume or finish in one activation.
- **What shows it.** The task's document shows the choice its next run would
  take (jig's `client/domain/contract.md`, 5.1).
- **The goal's template** takes Opus 5.5 at xhigh first and Opus 5.5 at high
  later, each with the first choice's sub-agents. That is
  `goal-as-designer.md`, section 8's choice, made at the goal's start, so
  nobody has to amend the goal after its first run, which the goal can't do
  itself.
- **Not per kind.** One run takes one model and may run for several reasons,
  so a choice per kind would need an order among them. The need measured so far
  is the first run against the rest (`notes.md`, N11).

## 6. The goal's charter in parts

The reference's goal charter, charter 5 (`../protocol/configuration.md`,
13.9), is written in four parts, each holding what
`reference-code-production.md`, section 13's rows say G reads and decides on
those wakes:

| Part | `on` | Holds |
|---|---|---|
| the cut | `first` | the first run (`goal-as-designer.md`, section 3): reading, surveys, design and its delivery, the batches, extra designers, the forge subscription; it never waits for or reacts to its changes |
| questions | `question`, `question_answered`, `question_ended`, `question_owed` | settling a relayed question from the draft and the design: an answer, an amendment, a prerequisite change, or a question to its person |
| results and holds | `result`, `notice`, `nothing_awaited`, `escalation`, `proposal`, `person_stalled` | a designer's result into its family's batch; a person task's result into the answer that waited for it; the last result's Left lists and the landing; the landing's result into the close-out; the close-out's result into the finish; a failure or a hold released, amended, cancelled or passed up, a landing's hold among them |
| news | `news`, `timer` | a catch-up; a red of g's tier; a red after landing |

The shared instructions hold who the goal is and its branch, the requester
protocol, the report note and notes, how a change's words are written (a spec,
its parts and its rules, since every part makes or amends changes), reading the
report note and the tree first at every wake and parking with `wait` once its
decisions are made, never writing g or making a second landing, and how to take
the messages every run is offered (section 4).

**A request that needs no design.** Where a request settles its changes with no
shape for the code to follow, as one made of documents does, the cut writes no
design, says so in its report note, and makes its changes. The first comparison
on the generated charter did so with no rule saying it may.

**The comparisons' charter** (`dogfood-hacks.md`, 2.10) is generated in the same
parts from the pipeline's prompts: its cut is the cut, its answerer the
questions, the reference's wake rules the results, the holds and the news. By
its own text, the cut would read about 17 KB, and an answer about 8, of the 31
KB every run reads today.

## 7. Focused charters

- **A charter holds what its role acts on.** A project's rules for building,
  testing, machines and landing reach the coding agent, which acts on them, and
  the foreman, which judges by them. The goal builds and tests nothing, so its
  charter holds only what a spec must carry of those rules: what a spec may ask
  a run to run, and what it may never ask.
- **A rule once.** A rule two parts need is in the shared instructions. A rule
  one part needs is in that part alone. A rule said twice is cut to one.
- **Not what the run is already told.** A charter doesn't repeat what its
  tools' descriptions and its brief say: a call's fields, a call's room, the
  wake policy's mechanics.

## 8. A charter that can run

- **A task no host takes** is designed already: the fleet refuses a worker's
  hello that offers less than a charter needs, and a due task's wait says so
  (jig's `domain/engine.md`, section 8). Where temper doesn't show it, that is a
  repair for its owner, not this draft's.
- **A charter too large to store says how large.** A change of settings refused
  because a charter's stored policy passes `root.journal.run_bytes`
  (`../domain/root.md`, section 10) names the charter, its size and the bound,
  as a refused start already does. Today a change says only that the charter
  is out of shape.
- **The reference's settings test** holds its charters within the stock
  worker's budget and the root's bound.

## 9. Order

1. **jig:** the reasons, parts, what a run is offered, the fence taking offered
   messages only, an undecided entry's wake, and `later`, design first: jig's
   `domain/tasks.md`, sections 3, 8 and 9; jig's `domain/engine.md`, 7.1, 7.2,
   7.4 and section 9; jig's `domain/hosts.md`, 6.4; jig's
   `protocol/configuration.md`, sections 3 and 6; the tasks child's take; the
   store's claim, charter and task records; the client's charter and task
   records.
2. **temper:** its settings form and `temper ctl settings`, with section 8's
   refusal; the reference's charter 5 in parts, and template 4's later choice
   (`../protocol/configuration.md`, 13.9).

## 10. Later

- Documents chosen per part.
- An LLM choice per kind of wake, if `later` proves too coarse.
- Parts for a charter whose runs resume: a part given as a message when a
  resumed run first meets its kind.
- Offering a run a message with its part, rather than keeping it for the next
  run.

## 11. Open questions

- **Is the later choice right at high?** The pipeline answers at medium
  (`goal-as-designer.md`, section 8). The next comparison's later runs, at high
  and reading a part, say whether medium would do.
- **Should the cut take entries?** An escalation reaching the cut waits for the
  run after it, and a cut longer than an escalation's stall passes it up to its
  person. The cut could release or amend, at the cost of the holds part's rules
  in every first run.
