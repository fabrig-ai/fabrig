# The agents temper runs

| Agent | Messages in | Tools its authority gives | Ends when |
|---|---|---|---|
| producing a change (../forge/change/README.md, 2) | none; where its change relays questions, the answers to its own (../forge/change/questions.md, 1) | the workspace's, from the landing branch or saved work, or started early with its dependencies' pushed heads merged in (../forge/change/questions.md, 1); `deliver` mid-run, as it goes; `finish` with a change; `message`, to ask its change, where it relays questions | its change passes its checks and is pushed |
| a repair (../forge/change/repairs.md) | as producing | the workspace's, from the change's branch, its brief carrying why; `finish` with a change; `message`, as producing | its new head passes its checks and is pushed |
| resolving a conflict (../forge/change/repairs.md) | as producing | the workspace's, from a merge in progress; `finish` with a change; `message`, as producing | the merge is resolved, checked and pushed |
| a review (../forge/change/gates.md, 1), a triage | none | reads; `finish` with a verdict | its verdict meets its contract |
| a foreman (../forge/change/questions.md, 1) | none | reads: the workspace's inspection, from the landing branch's tip, and the forge's; `finish` with a verdict | its verdict meets its contract |
| a chat | its person's words; its delegates' results and questions, and its person's answers to those questions (2) | most; `deliver` mid-run; `message`; `wait` | its person closes it |
| a coordinator | questions, results, overlapping landings, the answers to its own questions (2); proposals and escalations to decide; while its run lives, its changes' results its wake policy wakes it for (below) | `delegate`, `change`, `amend`, `cancel`, `release`, `decide`, `decide_escalation`, `propose`, `subscribe_forge`, `note`, `message`; `deliver` mid-run, its design straight to its goal branch, where its charter sets `modify` (`tools.md`; ../forge/branches.md, 3); `wait` | its goal's report is written |
| Ask temper (`ask-temper.md`) | its person's words; its proposals' outcomes: accepted, rejected with a reason, withdrawn as stale (1) | `see`, `document`, `remember`, `propose_settings` in its asking form; `wait` | its person closes it |

**A coordinator hears its changes while it designs.** A coordinator's
run doesn't wait for its changes, but goes on designing while they run.
While it lives it is offered, at its next turn, each of its changes'
results its wake policy wakes it for (jig's `domain/tasks/messages.md`, 3 and
6): under the stock's `last_or_failure`, a change failing or
cancelled, and the result of its last live change, a closer-ended goal's
included, since a live run makes no closer. smith offers what arrives
while the run works between two completions (smith's `domain/run.md`,
section 6, *Messages mid-work*),
so a design written beside its changes fits what landed or failed. A
result its run leaves unread is judged again when the run ends.

The change's agents run under the charters the change was made with,
kept on it whatever later changes of settings make them
(`../../protocol/configuration/settings.md`, 2): its producing, repairs and
resolutions under its producer's charter, the one its `change` named
(`tools.md`, 1) or else the forge's change charter, a coding agent's; its reviews
under the forge's review charter, a reviewer's, whose instructions are
to judge the change through its brief and the forge's reads and give a
verdict, or fail saying why it cannot (../forge/change/gates.md, 1), and whose tools
are the workspace's inspection and the forge's reads, never
modification or the shell: a reviewer changes nothing. Its foremen run
under the forge's foreman charter, whose instructions are to settle a
question from the change's words, the design in its checkout and the
change's pushed head, and give a verdict of answer, finish or escalate
(../forge/change/questions.md, 1), with a reviewer's tools: a foreman changes nothing.

The change's agents ask no one, and take no questions, answers or words
(2), unless the change relays their questions (../forge/change/questions.md, 1): then
its producing, its repairs and its resolutions ask the change, and hear
its answers. Its reviews and its foremen never ask.

A refused forge read keeps its refusing layer and reason. Connector authority
findings reach the agent as `ToolDenied`, with the findings intact. The root
also emits volatile operator facts for read refusals and call admission
refusals: task, attempt, tool, layer and reason, with authority findings or
the forge's error where present. The service routes these facts to its
operator notes and `temper-box serve` prints them on stderr. Call-record and
archive-read room pressure remains busy, retried by the host; observing it
changes no admission or grant. These bounded diagnostics are not store rows
or client wire fields, and overflow drops a fact and increments the root's
lost-fact count. Claude built-in Read refusals instead retain the permission
message in the recorded tool result (smith's `protocol/programs.md`, 5.4).
Forge file, directory, pull-file and compare replies fit the effective
core and receiving-agent body allowance (`tools.md`), retaining prefixes
or actionable size feedback. A forge read still exceeding the core settled
answer bound or `root.journal.transcript_bytes` returns a bounded error
naming the exceeded limit and measured bytes, including the tool name. Its refusal fact names
`host-render` and that limit. An oversized comparison points to `read_pull`
for the head and `read_pull_files` pages; an oversized pull-file page points
to directory and selected-file `read_file` calls at both endpoints with
`max_bytes`. Forge-client admission busy still uses the host's bounded
backoff.
A forge read answered with a kind no agent read asks for (`Runs`, `Run`,
`Commits`, `Tree` or `Queued`) is refused as such, and nothing stops
`temper-box serve`: the run's tool result, marked an error, names the read's
tool and the kind and says nothing was read, and the service notes a
read refusal for stderr at layer `host-render`, its reason naming the
kind.
A reviewer fails when it cannot judge the change.


## 1. Configuring

Ask temper (`ask-temper.md`), a chat from template 5 run by charter 11, both stock and
both settings like any other (`../../protocol/configuration/stock.md`), turns
how a person says they work into one proposal of settings: what temper's
engine must do becomes settings an owner accepts, and what only its agents
must do stays text its runs read. The proposal is a card in the chat and
waits in the Needs you of each project it belongs to (jig's
`domain/engine/settings.md`, 6; `../../client/pages/README.md`). Its instructions,
`ask.txt`'s *temper, as you explain it* and *Configuring*
(`../../draft/ask-temper.md`, section 8), say how it asks and proposes; it
reads by `see` (`ask-temper.md`, 1) and proposes by `propose_settings`' asking form
(`ask-temper.md`, 2), whose answers and checks do for it what instructions would
otherwise teach.

- **Its input** is the conversation: the person's words, a text they
  paste, or a file in an adopted repository they point to, which it reads
  with `see file OWNER/NAME PATH`. It reads the settings first, with `see
  settings`: the inventory, then each setting the change touches, whole,
  by the addresses the inventory's answer names; and it explains from
  what it reads and from temper's pages, by `see help` (`ask-temper.md`, 1).
- **The inventory** is the settings family's read with no member (`tools.md`),
  which `see settings` answers: the newest revision; each setting by kind
  and number with the revision that last changed it; the revisions held;
  the proposals of settings waiting. The authorized inventory also names the caller project's
  readable adopted repositories, each with the canonical host, owner and name
  from its retained repository record. Other projects and ended forges are
  excluded. These identities are read context, not settings or credentials;
  setting, revision and history reads carry no repository context. A charter
  document holds that host unchanged, including its port, and OWNER/NAME,
  which `propose_settings` writes from a repository named `OWNER/NAME`
  (`ask-temper.md`, 2), so that Ask temper names documents without a workspace or ctl.
  The same inventory names the caller project's people, one line each:
  its number, its login where temper holds one, its role's number, its
  account on each forge of the project's repositories, by the host their
  lines use, that forge's user's id or unknown (`../forge/adoption.md`): a
  box's person's by the forge's link (`../forge/adoption.md`, 6), with no adoption, and
  marks for the task requester and for temper's own forge writer, who never
  counts as a person. Someone known on none of them is said so once: an
  absent forge identity is explicit and cannot count for
  approvals. Services are left out. It then names each configured owned-forge
  account (`../../protocol/setup/init.md`), whose login alone verifies
  no project membership. The requester, a known forge identity and that
  configured account are separate facts; none establishes the others.
  Of these, a member's forge user id and an owned forge's person are more
  than a person's own `project show` prints. Only a run whose authority
  holds the settings family reads them, in the stock charter 11's through
  `see` (`ask-temper.md`, 1), of its own project alone, as for the repositories, and
  they hold no address, display name or credential.
  The same inventory names the offer to the chat's payer (jig's
  `client/domain/contract/documents.md`, 1), one line a model, with its provider,
  its efforts and its default effort, and says so where it holds none. It
  is the list of models the payer's accounts can run now. A charter it
  reads goes back whole and may name others. Connecting an account for a
  model none can run is the person's own act on Settings, Accounts, which
  its instructions have it name. The settings check validates the
  catalogue's models, not the current offer. A model is a setting it reads
  whole and proposes as any other: its window, output, efforts, default
  effort, prices and shown name, whether the picker shows it, and, as a
  removal, its withdrawal (`../../protocol/configuration/stock.md`). A
  model's setting names no account.
  A read of a project's ways or questions with none stored answers the
  stock (`../../protocol/configuration/stock.md`), never unknown; one of a
  project's words with none stored answers that the project has none,
  and its way's words stand. A read that
  finds nothing is an error whose words say it was not read and that this
  says nothing about the settings.
- **The kinds it reads and proposes** are a settings file's
  (`../../client/ctl/tools.md`, 3): `charter`, `template`, `policy`, `model`,
  `rules`, `landing-rule` (with its `project`, the deployment's when left
  out), `forge-charter`, `charter-words` (with the charter's number),
  `project-policy`, `project-ways`, `project-questions`, `project-classes` and
  `project-words` (each with its `project`), each named and valued as
  the file names and values it, so that a description's landing rules,
  the forge's charters and a project's own policy are proposed as its
  charters are. `see` reads them at the chat's own project (`ask-temper.md`, 1); a
  project's own policy is no address of its (`README.md`, section 8).
- **A goal template's work,** `template-goal` with the template's number
  (`../../protocol/configuration/settings.md`, 2, kind 11), `{"design": BOOL,
  "close_out": BOOL}`, is read and proposed as a template is, beside it:
  a proposal of a goal template whose charter designs the goal first or
  makes a close-out carries the template's work saying so, and a read of
  one with none stored answers that its coordinator plans and makes no
  close-out, never unknown. The tools' declarations name it, as they
  name every kind they take; Ask temper's instructions add no words for
  it, as they stand at their bound.
- **Structure and rules:** a sentence of a description that changes what
  temper's engine wakes, schedules, permits, pays for, lands or holds is
  structure, and becomes settings; one that only changes how an agent
  behaves is a rule, and stays text: a charter's instructions, or a
  document of an adopted repository the charter names
  (`../../protocol/configuration/settings.md`, 6), so that a change landed there
  reaches the next runs. Where temper has a mechanism for a rule, it offers
  it as structure and says what each choice guarantees; a rule becomes
  structure only by the person's choice. What settings cannot express it
  says plainly, offering the nearest. A provider temper isn't set up for,
  listeners, workers and limits belong to whoever runs temper here: it
  says so and never proposes them. Making a project public is no setting,
  done by its owner outside the chat (`../../client/ctl/tools.md`, 3), and no
  proposal's item carries it.
- **The approvals:** an approval counts a person with a known forge
  identity on the forge of the repository a rule covers, as the
  inventory shows them, a box's person by its link (`../forge/adoption.md`, 6);
  adoption seeds roles by the project's permission mapping, a stock admin
  as maintainer (1), never owner (0), keeping existing roles (`../forge/adoption.md`
  ). `propose_settings` judges each landing rule of the chat's
  own project that enforces approvals at the call, as acceptance judges it,
  and refuses one whose role has too few eligible people in acceptance's
  words, which ask for human collaborators added on the forge and the
  repository adopted again (`../forge/change/gates.md`, 3; `ask-temper.md`, 2); another project's
  eligibility is acceptance's, said to the owner who decides.
- **temper's own way:** asked to set a project up the way temper builds
  itself, in the person's own words, it
  reads the stock page `how-temper-builds-temper` with `see help`
  (`../../protocol/documentation.md`, 2.2), a page of temper's the person
  may read too, and proposes its settings for the deployment, its
  models among those the inventory's offer names. What the deployment cannot do
  yet it says, and leaves out; workers stay the deployment's.
- **A proposal, and its outcome:** once the person agrees, one proposal,
  whole, against the newest revision it read, its reason in their words,
  what temper enforces apart from what agents follow, and what the change
  does to running work; it tells them where the card waits and whose
  decision it is, as the tool's answer says (`ask-temper.md`, 2). A fresh
  `propose_settings` replaces its own proposal that waits (jig's
  `domain/tasks/proposals.md`). Asked to change one, it proposes again,
  whole, against the newest revision: the new card stands below, and the
  old one reads as replaced (`../../client/pages/settings.md`, 2). Refused, the
  one that waits stands. The outcome is heard in its inbox (jig's
  `domain/engine/settings.md`, 6): accepted, it says so in one line naming the
  revision; rejected, it revises from the person's reason; withdrawn as
  stale, it reads again, proposes afresh and says what changed.
- **It changes nothing itself.** Its tools are the reads and the proposal
  a person may make, through the same role and proposal checks: its
  task's authority, policy 1's for its role, holds the asking and settings
  families (`ask-temper.md`), and only an owner's decision changes settings (jig's
  `domain/engine/settings.md`, 6). A member's Ask temper proposes as an owner's
  does, and its proposal waits for an owner.
- **It writes down how it understood the person** as the chat's
  document, "How this project works", with `document` (`ask-temper.md`, 3), once it
  understands their way of working: a section a topic, open questions
  last, kept current as the conversation changes it (3).
- **Its tests** are `../../testing/checks.md`, 4's: scripted runs of charter 11 in
  the engine's domain world for its flows, and stories through ctl and
  Ask temper's page in the joined client world (`../../testing/checks.md`, 3).

## 2. Asking before the work

A person who wants to agree a plan before anything is changed: rung B0
of `../../draft/v0.1.0.md`, 2.1.

- **The work that waits asks; a change asks only as it is made to.** A
  change is decided work, asked for once what to change is agreed. Made
  with `questions` `"none"` (`tools.md`, 1), the default, its agents, the
  table's first four rows, ask no one: the change procedure delegates
  each with the Message family taken out of the tool families it passes
  on, which a child's authority otherwise keeps (jig's
  `domain/authority.md`, section 3). The change itself takes no
  questions, answers or words (../forge/change/README.md, 1); its amendments and its
  cancel reach it as ../forge/README.md says. A plan's maker that wants its
  changes' producers to ask makes them with `"foreman"` or `"up"`, and
  the change relays their questions (../forge/change/questions.md, 1); what follows is a
  chat's, whose changes take none.
- **The chat delegates the work as a goal** when its person asks to
  agree it first, with `goal` (`tools.md`, 2) and the person's words: a
  delegate under charter 1 holding the authority to call `change` and
  to ask; never `change` itself, which would start the work at once. The stock
  chat's instructions (charter 1, `../../protocol/configuration/stock.md`)
  carry the rule: work the person wants to agree before anything is
  changed goes to such a delegate, which tells its plan, asks, and calls
  `change` only after the answer, which reaches it from the person's
  inbox, not through the chat. When the person answers the goal's question,
  the goal has the answer and does the work; the chat never calls `change`
  or `goal` for that answer and tells the person the work goes ahead. The chat passes the person's words whole, at once, even where they
  ask for an advisor first, since the goal asks one itself (`tools.md`, 2): it
  never sets `advisor` for such work, writes the plan or asks the
  go-ahead in its own reply, nor calls `change`. The goal, once it has
  asked, waits for the answer: it never finishes before it, nor before
  every agreed change has landed. The brief opens with the caller's words
  so that the task's title is the work (`tools.md`, 2); a blank line follows,
  then the root's goal line and its line that the work is to be agreed,
  then the person's heading and words.
- **A held change or goal waits for the person at once.** jig skips
  conversations when it chooses the escalation's holder (jig's
  `domain/tasks/proposals.md`). The chat tells the person in words
  what its delegate waits on and that it waits in their inbox, then
  waits for their decision.
- Every Finite charter 1 run keeps `message` in smith's reserve; the owing fact is enforced at the root and is not known when the worker builds the charter.
- **A goal a chat calls asks its requester,** the chat, with `message`
  in the question form, before it calls `change`, and waits. jig puts the
  question in the inbox of the chat's person, since the person requested
  the chat (jig's `domain/tasks/messages.md`, 1; jig's `domain/people/README.md`,
  section 5). The chat cannot ask its own person: it says in its reply
  that a question waits for them.
- **A goal a person started asks that person.** A goal from the sheet or
  from Goals (`../../client/pages/goal.md`, 3) has a person as its requester.
  Its brief says `[Requested by person N]`, and `message` reaches tasks
  alone (jig's `domain/tasks/messages.md`, 5). So, wherever it would ask, it
  calls `delegate` with one person task addressed to its requester (jig's
  `domain/people/README.md`, section 7): executor `{kind: person, person: N}`,
  the question in the line form below as its spec's words, a report
  contract and an authority of nothing; then it waits. Its result is the
  person's answer, and it wakes the goal with those words, as any
  delegate's does (jig's `domain/tasks/lifecycle.md`, 7). The goal's paragraph of
  charter 1 says so in its one paragraph, `decided`'s Ask and the hint of a
  `message` refused for its reference say so too, and a goal a chat calls
  goes on asking its chat. A person's `Create` never sets `asks`, so the
  rail below, which a `message` question alone answers, holds no goal a
  person started; one whose brief says the work is to be agreed asks its
  plan the same way.
  - **Its permission is a role's,** as jig's executor permissions are
    (jig's `domain/authority.md`, section 3): the stock `rules.ceiling`
    and each project's `policy.ceiling` name roles 0, 1 and 2, those
    whose people may create work, and a role's authority names its own
    role alone. So what a person starts may address only someone holding
    that person's role. Another role is refused with
    `missing executor permission`; a person with no project role is
    refused with `Refused: executor`. A goal a chat calls loses every role
    kind as `goal` narrows its authority (`tools.md`, 1 names what it keeps), so
    it may address no one. Among holders of one role, the charter's words
    alone say "its requester" (`README.md`, section 8).
- **A question's line form.** jig's question carries words alone, so a
  question's options and its recommendation are in its words; the stock
  chat's instructions (charter 1, the goal's paragraph) have a goal
  write them in one form, which temper's client draws as a question,
  its context, what the agent saw and its choices (`../../client/pages/README.md`
  , *Needs you*). No protocol, store or tool knows the form:

  ```
  Fix the dropped watch in jig’s store, or keep it alive in temper-box serve?

  MakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local.

  Saw: The box test reproduces it every time.
  Choice: In jig’s store (recommended)
  Choice: In temper-box serve
  ```

  - The first line is the **question**, and never a trailing line.
  - The **trailing lines** are found walking up from the last line,
    blank lines passed over: each line beginning `Saw: ` or `Choice: `
    is one, and the first other line ends them. Such a line above that
    point is context, words like any other.
  - The lines between the question and the trailing lines are the
    **context**, without its first and last blank lines.
  - A trailing `Saw: ` line is **what the agent saw**, several joined by
    a line break, in their order.
  - A trailing `Choice: ` line is a **choice**, in its order, its words
    what follows the prefix, trimmed; one with no words, or past what an
    answer's words take in the client (`CUE_BYTES`, 256), is no choice.
    One ending ` (recommended)` is recommended, that ending no part of
    its words; past the first so marked, the ending is removed and
    recommends nothing.
  - **The fallback:** words with no choice, or words the read cut
    (`question.words.cut`), have no trailing lines: the first paragraph
    is the question, the rest the context, with nothing the agent saw
    and no choices, and they are answered by a reply alone. Any question
    a person or an older goal wrote draws this way, and one a change
    passes up with a foreman's reasons beneath it (../forge/change/questions.md, 1), whose
    last lines are those reasons.
- **The person answers in the inbox,** `temper ctl task answer TASK
  QUESTION TEXT` (`../../client/ctl/tools.md`, 3), Home's card or the inbox's
  page, and
  nobody else does: the answer reaches the goal itself, as its
  requester's, never through the chat's model, which may not answer a
  question that waits in its person's inbox (jig's `domain/people/README.md`,
  section 5): its `message` answer is refused, naming that its person
  answers it there. The chat hears a notice that the question was
  answered, with the person's answer words cut at 256 bytes on a UTF-8
  boundary and marked when cut, and says so. The goal is kept while its
  question waits, including when its ordinary turns expire after asking
  (jig's `domain/tasks/lifecycle.md`, 8): a run of it that finishes before the
  answer parks, and the answer wakes it. It then calls `change` with
  what was agreed, waits for its result and reports; the chat says what
  landed.
- **Nothing is changed before the answer:** no change task, branch or
  pull request exists until the goal calls `change`.
- **The asking is the root's rail, not the goal's choice.** The chat
  marks such work once, as it delegates it: `goal` with `agree` set, at
  the moment its person asks to agree it first, and the root makes the
  goal's task owing its requester a question (jig's `domain/tasks/lifecycle.md`,
  8, its spec's `asks`), the typed mark everything below reads, never
  the brief's text. `finish` is the run's own, so an agreed goal that
  finishes before it has asked is not refused but parks, kept by what it
  owes, and wakes at once to the notice "this work is to be agreed: ask
  your requester your plan with message, as a question, and wait"; its
  `change`, the root's host call, is refused before that question is
  answered ("wait for the answer to question N"), the run going on, as
  any refused host call does. Once answered,
  it changes and finishes as any goal; a person who declines is
  answered by a report, changing nothing. So a model that writes its
  plan as its result, or changes first, is turned back to the inbox.
- **Its stories:** `suite::fuzzy::green_light` on fakes (../../testing/real-loop.md), and
  rung B0 walked on real backends (`../../draft/v0.1.0.md`, 2.1).

## 3. Documents and reports

### A chat's document

A chat keeps one Markdown document beside its conversation: its agent writes
it, the person reads it in Chat, Split or Draft (`../../client/pages/README.md`,
*A chat's document*), and a goal starts from it. It is the note of jig's in the
chat's own scope (jig's `domain/engine/briefs.md`, section 2), the lowest numbered
whose description does not begin `work:`: there is no kind. A `work:` note, such
as the `work: settings` note a proposal's note source names (`tools.md`), is never drawn, and a chat
with only working notes has no document. Its description is its title, its body
Markdown. Ask temper's "How this project works" (`ask-temper.md`, 3) is the same
mechanism: its title is that literal, since a run knows its project's number
and not its name.

- **Who may write:** a chat's authority's note scopes hold a goal's, which for
  a root conversation is its own (`../../protocol/configuration/stock.md`), so a
  chat writes its own scope and only it. A goal uses its own scope for its
  report; an advisor keeps no note scopes. A run knows its task number from its brief.
- **Its instructions** are held in the binary, under the chat's own paragraph
  of `chat.txt`: when the person asks for a draft, or says "write this up", keep
  one document, with `note`, scope goal, the project and its own task number,
  description its title, its body Markdown beginning with `# TITLE` and a muted
  line "Draft, with the user." (no run is told the date), one `##` section per topic, open questions
  under "## Open questions". It writes a first version whole, then changes it by
  lines (jig's `domain/engine/briefs.md`, section 2), naming the note and the revision
  it read; it never rewrites it whole past a few KB, builds a long document in
  parts, and continues a line past one call with `join`. It says in a sentence
  what changed and doesn't repeat it. Where a sequence says more drawn, it writes a
  flow, and its words name the subset (`skein/docs/design/ui/markdown.md`, 1): the same
  block reads in a repository, where a forge renders Mermaid.
- **Ask temper's,** `ask.txt` (`ask-temper.md`): the same, through `document` (`ask-temper.md`, 3), its
  title "How this project works", kept once the person's way of working is
  understood, a section a topic and open questions last (1); its reply says in
  one sentence what the document now adds, and never repeats its text.
- **The tool's answer** gives the client what its line says (`../../client/pages/README.md`
  ): `Note N revision R written · 64 lines` for a first write, `Note N revision
  R written · +18 −6 · line 31 · 64 lines` for a later one, from jig's `lines`,
  `added`, `removed` and `first` (jig's `domain/engine/briefs.md`, section 2). The
  lines close each so that an append's `first` needs no counting. A refused call
  says what to do next: `Note refused: revision moved, now 7`, `Note refused:
  range past the end, 64 lines`, `Note refused: body past its bound, 70000 bytes
  of 65536`.
- **Its stories:** a scripted turn of the fake model that writes a document
  (`../../testing/real-loop.md`), proved through ctl and walked in the journey `chat-draft`
  (`../../client/web/journeys.md`, 9); and the real loop's
  `suite::notes::a_long_document_is_built_in_parts_and_revised_by_a_fresh_run`
  (`../../testing/process-tier.md`), whose script builds a 30 KB note in calls of under 10 KB
  each, one line longer than a call among it, then has a fresh run find the note
  in its brief and change it by lines.

### A goal's report

A goal tells its person where it stands in a **note**, one per goal, which
needs no record, read or wire field of its own. The client reads and draws it
(`../../client/pages/README.md`, *A goal's report*); a person reads it first.

- **Its shape:** scope goal (the tool's scope `goal`, the client's
  `NoteScope::Task`). Its description is `report: ` then the headline, one line. Its body
  is the report's paragraphs, separated by blank lines, then, each on its own
  line and each optional: `Left: …`, `Risk: …`, `Decided: first, 05:40 ·
  second, 05:12`. Only a line's start makes one of these; the words elsewhere
  are a paragraph's. Limits: jig's notes, 256 bytes of description and 8 KB of
  body in the standard profile.
- **Revised in place:** the agent names the note and its revision from its
  brief's index, so that one note stands, its scope's numbers from the
  brief's task section. Where several answer, the client takes the newest
  by `changed`.
- **When:** after planning, and whenever what is true of the goal changes: a
  task landed, failed or split, a decision taken, a risk found.
- **Its words:** the headline says where the goal stands, in a sentence a person
  reads first. The paragraphs say what happened and why, in the first person.
  `Left:` what remains; `Risk:` what could go wrong; `Decided:` what it decided
  itself, each with its time. A message that changes the plan becomes an
  amendment the goal shows the person first.
- **Who is told:** the goal's part of charter 1's instructions and the
  reference's charter 5 (`../../protocol/configuration/stock.md` and `../../protocol/configuration/reference.md`). The
  stock's authority holds the goal's scope (13.4); a charter that does not ask
  for a report writes none, and no note yet is a normal state, never an error.
- **Its stories:** the joint world's
  `suite::task::a_chats_goal_shows_its_report_from_its_goals_row` (`../../testing/process-tier.md`
  ): a chat's goal writes its report, and the person reaches it from Home's
  Goals link through the goal's row. The scripted fake model writes a report in
  its goal script, so that a person or a journey on `temper box --fake-backends`
  meets one (`../../protocol/configuration/directories.md`, 1).

## 4. The drafter

A design session is a chat with the drafter (`../../draft/surfaces.md`, 11.2):
its person and it design one piece of work, as a draft the person starts a
goal from. It is template 6, a chat run by charter 12, the drafter's
(`../../protocol/configuration/stock.md`), and a chat in every other respect: a
conversation its person ends, kept, listed among the project's chats.

- **Its instructions** are the `stock` module's `drafter.txt`, held in the
  binary as every stock charter's, every byte as written: read what the work
  touches before proposing; ask one question at a time, as numbered choices,
  the recommendation first with its reason and "Tell me something else" last;
  take each answer into the draft at once; keep the open points under
  `## Open questions`; say in one line when none blocks the work; and keep
  the draft as the chat's document (3), a first version whole, then
  changed by lines. temper/ui writes them, as a chat's.
- **It reads and starts nothing.** Its charter names every family a chat's
  does but delegate, so its runs are declared no `goal`, `change` or
  `delegate` (`tools.md`), and its task's authority, a chat's, would refuse none of
  them: the declaration is the rail. Its charter names no inspect, modify or
  shell, so its runs are given no repository (`charter.md`, 2): it reads the project's
  code through the forge's reads, `read_branch` for a repository's head and
  then `read_file`, as a chat does (`tools.md`, 1), and its model's sub-agents survey
  where it has them, its charter's `agents` set.
- **Its draft** is the chat's document (3): a `note` in its own task's
  scope, which its authority's goal scope admits, as a chat's.
- **A session is kept:** its task is kept and its charter resumes (`charter.md`, 5), so
  a session opened again continues its conversation; its instructions read
  the draft and its open points first and take up the first.
- **A goal starts from its draft** as from a chat's document, by "Start
  goal…" (`../../client/pages/README.md`), its ways the project's tracked
  templates (`../../client/pages/goal.md`, 3); the drafter calls no `goal`.
- **Where one starts:** on the web, the new chat with a draft where `me`
  lists template 6 (`../../client/pages/README.md`, *A new chat*); from `temper
  ctl`, `chat start --template 6` (`../../client/ctl/tools.md`, 3); in the
  terminal, `/draft` (`../../draft/surfaces.md`, 11.2), which the terminal's
  design takes up.

## 5. Open questions

- **A permission for its own requester alone** (`agents.md`, 2): a goal a person
  started may address anyone holding that person's role, since jig's
  executor permission is a role's, and a requester who has since changed
  role can no longer be addressed. A permission naming the requester would
  be jig's, a new executor kind. And `shapes::set_agreed_goal_from`, were it
  sent, would owe a question no person task answers. Default: the role, the
  charter's words, and that shape unsent.

- **Constraining models in settings reads and proposals** (`agents.md`, 1): whole
  charter values can name models outside the current offer. Restricting what
  a read exposes while still round-tripping that value, or rejecting a new
  unoffered choice without rejecting an unchanged charter, needs a settings
  contract. Default: the inventory is filtered; whole charter reads and
  configured-model validation stay. Ask temper's instructions impose no
  restriction to the current offer (`../../draft/ask-temper.md`, section 8).

- **The person's forge identity in a chat's brief,** rather than in the
  settings inventory (`agents.md`, 1), which Ask temper reads first.
  A brief section would spend every chat's room for what one charter
  uses. Default: the inventory.

- **A person known by a login alone** (`agents.md`, 1), where no link vouches for them:
  `temper-box serve` with several members, a managed Forgejo, a pasted App. Often
  no collaborator, adoption finds no person for them and the inventory's forge
  person has no forge user id; `read_permission` takes an id, so the agent
  cannot match them, and `propose_settings` refuses at the call, in
  acceptance's words, an approval only they could meet (`../forge/change/gates.md`, 3;
  `ask-temper.md`, 2), saying who to add on the forge before adopting again, in the browser the adoption
  form typed afresh (`../../client/forge/family.md`, 4). A family of links waits for
  `temper-box serve`'s members (`../../protocol/sign-in.md`, section 9).
  Default: adoption's seeds and the refusal above; a configured login alone
  supplies no verified forge identity.

- **What sizes temper's own way** (`agents.md`, 1): a read of the deployment's worker slots,
  an address of `see`'s, or an answer from the person.
  Default: the person's, as nothing reads
  them.

- **QA in temper's own way** (`agents.md`, 1): `../../draft/own-way.md`, 6, has it proposed once
  a tester is set; QA after landing isn't built (`../findings.md`, section 8).
  Default: the agent says so, and leaves it out.

- **Ask temper woken by a landing** of a document a charter
  names, where the landing touches structure, to say what it means for
  the settings ("the process document changed; here is what it means
  for your setup": `../../draft/configuring.md`, section 6). Default: a
  person starts it (`agents.md`, 1); a landed document reaches the next runs of the
  charters that name it (`../../protocol/configuration/settings.md`, 6).

- **A goal's question kept with its chat** (jig's `domain/tasks/messages.md`,
  5): the chat, woken by it, would answer from what it knows (its
  person's priorities, its notes), or ask its person a question of its
  own and answer the goal once it has the answer, in place of `agents.md`, 2's
  route through the person's inbox. Default: `agents.md`, 2's route.

- **Who else writes a question's line form** (`agents.md`, 2): the stock goal alone
  is taught it, by one sentence of charter 1, as the request of 2026-11-05
  for *Needs you* asks. A change's producers, its repairs and resolutions, and
  the reference's charters are not, so their questions read as words,
  answered by a reply. Default: as it is; a line in each charter that asks.

- **Ask temper changing the catalogue** (`agents.md`, 1): observe a real model
  reading and proposing a model setting, including estimated prices, and
  keeping its suggestions within the offer, which its instructions do not
  name. Default: scripted stories of `see` and `propose_settings`; the
  real-provider observation is the user's, by trying it (`ask-temper.md`).

- **A real model asking its person** (`agents.md`, 2; the request `ui-goal-asks`):
  the client world's sheet-to-answer story supplies a typed `delegate`
  call, and the stock and host-answer tests hold the instructions. They
  do not establish a real model choosing that call from the sheet or
  Goals, with Away on or off or with always-ask words. Default: those
  instructions and focused stories; real-model observation follows landing.

- **A real model revising a proposal** (`agents.md`, 1; the request `ui-proposal-2`
  of 2026-11-06): after landing, check that a request from "Change
  something" leads to a fresh proposal without a trip to Needs you, and
  that a request for a model no account runs leads to browser Settings,
  Accounts. Default: the stock instructions and scripted stories; the
  real-model observation follows landing.

- **A goal as the designer** (`tools.md`;
  `../../draft/goal-as-designer.md`, sections 3, 5, 6, 7 and 9): the delivery
  path is independent of the stock and reference instructions, their
  wakes and their location. Those words and settings are folded into the
  reference workflow together. Calls, briefs and reports take the room
  of `../../protocol/configuration/settings.md`, 7, with the reference's fallback.
  Default: the existing stock and reference words and wakes; test charters
  exercise goal delivery.
