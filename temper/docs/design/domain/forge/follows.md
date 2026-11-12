# Follows

Provisional, 2026-11-11, revised 2026-11-12 for a standing role as
owner, from `../../draft/follows.md`, sections 2 to 6.
A project may follow a branch of a repository another project writes:
each landing there that touches the paths it names becomes a change in
a repository of its own, made by code, ordered as the source landed,
owned by a person or a standing role of the project, and checked
against the source by a gate of its own. A follow is close to a tier (`../after-landing.md`,
section 3): both follow a branch's tip; a tier's answer is a verdict, a
follow's a change in another repository. What is still open is section
12; what waits, section 13.

## 1. In one page

- **A follow is a project's setting,** kind 12 (section 2): a source
  (a repository its project adopts as context, a branch, path
  prefixes), a target (a repository it owns, a branch), an owner (a
  person of the project, or one of its standing roles), how its changes
  start (`owner` or `code`), its producer's charter and LLM choices, and
  its check.
- **The forge hears the source's landings** (section 3): every move of
  the source branch's tip, by temper or by anyone, is compared from the
  follow's cursor; a landing that touches the paths, or whose files are
  unknown, joins the follow's queue, kept in the store across a restart.
- **A follow task makes the changes** (section 4): a procedure task of
  the forge's, `follow`, made with the setting, makes one follow change
  per queued landing, in order, each depending on the one before it,
  its words made by code from the landing, and parameter 18 naming the
  landings it follows. A follow change never starts early; under
  `owner` it waits for its owner's word. A standing role's follow task
  is its delegate, so its producers' questions and its changes' holds
  reach the role; a person's is the deployment's.
- **The owner acts** (section 5): `temper ctl follow start`, `hold`,
  `skip` and `merge`, or a standing role's `follow` tool, each applied
  by the follow task's next step, which amends the next follow change's
  dependency in the same decision as a skip's or a merge's cancels, so
  nothing is held for them.
- **The check** (section 6): every follow change carries a gate of its
  own, the follow's workflow dispatched at its head with the followed
  source commit as one more input; red is a repair naming the source's
  tests that fail. A red after landing is filed with the area its
  owner owns, so it reaches the owner.
- **Lag and parity** (section 7): the queue's landings neither followed
  nor skipped, each with its state, and the check's newest verdict, kept
  by code and read through the forge's `follows` document and `temper
  ctl follow list` and `follow show`.
- **Across projects, one deployment** (section 8): a repository one
  project writes is adopted by another as context, and that adoption is
  kept for each project; the source's project is told nothing.

## 2. A follow

A follow is a setting of the target's project, temper's kind 12
(`../../protocol/configuration/settings.md`, 2), at the project's scope,
numbered from 1 to the forge's `follows`, proposed and accepted as
every setting is, its decision that project's owners'. Its value, a
`Follow`:

| Member | What it holds | Absent or empty |
|---|---|---|
| `source` | a repository, by its forge's number and the forge's id for it (`adoption.md`), its project adopting it as context; `branch`, a branch of it; `paths`, at most `follow_paths` path prefixes of whole segments (`src`, `include/vsr.h`), each at most 256 bytes, with no leading, trailing or doubled `/` | `branch`: the repository's default branch as its adoption read it; `paths`: every path |
| `target` | a repository the project owns and temper writes (`adoption.md`); `branch`, the branch its follow changes land into | `branch`: the repository's landing branch |
| `owner` | `person`: a person of the project, by number, with any role in it; or `task`: a standing role of the project (`../agent/standing.md`), by its task's number, a live task of the project labelled 3 | refused |
| `start` | `owner`, each follow change waiting for its owner's word; or `code`, each starting once the one before it has landed | `owner` |
| `producer` | the charter and LLM choices a follow change runs its producer under, spelled as `change`'s parameter 11 is (`change/README.md`, 1) | the forge's change charter and the charter's defaults |
| `check` | `workflow`, a workflow file of the target, as a tier names one; `artifact`, the name of the artifact its run uploads; `format`, `junit` or `lines` (`../after-landing.md`, 2) | refused |

- **Its form in a settings file,** binding for `temper ctl` and the
  host tools' settings (`../../protocol/configuration/settings.md`, 2):
  ```json
  {"source": {"forge": 1, "repository": 12, "branch": "main", "paths": ["src", "include"]},
   "target": {"forge": 1, "repository": 13, "branch": ""},
   "owner": {"person": 4}, "start": "owner", "producer": "6;3.1;",
   "check": {"workflow": ".forgejo/workflows/vsr-tests.yml", "artifact": "results", "format": "junit"}}
  ```
  `branch`, `paths`, `start` and `producer` may be left out, as the
  table's last column says. A standing role's owner is `"owner":
  {"task": 52}`, spelled as an area's `task` owner is
  (`../../protocol/configuration/settings.md`, kind 14), never `role`,
  which there names a role of the project's people.
- **In temper's codec** (binding for the stored setting, family 20, and
  the client's wire, whose `Follow` the `follows` document carries):
  `FollowOwner` is `Person(FollowPerson { person: u64 })`, tag 0, or
  `Task(FollowOwnerTask { task: u64 })`, tag 1; the engine's
  `settings::FollowOwner` is `Person(u64)` or `Task(u64)`. An added
  variant: every stored follow still decodes.
- **Judged as any kind is:** the forge's structural judgement first, then
  the root's checks across domains (`change/gates.md`, 3), each refusal
  naming the follow and what fails, and the whole change refused with
  it:
  - the source not adopted by the project as context: `follow N: its
    source FORGE/REPOSITORY is not adopted by project P as context`;
  - the target not a repository the project owns and temper writes;
  - a `person` owner not a person of the project: `follow N: its owner
    X is not a person of project P`; a `task` owner not a live task of
    the project labelled 3: `follow N: its owner task T is not a live
    standing role of project P`;
  - paths past `follow_paths`, or one malformed;
  - a producer that doesn't spell parameter 11, or a charter the
    settings don't offer;
  - a check with an empty member, or one past 256 bytes;
  - a source or a target other than an existing follow's: `follow N: its
    source and target are fixed; remove it and add another`.
- **Accepted,** in the decision that keeps it, the forge saves the
  follow's record (section 9) and asks for its task (section 4). An edit
  of its start, producer, paths or check, or of its owner from one
  person to another, applies from the follow task's next step: the
  queue keeps what it held, a new path list keeps the landings heard
  after it, and a check names the workflow its changes dispatch after
  it. An edit of its owner that changes who requests its task, a person
  to a standing role, a role to a person or one role to another, makes
  a new follow task (section 4, *A new follow task*). A removal cancels
  its task, and jig cancels its live follow changes with it (jig's
  `domain/tasks/changes.md`); the record goes in that decision.
- **A standing role as owner** (`../../draft/standing-roles.md`, section
  2): its follow task is its delegate (section 4, *Under a standing
  role*), it acts with its `follow` tool (section 5), and a red after
  landing reaches it by its area (section 6). A role that ends takes its
  follow task with it, and the follow stops until an edit names another
  owner (section 4).

## 3. What the forge hears

- **The source branch is watched** for as long as a follow names it,
  beside tiers' branches, landing subscriptions and live tasks' names,
  in the one union the working set admits (`keeping-up.md`, 1).
- **The follow's cursor,** `heard`: the source commit up to which its
  landings are queued. The first read after the follow is accepted sets
  it at the branch's tip, the baseline: nothing before it is queued.
- **Each move of the source branch's tip** the forge sees, by temper's
  merge, by a landing's turn, or by anyone (every path that tells tiers
  of a move: `../after-landing.md`, section 3), starts, for each follow
  of that branch with no read in flight, a read from `heard` to the new
  tip: the files the range touched (skein's `Read::Compare`, names alone,
  read as `README.md`, 5.3 reads a landing's), and its commits with their
  messages, oldest first (`Read::Commits`). A move while a read is in
  flight is taken by the next read, from the cursor the first left.
- **One move is one landing,** `before` the cursor, `after` the tip read:
  what the forge saw move at once, as a tier's range is. It is kept when
  one of its files lies under one of the follow's paths (`README.md`,
  5.3's prefix rule: `src` covers `src/a.c`, not `src-old/a.c`), when the
  follow names no paths, or when its files are unknown: a comparison
  that names fewer paths than it counts, an unusable answer or a failed
  read, as 5.3's overlap is unknown. A follow change for nothing costs
  its owner a skip; a missed one costs parity.
- **Kept, it joins the queue** in the decision that takes the read's
  answers, and the cursor moves to `after` in that same decision; not
  kept, the cursor moves alone. An entry holds `before`, `after`, its
  commits (at most `follow_commits`, each an id and its message cut to
  `follow_message_bytes`, the rest counted), its files under the paths
  (at most `follow_files`, the rest counted) or unknown, and, where
  `after` is a merge temper made in the source, the change's pull
  request number from its `Landed` row.
- **Across a restart:** the record keeps the cursor and the queue; a
  read in flight is lost and asked again from the cursor, which moved
  only with what it queued. So no landing is lost, and none is queued
  twice: an entry is known by its `after`.
- **A full queue,** `follow_queue` entries, reads nothing more: the
  cursor stays, and when an entry leaves, the next read covers every
  move since as one landing.

## 4. The follow task and its changes

- **The follow task** is a procedure task of the forge's, `follow`,
  procedure 7, made when the follow is accepted, as a tier's run is
  (`../after-landing.md`, section 10). A person's follow's is requested
  by the deployment on the target project's behalf, with the project's
  policy ceiling, and funded as a queue repair's change is
  (`queues.md`), its spend the least of the policy's, its period's left
  and the deployment's run spend; a standing role's is the role's
  (*Under a standing role*, below). Its words: `Follow N of project P:
  SOURCE BRANCH into TARGET BRANCH`, each repository as `OWNER/NAME`.
  It never ends by itself: its setting's removal cancels it. One
  cancelled any other way, its role's end among them, leaves its follow
  stopped, its record saying so, and the next accepted edit of the
  setting makes a new one. Its record names it.
- **Under a standing role:** the follow task is the role's delegate,
  made by the root in a decision of its own with the role as its
  requester and its funder, as a goal's closer is its goal's
  (`branches.md`, section 2): its authority the role's, one level less
  deep; its spend the least of what a person's follow task would get
  (above) and the role's available budget. So its follow changes stand
  below the role, their questions reach it through the follow task
  (*Questions*, below), and their holds reach it as escalations,
  passing over the change and the follow task, procedures both (jig's
  `domain/tasks/proposals.md`). The follow's spend is the role's:
  held for want of spend (*Its steps*, below), the follow task's hold
  reaches the role, which may amend its budget from its own, refilled
  each period, and release it, or pass it to its person. A make jig
  refuses (the role's room, depth or funding) leaves the follow
  stopped, its record saying `its owner, task T, could not make its
  task: WHY`, and the next accepted edit asks again.
- **Its steps** are due whenever its record changes (an entry queued, an
  act taken, a delegate's end, hold, question or answer) and make what
  the record says is due: jig's procedure step, its batch, amendments,
  cancels and messages one decision (jig's `domain/connectors.md`, 6.6).
  A step that can't make a follow change for want of spend holds the
  follow task, saying so, for the project's escalation role.
- **One follow change per entry,** made in queue order: a change
  (`change/README.md`, 1) into the target's branch, depending on the
  follow change before it while that one lives (jig's
  `domain/tasks/lifecycle.md`, 8), on none when none does. Its
  parameters: 1, the target; 2, its branch; 11, where the follow names a
  producer; and 18, the landings it follows. It carries no parameter 14:
  parameter 18 makes its question route `up`, to the follow task, and
  keeps it within small's four parameters a task.
- **Parameter 18,** bytes, `N:COMMIT[,COMMIT]...`: the follow's number in
  decimal, a colon, then the `after` of each landing the change follows,
  oldest first, each 40 or 64 lowercase hexadecimal digits, at most
  `follow_merge`. The root checks it as it decides the delegate, as it
  checks `repairs` (`../after-landing.md`, section 7): the requester is
  that follow's task, in the change's project; each commit is an entry
  of that follow's queue, in its order, neither followed nor skipped,
  and named by no other live follow change. A refusal names the commit
  and why. `change` has no member for it: a model makes no follow
  change.
- **What parameter 18 does to a change:**
  - its start keeps it, the follow's number and its commits, with the
    change (`follows`), and a value outside the text form refuses the
    start as a malformed parameter;
  - **it never starts early:** it produces once its dependencies are
    done, never from a prerequisite's pushed head (`change/questions.md`,
    1, *Early starts*), so no dependency of a follow change is ever
    marked as started from, and amending it is always open;
  - under `owner`, it produces once its owner's word starts its entry
    (section 5); until then its panel says `waits for its follow's
    owner`, or, held by its owner, `held by its follow's owner: REASON`;
  - it carries its follow's check (section 6);
  - **its landing follows them:** in the decision that records it
    landed, its entries leave the queue, and the follow change after it
    has its dependency met.
- **Its words, made by code,** never by a model (binding):
  ```
  Follow SOURCE AFTER12: SUBJECT

  Port landing AFTER on SOURCE BRANCH (from BEFORE) into TARGET TBRANCH, for follow N of this project.
  Pull request: URL
  Compare: URL
  Diff: in the read-only checkout of SOURCE, `git diff BEFORE AFTER -- PATH...`

  Commits:
  - ID12 FIRST LINE
    THE REST OF ITS MESSAGE
  and K more commits

  Files in the follow's paths:
  - PATH
  and K more files
  ```
  SOURCE and TARGET are `OWNER/NAME`, AFTER12 and ID12 twelve digits,
  SUBJECT the newest commit's first line, the title line cut at 120
  bytes with `…`. `Pull request:` appears where the entry names one;
  `Compare:` is the forge's page of `BEFORE...AFTER` on the source; the
  `-- PATH...` part only where the follow names paths. Unknown files
  read `Files: unknown; the forge's comparison was incomplete.` A change
  following several landings has one block from `Port landing` on for
  each, oldest first, and the title `Follow SOURCE AFTER12 and K more:
  SUBJECT`, AFTER12 the newest. The whole is fitted to the deployment's
  `tasks.spec_bytes` less its parameters' bytes: the messages' rest
  cut first, then the file lists, each cut marked `…`; the head lines
  stay whole.
- **The producer reads the diff** in its workspace, where the source is
  checked out read only beside the target (section 8): a follow
  change's producer runs in the target's project, whose context
  repositories every working run gets (`README.md`, section 2).
- **Questions:** a follow change's agents' questions reach the follow
  task as its requester (`change/questions.md`, 1), and its next step
  passes each to the owner:
  - *a person,* as a person task addressed to them (jig's
    `domain/people/README.md`, section 7), its words `Follow N's change
    M asks (question Q): WORDS`; when that task ends, the step answers
    the change's question Q with its result's words, or, cancelled or
    failed, with `The follow's owner gave no answer.`;
  - *a standing role,* its requester, in the question form, as a goal's
    closer relays its change's (`branches.md`, section 2): its words
    `Follow N's change M asks (question Q):`, a blank line, then the
    question's words whole, cut from their end to the deployment's
    `tasks.message_bytes` and marked `…`. The role is woken by it as by
    any question and answers it as it answers its own work's. Its
    answer, in the answer form to the follow task's question, is taken
    at the step that reads it and relayed to change M in the answer
    form to Q, its words whole, cut and marked as above; the role's
    words answer nothing. Several stand at once, one per asking change,
    each kept on the follow's record with the number of the follow
    task's own question until it is answered. A change that ends with
    its question relayed has it dropped from the record, and the role's
    later answer to it is taken and dropped.
- **Holds:** a follow change held (its repairs past their bound, a review
  failed twice, its stall) holds those after it, which depend on it.
  jig's escalation passes over the change and the follow task,
  procedures, to the nearest agent above them (jig's
  `domain/tasks/proposals.md`): under a standing role, the role, which
  may release, amend or cancel the held change, or pass it to its
  person; under a person, the project's escalation role, as for any
  change a deployment's task made. The follow task hears the hold as
  its requester, and its entry keeps the hold's account for the lag.
- **An end without landing:** a follow change failed, or cancelled by
  anyone but its follow, leaves its landings in the queue, its entry
  `ended without landing: WORDS`, its result's first line; jig holds the
  change after it for its dependency (jig's `domain/tasks/lifecycle.md`,
  6), and the owner starts the entry again or skips it (section 5). The
  follow never releases that hold: a procedure releases none (jig's
  `domain/connectors.md`, 6.3). None of the follow changes after the
  ended one has produced, as each waits on it, so the act that starts
  or skips the ended entry remakes them: *the tail made again*, below.
- **The tail made again:** the step cancels every follow change after
  the ended entry, `made again after change M ended`, and marks their
  entries, with the ended one when it is started, to be made again, each
  keeping its owner's word and hold; the same step makes them as above,
  in queue order, each depending on the one made before it and the
  first on the live follow change before them, or on none (a step a
  batch's bound stops leaves the rest to the next). So no follow change
  stays held for an ended dependency, no hold is released, and no entry
  has two live changes.
- **A new follow task,** made by an accepted edit after its follow
  stopped, or by an edit of its owner that changes who requests it
  (section 2), in which case the old task, still live, is cancelled in
  the decision that saves the edit, `its owner changed`, and jig cancels
  its live follow changes and person tasks with it. The new task's
  steps make the queue's follow changes again, in queue order, up to
  the first entry ended without landing: each entry whose change is no
  longer live, depending on the one made before it and the first on
  none, keeping its owner's word and hold, as *the tail made again*
  does. An entry ended without landing stays so, and those after it are
  made when its owner starts or skips it (*the tail made again*). The
  old task's questions are dropped from the record: their changes,
  cancelled, ask no more. A change that was producing starts over.

## 5. The owner's acts

- **Who:** a person owner, or an owner of its project, through `temper
  ctl` (`../../client/ctl/tools.md`, 3), each a keyed request of the
  forge's family answered once durable; a standing role owner through
  its `follow` tool (below). The root checks the person's role in the
  project, or that the calling task is the follow's owner; the forge
  checks the act against the follow's record and keeps it there in the
  decision that takes it; the follow task's next step, due at once,
  applies it. The acts below are written as `temper ctl` takes them;
  the tool takes the same.
- **`follow start --project P FOLLOW COMMIT`:** the entry whose `after`
  is COMMIT may produce once its dependency is done: its owner's word,
  under `owner`, or the end of its owner's hold, under either. On an
  entry ended without landing, the step makes the entry's follow change
  again, depending on the change before it, and the tail after it made
  again (section 4) in the same decision; no hold is released.
- **`follow hold --project P FOLLOW COMMIT --reason TEXT`:** an entry
  whose change hasn't started producing waits, and the lag shows its
  reason; `start` lifts it.
- **`follow skip --project P FOLLOW COMMIT --reason TEXT`:** an entry not
  followed counts as followed, and the follow keeps it among its skips
  with its reason. Its change, if made, is cancelled with the reason, and
  the follow change after it is amended to depend on the change before
  the skipped one, or on none, in the same step. A skipped entry ended
  without landing has the tail after it made again instead (section 4).
- **`follow merge --project P FOLLOW COMMIT COMMIT...`:** two to
  `follow_merge` consecutive entries, none followed, none skipped, none of
  their changes producing: the first's change is amended to follow them
  all, its parameter 18 naming each and its words each one's block; the
  others' changes are cancelled, `merged into change M`; and the follow
  change after the last is amended to depend on M. A role that waits for
  the source's next fix to the same functions, to port both together,
  holds the first and merges.
- **The chain stays whole:** a step's amendments apply before its
  cancels (jig's `domain/tasks/changes.md`), so a skip's or a merge's
  cancelled changes are no one's dependency by the time they end, and no
  follow change is held for them.
- **Refused,** naming the entry and why: a commit no entry has, one
  followed or skipped, a started change for `hold` or `merge`, an entry
  ended without landing for `merge` (start or skip it first), entries
  not consecutive or past `follow_merge`, a reason past
  `follow_reason_bytes`, a person neither the owner nor an owner of the
  project, a task not the follow's owner: `follow N is not yours: its
  owner is OWNER`, OWNER `person X` or `task T`.
- **The role's `follow` tool** (`../agent/tools.md`), of the delegate
  family, declared where `finding` is, so that a standing role acts on
  the follows it owns in its task's project. One tool, its `act` saying
  which:
  - `{"act": "start", "follow", "commit"}`, `{"act": "hold", "follow",
    "commit", "reason"}` and `{"act": "skip", "follow", "commit",
    "reason"}`; `{"act": "merge", "follow", "commits"}`, two to
    `follow_merge` commits in queue order; each commit 40 or 64
    lowercase hexadecimal digits, as `temper ctl` takes it. Taken, the
    answer is `Kept: follow N's next step applies it.`; refused, the
    refusal's words above, marked an error, and the run goes on;
  - `{"act": "list"}`: the project's follows, a line each, section 7's
    `follow list` line, cut to the answer's bound with `and K more`;
  - `{"act": "show", "follow"}`: one follow whole, as `temper ctl follow
    show` prints it.

  The project is the calling task's; the acts are its owner's alone, and
  `list` and `show`, reads, are open to any task of the project. The
  writes are named and decided once, as `finding`'s are.

## 6. The check

- **A gate of the change's own** (`change/gates.md`, 1), carried by every
  follow change: blocking, exact, run once CI passes at the head, as
  gates are, and judged on the change under the queue and under the turn
  alike (`../landing.md`, section 3).
- **Dispatched by the change,** as a gate is (`../landing.md`, 7.1), on
  its own branch at its head: the follow's `workflow`, with the inputs
  `temper_commit`, the head, `temper_key`, its key in hexadecimal, and
  `temper_follows`, the newest commit the change follows, its last in
  parameter 18, as 40 or 64 lowercase hexadecimal digits. The run is
  found and followed as there.
- **Its key** is `../landing.md`, 6.1's, its gate's name `follow.N`, its
  definition the check's three members, its tree the head's whole, and
  the followed commit's 32 bytes (a SHA-1 one padded) after the
  definition's, so two followed commits never share a receipt. Its
  receipt is kept in `Receipt` and reused at the same key (`../landing.md`,
  6.2).
- **Its report** is a tier's run's (`../after-landing.md`, section 4):
  passed, or its failing tests by id from the artifact; a failed run that
  names none, or an artifact absent or unreadable, is a fault.
- **Passed:** the gate approves at that head.
- **Failed:** a repair (`change/repairs.md`), its remarks `The follow's
  check failed at HEAD12 against SOURCE AFTER12: K of the source's tests
  fail on the target.`, then each test, one per line (`many` past
  `failing_tests`), then `Run: URL`. A change that keeps failing holds
  on its repairs' bound, and its owner reads it in the lag (section 4).
- **A fault** dispatches it again, up to the project's `tier_retries`;
  past them the change holds, `follow check held after N runner faults,
  last: KIND`.
- **Merged,** the followed commit is the new last, so a verdict at the
  head for an older one no longer counts.
- **After landing,** the target's tiers follow its branch as any
  repository's (`../after-landing.md`, section 3). A follow change has
  no goal, its requester a procedure under the deployment or a standing
  role, neither tracked, so a red its landing raises is a red of no
  goal's, filed by the root as a finding (`../after-landing.md`,
  section 7; `../findings.md`, section 4).
- **Routed to the follow's owner by its area:** that finding is filed
  with the follow's owner's area, not with none, so that it is the
  owner's at once, with no agent's run, and the owner hears it as
  `../areas.md`, section 5 says, a role as news, a person in Needs you:
  - *the follow* is the one named by the `follows` of the landing's
    member changes whose rows the forge still keeps, the lowest-numbered
    change first;
  - *its owner's area* is the first of the project's areas (kind 14), in
    their order, whose owner stands and gives the follow's owner as its
    holder (`../areas.md`, sections 3 and 4): `Holder::Task(T)` for a
    `task` owner, `Holder::Party(X)` for a `person` one;
  - *with none,* where no member's row names a follow, or the owner owns
    no area that stands, the finding is filed with none, for the
    project's triager, as any red of no goal's is.

  The finding is otherwise as `../findings.md`, section 4 files a red of
  no goal's: its kind, severity, title, body and once per landing.
- **Proof stays in tests:** code compares no follow change's diff with
  the source's; the check and the change's reviews are the proof.

## 7. Lag and parity

- **Lag:** the queue's entries, oldest first, each with its `after`, its
  change, and its state: `not made yet`; `waits for its owner's word`;
  `held by its owner: REASON`; `waits for change M`; `producing as change
  M`; `held: ACCOUNT`, the change's hold; or `ended without landing:
  WORDS`. Their count is the lag, 0 when caught up. A followed entry
  leaves the queue; a skipped one leaves it for the follow's skips, its
  last `follow_skips` kept with their reasons.
- **Parity:** the newest verdict of the follow's check, kept on its
  record: the source commit it was given, the target's head, passed or
  `K failing` (`many` past `failing_tests`), and its run's link; none
  before the first.
- **Kept by code,** in the decisions that change them, never by a model.
- **Read through the forge's `follows` document,** a project's, which its
  members may read: each follow, its setting, its lag's entries, its
  skips, its parity and its task. `temper ctl follow list --project P`
  prints a line a follow (binding):
  `follow N: SOURCE BRANCH → TARGET TBRANCH · LAG · PARITY`, LAG `caught
  up` or `K behind`, PARITY `check passes at AFTER12`, `K failing at
  AFTER12` or `no check yet`; `follow show --project P N` prints one
  whole (`../../client/ctl/tools.md`, 2). Home's, Team's and How work
  runs' lines are drawn from that document (`../../draft/surfaces.md`,
  sections 3, 6 and 7): Team's on the owner's row, and on a standing
  role's card (`../../client/pages/team.md`, 1, *Follows*).
- **The project's dependencies** ride in the same document, after its
  follows, and `follow list` prints a line each after the follows'
  (`dependencies.md`, section 4).

## 8. Across projects

- **One deployment:** the source is a repository the target's project
  adopts as context (`adoption.md`), on any forge the deployment keeps:
  another project's owned repository, or one no project writes.
- **A context adoption is each project's.** A repository one project
  writes keeps its writer's row, and beside it each other project that
  adopts it as context, saved with it, within `judge_projects`. Every
  lookup by project (its repositories, a tag it names, its runs' context
  checkouts, its subscriptions' admission) answers such a repository
  with the context role: reads alone. Adopting it as context again is
  idempotent; removing the writer's adoption or the forge removes them.
- **Read as context,** so a follow change's producer and its owner's
  runs check the source out read only beside the target
  (`README.md`, section 2, *Workspace items*).
- **Heard for its follows:** the forge watches the source branch for as
  long as a follow names it (section 3), whether or not anything else of
  its project's does.
- **Who makes one:** an owner of the target's project, by accepting a
  proposal of kind 12. The source's project is told nothing and gives
  nothing: reading is all a follow does there.
- **Refused:** a follow of a repository its project hasn't adopted as
  context (section 2).

## 9. Effects, reads and the store

- **Procedure 7, `follow`:** its steps make no forge effect of their own;
  its delegates are change tasks (procedure 2) and person tasks for its
  questions. Its authority is the project's ceiling for a person's
  follow and the role's for a standing role's (section 4), and each
  follow change's is made from it as `change` makes a change's.
- **Reads:** the forge's own `Read::Compare` and `Read::Commits` on the
  source, as it reads a landing's files (`README.md`, 5.3), under no
  task's grant.
- **The check's dispatch** is the change's: kind 16 on the target, a
  rerun kind 17 (`../landing.md`, section 11), a purpose of the change's
  step (tag 2, `README.md`, section 4).
- **In the store,** family 42, `Follow`, keyed by the project and the
  follow's number: the follow's source repository and branch, its
  cursor, its task, whether it is stopped and why, its queue (each
  entry's `before`, `after`, pull request, commits, files or unknown,
  change, state and the act it waits on), its skips with their reasons,
  its parity, and its open questions (each the asking change, its
  number for the question, its words, and the person task asked or,
  under a standing role, the follow task's own question to it, with the
  answer once given); at most `follows` per project, its queue within
  `follow_queue`. Saved in the decisions section 3 to 7 name, erased with
  its setting's removal or its project's (`../../protocol/store.md`,
  section 2).
- **A change's `follows`,** the follow's number and its commits, are kept
  with its `Change` row, as `repairs` is.
- **Kind 12's values** are kept in family 20, `Setting`, as the forge's
  other kinds are.

## 10. Limits

The forge's (`../../protocol/configuration/limits.md`), `small` then
`standard`: `follows` per project, 2 and 8; `follow_paths` per follow,
8; `follow_queue`, 16 and 64; `follow_merge`, landings per follow change,
8; `follow_commits` per entry, 8; `follow_files` per entry, 32;
`follow_message_bytes` per commit, 512; `follow_skips` kept, 8;
`follow_reason_bytes`, 256.

## 11. The world

Focused stories in temper's domain world (`../../testing/README.md`,
section 4) with the fake forge and the fake model, two projects on it:
the source's owning its repository, the target's owning its own and
adopting the source's as context, and a follow owned by a person:

- a landing in the source touching the paths makes one follow change
  naming it, its message and files in its words; one outside the paths
  makes none; one whose files are unknown makes one;
- two landings make two follow changes, the second depending on the
  first; under `owner`, neither produces before its word; the first
  held by its owner, the lag is 2 with its reason;
- the owner merges two into one change naming both, and skips one with
  a reason: each counts as followed;
- a merge and a skip in the middle of a chain of four: the next change's
  dependency is amended in the same decision, and none is held for a
  cancelled dependency;
- a follow change's producer's question reaches the owner as a person
  task, and their answer reaches the producer; nothing reaches the
  source's project;
- the check is dispatched with the followed commit; red, the change
  repairs with the failing tests named; green, it lands and the lag
  falls to zero, parity green;
- the engine stopped and started with three in the queue: none lost,
  none made twice;
- a follow of a repository its project hasn't adopted as context is
  refused; the second project's context adoption of the first's
  repository is read by its runs and its follow.

And with a follow owned by a scripted standing role, started from
template 7 (`../agent/standing.md`, section 2):

- a `task` owner that is no live standing role of the project is
  refused, in section 2's words;
- the follow task is the role's delegate, funded by it; a follow
  change's producer's question reaches the role in the question form,
  and the role's answer reaches the producer;
- a held follow change escalates to the role, which releases it;
- the role starts, holds, skips and merges with its `follow` tool, each
  applied as the same act through `temper ctl` is, and another task's
  act is refused, `follow N is not yours`;
- the owner edited from a person to the role, with three in the queue,
  one producing: a new follow task under the role remakes their
  changes, each keeping its owner's word, none held for a cancelled
  dependency;
- a red after landing of a follow change is filed with the area the
  role owns and reaches it as news with no agent's run; with no area of
  the role's, it is filed with none, for the triager.

Fuzzy: random runs of source landings, in and out of the paths or with
unknown files, the owner's starts, merges, holds and skips, reds at the
check, restarts, and edits of its owner between a person and a
standing role. Always: each landing in the paths is followed or
skipped once, never twice; follow changes land in the source's order;
the lag equals the landings neither followed nor skipped; no follow
change is held for a cancelled dependency; no entry has two live
changes.

## 12. Open questions

- **The owner's own hands:** whether a role's session may produce a
  small follow change itself (`../../draft/follows.md`, section 10).
- **Holds and a person owner:** whether the follow tells a person owner
  of each hold of its changes, beside the project's escalation role that
  jig's escalation reaches (section 4); a standing role owner hears them
  as escalations.
- **The follow task's budget** is given once, at its making: under a
  standing role, the role tops it up by amendment from its own budget,
  refilled each period (section 4); a person's follow task refilled
  each period, as a standing task's budget is, is open.
- **A standing role's follow across a restart of the role:** a role
  started again under the same charter is a new task, so its follows
  name the old one and stop with it; an owner named by charter, as an
  area's `charter N` is (`../areas.md`, section 3), would keep them.

## 13. Later

- **A source on another deployment,** or on a forge the deployment doesn't
  keep, read through its public reads alone.
- **A follow started at an old commit,** with the history before it as a
  backlog.
