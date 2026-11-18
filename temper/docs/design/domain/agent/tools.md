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
  (`../forge/README.md`, 5.2) count too, and so does the project's
  findings topic the root keeps for a finding's holder, which alone is
  admitted one past that room (`../areas.md`, section 5). A call past that room is refused, saying
  why, as jig's `domain/engine/executors.md`, 3 has every refusal do:
  `subscribe` answers "Refused: task 4 busy: it holds its most
  subscriptions (at most 4, counting the topics the engine keeps for a
  goal); unsubscribe one it made to make room", and `subscribe_forge`
  names `tasks.subscriptions` as its own room's refusals do. A
  subscription that would leave no room in the task's inbox for its news
  is refused as any call past the inbox is, naming the inbox.
- **A charter's room** is `room.md`'s: smith's host part's budget, the
  allowance a charter is fitted to, how its brief is cut, what a refusal
  names when the rest alone doesn't fit, and the checks at start and at
  each change of settings.
- **`repository`,** one of the project's repositories as `owner/name`:
  among its adopted repositories of any role, on forges not ended, the
  one named, or with no name the only one; with several and no name, or
  a name none has, refused, naming them. The root resolves it to the
  repository's resource as it builds a change's delegate (1),
  before it decides the call, so that the call's authority is checked
  on that resource (`../routing.md`, 2) and the run never names a number.
- **The forge's reads are three tools,** each taking a `what` that names
  the read, `repository` beside it, and the union of its reads' members,
  each member described once (`../../draft/tools-by-role.md`, 4; the split
  in three keeps each within `declarations.md`, 3's bound):

  | Tool | `what` | Required | Optional | Formerly |
  |---|---|---|---|---|
  | `read_forge` | `branch` | `branch` | | `read_branch` |
  | | `branches` | `prefix` | | `read_branches` |
  | | `compare` | `before`, `after` | | `read_compare` |
  | | `file` | `commit`, `path` | `max_bytes` | `read_file`, its `head` |
  | | `protection` | `branch` | | `read_protection` |
  | | `settings` | | | `read_settings` |
  | | `collaborators` | | `page` | `read_collaborators` |
  | | `permission` | `user` | | `read_permission` |
  | `read_issues` | `items` | | `since`, `page`, `item_kind` | `read_items` |
  | | `item` | `number` | `comment` | `read_item`, its `after` |
  | | `pull` | `number` | | `read_pull` |
  | | `pull_for` | `branch`, `base` | | `read_pull_for`, its `head` |
  | | `reviews` | `number` | `page`, `skip` | `read_reviews` |
  | | `remarks` | `number`, `review` | `page`, `skip` | `read_remarks` |
  | | `pull_files` | `number`, `commit` | `page`, `skip` | `read_pull_files`, its `head` |
  | `read_ci` | `checks` | `commit` | | `read_checks` |
  | | `job` | `commit`, `run`, `job`, `attempt` | `max_bytes` | `read_job`, its `head` |
  | | `statuses` | `commit` | `page` | `read_statuses` |

  - **A member keeps one meaning** across the three: `commit` a commit's
    id, 40 or 64 lowercase hexadecimal digits; `branch` a branch's name,
    for `pull_for` the branch the pull merges from, and `base` the one it
    merges into; `before` and `after` the earlier and later commits of a
    comparison; `comment` a comment's id, the item's comments after it,
    0 when left out; `number` an issue's or a pull's; `review` a review's
    id; `user` a forge user's id; `page` from 1, 1 when left out; `skip`
    the rows of `page` already read, 0 when left out; `since` a time in
    nanoseconds, 0 when left out; `item_kind` `issue` or `pull`;
    `max_bytes` the read's limit when left out.
  - **`job`'s members** are exactly as `checks` and `statuses` name each
    Actions job: the job attempt skein resolved from the check's link (its
    run, the forge's own job id, its attempt: skein's `forge/forgejo.md`,
    2), never the link's index of the job in its run, which the answer may
    show beside it but no read takes.
  - **A `what` missing a member it requires** is answered at decode,
    deciding nothing, by naming what that `what` needs, as `read_ci job
    needs commit, run, job and attempt`; an unknown `what` names the
    tool's. A member another `what` takes is ignored, as a member no
    decoder reads is.
  - **Each `what` is answered exactly as its former tool was:** the same
    route, renderers, bounds and pages (`../../draft/forge-reads-targeted.md`).
    The former tools' names are decoded too, each as its `what`, but
    declared to no run (`declarations.md`, 4).
  - **Words name a read by its tool and `what`,** as `read_forge
    what=file`: the answers' and refusals' guidance below, the brief's cut
    lines (`room.md`, 2) and the stock texts.
- **A page the answer's bytes cut names how to read on.** A
  `read_issues` `reviews`, `remarks` or `pull_files` page cut at
  `forge.client.answer_bytes` (skein's `forge/README.md`, 3.2 and 3.3)
  ends with `Page cut: read on with page P, skip S.`, its own page and
  the rows read of it, so the next read loses no row; a page that ends
  uncut and has more says `More pages available.`, as today.
- **`read_forge`'s `permission` names the person:** its answer is the permission,
  the forge user's id and the login the forge names them by, as
  `Write; forge user 3, login ada` (skein's `forge/README.md`, 3.3), so that a
  run can match a name to an id; with no login from the forge, the same
  without it. `collaborators` lists ids, for `permission` to
  name.

- **A compare is by names, with no contents.** `read_forge`'s `compare` answers
  how many commits `after` has that `before` lacks and how many it lacks,
  whether `after` contains `before`, and the changed files' paths, at
  most `forge.client.rows` (8 as configured) with how many it left out
  (skein's `forge/README.md`, 3.2 and 3.3). It reads no file's contents, so two
  branches however far apart compare. Its answer, a line each:
  `AFTER is A commits ahead of BEFORE and B behind it.`, then `AFTER
  contains BEFORE.` or `AFTER lacks BEFORE.`, then `Changed files: N`,
  each path it holds, and, where it holds fewer than N, `[K not listed:
  read_issues what=pull_files lists a pull's files; read_forge what=file
  reads a path at each commit]`. A file's contents are `file` at `before` and at
  `after`, a side `Missing` where the file is absent; its description
  says so. A `compare` refusal (a forge that cuts its own listing)
  names its bound as any read's does, says that the commit ids were read
  and that no other id helps, since a run that isn't told retries with
  one it made up, and directs it to `pull_files` or `file`. A
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
  directory reads at both commits. It directs selected `file` reads
  at the head for after and the comparison's base commit for before;
  the base commit must be resolved separately. Advancing pages alone cannot
  recover the oversized page. A file or pull-file forge size refusal, whose
  measured size is unknown, names that uncertainty, the forge bounds with
  their configured figures and the numeric host bound with the same
  tool-specific recovery. A compare's forge size refusal follows the compare
  guidance above. At a host bound too small
  for the guidance, the refusal itself is cut to fit.
- **A refused branch read says what to read instead.** Both refusals
  are right: `branches` takes a nonempty prefix, an empty one
  ending as `Error::Empty` before any request (skein's `forge/forgejo.md`,
  section 2), and `branch` of a name the forge lacks is
  `Error::Missing`. That is so for a goal's own branch where its goal
  holds none (a chat's `goal` names none, 2), and for a run branch
  before its first push (`../forge/branches.md`). `branches`'s
  `prefix` is declared "A nonempty prefix of the names, as temper/."
  Each refusal keeps its first line, `Forge read failed: empty` or
  `Forge read failed: missing`, and goes on after a full stop:
  - `branches`, `Empty`: "read_forge what=branches needs a nonempty
    prefix, as temper/ for temper's branches. read_forge what=settings
    names the default branch, and read_forge what=branch reads one branch
    by its name."
  - `branch`, `Missing`: "The forge has no branch of that name in
    this repository. read_forge what=settings names its default branch,
    and read_forge what=branches lists the names under a prefix, as
    temper/. A goal's
    own branch is there only where its goal holds one, and a run branch
    from its first push."
  - `branches`, `TooLarge`, first line `Forge read failed: too
    large`: "The names under this prefix pass forge.client.answer_bytes,
    the one bound a branch list has: name a longer prefix, as
    temper/goal/ in place of temper/, or read one branch by its name
    with read_forge what=branch." No row bound is named, since none applies to
    branch names (skein's `forge/README.md`, 3.7). The tool's description says
    the same in its words: the names fit forge.client.answer_bytes, and
    a longer prefix reads fewer.

  Every other tool's `Missing` and `Empty` keep their one line. A forge read
  refused as too large names the bound it passed and the size it met, in a
  bound's words (`../forge/change/repairs.md`): `Forge read failed: too large: past
  forge.client.rows (8): 22 rows.`, then how to read on, as its tool's
  description says. Each forge read's description names the bounds its
  answer meets and how to read past them: a listing by its next page, a
  file by path within `max_bytes` (`file`), a pull's files by page
  or a path at the head or the base, a comparison by names with each
  file's contents by path, a branch list by a longer prefix. The refusal
  stays marked an error and is cut to fit the host answer's bound, as
  the compare's is.
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
  - `{"act": "list", "state", "mine"}`, both optional: the project's
    areas, a line each, `NAME (OWNER, HOLDER): WORDS`, its triager's
    line, then its findings newest first, a line each, `P/N STATE KIND
    SEVERITY AREA HOLDER: TITLE`, cut to the answer's bound with `and K
    more`;
  - `{"act": "show", "number"}`: one finding whole, as `temper ctl
    finding show` prints it.

  `HOLDER` is `task N`, `party N` or `role N`, by `../areas.md`, section
  4. Its refusals are `../areas.md`, section 7's words, marked an error,
  and the run goes on. The writes are named and decided once, as
  `change`; `list` and `show` are reads.

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
- **Jobs are smith's** (smith's `domain/tools.md`), not `read_ci`'s `job`,
  which reads a CI job's log. A `shell` command that runs past its
  yield goes on as a job in the agent's process, under the charter's
  `shell` (from `policy.shell`); the run reads or stops it with smith's
  `job`. Neither the engine nor the worker routes or answers a job.
  `temper-engine-smith`'s `turn` takes a turn request alone, and
  answers any other request, a job's included, as not a turn
  (`Problem::Type`).

## 1. A change, in the run's words

The stock chat first reads the code a repository change touches with the
forge's `read_forge what=branch` for its head, then `what=file`. It calls `change`
with the person's request and only what it read, never describing code
it has not read, its words opening with a title line written for a
person (`charter.md`, 4, *Words a person reads first*). Repository changes go through `change`, never through
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

**A person's report has a floor.** jig refuses a report contract bound at
0 words addressed to a person or a role (jig's `domain/people/README.md`,
7), and the run learns it before and after the call:

- **The schema says it:** `delegate`'s `contract` description ends
  `A person's or a role's report: words at least 1.`; its `words` keeps
  `minimum` 0, which an agent's report and a verdict's choices still
  take.
- **The refusal names the bound:** a batch refused as `Refusal::Contract`
  reads `task <N> result contract (words at most the result bound; a
  person's or a role's report at least 1)`, whatever the tool that made
  the batch, so that a run can mend the member it names.

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
  the work, written for a person (`charter.md`, 4, *Words a person reads
  first*). The root's line tells it to read the
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

- **A collaborator's login in `read_forge`'s `collaborators`** (`tools.md`). Default: ids,
  with `permission` naming each.

- **`permission`'s login held to the settings family** (`tools.md`). It
  reaches every run with the forge reads, as the forge's own answer about
  a user of a repository that run may already read; holding it would be a
  condition in `temper-engine-domain`'s render. Default: every such run.

- **When to delegate a task rather than a sub-agent.** A run is declared
  its charter's families' tools alone (`declarations.md`, 1); how a brief
  teaches an LLM to choose a delegated task over a sub-agent of its own is
  open. Default: the charters' words.

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
