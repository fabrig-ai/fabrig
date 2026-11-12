# The agent's tools

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
- **Every host tool below,** declared in every charter, each with the
  charter's call timeout, but `read_task`, offered only with the task's read
  family, and those of the `settings` family, declared
  only where the run's charter names that family in its `tools` (jig's
  `domain/engine/executors.md`, 1) and its task's authority has it, and the
  charter is no asking one (below): no stock charter, since charter 11 is
  an asking charter, and a charter of a deployment's own that names the
  family (`../../protocol/configuration/stock.md`). The delegate family's,
  `goal`, `change` and `delegate`, are declared only where the run's
  charter names that family in its `tools`, so that a charter that starts
  nothing, the drafter's (`agents.md`, 4), is offered none of them; where it does, its
  task's authority decides each call, as below.
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
  request and blurs what it does, so for these charters the rule above,
  every host tool in every charter, does not hold. Authority decides
  each call by its family as for every charter: `see`, `document` and
  `remember` by the asking family, `propose_settings` by the settings family. The task's authority decides
  each call by its family (jig's `domain/authority.md`, sections 3 and
  8.5): a call beyond it is answered beyond authority, naming what it
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
  its `after` names (1) (jig's `domain/engine/executors.md`, 3).
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
  (`../forge/README.md`, 5.2) count too. A call past that room is refused, saying
  why, as jig's `domain/engine/executors.md`, 3 has every refusal do:
  `subscribe` answers "Refused: task 4 busy: it holds its most
  subscriptions (at most 4, counting the topics the engine keeps for a
  goal); unsubscribe one it made to make room", and `subscribe_forge`
  names `tasks.subscriptions` as its own room's refusals do. A
  subscription that would leave no room in the task's inbox for its news
  is refused as any call past the inbox is, naming the inbox.
- **The host part's budget.** smith refuses a run whose host part (its
  host tools, instructions, brief, and finish and deliver schemas) passes
  its bound, the room of the agent's smallest window (`charter.md`, 5), before any
  turn. The largest stock charter's non-brief host part is charter 1's
  with a Change contract and the control family's plan form of
  `change` (1); with the authorised brief, its task section a whole
  spec (`../../protocol/configuration/limits.md`), and 1,472 bytes for
  64 stock sections' titles and framing (16 title bytes and 7 framing
  bytes each), it leaves a margin at `temper-box init`'s models
  (`../../protocol/configuration/README.md`, section 9); `temper-service`'s step tests hold each stock charter's
  figures. A gathered brief spends what the non-brief host part leaves. So a new
  host tool is offered only where a family and the charter gate it,
  never to every run, and a charter's room is measured before a tool
  joins it.
- **The brief fits the room its charter leaves.** smith counts a run's
  brief in its host part, beside the charter's offered tools, its
  instructions and the finish and deliver schemas. Before sending an
  assignment, the engine fits gathered sections to its charter bound for
  both the encoded charter, including framing, and its decoded ownership.
  The bound is the smaller of the configured `hub.charter_bytes` and the
  charter allowance of the worker that receives it, so the engine never
  sends a run a charter its worker would refuse: a worker refuses at
  decoding a charter whose ownership passes its smith `run_bytes`, before
  its own fit. Where the process has a local agent, that allowance is the
  agent's `run_bytes`, as a task read's room is its `host_reply_text`
  (`../routing.md`, 2). Where it doesn't know its worker's, it takes smith's
  at its floor, 65,536, which the stock agent's `tool_payload` of 10,662
  does not raise (smith's `protocol/limits.md`, 3.10): the smaller of the
  two defaults, and how the engine learns a remote worker's is `README.md`, section 8
  's. Instructions, tools, contracts,
  models and run identity stay whole. The worker service also fits the
  brief after the workspace is prepared and before encoding the start,
  using the receiving run's limits and measured schema costs. It fits both
  rendered host room and the decoded charter ownership left after smith
  charges the prepared workspace against `run_bytes`, including resolved
  host paths and conflict paths. Smith supplies that remaining allowance
  through `charter_room`; temper keeps the incoming charter allowance too.
  `temper-protocol-brief` owns
  the fitter, which the engine's translation, `temper-engine-smith`, and
  the worker service each link; the translation's typed start takes
  explicit receiving limits and fits the brief too. A brief already within each entrance's room is kept
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
  (`../../protocol/configuration/limits.md`). A stored proposal's
  acceptance, a restore and a reset carry values that passed an
  entrance or the start, and the bound does not move while the engine
  runs. The sum is a table of each host family's declaration bytes and
  the charter's own text, where the domain reads it, not a call into
  smith; a step test in `temper-engine-smith` holds the table equal to
  smith's own measure. The bound is the engine's size profile's, the
  one its workers are configured from (51,717 in the small one, for a
  window of 177,523 tokens; in `standard`, the room of luna's window,
  `charter.md`, 5); an
  engine without that figure skips the start's check. A run smith
  refuses for its host part anyway is held with smith's reason, the
  bound and the figure, never a bare "invalid".
- **`repository`,** one of the project's repositories as `owner/name`:
  among its adopted repositories of any role, on forges not ended, the
  one named, or with no name the only one; with several and no name, or
  a name none has, refused, naming them. The root resolves it to the
  repository's resource as it builds a change's delegate (1),
  before it decides the call, so that the call's authority is checked
  on that resource (`../routing.md`, 2) and the run never names a number.
- **The read's own members,** beside it: `read_branch` `branch`;
  `read_branches` `prefix`; `read_pull` `number`; `read_pull_for` `head`
  and `base`, branch names; `read_pull_files` `number`, `head`,
  `page` and `skip`; `read_reviews` `number`, `page` and `skip`;
  `read_remarks` `number`, `review`, `page` and `skip`; `read_statuses` `commit` and `page`;
  `read_checks` `commit`; `read_compare` `before` and `after`;
  `read_file` `head`, `path` and `max_bytes`; `read_job` `head`, `run`,
  `job`, `attempt` and `max_bytes`, exactly as `read_checks` and
  `read_statuses` name each Actions job: the job attempt skein resolved
  from the check's link (its run, the forge's own job id, its attempt:
  skein's `forge/forgejo.md`, 2), never the link's index of the job in its run,
  which the answer may show beside it but no read takes; `read_items` `since`, `page` and
  `item_kind` (`issue` or `pull`); `read_item` `number` and `after`;
  `read_protection` `branch`; `read_settings` none;
  `read_collaborators` `page`; `read_permission` `user`, a forge user's
  id. A commit or a head is its id, 40 or 64 lowercase hexadecimal
  digits; `page` is from 1, and 1 when left out; `since` is a time in
  nanoseconds, 0 when left out; `after` a comment's id, 0 when left
  out; `max_bytes`, the read's limit when left out; `skip`, the rows of
  `page` already read, 0 when left out.
- **A page the answer's bytes cut names how to read on.** A
  `read_reviews`, `read_remarks` or `read_pull_files` page cut at
  `forge.client.answer_bytes` (skein's `forge/README.md`, 3.2 and 3.3)
  ends with `Page cut: read on with page P, skip S.`, its own page and
  the rows read of it, so the next read loses no row; a page that ends
  uncut and has more says `More pages available.`, as today.
- **`read_permission` names the person:** its answer is the permission,
  the forge user's id and the login the forge names them by, as
  `Write; forge user 3, login ada` (skein's `forge/README.md`, 3.3), so that a
  run can match a name to an id; with no login from the forge, the same
  without it. `read_collaborators` lists ids, for `read_permission` to
  name.

- **A compare is by names, with no contents.** `read_compare` answers
  how many commits `after` has that `before` lacks and how many it lacks,
  whether `after` contains `before`, and the changed files' paths, at
  most `forge.client.rows` (8 as configured) with how many it left out
  (skein's `forge/README.md`, 3.2 and 3.3). It reads no file's contents, so two
  branches however far apart compare. Its answer, a line each:
  `AFTER is A commits ahead of BEFORE and B behind it.`, then `AFTER
  contains BEFORE.` or `AFTER lacks BEFORE.`, then `Changed files: N`,
  each path it holds, and, where it holds fewer than N, `[K not listed:
  read_pull_files lists a pull's files; read_file reads a path at each
  commit]`. A file's contents are `read_file` at `before` and at
  `after`, a side `Missing` where the file is absent; its description
  says so. A `read_compare` refusal (a forge that cuts its own listing)
  names its bound as any read's does, says that the commit ids were read
  and that no other id helps, since a run that isn't told retries with
  one it made up, and directs it to `read_pull_files` or `read_file`. A
  successful comparison whose rendered form exceeds the host answer's
  bound returns measured-size feedback with the same recovery instead of
  losing its body at the worker.
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
  recover the oversized page. A file or pull-file forge size refusal, whose
  measured size is unknown, names that uncertainty, the forge bounds with
  their configured figures and the numeric host bound with the same
  tool-specific recovery. A compare's forge size refusal follows the compare
  guidance above. At a host bound too small
  for the guidance, the refusal itself is cut to fit.
- **A refused branch read says what to read instead.** Both refusals
  are right: `read_branches` takes a nonempty prefix, an empty one
  ending as `Error::Empty` before any request (skein's `forge/forgejo.md`,
  section 2), and `read_branch` of a name the forge lacks is
  `Error::Missing`. That is so for a goal's own branch where its goal
  holds none (a chat's `goal` names none, 2), and for a run branch
  before its first push (`../forge/branches.md`). `read_branches`'s
  `prefix` is declared "A nonempty prefix of the names, as temper/."
  Each refusal keeps its first line, `Forge read failed: empty` or
  `Forge read failed: missing`, and goes on after a full stop:
  - `read_branches`, `Empty`: "read_branches needs a nonempty prefix,
    as temper/ for temper's branches. read_settings names the default
    branch, and read_branch reads one branch by its name."
  - `read_branch`, `Missing`: "The forge has no branch of that name in
    this repository. read_settings names its default branch, and
    read_branches lists the names under a prefix, as temper/. A goal's
    own branch is there only where its goal holds one, and a run branch
    from its first push."
  - `read_branches`, `TooLarge`, first line `Forge read failed: too
    large`: "The names under this prefix pass forge.client.answer_bytes,
    the one bound a branch list has: name a longer prefix, as
    temper/goal/ in place of temper/, or read one branch by its name
    with read_branch." No row bound is named, since none applies to
    branch names (skein's `forge/README.md`, 3.7). The tool's description says
    the same in its words: the names fit forge.client.answer_bytes, and
    a longer prefix reads fewer.

  Every other tool's `Missing` and `Empty` keep their one line. A forge read
  refused as too large names the bound it passed and the size it met, in a
  bound's words (`../forge/change/repairs.md`): `Forge read failed: too large: past
  forge.client.rows (8): 22 rows.`, then how to read on, as its tool's
  description says. Each forge read's description names the bounds its
  answer meets and how to read past them: a listing by its next page, a
  file by path within `max_bytes` (`read_file`), a pull's files by page
  or a path at the head or the base, a comparison by names with each
  file's contents by path, a branch list by a longer prefix. The refusal
  stays marked an error and is cut to fit the host answer's bound, as
  the compare's is.
- **Reads and writes:** the forge's reads, `recall`, `settings` and `read_docs` are reads; every
  other host tool is a write, which smith runs alone (smith's
  `domain/run.md`, 5.2).
- **`decided`,** of the control family, offered as every host tool is, to
  every task whose question would reach a person (`../forge/attention.md`, 5): `{
  question, options, took, why, kind, paths, under }`, `options` 2 to 6
  words of the agent's, `took` an index into them, `kind` one of the ten
  of `../forge/attention.md`, 6, `wording`, `retry`, `implementation`, `tests`,
  `dependency`, `departure`, `money`, `irreversible`, `rules` and `other`;
  `paths`, optional, at most 16 repository-relative paths of at most 256
  bytes that the choice would change; `under`, optional, the number of an
  item with words of the project's classes the choice falls under;
  `question` at most 400 bytes, each option 120, `why` 400, a call past
  those refused at decoding, saying so. The forge judges it (`../forge/attention.md`,
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
    to the classes, a landing rule or a standing role's rules is `rules`.
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
- **Declared, decoded and rendered in one place,** `temper-engine-smith`:
  each tool's JSON Schema, description and effect, written once
  (`delegate`'s and `amend`'s schemas name every field of the spec and
  the authority, so that a run writes them from its tools alone, and
  both take a priority as `{priority}`, where jig's own `delegate` takes
  a bare number, so that a run learns one shape); a
  call's input decoded into the root's `Tool`, or answered at once as an
  error naming what is wrong, deciding nothing; an answer rendered as
  smith's text, marked an error or not (`../../protocol/llm.md`, section 4).
- **Calls are named and decided once** (jig's `domain/engine/executors.md`, 3):
  the engine answers a call it has seen before from its record, and smith
  asks again under the same name after busy or a lost answer (smith's
  `domain/run.md`, 5.2).
- **An answer fits what the agent takes** (`../worker/README.md`, 4.3):
  rendering a forge read or a task read uses the minimum of `core.call_answer_bytes` less
  the tool name and the receiving agent's reply bound, the root's
  `Domain::reply_bound` for the call's key (`../routing.md`, 2): the
  `agent.limits.bodies.host_reply_text` the run's worker names in its
  hello, from its agent's `agent.json` `profile.declared.tool_payload`
  (`../worker/README.md`, section 5); where it names none, the local agent's.
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
- **A call names what it saw** (jig's `domain/engine/executors.md`, 3): the
  revisions above are those its brief, its messages and its earlier
  answers showed it. One whose object has moved since is answered moved,
  with what it is now, and the run decides again on that.
- **Not the engine's:** `wait`, `finish` and `deliver` (jig's
  `domain/engine/executors.md`, 3). `deliver` is a push (`README.md`, section 5).
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
`temper-box serve: restart refused: ` and the same diagnostic (2026-11-04).

## 1. A change, in the run's words

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
- **the delegate:** the change procedure (../forge/change/README.md, 1) with the words,
  the repository's resource and its landing branch: the one the call
  names (below), or with none the applicable held goal branch, otherwise
  the repository's default branch; a
  change's contract, its words bounded by the deployment's
  `tasks.result_bytes`;
- **its authority, the run's own narrowed:** the run's grants of the
  forge's reads (kind 1, ../forge/README.md, section 2) and of the kinds a change
  makes (../forge/README.md, section 4: a push, open, merge, edit, update, close
  or reopen a pull request), on the change's repository, which every
  agent of the change holds, so that its reviewer and its repairs read
  the pull request, its files and CI's logs; the change's charters, its
  producer's and its reviews' (../forge/change/gates.md, 3), as the only kinds it may
  delegate, the merge kind, job 3, where the run's authority delegates
  to it (../forge/change/repairs.md), and a person's role where a landing rule
  covering the branch names a person gate; at most
  `forge.change_policy.tasks` tasks (`../../protocol/configuration/limits.md`
  : 6 in `small`, a producer, a review, and two repairs each
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

- **`landing`:** `"goal"`, the nearest goal branch held by the requester
  or one of its ancestors in the change's repository (../forge/branches.md, 1);
  or a branch's name in the repository. Left out, that held goal branch,
  otherwise the repository's default branch. Ordinary child changes,
  including calls from the goal's subtree, cannot name a destination
  outside that held branch: the whole call is refused before any delegate
  is made. `"branch": "mine"` deliberately lands the requester's held goal
  branch whole, defaulting to the repository's default branch. `"goal"`
  with no applicable held goal branch is refused, saying so; a
  name that is not a branch's (an empty segment) is refused, naming it;
  a branch the repository lacks holds the change at its opening, the
  forge refusing its pull request (../forge/change/README.md, 2).
- **`branch`:** `"mine"`, its requester's held branch: the goal branch
  the run's task holds in the change's repository (../forge/branches.md, 1),
  which the change lands whole as it stands, with no producer (../forge/change/README.md,
  1); left out, the change's own branch, which its producer pushes.
  `"mine"` from a run whose task holds no goal branch in that repository
  is refused, saying so; with `"landing": "goal"`, a branch into
  itself, refused, saying so; while another live change of the task is
  on that branch, refused, naming its task. Any other value refused,
  naming it, `"run"` among them: a goal's runs deliver their design
  straight to its goal branch (`tools.md`; ../forge/branches.md, 3), so no change lands
  a run branch. The schema offers `"enum": ["mine"]`, described `mine:
  goal into base; omit: a new.`
- **`name`:** a name for the change within the call, at most 64 bytes,
  unique in it, which another change of the same call names in its
  `after`; it names nothing once the call is decided.
- **`after`:** the change's dependencies (jig's `domain/tasks/README.md`,
  section 4): a task number, one of the run's task's live delegates, or
  the `name` of an earlier change in the same call. The change waits on
  them as any task does, so it starts once each has ended done; a name
  the call does not give before it, or a task jig refuses, refuses the
  call, naming it. Its producer, and every other task it delegates to, may
  read each of them with `read_task`, live or retained, read only (jig's
  `domain/engine/executors.md`, 3): a dependency is no reference, and
  carries no message.
- **`merge`:** a catch-up (../forge/README.md, section 6): the commit to take, as
  40 or 64 hexadecimal digits, which the merge job merges into a branch
  made at the landing branch's tip, as the change's producer in place of
  an agent; the words say what the commit brings and why. Refused,
  naming the setting, where the change's project names no `merge_job`
  (`../../protocol/configuration/engine.md`), and naming the kind,
  where the run's authority does not delegate to the merge kind; a
  commit in another form is refused, naming it. Whether the repository
  has the commit is the job's to find: one the forge cannot fetch
  fails the job, and holds the change.
- **`changes`:** several changes in one call, each an object of the
  members above but `changes`, made as one batch (jig's
  `domain/tasks/README.md`, section 4): all of them or none, the call's answer
  naming each change's task in the call's order. A call names either
  `words` or `changes`, never both; `changes` holds at least one.
  A batch's change takes every member from its own object alone, the
  members below `changes` and those to come too, and nothing from the
  call's: a call with `changes` names no other member. One that does is
  refused as it is decoded, before any change is made, naming the first
  such member in the declaration's order (`repository`, `name`,
  `landing`, `branch`, `merge`, `after`, `producer`, `llm`, `repairs`,
  `questions`) and saying it belongs in each change: "host call member
  belongs in each change, not beside changes: landing". A landing, a
  producer, a route for questions or an LLM choice given once for a
  whole batch is never dropped unheard, and the run gives it again in
  each change's object. A member's own value is the change's, as in the
  single form; `words` beside `changes` is refused naming `changes`, as
  above.
- **`producer`:** the number of the charter the change's producing
  task, its repairs, its resolutions and its checks' gates run under
  (../forge/change/README.md, 1); left out, the forge's change charter, kind 3's number
  1 (`../../protocol/configuration/settings.md`, 2). A charter the newest
  settings do not offer (jig's `domain/engine/settings.md`, 3), or one the
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
  each as jig checks a batch's (jig's `domain/tasks/README.md`, section 4),
  against the charter its role runs under at the newest revision and
  the run's authority: a model not configured or named twice, an effort
  its model does not declare, a model, main or sub-agent, outside that
  charter's models or the authority's refuses the call, naming the
  model and why. A role left out names none, and each of its agents takes
  its models at each run's check (jig's `domain/tasks/README.md`, section 3).
  A role's choice carries no `later` (`../../draft/per-wake.md`, section
  5): a change's producers resume or finish in one activation, so each
  of its agents takes the one choice at every run, and the root makes
  each with no `later`.
- **`repairs`:** the red landing the change repairs
  (`../after-landing.md`, section 7), by its merge commit's full
  hexadecimal digits, as the red's news names it: a landing of temper's
  in the change's repository, on the branch the change lands into, red
  in some tier as the root decides the delegate. A value that is not a
  commit's digits is refused, naming it; a commit no live landing of
  temper's there has, or one red in no tier, refused, naming the commit
  and why. The change keeps it, and its own landing clears the red one
  when a later tier run passes the failures blamed on it
  (`../after-landing.md`, 5.3); left out, the change repairs nothing.
- **Who may set them:** the members past `words` and `repository` are a
  plan's, and only a run whose task's authority holds jig's control
  family (amend and cancel, with `release`: jig's `domain/authority.md`,
  section 3) may set them: a plan's maker amends, cancels and releases
  what it makes. The stock policy's families give it
  (`../../protocol/configuration/stock.md`), so a chat or a goal under the
  stock sets them. `change`'s declaration offers them only where the run's
  authority holds that family (`tools.md`, the host part's budget), and the root
  refuses a call naming one from a run without it, beyond authority,
  naming the family, as it decides the delegate. What each member sets is
  checked as the rest of the batch is: its landing branch by the change's
  grants on the repository, its dependencies by jig's batch.
- **`questions`:** how the change takes its agents' questions (../forge/change/questions.md,
  1): `"none"`, a change that asks no one (`agents.md`, 2); `"up"`, each
  passed straight to the requester; left out, the
  project's setting (`../../protocol/configuration/settings.md`, 2, kind 6), whose
  default is `"none"`. Any other value is refused, naming it, `"foreman"`
  among them, alone and in a batch's member alike: that route is retired
  (`../../draft/stock-charters.md`, 3), and the schema offers `"none"` and
  `"up"` alone.
- **Members to come** join each change's object, each optional and
  checked by the root as it decides that change's delegate, leaving
  these as they are.

## 2. A goal, in the run's words

A chat cannot know what a goal's `delegate` needs either: its own
authority's tool families, the grants' vocabulary on the forge's
resources, the delegation's kinds and its funding, none of which a run
sees. `goal` takes the words alone, whether the work is to be agreed
first (`agree`, `agents.md`, 2), and whether the goal is an advisor,
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
  (`agents.md`, 2), the root's lines are followed by its person's last words to the
  chat, whole, under their own heading, so that a request the chat cut short
  still reaches the goal entire (cut only past the brief's room, and
  then marked with the bytes cut). The goal takes its label and flags
  from the goal template (template 2): label 2, listed, no conversation,
  kept, and ended by its result. It is tracked
  at the run's own priority, or last among the project's goals where the
  run is untracked: one past the largest tracked priority in the
  project, saturating, or 0 with none (the lower first, jig's
  `domain/tasks/README.md`, section 3), with a report's contract bounded by
  the deployment's largest result, jig's `tasks.result_bytes` (the box's
  agent's message, `../../protocol/configuration/stock.md`), whatever the run's own contract's
  bound (a chat's bound reaches neither its brief nor its replies,
  since its requester ends it, jig's `domain/tasks/lifecycle.md`, 9), since jig
  refuses a batch whose contract is above it (`charter.md`, 3): the work a person
  asked for, which plans, asks its requester with `message` where its
  words say to (`agents.md`, 2), calls `change`
  for each change, and reports. The contract bounds the result, not
  the work: asking first is the instructions', and the rail is
  authority's;
- **its authority, the run's own narrowed,** never wider in any part
  (jig's `domain/authority.md`, sections 3 and 5): the run's models and
  deadline; its tool families (delegation, messages, notes and reads, so
  that it may call `change`, ask, write its report and read the forge);
  its grants of the kinds
  a change makes and its reads, so that `change` can narrow them again
  (1); its delegation kinds, with the tasks its subtree may still
  make, as jig counts them, less one for the goal itself, and a depth
  one less, refused when either is spent; notes of the goal's own scope
  alone, where the run's own authority holds that scope, so that it
  writes its report (`agents.md`, 3); half of what its
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
by it (`agents.md`, 2); a goal is where the work is planned, so an advisor asked
for elsewhere leaves the work to be planned in a chat's reply. So one
fact of the calling run, whether the root built it as a goal, decides
both. For a tracked goal whose spec marks the work to be agreed (`agents.md`, 2),
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

`goal`'s declaration says when a run calls it, as `delegate`'s says
to call `change` for a change to a repository, since a run chooses
among its tools by what their declarations say as much as by its
instructions: "Have one goal tracked, only for work your person wants
agreed first (set agree) or a survey that changes nothing. For a change
to a repository, call change." It fits a declaration's 160 bytes
(`../../protocol/documentation.md`, 8), so it says when alone: what a goal
may do is its charter's, and what `advisor` does is that member's
description. The root decides a chat's `goal` without
`agree` as before: a goal made for a plain change still makes its
change, and the change lands, at a goal's run more than the chat's own
`change` would take.

## 3. Open questions

- **Proposal-form diagnostics** (`tools.md`): `settings` reads name the kind
  and its missing member; `propose_settings` keeps the shared settings-file
  parser's errors. Default: those words, until a proposal needs the same
  kind-specific guidance. Both tools' schemas name the numbered kinds.

- **A charter where the process has no local agent.** The engine does not
  know its remote worker's charter allowance, so it fits a charter
  to the smaller of `hub.charter_bytes` and smith's 65,536 (`tools.md`), and a
  remote worker that takes more, the reference's 127,480 among them,
  receives its brief cut at 65,536. The worker's hello on jig's link, which
  names its reply bound (`tools.md`), could carry the allowance beside it, or the
  engine's document could name it in `hosts.workers`, and the request
  `landing-behind-main` of 2026-11-08 asks it be sized as that bound is.
  Default: the smaller of the two defaults.

- **A collaborator's login in `read_collaborators`** (`tools.md`). Default: ids,
  with `read_permission` naming each.

- **`read_permission`'s login held to the settings family** (`tools.md`). It
  reaches every run with the forge reads, as the forge's own answer about
  a user of a repository that run may already read; holding it would be a
  condition in `temper-engine-domain`'s render. Default: every such run.

- **Which host tools a charter offers.** jig's charter carries the host
  families its charter's `tools` names and its task's authority holds
  (jig's `domain/engine/executors.md`, 1), each stock charter's named in
  `../../protocol/configuration/stock.md`; temper's declares by them the
  `settings` and delegate families alone, an asking charter its own tools
  alone, and every other host tool in every other charter (`tools.md`), and the
  authority refuses what it does not allow. Whether to declare only what the
  charter names for the other families too, which the host part's budget (`tools.md`)
  may force, and how a brief teaches an LLM when to delegate a task rather
  than a sub-agent. `withdraw`, offered to a chat and refused by its
  authority, since policy 1 gives no decide family
  (`../../protocol/configuration/stock.md`), is one such tool.

- **Who may name a plan's members of `change`:** the control family
  says it (`tools.md`, 1), since a plan's maker amends, cancels and releases what
  it makes. A family of its own would let a policy grant control without
  plans, or plans without control.

- **A cancel's grace:** jig's `cancel` takes one, if the run gives it
  (jig's `domain/engine/executors.md`, 3); temper's takes none.

- **A change produced by a job of any kind.** `producer` names a
  charter, and `merge` the merge job, a catch-up's producer (`tools.md`, 1):
  every producer a reference change needs
  (`../../draft/reference-code-production.md`, section 3). Default: no
  other job kind produces a change; a project whose tooling makes
  changes with no model would name its kind as a member of `change`.

- **A real chat choosing `change`** (`tools.md`): the owned-forge
  landing story requires the chat's own successful `change`, rather than
  a goal's change. Observe whether `goal`'s declaration steers a plain
  repository change there, and whether the branch-read guidance gives a
  real run its next read. Default: declarations and bounded refusal
  guidance, without a new admission restriction; the existing live story
  runs after landing (`../../testing/live-tier.md`). A scripted call establishes
  neither model choice nor recovery.
