# The agent's tools

## 4. Tool families

- **smith's own families** by the charter's policy: inspecting,
  modifying and the shell in the workspace, and sub-agents, each as the
  charter's `inspect`, `modify`, `shell` and `agents` say; `wait`, always;
  `deliver`, for a finite change's contract and for a goal's run (below)
  (jig's `domain/hosts/charters.md`; smith's `domain/run.md`, 8.4),
  never in an Open run (`charter.md`, 5). A run that cannot deliver cannot modify
  files: jig's charter translation grants `modify` only with `deliver`
  and the charter's permission (jig's `domain/hosts/charters.md`). A
  chat's, an advisor's and a reviewer's runs inspect alone, whatever
  their charter's `modify` (a rail, not an instruction). Stock charter 1
  disables `modify`; repository edits, including small fixes, go through
  `change` (1). The stock coder has read, write, edit and remove: a
  file the change drops is removed once read.
- **`save`, for a change's producer** (smith's `domain/run.md`, 8.5; jig's
  `domain/hosts/charters.md`): granted with `deliver` to a finite run with
  a change among its outcomes, and to no other run, a goal's included. It
  pushes the checkout's tree to the change's own branch at once, with no
  check run, marked unchecked (`README.md`, section 5), so that a producer that stops to
  ask a question has its work on the branch first, unrefused for being
  unfinished. Only a `deliver` or a `finish` that delivers runs the checks,
  and only a producer's successful result, whose finish ran them, has its
  change merged into the goal's branch (`../forge/change/README.md`, 1).
- **A goal's run may modify and deliver** where its charter's `modify` is
  set and its task holds a goal branch in the repository; otherwise it
  inspects alone. It delivers straight to its goal branch, a design
  writer of it (`../forge/branches.md`, 3), with a `title` and a `body` as a
  change's delivery has them, at any point in its run and as often as it
  likes, and the project's checks run at it (`README.md`, section 5; smith's
  `domain/run.md`, 8.1). A design delivery carries Markdown alone, takes
  the goal branch's moved tip first and is never stale (`README.md`, section 5); code
  goes through `change`. Delivering isn't finishing: it has no change
  among its outcomes, its `finish` offers a report and a failure, and it
  goes on after a delivery. What it delivered is on its goal branch once
  the receipt says so: no change lands it.
- **Each host tool below is declared by its family,** with the charter's
  call timeout (`declarations.md`, 1): to a charter that isn't asking,
  where its `tools` names the tool's family (jig's
  `domain/engine/executors.md`, 1) and its run's authority holds it, so
  that a charter that starts nothing, the drafter's (`agents.md`, 4), is
  offered no `goal`, `change` or `delegate`, and only a charter of a
  deployment's own that names the settings family is offered `settings`
  and the configuring form of `propose_settings`
  (`../../protocol/configuration/stock.md`); `read_docs` to every run but
  an asking one. Its task's authority still decides each call, as below.
  Each is declared in a short form, its whole form on the `host-tools`
  page (`declarations.md`, 2).
- **A charter that names the asking family is declared its own tools
  alone** (jig's `domain/authority.md`, section 3): an asking charter is
  one whose `tools` name the asking family and no family but it and
  settings, so that a mask of every family, as a record with none decodes
  (jig's `RunPolicy::EVERY_FAMILY`), is none. It is declared `see`, `document`
  and `remember`, the asking family's, where its task's authority holds that
  family, and `propose_settings` in its asking form where the charter names
  the settings family and its task's authority holds it (`ask-temper.md`); no other host
  tool, of any family, whatever the charter's other families, and
  neither `settings` nor `read_docs`, which `see` reads in their place.
  smith's `wait` is granted as for any Open run. The asking tools are
  declared to no other charter. A tool a run never calls costs every
  request and blurs what it does, so these charters are declared no other
  family's tools, whatever they name. Authority decides
  each call by its family as for every charter: `see`, `document` and
  `remember` by the asking family, `propose_settings` by the settings family. The task's authority decides
  each call by its family (jig's `domain/authority.md`, section 3, and `domain/authority-checks.md`, section 1.5
  ): a call beyond it is answered beyond authority, naming what it
  lacked, and the run may `propose` it instead. A refused tool family
  is named from the call's tool, and repeated findings for that family
  print once; the refusal grants no additional authority.
- **The settings family's tools,** for a charter of a deployment's own
  that names the family and not the asking family:
  - `settings`, a read: with no member, the inventory (`agents.md`, 1); with a
    `setting`, its kind and the `number` or `project` the kind takes,
    that setting whole at the newest revision; a revision's record; or a
    page of the history. Its pages use `core.call_answer_bytes` less the
    tool name, reserving escaped framing and continuation markers. A long
    value fills the available page even after its heading; `next:
    OFFSET` counts original UTF-8 text bytes for exact reconstruction.
    This allowance is separate from `read_docs`' fixed bound
    (`../../protocol/documentation.md`, 5). A read that leaves out the
    member its kind needs is answered with what to do, "landing-rule
    needs number; read the inventory and name what it lists", as a
    failed call, which a person's page keeps showing.
  - `propose_settings`, its configuring form: whole settings against the
    newest revision the run read, with a `reason`, the kinds a settings
    file's (`agents.md`, 1). Its source is exactly one of inline `items` or `note:
    {name, revision}`, a note of the goal scope described `work:
    settings` whose body is a whole settings file `{"items":[...]}`,
    built by bounded `note` writes, `first`, `remove` and `join` (`agents.md`, 3).
    For a note, the root checks the settings and recall families and
    loads the whole named note within the configured notes and load
    bounds. Its scope must be readable by the proposing task and its
    revision must match. The shared settings-file parser decodes its
    body, and the same whole-value validation, base-revision checks and
    owner review apply to either source. Missing, inaccessible, moved or
    malformed sources create no proposal and apply nothing. A decided
    call is replayed before loading the note again; later note edits
    cannot change its proposal. No fragment state is added; the note
    keeps jig's revision, retention and cleanup rules. A fresh call
    replaces the run's own proposal that waits (jig's `domain/tasks/proposals.md`
    ); refused, the one that waits stands. One decoder and one
    answer serve this form and the asking one (`ask-temper.md`, 2), but `more`, which
    is refused to a charter that is not asking; this form's approvals are
    judged at acceptance alone (`../forge/change/gates.md`, 3).
  - Both tools' schemas name the numbered kinds. `read_docs` is declared
    to every charter but an asking one (`../../protocol/documentation.md`,
    section 5).
## 5. Task reads and subscriptions

- **A coordinator's task reads.** `read_task {task, field?, from?}` returns a
  related task's specification by default, state, dependencies and spec revision,
  and its hold revision, reason and connector-supplied account. Retained ended
  direct delegates remain readable, and so is every task below the run's task,
  at any depth, reached through a proven same-project requester chain of live
  or retained intermediate tasks, its specification and its result alike: a
  goal reads the coding agent under one of its changes, or a task under its
  closer, by the number its messages name. The dependencies of the run's
  task's requester, live or retained, are read too, read only: a change's
  coding agent, and a repair or resolution under that change, read the changes
  its `after` names (1) (jig's `domain/engine/executors-tools.md`, 1).
  `field=result` reads kept terminal reports. A live task's `field=result`
  read answers as its specification read does: its header, with its state,
  hold and questions, then the line `Result: none yet; it comes when the task
  ends. Its specification follows; continue it with field left out.` and the
  specification's first page. At an offset past 0 it is unknown, as an ended
  task's missing report is.
  One answer holds as much of the words as the run's reply takes, whatever a
  client's `documents.text_bytes`, so a spec within that room comes back in one
  call: the reply bound its worker names in its hello, else the local agent's.
  Where neither names one, it holds a piece of `documents.text_bytes`
  (`../routing.md`, 2).
  A cut answer prints `next: OFFSET`; pass that byte offset as `from` with the
  same field, and restart specification reads at zero if the spec revision changes.
  Task reads decide nothing and keep no named-call record. Escalations name
  their task and hold revision and give this recovery call. Read access does
  not grant messages or amendments to ended tasks, or the subscribe family.
- **A run's subscriptions take its task's room.** `subscribe`, to a task
  or a timer, and `subscribe_forge` each take one of the task's
  subscriptions, at most `tasks.subscriptions` (jig's `domain/tasks/messages.md`, 4, and `jig/docs/design/domain/tasks/README.md`, 6
  ), where the topics the core keeps for a goal
  (`../forge/topics.md`, 1.2) count too, and so does the project's
  findings topic the root keeps for a finding's holder, which alone is
  admitted one past that room (`../areas.md`, section 5). A call past that room is refused, saying
  why, as jig's `domain/engine/executors-tools.md`, 1 has every refusal do:
  `subscribe` answers "Refused: task 4 busy: it holds its most
  subscriptions (at most 4, counting the topics the engine keeps for a
  goal); unsubscribe one it made to make room", and `subscribe_forge`
  names `tasks.subscriptions` as its own room's refusals do. A
  subscription that would leave no room in the task's inbox for its news
  is refused as any call past the inbox is, naming the inbox.
- **A batch's resource reports** (jig's `jig/docs/design/domain/tasks/holds.md`,
  2) answer every call that makes tasks, `change` and `delegate` among
  them, within `tasks.hold_wait`. A batch the reports' table cannot hold
  answers "Refused: busy: the engine keeps reports of at most 256
  resources, those live tasks and waiting batches hold, and this batch's
  do not fit; call again once tasks have ended", 256 the refusal's
  bound. One whose reports have not all come by then answers "Refused:
  task 9871 unreported: no report came within tasks.hold_wait for a
  resource held by tasks 9871 9873; nothing was made; call again",
  9871 its first member so named and the numbers its `blocked_by`, in
  its order, each after a space as other refusals' tasks are.
- **A charter's room** is `room.md`'s: smith's host part's budget, the
  allowance a charter is fitted to, how its brief is cut, what a refusal
  names when the rest alone doesn't fit, and the checks at start and at
  each change of settings.
## 6. Forge reads

Moved to [`tools-forge.md`](tools-forge.md), section 1.

## 7. Control and findings

- **Reads and writes:** the forge's reads, `recall`, `settings` and `read_docs` are reads; every
  other host tool is a write, which smith runs alone (smith's
  `domain/run.md`, 5.2).
- **`decided`,** of the control family, declared where its charter names
  that family and its authority holds it (`declarations.md`, 1), so to the
  chat's, the goal's and the standing roles' runs, whose questions reach a
  person (`../forge/attention.md`, 5), and not to a coder's or a reviewer's: `{
  question, options, took, why, kind, paths, under }`, `options` 2 to 6
  words of the agent's, `took` an index into them, `kind` one of the eleven
  of `../forge/attention.md`, 6, `wording`, `retry`, `implementation`, `tests`,
  `dependency`, `departure`, `money`, `irreversible`, `rules`, `other` and
  `findings`, the schema's enumeration in that order;
  `paths`, optional, at most 16 repository-relative paths of at most 256
  bytes that the choice would change; `under`, optional, the number of an
  item with words of the project's classes the choice falls under;
  `question` at most 400 bytes, each option 120, `why` 400, a call past
  those refused at decoding, saying so. Within them, the charters' words
  have the question one sentence and each option one line, for the
  person who reads them (`charter.md`, 4, *Words a person reads first*);
  nothing checks it. The forge judges it (`../forge/attention.md`,
  5 and 6): "Decided: carry on with TOOK." with a `Decided` row
  saved, or "Ask: REASON. HOW", REASON from its reach: "money always waits for your
  person", "implementation asks your person, even when away",
  "implementation asks your person while they are here", "changes to
  path set N ask your person, even when away", whose name `settings
  project-classes` reads (`ask-temper.md`, 1), "your words say Asks: always", "your person's decided list is full for today" where the
  class decided and no row had room, and, for a caller with no person,
  "no person is asked through you". Named and decided
  once, as `change`.
  - **HOW names the one way its caller can ask,** never a tool it
    lacks. The root reads it from the caller's task as it renders the
    answer, and passes it to the renderer beside the answer (the
    decision, `took` and `reach`, stays as stored): `Ask your requester
    with message, as a question.` where the caller's requester is a task
    and its authority holds the Message family; `Ask person N with
    delegate, a person task addressed to them.` where its requester is
    person N and its charter names the delegate family (`agents.md`, 2);
    and, where neither holds, the answer reads "Not decided: REASON.
    Nothing you hold asks anyone: decide within your brief, or leave
    the choice and your recommendation in your result." in place of
    the `Ask:` form. So a change's review, or an agent of a change with
    questions `none` (`../forge/change/README.md`, 1), is never told to
    call `message`; a repair of a change with questions `up` is told to
    ask its change.
  - Funds, sign-ins, machines, a plan's go-ahead and
    anything hard to reverse are `money` or `irreversible` and always wait,
    by the charter's words (`../../protocol/configuration/stock.md`); a change
    to the classes, a landing rule or a standing role's rules is `rules`;
    what becomes of findings the asker holds, a goal started for them,
    the backlog or a hand-off, is `findings`, the triager's question over
    a group (`../areas.md`, 6, *Its groups*).
  - **`findings` moved the host tools' contract:** the enumeration's
    added value is a layout change of `decided`'s line, so the contract's
    version moves by one with its kept text, and each kept version from
    the floor whose `decided` line differs has its adapter, which hands
    the old call's input on unchanged, since every kind an older contract
    names is a current one (`../../protocol/wires.md`, 7). The floor
    stays.
    A task that overturned words reach (`../../client/pages/README.md`) asks it
    by its requester's route (`agents.md`, 2), and waits. Nothing in the engine
    answers a question for it, and one that skips the tool waits as ever.
- **What a run reads names what it holds.** A declaration's description,
  or a member's, names another host tool as the way for the run to do
  something only where every run offered the first is offered the
  second; elsewhere it says what to do in words that name no tool. A
  member that takes another tool's shape (`propose`'s `amendment`, "as
  amend's amendment") names no call, nor does a tool's effect on other
  tasks (`introduce`'s). So `decided` reads "Ask whether you may decide
  this for your person. Carry on only if decided; otherwise its answer
  says how you can ask.", its `under` "The item with words of the
  project's classes that covers this choice.", and `read_task`, offered
  with the read family alone, "Specs/results, from=next. Durable
  question numbers and answerers, for an answer."
- **A proposed `amend` takes `amend`'s amendment** (jig's
  `domain/tasks/proposals.md`): one naming no authority proposes the
  target's authority kept, as `amend` takes it, and what it needs is
  read from the target's authority; `authority shape` refuses only an
  authority it names that is malformed.
- **`finding`,** of the delegate family, declared where `goal` and
  `change` are, so that a chat, a goal's coordinator and a standing role
  file and triage findings in their task's project (`../findings.md`,
  sections 4 and 6; `../areas.md`, sections 5 and 7). One tool, its
  `act` saying which:
  - `{"act": "file", "kind", "severity", "area", "title", "body"}`,
    `kind` one of `qa`, `flake`, `red`, `review` and `feedback`,
    `severity` one of `low`, `medium` and `high`, `area` optional, title
    and body within `../findings.md`, section 2's bounds: "Filed finding
    P/N; it is HOLDER's.";
  - `{"act": "hand", "number", "area", "why"}`: "Handed finding P/N to
    area NAME; it is HOLDER's.";
  - `{"act": "route", "number", "goal"}`, `{"act": "backlog",
    "number"}` and `{"act": "drop", "number", "reason"}`: "Routed
    finding P/N to task G.", "Finding P/N is in the backlog.", "Dropped
    finding P/N.";
  - `{"act": "repair", "numbers", "words"}`, `numbers` an array of 1 to
    16 finding numbers of the task's project, oldest first as given,
    `words` 1 to 4,096 bytes: "Repair goal G made for findings P/N,
    P/M." (`../red-repairs.md`, section 4);
  - `{"act": "blame", "number", "commit", "why"}`, `commit` 40 or 64
    hex digits, absent for none, `why` as `hand` takes it: "Moved
    finding P/N's blame to landing COMMIT." or "Moved finding P/N's
    blame to no landing.", COMMIT its first twelve digits
    (`../red-repairs.md`, section 3);
  - `{"act": "list", "state", "mine"}`, both optional: the project's
    areas, a line each, `NAME (OWNER, HOLDER): WORDS`, its triager's
    line, then its findings newest first, a line each, `P/N STATE KIND
    SEVERITY AREA HOLDER: TITLE`, cut to the answer's bound with `and K
    more`;
  - `{"act": "show", "number"}`: one finding whole, as `temper ctl
    finding show` prints it.

  `HOLDER` is `task N`, `party N` or `role N`, by `../areas.md`, section
  4. Its refusals are `../areas.md`, section 7's words, and
  `repair`'s and `blame`'s `../red-repairs.md`, 4.2's and section 3's,
  marked an error, and the run goes on. The writes are named and
  decided once, as `change`; `list` and `show` are reads. `repair` and
  `blame` came with the host tools' contract's next version; a
  `finding` call of the version before is handed on unchanged.

- **`red`,** of the delegate family, declared where `finding` is, so
  that a goal's coordinator records the failures its branch carries
  that no live change owns, and ends them (`../forge/reds.md`, section
  3, which gives its members, answers and refusals): its `act` one of
  `record`, `end` and `list`, answered for a run whose task holds a goal
  branch alone. `record` and `end` are named and decided once, as
  `finding`'s writes; `list` is a read.

- **`follow`,** of the delegate family, declared where `finding` is, so
  that a standing role acts on the follows it owns in its task's
  project, as their owner (`../forge/follows.md`, section 5, *The role's
  `follow` tool*, which gives its members, answers and refusals): its
  `act` one of `start`, `hold`, `skip` and `merge`, the owner's acts,
  and `list` and `show`, reads of the project's `follows` document open
  to any task of the project. A charter that offers the delegate family
  offers it.

## 8. Call contracts

- **Declared, decoded and rendered in one place,** `temper-engine-smith`:
  each tool's declared form and whole form (`declarations.md`, 2),
  description and effect, written once (`delegate`'s and `amend`'s whole
  forms name every field of the spec and the authority, their declared
  forms the members a call usually sets, and both take a priority as
  `{priority}`, where jig's own `delegate` takes a bare number, so that a
  run learns one shape); a
  call's input decoded into the root's `Tool`, or answered at once as an
  error naming what is wrong, deciding nothing; an answer rendered as
  smith's text, marked an error or not (`../../protocol/llm.md`, section 4).
- **Calls are named and decided once** (jig's `domain/engine/executors-tools.md`, 1):
  the engine answers a call it has seen before from its record, and smith
  asks again under the same name after busy or a lost answer (smith's
  `domain/run.md`, 5.2).
- **An answer fits what the agent takes** (`../worker/hosted-runs.md`, 1.3):
  rendering a forge read or a task read uses the minimum of `core.call_answer_bytes` less
  the tool name and the receiving agent's reply bound, the root's
  `Domain::reply_bound` for the call's key (`../routing.md`, 2): the
  `agent.limits.bodies.host_reply_text` the run's worker names in its
  hello, from its agent's `agent.json` `profile.declared.tool_payload`
  (`../worker/agent-processes.md`, section 1); where it names none, the local agent's.
  Raising `core.call_answer_bytes` alone cannot overcome a smaller agent
  allowance. Where neither names one, the configured core allowance stands.
  A `see` answer is rendered within the local agent's bound alone
  (`README.md`, section 8).
  A call past the worker's input bound, `core.call_answer_bytes`, is
  answered `call too large` under its name, and the run goes on (jig's
  worker link). An answer past the agent's `host_reply_bytes` (smith's,
  its channel's `host_reply_text`, the agent's `tool_payload`) reaches
  the run as smith's `TooLarge`, which says the answer was too large
  and not shown, and the run goes on. The engine keeps the answer it
  decided, so a call asked again gets the same reply.
- **A call names what it saw** (jig's `domain/engine/executors-tools.md`, 1): the
  revisions above are those its brief, its messages and its earlier
  answers showed it. One whose object has moved since is answered moved,
  with what it is now, and the run decides again on that.
- **Not the engine's:** `wait`, `finish` and `deliver` (jig's
  `domain/engine/executors-tools.md`, 1). `deliver` is a push (`README.md`, section 5).
- **Jobs are smith's** (smith's `domain/tools.md`), not `read_ci`'s `job`,
  which reads a CI job's log. A `shell` command that runs past its
  yield goes on as a job in the agent's process, under the charter's
  `shell` (from `policy.shell`); the run reads or stops it with smith's
  `job`. Neither the engine nor the worker routes or answers a job.
  `temper-engine-smith`'s `turn` takes a turn request alone, and
  answers any other request, a job's included, as not a turn
  (`Problem::Type`).

## 1. A change, in the run's words

Moved to [`tools-change.md`](tools-change.md), section 1.

## 2. A goal, in the run's words

Moved to [`tools-goal.md`](tools-goal.md), section 1.

## 9. Open questions

- **Proposal-form diagnostics** (`tools.md`): `settings` reads name the kind
  and its missing member; `propose_settings` keeps the shared settings-file
  parser's errors. Default: those words, until a proposal needs the same
  kind-specific guidance. Both tools' schemas name the numbered kinds.

- **When to delegate a task rather than a sub-agent.** A run is declared
  its charter's families' tools alone (`declarations.md`, 1); how a brief
  teaches an LLM to choose a delegated task over a sub-agent of its own is
  open. Default: the charters' words.

- **A cancel's grace:** jig's `cancel` takes one, if the run gives it
  (jig's `domain/engine/executors-tools.md`, 1); temper's takes none.
