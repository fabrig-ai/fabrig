# The agent

Provisional, 2026-10-05, revised 2026-11-05. temper's agent is smith's:
the standard agent built from smith, the kit for LLM agents, on skein
like temper. smith's design is its own (smith's `domain/run.md`,
`domain/session.md`, `domain/tools.md`, `domain/host.md`), and how a
run's charter, tools, contract, messages and delivery become smith's is
jig's (jig's `domain/hosts.md`, sections 8 and 9). This document says
what temper fills in: the charters its engine writes, the host tools its
engine serves (4.3, the one table of them), its result contracts and
conventions, delivery as a push, and the agents temper runs. What is
still open is listed in section 10.

## 1. In one page

- **temper runs smith's stock agent,** on workers, one process per run:
  temper's own binary run as `temper internal agent`, smith's agent shell
  (`../protocol/README.md`, section 4). temper builds no code into it. A
  run's charter and its host's answers are all that make it temper's.
- **The engine writes the charter** (section 4): the task's instructions,
  its brief rendered, every host tool, a result contract with temper's
  verdicts and a change's `title` and `body`, `.temper/pre-pr` and
  `AGENTS.md` as conventions, and a budget in the deployment's unit with
  its models' prices.
- **The host tools are temper's** (4.3): jig's core's tools under temper's
  own names, and the forge's reads, effect and topics.
- **The worker is smith's host** (section 6; worker.md): it prepares the
  workspace from the forge, supervises the agent's process with smith's
  host domain, delivers a change by pushing it, and relays the rest to
  the engine.
- **The engine keeps what smith leaves to hosts:** transcripts, turn by
  turn, in its store; host tools' calls, decided once; what follows an
  answer (jig's `domain/engine.md`, section 7).
- **One translation, temper's own.** `temper-engine-smith`, which the
  engine's routing calls, is the only place where temper's charter,
  tools and results become smith's, and smith's back. temper uses
  neither `jig-protocol-charter` nor `jig-protocol-tools`, since its root
  has a `Tool` vocabulary of its own (`../protocol/llm.md`, sections 4
  and 5). smith interprets no temper vocabulary, and the worker none of
  either.

## 2. The agent in the system

```
engine   jig's core and the forge, the store; writes each run's charter and serves its tools
worker   smith's host: workspaces, agent processes, pushing their changes; relays the rest
agent    smith: LLM work, one run per process, reporting to its worker
```

- **The engine is the only client** of the store and of the forge. It
  answers the host tools a run calls, and commits the run's turns and
  result. The worker's one write is pushing a run's branch, over git.
- **The engine never reads a repository.** It works from what the forge
  connector reads; what a workspace holds, such as a repository's
  `AGENTS.md` or its checks, is for the run to find.
- **The agent acts on the world only through the worker:** a delivery,
  which the worker makes as a push, and host tools, which the worker
  relays to the engine. It never pushes, merges or writes to a system on
  its own authority.
- **Retrying a failed run is decided above the run,** by the engine,
  across attempts. A run answers once and is done.

## 3. What temper takes from smith

- **In the domains:** the engine's root links no smith crate; charters,
  transcripts and results are bytes to jig's core (jig's
  `domain/hosts.md`, 6.8). The worker's root composes `smith-host-domain`
  as its agent child (worker.md, section 3). `temper-engine-smith`, the
  translation of section 4, is step code over `smith-domain`,
  `smith-domain-run` and `smith-domain-session`.
- **In the protocol layers:** smith's codecs, its host half and its agent
  shell (`../protocol/README.md`, section 5).
- **In the worlds:** smith's agent domain over skein's fake LLM, its
  agent processes, and its codecs (section 9).
- **LLM accounts are jig's,** made from the client, on skein's accounts
  driver. A run's grants reach it through its worker and smith's host
  half: for each LLM provider it uses, a credential with its scheme, or
  a program source's name, whose login the host checks and the program
  reads (`../protocol/credentials.md`, section 3; jig's
  `protocol/llm.md`, section 6). The agent runs the loop itself, or
  through the program (smith's `protocol/programs.md`, section 2).

## 4. temper's charter

The core decides a run's charter in its own terms (jig's
`domain/engine.md`, 7.1); `temper-engine-smith` writes it as smith's
(smith's `domain/run.md`, 3.1) from the root's assignment, and the
engine's routing encodes it (`../protocol/llm.md`, section 5).

### 4.1 Instructions and brief

- **Instructions:** the charter's, from the deployment's configuration
  (jig's `protocol/configuration.md`, section 3): a chat's, a
  coordinator's, a reviewer's with a lens, a change's producer's; until
  an amendment gives a task its own (jig's `domain/tasks.md`, section
  7), whole, where an empty text leaves it none, not its charter's. The
  task's next run carries them; a run already live hears them as the
  amendment's words. A charter is a setting in the store, read from the
  revision the task runs under (jig's `domain/engine.md`, 17.3),
  temper's stock chat's and coding agent's held in the binary as their
  defaults (`../protocol/configuration.md`, 13.4).
- **The brief:** the sections jig's core gathers (jig's
  `domain/engine.md`, section 9) and the forge's (forge.md, section 2),
  each rendered as one of smith's titled sections, titled by its kind:
  `Task`, `Lineage`, `Dependencies`, `Plan`, `Earlier attempts`,
  `Calls`, `Waiting`, `Notes`, `Transcript tail`, and the forge's
  `Pull request`, `CI`, `Reviews` and, for each document the charter
  names, `Document` followed by its repository, path and heading
  (`../protocol/configuration.md`, 13.7). A section its gather could not fill
  says why: its read failed, timed out, or exceeded its limit. smith adds
  the sections about its own mechanics, and each repository's guide.
  The small profile admits 14,592 body bytes across at most 64 brief
  sections; the forge reviews budget is 4,608 bytes within that total;
  `standard`, which `temper init`'s box runs, gathers what its host part
  leaves the largest stock charter, its task section a whole spec
  (`../protocol/configuration.md`, section 7). Assignment and receiving
  fitting (4.3) may shorten those gathered sections to the charter and
  workspace ownership bound; the durable specification stays whole.

### 4.2 Workspace

The connectors name the repositories, where each starts, which may be
written and the branch a change goes to (jig's `domain/engine.md`, 7.1;
worker.md, 4.1); the worker prepares them and gives smith the workspace:
each repository's directory and name, whether it may be written, and for
a merge in progress the files in conflict (worker.md, section 5). Every
temper workspace is git repositories. A run with no repository, such as
a chat that names none, runs with no workspace (worker.md, 4.1).

### 4.3 Tools

- **smith's own families** by the charter's policy: inspecting,
  modifying and the shell in the workspace, and sub-agents, each as the
  charter's `inspect`, `modify`, `shell` and `agents` say; `wait`, always;
  `deliver`, for a finite change's contract (jig's `domain/hosts.md`,
  section 8; smith's `domain/run.md`, 8.4), never in an Open run (4.6).
  A run that cannot deliver cannot modify files: jig's charter
  translation grants `modify` only with `deliver` and the charter's
  permission (jig's `domain/hosts.md`, section 8). A chat's, a goal's, an advisor's and a reviewer's
  runs inspect alone, whatever their charter's `modify` (a rail, not an
  instruction). Stock charter 1 disables `modify`; repository edits,
  including small fixes, go through `change` (4.3.1). The stock coder
  has read, write, edit and remove: a file the change drops is removed
  once read.
- **Every host tool below,** declared in every charter, each with the
  charter's call timeout, but `read_task`, offered only with the task's read
  family, and those of the `settings` family, declared
  only where the run's charter names that family in its `tools` (jig's
  `domain/engine.md`, 7.1) and its task's authority has it: charter 3
  alone in the stock
  (`../protocol/configuration.md`, 13.4). The task's authority decides
  each call by its family (jig's `domain/authority.md`, sections 3 and
  8.5): a call beyond it is answered beyond authority, naming what it
  lacked, and the run may `propose` it instead. A refused tool family
  is named from the call's tool, and repeated findings for that family
  print once; the refusal grants no additional authority.
- **A coordinator's task reads.** `read_task {task, field?, from?}` returns a
  related task's specification by default, state, dependencies and spec revision,
  and its hold revision, reason and connector-supplied account. Retained ended
  direct delegates remain readable. `field=result` reads kept terminal reports,
  including descendants reached through a proven same-project requester chain
  of live or retained intermediate tasks (jig's `domain/engine.md`, 7.3).
  A cut answer prints `next: OFFSET`; pass that byte offset as `from` with the
  same field, and restart specification reads at zero if the spec revision changes.
  Task reads decide nothing and keep no named-call record. Escalations name
  their task and hold revision and give this recovery call. Read access does
  not grant messages or amendments to ended tasks, or the subscribe family.
- **The host part's budget.** smith refuses a run whose host part (its
  host tools, instructions, brief, and finish and deliver schemas) passes
  its bound, the room of the agent's smallest window (4.6), before any
  turn. The largest stock charter's non-brief host part is charter 1's
  with a Change contract and the control family's plan form of
  `change` (4.3.1); with the authorised brief, its task section a whole
  spec (`../protocol/configuration.md`, section 7), and 1,472 bytes for
  64 stock sections' titles and framing (16 title bytes and 7 framing
  bytes each), it leaves at least 361 bytes at `temper init`'s
  models; `temper-service`'s step tests hold each stock charter's
  figures. A gathered brief spends what the non-brief host part leaves. So a new
  host tool is offered only where a family and the charter gate it,
  never to every run, and a charter's room is measured before a tool
  joins it.
- **The brief fits the room its charter leaves.** smith counts a run's
  brief in its host part, beside the charter's offered tools, its
  instructions and the finish and deliver schemas. Before sending an
  assignment, the engine fits gathered sections to the configured
  `hub.charter_bytes` envelope for both the encoded charter, including
  framing, and its decoded ownership. Instructions, tools, contracts,
  models and run identity stay whole. The worker service also fits the
  brief after the workspace is prepared and before encoding the start,
  using the receiving run's limits and measured schema costs. It fits both
  rendered host room and the decoded charter ownership left after smith
  charges the prepared workspace against `run_bytes`, including resolved
  host paths and conflict paths. Smith supplies that remaining allowance
  through `charter_room`; temper keeps the incoming charter allowance too.
  `temper-engine-smith` owns
  the fitter; its typed start takes explicit receiving limits and fits
  the brief too. A brief already within each entrance's room is kept
  whole. Otherwise, fitting cuts bodies from the last
  section in the core's order, at UTF-8 boundaries, keeping each cut
  section's title and ending its body with its own line:
  `[cut N bytes; whole: READ]`. `N` counts original body bytes removed,
  excluding the added separator and cut line. `READ` is
  `read_job/read_checks` for `CI`, `read_pull/read_pull_files` for
  `Pull request`, `read_reviews/read_remarks` for `Reviews`, `read_docs`
  for `Home`, `recall` for `Notes`, `the task's plan` for `Plan`, and
  `the task's activity` for the other core sections; other titles use
  `read_docs or the task's activity`.
  When a title and its minimum cut line cannot fit, whole trailing
  sections are omitted. One line records them, in their original order:
  `[N sections omitted: TITLE, TITLE; whole: the task's activity]`.
  It follows the last retained body, or stands in an untitled section
  if none remains, only when it fits without cutting more retained
  text. If even that line cannot fit, it is left out, down to an empty
  brief at zero room. Fitting also keeps the charter within its admitted
  ownership allowance. No run is refused for its brief's size; if the
  non-brief host part or fixed declarations plus workspace alone exceed
  their receiving bound, smith's declaration refusal still applies.
- **Checked before a run meets it:** at start, and at temper's three
  entrances of a change of settings that touches a charter (the
  engine's `ChangeSettings` and `ProposeSettings` routes and a run's
  `propose_settings`), temper sums each charter's offered tools, its
  instructions and the fixed parts, and refuses one that leaves its
  brief less than 2,048 bytes, naming the charter and both figures
  (`../protocol/configuration.md`, section 7). A stored proposal's
  acceptance, a restore and a reset carry values that passed an
  entrance or the start, and the bound does not move while the engine
  runs. The sum is a table of each host family's declaration bytes and
  the charter's own text, where the domain reads it, not a call into
  smith; a step test in `temper-engine-smith` holds the table equal to
  smith's own measure. The bound is the engine's size profile's, the
  one its workers are configured from (51,717 in the small one, for a
  window of 177,523 tokens; in `standard`, the room of luna's window,
  4.6); an
  engine without that figure skips the start's check. A run smith
  refuses for its host part anyway is held with smith's reason, the
  bound and the figure, never a bare "invalid".
- **`repository`,** one of the project's repositories as `owner/name`:
  among its adopted repositories of any role, on forges not ended, the
  one named, or with no name the only one; with several and no name, or
  a name none has, refused, naming them. The root resolves it to the
  repository's resource as it builds a change's delegate (4.3.1),
  before it decides the call, so that the call's authority is checked
  on that resource (`root.md`, 5.2) and the run never names a number.
- **The read's own members,** beside it: `read_branch` `branch`;
  `read_branches` `prefix`; `read_pull` `number`; `read_pull_for` `head`
  and `base`, branch names; `read_pull_files` `number`, `head` and
  `page`; `read_reviews` `number` and `page`; `read_remarks` `number`,
  `review` and `page`; `read_statuses` `commit` and `page`;
  `read_checks` `commit`; `read_compare` `before` and `after`;
  `read_file` `head`, `path` and `max_bytes`; `read_job` `head`, `run`,
  `job`, `attempt` and `max_bytes`, exactly as `read_checks` and
  `read_statuses` name each Actions job: the job attempt skein resolved
  from the check's link (its run, the forge's own job id, its attempt:
  skein's `forge.md`, 9), never the link's index of the job in its run,
  which the answer may show beside it but no read takes; `read_items` `since`, `page` and
  `item_kind` (`issue` or `pull`); `read_item` `number` and `after`;
  `read_protection` `branch`; `read_settings` none;
  `read_collaborators` `page`; `read_permission` `user`, a forge user's
  id. A commit or a head is its id, 40 or 64 lowercase hexadecimal
  digits; `page` is from 1, and 1 when left out; `since` is a time in
  nanoseconds, 0 when left out; `after` a comment's id, 0 when left
  out; `max_bytes`, the read's limit when left out.

- **A compare is whole, with no pages.** Its files and commits each fit
  `forge.client.rows`, and its decoded answer fits
  `forge.client.answer_bytes` (skein's `forge-client.md`, 2.3, and
  `forge.md`, 5.2 and 5.3). A `read_compare` size refusal names both
  limits without claiming which was exceeded. It directs a run with
  an existing pull to `read_pull` for its head, then `read_pull_files`
  at that head, from page 1 while more follows. Pull-file sides must
  also fit whole. Otherwise, or when that page is too large, the run
  lists directories with `read_file` at both endpoints and reads
  selected files at each within `max_bytes`. The refusal stays marked
  an error and fits the host answer's byte bound. A successful comparison
  whose rendered form exceeds that bound returns measured-size feedback
  with the same recovery instead of losing its body at the worker.
- **File reads fit the host answer's bound.** The adapter measures rendered
  file, directory and pull-file answers, including metadata and byte escaping.
  A file keeps the largest UTF-8 prefix that fits with its metadata,
  recovery feedback when room permits, and a visible `[file truncated]`
  marker, counting escaping (up to four output
  bytes per input byte). An explicit `max_bytes` still bounds the raw forge
  prefix; host fitting may cut it further. A cut is never the whole file,
  and there is no offset. A host-fitted prefix also names the measured
  rendered size and effective body bound, and gives configuration recovery
  when room permits; tiny bounds retain the cut marker alone. If metadata
  leaves no room for content, bounded error feedback directs a shorter path
  or raising the limiting core or agent allowance;
  lowering `max_bytes` cannot fit metadata. An oversized directory answer
  is an error naming its rendered byte size and numeric host bound. An oversized
  pull-file page retains a changed-path inventory as room permits, marks
  content omitted separately from absent sides, marks omitted paths, and
  keeps whether more pages follow. Omitted paths are recovered through
  directory reads at both commits. It directs selected `read_file` calls
  at the head for after and the comparison's base commit for before;
  the base commit must be resolved separately. Advancing pages alone cannot
  recover the oversized page. A forge size refusal, whose measured size
  is unknown, names that uncertainty, the forge bounds and the numeric host
  bound with the same tool-specific recovery. At a host bound too small
  for the guidance, the refusal itself is cut to fit.
- **Reads and writes:** the forge's reads, `recall`, `settings` and `read_docs` are reads; every
  other host tool is a write, which smith runs alone (smith's
  `domain/run.md`, 5.2).
- **`decided`,** of the control family, offered as every host tool is, a
  goal's alone by what it answers: `{ question, options, took, why, kind
  }`, `options` 2 to 6 words of the agent's, `took` an index into them,
  `kind` one of `implementation`, `retry`, `wording`, `dependency` and
  `other`; `question` at most 400 bytes, each option 120, `why` 400, a
  call past those refused at decoding, saying so. The forge judges it
  (`forge.md`, 5.9): "Decided: carry on with TOOK." with a `Decided` row
  saved, or "Ask: your person is here or this kind waits for them. Ask
  with message, as a question." Named and decided once, as `change`. Funds,
  sign-ins, machines, a plan's go-ahead and anything hard to reverse are
  kind `other`, by the charter's words (`../protocol/configuration.md`,
  13.4). A goal that overturned words reach (`../client/README.md`, 3.5)
  asks it with `message`, as a question, and waits. Nothing in the engine
  answers a question for it, and one that skips the tool waits as ever.
- **Declared, decoded and rendered in one place,** `temper-engine-smith`:
  each tool's JSON Schema, description and effect, written once
  (`delegate`'s and `amend`'s schemas name every field of the spec and
  the authority, so that a run writes them from its tools alone, and
  both take a priority as `{priority}`, where jig's own `delegate` takes
  a bare number, so that a run learns one shape); a
  call's input decoded into the root's `Tool`, or answered at once as an
  error naming what is wrong, deciding nothing; an answer rendered as
  smith's text, marked an error or not (`../protocol/llm.md`, section 4).
- **Calls are named and decided once** (jig's `domain/engine.md`, 7.3):
  the engine answers a call it has seen before from its record, and smith
  asks again under the same name after busy or a lost answer (smith's
  `domain/run.md`, 5.2).
- **An answer fits what the agent takes** (`domain/worker.md`, 4.3):
  forge-read rendering uses the minimum of `core.call_answer_bytes` less
  the tool name and the checked receiving `agent.limits.bodies.host_reply_text`.
  The latter comes from the local agent's `agent.json`
  `profile.declared.tool_payload`. Raising `core.call_answer_bytes` alone
  cannot overcome a smaller agent allowance. Engine-only processes without
  local agent metadata retain their configured core allowance.
  A call past the worker's input bound, `core.call_answer_bytes`, is
  answered `call too large` under its name, and the run goes on (jig's
  worker link). An answer past the agent's `host_reply_bytes` (smith's,
  its channel's `host_reply_text`, the agent's `tool_payload`) reaches
  the run as smith's `TooLarge`, which says the answer was too large
  and not shown, and the run goes on. The engine keeps the answer it
  decided, so a call asked again gets the same reply.
- **A call names what it saw** (jig's `domain/engine.md`, 7.3): the
  revisions above are those its brief, its messages and its earlier
  answers showed it. One whose object has moved since is answered moved,
  with what it is now, and the run decides again on that.
- **Not the engine's:** `wait`, `finish` and `deliver` (jig's
  `domain/engine.md`, 7.3). `deliver` is a push (section 6).
- **Jobs are smith's** (smith's `domain/tools.md`), not `read_job`'s,
  which reads a CI job's log. A `shell` command that runs past its
  yield goes on as a job in the agent's process, under the charter's
  `shell` (from `policy.shell`); the run reads or stops it with smith's
  `job`. Neither the engine nor the worker routes or answers a job.
  `temper-engine-smith`'s `turn` takes a turn request alone, and
  answers any other request, a job's included, as not a turn
  (`Problem::Type`).

The refusal says `charter N leaves its brief R bytes of its host part's B;
it needs at least 2,048`, with thousands' commas in the figures and room
zero when the non-brief part consumes the bound. A party's direct change or
proposal is refused as a judged charter item, naming its item index; a run's
proposal receives a host error with these words and its run continues. On
restart, a stored current charter that fails the check ends startup with
`temper serve: restart refused: ` and the same diagnostic (2026-11-04).

#### 4.3.1 A change, in the run's words

The stock chat first reads the code a repository change touches with the
forge's `read_branch` for its head, then `read_file`. It calls `change`
with the person's request and only what it read, never describing code
it has not read. Repository changes go through `change`, never through
writes to the chat's workspace. After delegation the chat says only that
work started until the delegate reports. When a change reports successful
`NoChange`, the chat tells the person that nothing changed and explains why
using the delegate's conclusion, without claiming landing, implementation
or tests. For a fix, if the named fault is absent, it changes nothing and
says what the code has. The stock coder likewise writes
nothing and finishes with the successful `no_change` verdict saying what the code has when the
named fault is absent, never adding a feature in a fix's place. Where
the change's words describe the code otherwise than it is, the coder
builds what they ask to add in the code there is, under its names,
finishing with `no_change` only when it cannot be built there.
An error that prevents production is a failure and keeps its recovery hold. When a change
lands, the chat names the pull request and merge commit from its result's
`Landed` line, and describes what changed only as the result or pull request says,
never by restating the request as done.

A chat cannot know what a change's `delegate` needs: the procedure's
numbers, the repository's resource, its landing branch, the contract's
kind and the grants a change makes. `change` takes the words alone, and
the root builds the one delegate from what it knows, then decides it as
any `delegate` (jig's authority, named and decided once):

- **the repository:** among the project's adopted repositories it may
  write, on forges not ended, the one named, or with no name the only
  one; with several and no name, or a name none has, refused, naming
  them. Never the home by default: a project's home may be another
  forge's (the demo's);
- **the delegate:** the change procedure (forge.md, 8.1) with the words,
  the repository's resource and its landing branch: the one the call
  names (below), or with none the repository's default branch; a
  change's contract, its words bounded by the deployment's
  `tasks.result_bytes`;
- **its authority, the run's own narrowed:** the run's grants of the
  forge's reads (kind 1, forge.md, section 2) and of the kinds a change
  makes (forge.md, section 6: a push, open, merge, edit, update, close
  or reopen a pull request), on the change's repository, which every
  agent of the change holds, so that its reviewer and its repairs read
  the pull request, its files and CI's logs; the change's charters, its
  producer's and its reviews' (forge.md, 8.9), as the only kinds it may
  delegate, the merge kind, job 3, where the run's authority delegates
  to it (forge.md, 8.4), and a person's role where a landing rule
  covering the branch names a person gate; at most
  `forge.change_policy.tasks` tasks (`../protocol/configuration.md`,
  section 7: 6 in `small`, a producer, a review, and two repairs each
  followed by its review; 80 in `standard`), one fewer than the run has left, and a depth of at most 2; no notes; half of
  what the run's budget has left, so that it can still read the result
  and answer, refused when that is no more than a run's minimum. That
  half is a default, open to tuning. Several changes in one call share
  that half equally.

A `delegate` refused for an executor or a grant says, too, that a
repository change is `change`'s.

**What a plan decides.** Beside the words and the repository, a call
names what a plan decides of each change, and the root still builds the
delegate from what it knows:

```
{"words": "...", "repository": "owner/name",
 "name": "parser", "landing": "goal", "after": [41, "lexer"]}
{"changes": [{"name": "lexer", "words": "...", "landing": "goal"},
             {"name": "parser", "words": "...", "landing": "goal", "after": ["lexer"]}]}
{"words": "...", "branch": "mine"}
```

- **`landing`:** `"goal"`, the requester's own goal branch,
  `temper/<root>/g<task>` in the change's repository (forge.md, section
  11), which the requester holds; or a branch's name in the repository;
  left out, the repository's default branch. `"goal"` from a run whose
  task holds no goal branch in that repository is refused, saying so; a
  name that is not a branch's (an empty segment) is refused, naming it;
  a branch the repository lacks holds the change at its opening, the
  forge refusing its pull request (forge.md, 8.2).
- **`branch`:** `"mine"`, its requester's held branch: the goal branch
  the run's task holds in the change's repository (forge.md, 11.1),
  which the change lands whole as it stands, with no producer (forge.md,
  8.1); left out, the change's own branch, which its producer pushes.
  `"mine"` from a run whose task holds no goal branch in that repository
  is refused, saying so; with `"landing": "goal"`, a branch into
  itself, refused, saying so; while another live change of the task is
  on that branch, refused, naming its task; any other value refused,
  naming it.
- **`name`:** a name for the change within the call, at most 64 bytes,
  unique in it, which another change of the same call names in its
  `after`; it names nothing once the call is decided.
- **`after`:** the change's dependencies (jig's `domain/tasks.md`,
  section 4): a task number, one of the run's task's live delegates, or
  the `name` of an earlier change in the same call. The change waits on
  them as any task does, so it starts once each has ended done; a name
  the call does not give before it, or a task jig refuses, refuses the
  call, naming it.
- **`merge`:** a catch-up (forge.md, section 10): the commit to take, as
  40 or 64 hexadecimal digits, which the merge job merges into a branch
  made at the landing branch's tip, as the change's producer in place of
  an agent; the words say what the commit brings and why. Refused,
  naming the setting, where the change's project names no `merge_job`
  (`../protocol/configuration.md`, section 3), and naming the kind,
  where the run's authority does not delegate to the merge kind; a
  commit in another form is refused, naming it. Whether the repository
  has the commit is the job's to find: one the forge cannot fetch
  fails the job, and holds the change.
- **`changes`:** several changes in one call, each an object of the
  members above but `changes`, made as one batch (jig's
  `domain/tasks.md`, section 4): all of them or none, the call's answer
  naming each change's task in the call's order. A call names either
  `words` or `changes`, never both; `changes` holds at least one.
- **`producer`:** the number of the charter the change's producing
  task, its repairs, its resolutions and its checks' gates run under
  (forge.md, 8.1); left out, the forge's change charter, kind 3's number
  1 (`../protocol/configuration.md`, 13.2). A charter the newest
  settings do not offer (jig's `domain/engine.md`, 17.3), or one the
  run's authority may not delegate to (jig's `domain/authority.md`,
  section 3), refuses the call, naming it, as a `delegate` naming it
  would be refused.
- **`llm`:** an LLM choice for each of the change's agents' roles, each
  in jig's form, its models and efforts by their configured names and no
  account (jig's `protocol/configuration.md`, 3.1):

  ```
  {"words": "...", "landing": "goal", "producer": 6,
   "llm": {"producer": {"main": {"model": "gpt-6.1-sol", "effort": "high"}},
           "review": {"main": {"model": "gpt-6.1-sol", "effort": "medium"}}}}
  ```

  `producer` for its producing task, its repairs, its resolutions and
  its checks' gates; `review` for its agents' gates. The root checks
  each as jig checks a batch's (jig's `domain/tasks.md`, section 4),
  against the charter its role runs under at the newest revision and
  the run's authority: a model not configured or named twice, an effort
  its model does not declare, a model, main or sub-agent, outside that
  charter's models or the authority's refuses the call, naming the
  model and why. A role left out takes its charter's default as each of
  its agents is made (jig's `domain/tasks.md`, section 3).
- **`repairs`:** the red landing the change repairs
  (`after-landing.md`, section 7), by its merge commit's full
  hexadecimal digits, as the red's news names it: a landing of temper's
  in the change's repository, on the branch the change lands into, red
  in some tier as the root decides the delegate. A value that is not a
  commit's digits is refused, naming it; a commit no live landing of
  temper's there has, or one red in no tier, refused, naming the commit
  and why. The change keeps it, and its own landing clears the red one
  when a later tier run passes the failures blamed on it
  (`after-landing.md`, 5.3); left out, the change repairs nothing.
- **Who may set them:** the members past `words` and `repository` are a
  plan's, and only a run whose task's authority holds jig's control
  family (amend and cancel, with `release`: jig's `domain/authority.md`,
  section 3) may set them: a plan's maker amends, cancels and releases
  what it makes. The stock policy's families give it
  (`../protocol/configuration.md`, 13.4), so a chat or a goal under the
  stock sets them. `change`'s declaration offers them only where the run's
  authority holds that family (4.3, the host part's budget), and the root
  refuses a call naming one from a run without it, beyond authority,
  naming the family, as it decides the delegate. What each member sets is
  checked as the rest of the batch is: its landing branch by the change's
  grants on the repository, its dependencies by jig's batch.
- **`questions`:** how the change takes its agents' questions (forge.md,
  8.11): `"none"`, a change that asks no one (8.2); `"foreman"`, each
  question settled first by a foreman of its own; `"up"`, each passed
  straight to the requester, a designer's change's way; left out, the
  project's setting (`../protocol/configuration.md`, 13.2, kind 6), whose
  default is `"none"`. Any other value is refused, naming it. A change
  taking `"foreman"` is given one more task per question of its bound
  (`forge.change_policy.questions`, `../protocol/configuration.md`,
  section 7), beside the tasks above, for its foremen.
- **Members to come** join each change's object, each optional and
  checked by the root as it decides that change's delegate, leaving
  these as they are.

#### 4.3.2 A goal, in the run's words

A chat cannot know what a goal's `delegate` needs either: its own
authority's tool families, the grants' vocabulary on the forge's
resources, the delegation's kinds and its funding, none of which a run
sees. `goal` takes the words alone, whether the work is to be agreed
first (`agree`, 8.2), and whether the goal is an advisor,
and the root builds the one delegate from the run's own, then decides
it as any `delegate`:

- **the delegate:** under the run's charter (a chat's, charter 1),
  whose instructions stay its instructions, the caller's words first in
  its brief, then a blank line and the root's own line that it is a goal,
  the work its person asked for, as an advisor's follows its caller's
  words with its own line. The task's title, its spec's first line, is
  the work. The root's line tells it to read the
  instructions meant for a goal as its own and those meant for a chat
  as not (charter 1's instructions name each). For an agreed goal
  (8.2), the root's lines are followed by its person's last words to the
  chat, whole, under their own heading, so that a request the chat cut short
  still reaches the goal entire (cut only past the brief's room, and
  then marked with the bytes cut). The goal takes its label and flags
  from the goal template (template 2): label 2, listed, no conversation,
  kept, and ended by its result. It is tracked
  at the run's own priority, or last among the project's goals where the
  run is untracked: one past the largest tracked priority in the
  project, saturating, or 0 with none (the lower first, jig's
  `domain/tasks.md`, section 3), with a report's contract bounded by
  the deployment's largest result, jig's `tasks.result_bytes` (the box's
  agent's message, `../protocol/configuration.md`, 13.4), whatever the run's own contract's
  bound (a chat's bound reaches neither its brief nor its replies,
  since its requester ends it, jig's `domain/tasks.md`, 5.8), since jig
  refuses a batch whose contract is above it (4.4): the work a person
  asked for, which plans, asks its requester with `message` where its
  words say to (8.2), calls `change`
  for each change, and reports. The contract bounds the result, not
  the work: asking first is the instructions', and the rail is
  authority's;
- **its authority, the run's own narrowed,** never wider in any part
  (jig's `domain/authority.md`, sections 3 and 5): the run's models and
  deadline; its tool families (delegation, messages, notes and reads, so
  that it may call `change`, ask, write its report and read the forge);
  its grants of the kinds
  a change makes and its reads, so that `change` can narrow them again
  (4.3.1); its delegation kinds, with the tasks its subtree may still
  make, as jig counts them, less one for the goal itself, and a depth
  one less, refused when either is spent; notes of the goal's own scope
  alone, where the run's own authority holds that scope, so that it
  writes its report (8.3); half of what its
  budget has left, refused when that is no more than a run's minimum, as
  `change`'s. A change the goal then makes has half of the goal's,
  so at most a quarter of the chat's: defaults, open to tuning;
- **an advisor,** with `advisor` set, its report bounded as a goal's,
  its brief opening with the caller's words, then a blank line and the
  root's own line that it is an advisor, which reads and reports and
  changes nothing, so that its task's title is the work, the line
  counted with the words within the spec's bytes (a request is refused
  as too long with the line included, never cut to make room), so that
  it knows what it is without a convention in those words: unlabelled,
  with plain flags (not listed, no conversation, kept, ended by its
  result), and untracked, so that no goal of its is projected on the
  forge; reads alone, no grant of a forge effect, no delegation (no
  kinds, tasks or depth) and no notes, so that it reads and reports and
  can change nothing beyond its own workspace, which it has no grant to
  push (a rail, not an instruction).

The root builds it where it builds `change`'s delegate, in the engine's
route of host tools, and decides it as any `delegate`; the smith
translation only decodes the call.

The goal's own run takes `goal` too: an advisor it asks for, before it
asks its requester, is its survey before a plan. An advisor is all it
takes, and only it takes one: a goal is the work its person asked for,
and a goal under it would ask the goal, not the person, and be answered
by it (8.2); a goal is where the work is planned, so an advisor asked
for elsewhere leaves the work to be planned in a chat's reply. So one
fact of the calling run, whether the root built it as a goal, decides
both. For a tracked goal whose spec marks the work to be agreed (8.2),
while it has not asked its requester or a requester question remains
unanswered, the root first refuses `goal` with `agree` or without
`advisor` with "this work is to be agreed: ask your requester your plan
with message, as a question, and wait". Once the requester has answered and no requester
question remains pending, the ordinary refusal below points to `change`.
An advisor without `agree` remains admitted.
Otherwise the root refuses a goal's `goal` without `advisor` (a goal starts
an advisor alone: set `advisor`, or make the change with `change`), and
any other run's `goal` with `advisor` set (to change a repository, call
`change`; a goal is for work they want to agree first or a survey, and
its goal asks for an advisor), and any `goal` with both `agree` and
`advisor` set (an advisor agrees nothing: set agree or advisor, not
both), and the run goes on, as any refused host call does. A person who
wants a survey alone gets a goal that asks its advisor and reports,
changing nothing.

### 4.4 Result contracts

A task's result contract (jig's `domain/tasks.md`, section 3) becomes
smith's (smith's `domain/run.md`, 7.1):

| temper's | In smith |
|---|---|
| a report: words, bounded | a report, and a failure, within the same bound |
| a verdict from a closed list, each with the fields it requires and the follow-up tasks it may propose | a verdict for each choice, named by its number, with its fields and its words' bound; follow-ups as items, of the kinds of task the verdict may propose; a failure of at most 1,024 bytes |
| a connector's result: a change landed or a successful no-change conclusion | a change, requiring the fields `title` and `body`, each within the contract's bound, its checks required; a `no_change` verdict with bounded text and no fields or items; a failure within the same bound |
| an approval | a failure only, of at most 1,024 bytes: approvals are people's |

The `no_change` verdict becomes jig's successful `NoChange` result. It
retains the explanation, supplies no resource or landing, and ends the
change without a hold or escalation. The requester receives that result.

A change's result names the resource the worker's push and the forge
settled, never one from the LLM's fields. Its explicit `title` and `body`
are preserved independently through jig's typed result and durable codec, each
within the contract's words bound. The change procedure uses the title for
the pull request, without deriving it from the production brief.
The core judges a result again as it commits it, against the contract
and the state as it is now (jig's `domain/tasks.md`, 5.6).

### 4.5 Conventions

Checks at `.temper/pre-pr`, guides at `AGENTS.md` (smith's
`domain/run.md`, 3.1 and 8.1).

### 4.6 Budget, models, waiting

- **Budget:** the run's turns, spend and time, with its reserve of turns
  and time for finishing (smith's `domain/run.md`, 9.1), from its
  charter in configuration, at the task's revision; its spend is what
  its task has left, capped per run, reserved at its claim, in the
  deployment's unit (jig's `domain/hosts.md`, section 8).
- **Models:** the main session's and those its sub-agents may use, each
  with its endpoint, window, output and prices (`../protocol/llm.md`,
  section 3): the task's LLM choice among those its charter allows,
  without one the charter's first whose LLM provider has an account
  usable for the payer (jig's `domain/tasks.md`, section 3),
  and the account the run's claim chose for each LLM provider (jig's
  `domain/accounts.md`, 4.3). Their credentials, or a program source's
  name, come to the agent as grants through the worker
  (`../protocol/credentials.md`, section 3). A run on a program has
  smith's sub-agents, inspect only, each a program session of its own
  (smith's `protocol/programs.md`, 5.10).
  The stock demo is temper's fake model, declared with `gpt-6-luna`'s
  window, 1,048,576 tokens, and an output of 8,192, so it leaves every
  charter the room luna does. At smith's 4/5 threshold, after two
  outputs and less smith's own part, half of the remaining room gives
  the host part and a quarter the offer (smith's `protocol/limits.md`,
  3.9): ⌊(⌊1,048,576 × 4/5⌋ − 2 × 8,192 − own) / 2⌋, about 400,000
  bytes; `temper-service`'s step tests hold the figure at init's agent.
  Every stock charter retains that rendered room, which holds the stock
  tools and the gathered brief with a whole spec. Delivery also enforces
  the separate combined ownership bound, shortening the brief when needed
  (4.3). A message offered a run is at
  most an offer and the agent's tool payload, which the model's output
  bounds (`../protocol/configuration.md`, 13.8).
- **Waiting and resuming:** the waiting time is the charter's; a
  coordinator's is zero. A chat resumes its transcript, and a
  coordinator starts fresh from its brief (jig's `domain/engine.md`,
  7.4). A goal under stock charter 1 also resumes. Its granted `wait` may
  park in Finite wind-down, including the eighth completion after seven
  ordinary completions. The pending plan question and its answer route remain
  live while parked. The answer starts a fresh bounded activation with eight
  turns and one reserved turn; its waking prompt supersedes prior exhaustion.
  The task's monetary spending remains cumulative and each activation is
  funded only from what remains.
- **Its mode is its task's ending** (jig's `domain/tasks.md`, 5.8;
  `domain/hosts.md`, 8.1), the assignment's `ended_by`, whatever the
  charter: a task its result ends runs smith's `Finite` mode, and one
  its requester ends runs `Open`, granted the wait and neither `finish`
  nor `deliver`. temper's chats are Open (`../protocol/configuration.md`,
  section 2): ordinary text listens for the person's next words. Idle or
  wind-down parks the run, committing its reply with no result and no try
  spent. Close carries the grace's finish request live or in Start; its
  admission puts the Open run into wind-down, so its next text parks.
  Cancellation after the grace keeps its existing meaning; only the
  person's Close, or a cancel, ends the chat. Goals and every delegate
  run Finite and end by their results.

## 5. Messages

What a task hears while its run is live reaches smith as messages, as
jig's `domain/hosts.md`, section 9 says: each named, with its sender's
label, its kind and its words whole, the start carrying those waiting
in the task's inbox. Words read `2: …`; a question reads `2 asks
(question 4): …`, its number the one `message`'s answer form takes;
an answer reads `1 answers question 4: …`. A person's inbox answer
notifies the asked task with `Person answered task 2's question 4 in their inbox; task 2 has the answer and does the work:\n…` when task 2 asked it.
A question the receiving task asked itself keeps `Person answered question 4 in their inbox:\n…`,
including the answer words within 256 bytes, cut at a UTF-8 boundary and
marked when cut; an empty stored notice keeps the line without words.
So a goal hears its person's answer to question 4 by the number it asked,
and a chat, which answers no question waiting in its person's inbox (8.2), hears that
question 4 was answered, with the person's words and the asker that owns the work. Proposals and
escalations waiting for the task's decision are relayed the same way, each naming what `decide` or
`decide_escalation` takes.

## 6. Delivery is a push

smith's `deliver` is the worker's push (worker.md, section 5): the
worker commits exactly the tree the run checked, with the result's
`title` and `body`, and pushes it as a fast-forward, never forced, to the
branch the assignment names; a merge in progress is committed as the
merge, with both parents. Its outcomes, in smith's terms (smith's
`domain/run.md`, 8.2):

| The push | smith's delivery |
|---|---|
| landed in every repository with a change, or already landed by this hold's last push | delivered, naming each head |
| nothing to push | nothing |
| a branch moved, as a fast-forward cannot take it or its head is not the one expected | stale: the run ends, and the engine decides afresh |
| a conflicted file still holds a marker | refused, naming the file |
| refused by the forge, unreachable, timed out, broken, too large, missing the repository, a branch or a commit | failed, with the reason and the last 512 bytes of git's diagnostic output |

A push that lands is the result, even if the run was winding down. The
answer carries what the run left on the forge (worker.md, 4.2), and the
forge's change procedure goes on from it (forge.md, section 8).

## 7. Turns and transcripts

Turns are smith's bytes (smith's `domain/session.md`, section 3), which
the worker relays and the engine keeps opaque, by conversation and
window (jig's `domain/engine.md`, 7.2; jig's `domain/hosts.md`, 6.4).
temper's client decodes them as jig's client does (jig's
`client/domain/client.md`, section 10). A run that resumes is given its
transcript's current window and the calls committed after its last turn;
past the resume limit, or when smith refuses the transcript, the next
run starts fresh, its brief carrying the tail when it is readable. If
the saved window is unread, the first load spends a transient try; its
retry starts fresh without the tail, keeping the person's pending words
(jig's `domain/engine.md`, 7.2).

## 8. The agents temper runs

| Agent | Messages in | Tools its authority gives | Ends when |
|---|---|---|---|
| producing a change (forge.md, 8.2) | none; where its change relays questions, the answers to its own (forge.md, 8.11) | the workspace's, from the landing branch or saved work, or started early with its dependencies' pushed heads merged in (forge.md, 8.11); `deliver` mid-run, as it goes; `finish` with a change; `message`, to ask its change, where it relays questions | its change passes its checks and is pushed |
| a repair (forge.md, 8.4) | as producing | the workspace's, from the change's branch, its brief carrying why; `finish` with a change; `message`, as producing | its new head passes its checks and is pushed |
| resolving a conflict (forge.md, 8.4) | as producing | the workspace's, from a merge in progress; `finish` with a change; `message`, as producing | the merge is resolved, checked and pushed |
| a review (forge.md, 8.3), a triage | none | reads; `finish` with a verdict | its verdict meets its contract |
| a foreman (forge.md, 8.11) | none | reads: the workspace's inspection, from the landing branch's tip, and the forge's; `finish` with a verdict | its verdict meets its contract |
| a chat | its person's words; its delegates' results and questions, and its person's answers to those questions (8.2) | most; `deliver` mid-run; `message`; `wait` | its person closes it |
| a coordinator | questions, results, overlapping landings, the answers to its own questions (8.2); proposals and escalations to decide | `delegate`, `amend`, `cancel`, `release`, `decide`, `decide_escalation`, `propose`, `subscribe_forge`, `note`, `message`; `wait` | its goal's report is written |
| the configuring agent (8.1) | its person's words; its proposals' outcomes: accepted, rejected with a reason, withdrawn as stale | `settings`, `propose_settings`, the forge's reads, `read_docs`; `wait` | its person closes it |

The change's agents run under the charters the change was made with,
kept on it whatever later changes of settings make them
(`../protocol/configuration.md`, 13.2): its producing, repairs and
resolutions under its producer's charter, the one its `change` named
(4.3.1) or else the forge's change charter, a coding agent's; its reviews
under the forge's review charter, a reviewer's, whose instructions are
to judge the change through its brief and the forge's reads and give a
verdict, or fail saying why it cannot (forge.md, 8.3), and whose tools
are the workspace's inspection and the forge's reads, never
modification or the shell: a reviewer changes nothing. Its foremen run
under the forge's foreman charter, whose instructions are to settle a
question from the change's words, the design in its checkout and the
change's pushed head, and give a verdict of answer, finish or escalate
(forge.md, 8.11), with a reviewer's tools: a foreman changes nothing.

The change's agents ask no one, and take no questions, answers or words
(8.2), unless the change relays their questions (forge.md, 8.11): then
its producing, its repairs and its resolutions ask the change, and hear
its answers. Its reviews and its foremen never ask.

A refused forge read keeps its refusing layer and reason. Connector authority
findings reach the agent as `ToolDenied`, with the findings intact. The root
also emits volatile operator facts for read refusals and call admission
refusals: task, attempt, tool, layer and reason, with authority findings or
the forge's error where present. The service routes these facts to its
operator notes and `temper serve` prints them on stderr. Call-record and
archive-read room pressure remains busy, retried by the host; observing it
changes no admission or grant. These bounded diagnostics are not store rows
or client wire fields, and overflow drops a fact and increments the root's
lost-fact count. Claude built-in Read refusals instead retain the permission
message in the recorded tool result (smith's `protocol/programs.md`, 5.4).
Forge file, directory, pull-file and compare replies fit the effective
core and receiving-agent body allowance (section 4.3), retaining prefixes
or actionable size feedback. A forge read still exceeding the core settled
answer bound or `root.journal.transcript_bytes` returns a bounded error
naming the exceeded limit and measured bytes, including the tool name. Its refusal fact names
`host-render` and that limit. An oversized comparison points to `read_pull`
for the head and `read_pull_files` pages; an oversized pull-file page points
to directory and selected-file `read_file` calls at both endpoints with
`max_bytes`. Forge-client admission busy still uses the host's bounded
backoff.
A reviewer fails when it cannot judge the change.


### 8.1 The configuring agent

A chat from template 3, run by charter 3, both stock and both settings
like any other (`../protocol/configuration.md`, 13.4), which turns how
a person says they work into one proposal of settings: what temper's
engine must do becomes settings an owner accepts, and what only its
agents must do stays text its runs read.

- **Its input** is the conversation: the person's words, a text they
  paste, or a document in an adopted repository they point to by its
  path, which it reads with `read_file`. It reads the settings first,
  with `settings`: the inventory, then each setting the description
  touches, whole. The authorized inventory also names the caller project's
  readable adopted repositories, each with the canonical host, owner and name
  from its retained repository record. Other projects and ended forges are
  excluded. These identities are read context, not settings or credentials;
  setting, revision and history reads carry no repository context. A charter
  document copies that host unchanged, including its port, and OWNER/NAME.
  This lets a configuring chat name documents without a workspace or ctl.
  Settings pages use `core.call_answer_bytes` less the tool
  name, reserving escaped framing and continuation markers. A long value
  fills the available page even after its heading; `next: OFFSET` counts
  original UTF-8 text bytes for exact reconstruction. This allowance is
  separate from `read_docs`' fixed bound (`../protocol/documentation.md`, 5).
- **temper's own way:** asked to set a project up the way temper builds
  itself ("Start from it", `../client/README.md`, 3.5), it reads the stock
  page `how-temper-builds-temper` with `read_docs`
  (`../protocol/documentation.md`, 2.2), the same page the person reads
  beside the page, and proposes its settings sized for the deployment: the
  slots its workers have, the models its accounts reach. What the deployment
  cannot do yet it says, and leaves out; workers stay the deployment's. Its
  "How PROJECT works" keeps a column of temper's own values beside the
  project's, so that each difference is a sentence the person can change:
  its words, not a view of the client's.
- **Its instructions** are held in the binary, in the `stock` module's
  `configuring.txt`, and say, in the person's register:
  - the test for each sentence of a description: would temper's engine
    act differently without it? A sentence that changes what temper
    wakes, schedules, permits, pays for, lands or holds is structure,
    and becomes settings; one that only changes how an agent behaves
    is a rule, and stays text that runs read;
  - where a rule goes: one the person wrote in a document of an adopted
    repository stays there, and the charters of the roles it concerns
    name that document (`../protocol/configuration.md`, 13.7), so that
    a change landed there reaches their next runs; one typed in the
    chat, with no document behind it, goes into those charters'
    instructions, in the person's words;
  - to ask until it can propose, one question at a time, and to offer
    a rule as structure where temper's settings can say it, saying
    what each guarantees: "I can make each family's changes wait for
    its designer, as dependencies temper holds; 'push early' stays a
    rule your producers follow". The person chooses what is
    guaranteed; a rule is never made structure without their choice;
  - to assemble settings past a few KB in a separate goal-scoped jig note,
    described `work: settings`, whose body is a whole settings file
    `{"items":[...]}`. Bounded `note` writes use `first` and `remove`,
    and `join` to continue a line longer than one call. This note is separate
    from "How PROJECT works" (8.3); it applies no settings;
  - to propose once, with `propose_settings`, against the newest
    revision it read, with a reason whose first line is the
    proposal's title and whose next lines say, under "temper
    enforces", each change of structure in the person's words, under
    "your agents follow", each rule and where it comes from, and what
    the change does to running work;
  - to say what temper's settings cannot say, naming it and offering
    the nearest setting there is, never proposing a stand-in as if it
    were the thing asked; and what needs a restart (the document's
    members: a provider, a forge, a listener, a worker, a limit),
    never proposing it;
  - to explain from the settings and the documentation (`read_docs`),
    naming its pages, never their sections or offsets: the concepts
    `settings`, `revision` (moving running work and restoring a version
    among it), `proposal`, `charter`, `template`, `policy` and
    `landing-rule`, each with what it cannot do and what needs a
    restart; `settings-fields`, each setting's fields, bounds and
    defaults; `configuration` and `configuration-engine`, the
    document's members, and `limits` and `limits-engine`, the limits
    it sets; and the recipes. What happens when work starts, which role runs where, why
    a task's run had what it had, given the task's `settings:` line the
    person reads to it; and, asked, the settings described back as a
    workflow: who does what, under which role, and what temper holds
    them to;
  - on its proposal's outcome, heard in its inbox (jig's
    `domain/engine.md`, 17.6): accepted, to say so in one line with the
    revision it made; rejected, to take the reason as the person's
    words and revise; withdrawn as stale, to read the settings again
    and propose afresh, saying what changed.
- **Its tools** are the reads and the proposal a person may make,
  through the same role and proposal checks: charter 3's `tools` name
  the settings and read families (`../protocol/configuration.md`,
  13.4), so its runs are offered `settings` and `propose_settings`
  beside every other host tool (4.3), and its task's authority, policy
  1's, holds the settings, read, delegation and message families. It
  has no workspace, its instructions never delegate, and it never
  calls `withdraw`, whose family, the decide family, policy 1 does not
  give: a proposal it no longer wants is replaced by a fresh one, and
  the first one an owner accepts leaves the other stale.
- **The kinds it reads and proposes** are a settings file's
  (`../client/tools.md`, 5.3): `charter`, `template`, `policy`,
  `rules`, `landing-rule` (with its `project`, the deployment's when
  left out), `forge-charter` and `project-policy` (with its
  `project`), each named and valued as the file names and values it,
  so that a description's landing rules, the forge's charters and a
  project's own policy are proposed as its charters are.
- **A proposal's source** is exactly one of inline `items` or
  `note: {name, revision}`. The root checks the settings and recall families
  and loads the whole named note within the configured notes and load bounds.
  Its scope must be readable by the proposing task and its revision must match.
  The shared settings-file parser decodes its body, and the same whole-value
  validation, base-revision checks and owner review apply to either source.
  Missing, inaccessible, moved or malformed sources create no proposal and
  apply nothing. A decided call is replayed before loading the note again;
  later note edits cannot change its proposal. No fragment state is added;
  the note keeps jig's revision, retention and cleanup rules.
- **It changes nothing itself.** Its charter's `settings` family reads
  and proposes; only an owner's decision changes settings (jig's
  `domain/engine.md`, 17.6). A member's configuring chat proposes as
  an owner's does, and its proposal waits for an owner.
- **It writes down how it understood the person** as the chat's document, "How
  PROJECT works", kept current as the conversation changes it (8.3).
- **Started from ctl** with `chat start --project PROJECT --template 3
  TEXT` (`../client/tools.md`, 5.3), and from the client's `Configure`,
  which "Tell temper how you work" opens (`../client/README.md`, 3.5).
- **Its tests** are `testing.md`, 6.4's: scripted runs in the engine's
  domain world for its flows, and one story through ctl in the joined
  client world (`testing.md`, 6.3).

### 8.2 Asking before the work

A person who wants to agree a plan before anything is changed: rung B0
of `../draft/v0.1.0.md`, 2.1.

- **The work that waits asks; a change asks only as it is made to.** A
  change is decided work, asked for once what to change is agreed. Made
  with `questions` `"none"` (4.3.1), the default, its agents, the
  table's first four rows, ask no one: the change procedure delegates
  each with the Message family taken out of the tool families it passes
  on, which a child's authority otherwise keeps (jig's
  `domain/authority.md`, section 3). The change itself takes no
  questions, answers or words (forge.md, 8.1); its amendments and its
  cancel reach it as forge.md says. A plan's maker that wants its
  changes' producers to ask makes them with `"foreman"` or `"up"`, and
  the change relays their questions (forge.md, 8.11); what follows is a
  chat's, whose changes take none.
- **The chat delegates the work as a goal** when its person asks to
  agree it first, with `goal` (4.3.2) and the person's words: a
  delegate under charter 1 holding the authority to call `change` and
  to ask; never `change` itself, which would start the work at once. The stock
  chat's instructions (charter 1, `../protocol/configuration.md`, 13.4)
  carry the rule: work the person wants to agree before anything is
  changed goes to such a delegate, which tells its plan, asks, and calls
  `change` only after the answer, which reaches it from the person's
  inbox, not through the chat. When the person answers the goal's question,
  the goal has the answer and does the work; the chat never calls `change`
  or `goal` for that answer and tells the person the work goes ahead. The chat passes the person's words whole, at once, even where they
  ask for an advisor first, since the goal asks one itself (4.3.2): it
  never sets `advisor` for such work, writes the plan or asks the
  go-ahead in its own reply, nor calls `change`. The goal, once it has
  asked, waits for the answer: it never finishes before it, nor before
  every agreed change has landed. The brief opens with the caller's words
  so that the task's title is the work (4.3.2); a blank line follows,
  then the root's goal line and its line that the work is to be agreed,
  then the person's heading and words.
- **A held change or goal waits for the person at once.** jig skips
  conversations when it chooses the escalation's holder (jig's
  `domain/tasks.md`, section 9). The chat tells the person in words
  what its delegate waits on and that it waits in their inbox, then
  waits for their decision.
- Every Finite charter 1 run keeps `message` in smith's reserve; the owing fact is enforced at the root and is not known when the worker builds the charter.
- **The goal asks its requester,** the chat, with `message` in the
  question form, before it calls `change`, and waits. jig puts the
  question in the inbox of the chat's person, since the person requested
  the chat (jig's `domain/tasks.md`, 8.1; jig's `domain/people.md`,
  section 6). The chat cannot ask its own person: it says in its reply
  that a question waits for them.
- **A question's line form.** jig's question carries words alone, so a
  question's options and its recommendation are in its words; the stock
  chat's instructions (charter 1, the goal's paragraph) have a goal
  write them in one form, which temper's client draws as a question,
  its context, what the agent saw and its choices (`../client/README.md`,
  3.5, *Needs you*). No protocol, store or tool knows the form:

  ```
  Fix the dropped watch in jig’s store, or keep it alive in temper serve?

  MakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local.

  Saw: The box test reproduces it every time.
  Choice: In jig’s store (recommended)
  Choice: In temper serve
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
    passes up with a foreman's reasons beneath it (forge.md, 8.11), whose
    last lines are those reasons.
- **The person answers in the inbox,** `temper ctl task answer TASK
  QUESTION TEXT` (`../client/tools.md`, 5.3), Home's card or the inbox's
  page, and
  nobody else does: the answer reaches the goal itself, as its
  requester's, never through the chat's model, which may not answer a
  question that waits in its person's inbox (jig's `domain/people.md`,
  section 6): its `message` answer is refused, naming that its person
  answers it there. The chat hears a notice that the question was
  answered, with the person's answer words cut at 256 bytes on a UTF-8
  boundary and marked when cut, and says so. The goal is kept while its
  question waits, including when its ordinary turns expire after asking
  (jig's `domain/tasks.md`, 5.7): a run of it that finishes before the
  answer parks, and the answer wakes it. It then calls `change` with
  what was agreed, waits for its result and reports; the chat says what
  landed.
- **Nothing is changed before the answer:** no change task, branch or
  pull request exists until the goal calls `change`.
- **The asking is the root's rail, not the goal's choice.** The chat
  marks such work once, as it delegates it: `goal` with `agree` set, at
  the moment its person asks to agree it first, and the root makes the
  goal's task owing its requester a question (jig's `domain/tasks.md`,
  5.7, its spec's `asks`), the typed mark everything below reads, never
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
- **Its stories:** `suite::fuzzy::green_light` on fakes (testing.md, 2.4), and
  rung B0 walked on real backends (`../draft/v0.1.0.md`, 2.1).

### 8.3 Documents and reports

#### A chat's document

A chat keeps one Markdown document beside its conversation: its agent writes
it, the person reads it in Chat, Split or Draft (`../client/README.md`, 3.5,
*A chat's document*), and a goal starts from it. It is the note of jig's in the
chat's own scope (jig's `domain/engine.md`, section 10), the lowest numbered
whose description does not begin `work:`: there is no kind. A `work:` note, such
as the configuring agent's `work: settings` (8.1), is never drawn, and a chat
with only working notes has no document. Its description is its title, its body
Markdown. The configuring chat's "How PROJECT works" is the same mechanism.

- **Who may write:** a chat's authority's note scopes hold a goal's, which for
  a root conversation is its own (`../protocol/configuration.md`, 13.4), so a
  chat writes its own scope and only it. A goal uses its own scope for its
  report; an advisor keeps no note scopes. A run knows its task number from its brief.
- **Its instructions** are held in the binary, under the chat's own paragraph
  of `chat.txt`: when the person asks for a draft, or says "write this up", keep
  one document, with `note`, scope goal, the project and its own task number,
  description its title, its body Markdown beginning with `# TITLE` and a muted
  line "Draft, DATE, with the user.", one `##` section per topic, open questions
  under "## Open questions". It writes a first version whole, then changes it by
  lines (jig's `domain/engine.md`, section 10), naming the note and the revision
  it read; it never rewrites it whole past a few KB, builds a long document in
  parts, and continues a line past one call with `join`. It says in a sentence
  what changed and doesn't repeat it. Mermaid isn't drawn, so it doesn't draw.
- **The configuring agent's,** `configuring.txt` (8.1): the same, its title "How
  PROJECT works", written once the person's way is understood, kept current as
  the conversation changes it.
- **The tool's answer** gives the client what its line says (`../client/README.md`,
  3.5): `Note N revision R written · 64 lines` for a first write, `Note N revision
  R written · +18 −6 · line 31 · 64 lines` for a later one, from jig's `lines`,
  `added`, `removed` and `first` (jig's `domain/engine.md`, section 10). The
  lines close each so that an append's `first` needs no counting. A refused call
  says what to do next: `Note refused: revision moved, now 7`, `Note refused:
  range past the end, 64 lines`, `Note refused: body past its bound, 70000 bytes
  of 65536`.
- **Its stories:** a scripted turn of the fake model that writes a document
  (`testing.md`, 2.4), proved through ctl and walked in the journey `chat-draft`
  (`../client/journeys.md`, 9); and a walk's own script, which builds a 30 KB note
  in calls of under 10 KB each, one line longer than a call among it, then has a
  fresh run find the note in its brief and change it by lines.

#### A goal's report

A goal tells its person where it stands in a **note**, one per goal, which
needs no record, read or wire field of its own. The client reads and draws it
(`../client/README.md`, 3.5, *A goal's report*); a person reads it first.

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
  reference's charter 5 (`../protocol/configuration.md`, 13.4 and 13.9). The
  stock's authority holds the goal's scope (13.4); a charter that does not ask
  for a report writes none, and no note yet is a normal state, never an error.
- **Its story:** the scripted fake model writes a report in its goal script, so
  that a walk on `temper box --fake-backends` shows one
  (`../protocol/configuration.md`, 12.1).

## 9. The world

smith's worlds test the agent itself (smith's documents, each section
"The world"). temper's test what it fills in, and the agent under its
host:

- **`temper-engine-smith`'s step tests:** the root's charter becomes
  smith's start, its brief in order and its budget exact; every declared
  tool decodes its least shape and refuses a wrong one; `amend` and
  `release` name their revision; every answer renders as smith's text
  within smith's limits, or as an error; a job's answer names its failed
  step before its log; smith's terminals keep their spend and the root's
  contract.
- **The routing's tests:** a call decoded into the root's `Tool`; one of
  another activation, or of an unknown tool, answered as an error with no
  commit; a result that does not decode failing its attempt as invalid;
  a change's result kept only with its settled resource; each settled
  delivery the hub's answer (`../protocol/llm.md`, section 6).
- **The worker's and the system worlds** (worker.md, section 10): smith's
  scripted agent for the agent child, and smith's real agent domain over
  skein's fake LLM, its tools working in the trees the worker prepared.
- **Stories that are temper's:** a change produced, checked and pushed,
  and its push found moved; a coordinator that delegates a batch, hears
  results, and proposes what is beyond its authority; a chat that waits,
  hears its person's words, parks, and resumes from its transcript; a
  host tool answered busy by the engine, asked again and decided once.

## 10. Open questions

- **Which host tools a charter offers.** jig's charter carries the host
  families its charter's `tools` names and its task's authority holds
  (jig's `domain/engine.md`, 7.1), each stock charter's named in
  `../protocol/configuration.md`, 13.4; temper's declares by them the
  `settings` family alone and every other host tool in every charter
  (4.3), and the authority refuses what it does not allow. Whether to
  declare only what the charter names for the other families too, which
  the host part's budget (4.3) may force, and how a brief teaches an LLM
  when to delegate a task rather than a sub-agent.
- **The configuring agent woken by a landing** of a document a charter
  names, where the landing touches structure, to say what it means for
  the settings ("the process document changed; here is what it means
  for your setup": `../draft/configuring.md`, section 6). Default: a
  person starts it (8.1); a landed document reaches the next runs of the
  charters that name it (`../protocol/configuration.md`, 13.7).
- **A settings value past one call** (8.1): `propose_settings` carries each
  item's value in its call, so a value past what one call holds
  (`../protocol/configuration.md`, 13.8), as a 23 KB one is, can't be proposed,
  though a note now holds it, built by line ranges (jig's `domain/engine.md`,
  section 10). Naming a note there is left to a plan of its own (the request
  `note-parts` of 2026-11-05). Default: a value fits one call.
- **Who may name a plan's members of `change`:** the control family
  says it (4.3.1), since a plan's maker amends, cancels and releases what
  it makes. A family of its own would let a policy grant control without
  plans, or plans without control.
- **A goal's question kept with its chat** (jig's `domain/tasks.md`,
  8.5): the chat, woken by it, would answer from what it knows (its
  person's priorities, its notes), or ask its person a question of its
  own and answer the goal once it has the answer, in place of 8.2's
  route through the person's inbox. Default: 8.2's route.
- **Who else writes a question's line form** (8.2): the stock goal alone
  is taught it, by one sentence of charter 1, as the request of 2026-11-05
  for *Needs you* asks. A change's producers, its repairs and resolutions, and
  the reference's charters are not, so their questions read as words,
  answered by a reply. Default: as it is; a line in each charter that asks.
- **A change's foremen's LLM choice** (forge.md, 8.11): `llm` (4.3.1)
  names the producer's and the reviews' roles, and a change's foremen
  run at their charter's default, where the reference's draft gives
  each change one for its foremen too
  (`../draft/reference-code-production.md`, sections 5 and 14). Default:
  the foreman's charter's, the reference setting its model there
  (`../protocol/configuration.md`, 13.9); a `foreman` role once a plan
  needs foremen of different weights.
- **A cancel's grace:** jig's `cancel` takes one, if the run gives it
  (jig's `domain/engine.md`, 7.3); temper's takes none.
- **A code graph** for a workspace (worker.md, section 12), as an MCP
  tool source once smith has one.
- **A change produced by a job of any kind.** `producer` names a
  charter, and `merge` the merge job, a catch-up's producer (4.3.1):
  every producer a reference change needs
  (`../draft/reference-code-production.md`, section 3). Default: no
  other job kind produces a change; a project whose tooling makes
  changes with no model would name its kind as a member of `change`.
