# Follows

Provisional, 2026-11-11, from `../../draft/follows.md`, sections 2 to 6.
A project may follow a branch of a repository another project writes:
each landing there that touches the paths it names becomes a change in
a repository of its own, made by code, ordered as the source landed,
owned by a person of the project, and checked against the source by a
gate of its own. A follow is close to a tier (`../after-landing.md`,
section 3): both follow a branch's tip; a tier's answer is a verdict, a
follow's a change in another repository. What is still open is section
12; what waits, section 13.

## 1. In one page

- **A follow is a project's setting,** kind 12 (section 2): a source
  (a repository its project adopts as context, a branch, path
  prefixes), a target (a repository it owns, a branch), an owner (a
  person of the project), how its changes start (`owner` or `code`),
  its producer's charter and LLM choices, and its check.
- **The forge hears the source's landings** (section 3): every move of
  the source branch's tip, by temper or by anyone, is compared from the
  follow's cursor; a landing that touches the paths, or whose files are
  unknown, joins the follow's queue, kept in the store across a restart.
- **A follow task makes the changes** (section 4): a procedure task of
  the forge's, `follow`, made with the setting, makes one follow change
  per queued landing, in order, each depending on the one before it,
  its words made by code from the landing, and parameter 18 naming the
  landings it follows. A follow change never starts early; under
  `owner` it waits for its owner's word.
- **The owner acts** (section 5): `temper ctl follow start`, `hold`,
  `skip` and `merge`, each applied by the follow task's next step, which
  amends the next follow change's dependency in the same decision as a
  skip's or a merge's cancels, so nothing is held for them.
- **The check** (section 6): every follow change carries a gate of its
  own, the follow's workflow dispatched at its head with the followed
  source commit as one more input; red is a repair naming the source's
  tests that fail.
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
| `owner` | `person`: a person of the project, by number, with any role in it | refused |
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
  table's last column says.
- **Judged as any kind is:** the forge's structural judgement first, then
  the root's checks across domains (`change/gates.md`, 3), each refusal
  naming the follow and what fails, and the whole change refused with
  it:
  - the source not adopted by the project as context: `follow N: its
    source FORGE/REPOSITORY is not adopted by project P as context`;
  - the target not a repository the project owns and temper writes;
  - the owner not a person of the project;
  - paths past `follow_paths`, or one malformed;
  - a producer that doesn't spell parameter 11, or a charter the
    settings don't offer;
  - a check with an empty member, or one past 256 bytes;
  - a source or a target other than an existing follow's: `follow N: its
    source and target are fixed; remove it and add another`.
- **Accepted,** in the decision that keeps it, the forge saves the
  follow's record (section 9) and asks for its task (section 4). An edit
  of its owner, start, producer, paths or check applies from the follow
  task's next step: the queue keeps what it held, a new path list keeps
  the landings heard after it, and a check names the workflow its
  changes dispatch after it. A removal cancels its task, and jig
  cancels its live follow changes with it (jig's
  `domain/tasks/changes.md`); the record goes in that decision.
- **A standing role as owner** joins with standing roles
  (`../../draft/standing-roles.md`, section 2): until then a follow's
  owner is a person.

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
  (`../after-landing.md`, section 10): requested by the deployment on the
  target project's behalf, with the project's policy ceiling, and funded
  as a queue repair's change is (`queues.md`), its spend the least of
  the policy's, its period's left and the deployment's run spend. Its
  words: `Follow N of project P: SOURCE BRANCH into TARGET BRANCH`, each
  repository as `OWNER/NAME`. It never ends by itself: its setting's
  removal cancels it. One cancelled any other way leaves its follow
  stopped, its record saying so, and the next accepted edit of the
  setting makes a new one. Its record names it.
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
  task as its requester (`change/questions.md`, 1). Its next step passes
  each to the owner, as a person task addressed to them (jig's
  `domain/people/README.md`, section 7), its words `Follow N's change M
  asks (question Q): WORDS`; when that task ends, the step answers the
  change's question Q with its result's words, or, cancelled or failed,
  with `The follow's owner gave no answer.`
- **Holds:** a follow change held (its repairs past their bound, a review
  failed twice, its stall) holds those after it, which depend on it.
  jig's escalation passes over the follow task, a procedure, to the
  project's escalation role (jig's `domain/tasks/proposals.md`), as for
  any change a deployment's task made; the follow task hears the hold
  as its requester, and its entry keeps the hold's account for the lag.
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

## 5. The owner's acts

- **Who:** the follow's owner, or an owner of its project, through
  `temper ctl` (`../../client/ctl/tools.md`, 3), each a keyed request of
  the forge's family answered once durable. The root checks the
  person's role in the project; the forge checks the act against the
  follow's record and keeps it there in the decision that takes it; the
  follow task's next step, due at once, applies it.
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
  project.
- **A role's tool** for the same acts joins with standing roles
  (`../../draft/standing-roles.md`, section 3).

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
  repository's (`../after-landing.md`, section 3), and a red there is a
  finding as any (`../findings.md`, section 4); routing it to the follow's
  owner by area joins with standing roles (`../../draft/standing-roles.md`,
  section 4).
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
  sections 3, 6 and 7).

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
  questions. Its authority is the project's ceiling, and each follow
  change's is made from it as `change` makes a change's.
- **Reads:** the forge's own `Read::Compare` and `Read::Commits` on the
  source, as it reads a landing's files (`README.md`, 5.3), under no
  task's grant.
- **The check's dispatch** is the change's: kind 16 on the target, a
  rerun kind 17 (`../landing.md`, section 11), a purpose of the change's
  step (tag 2, `README.md`, section 4).
- **In the store,** family 41, `Follow`, keyed by the project and the
  follow's number: the follow's source repository and branch, its
  cursor, its task, whether it is stopped and why, its queue (each
  entry's `before`, `after`, pull request, commits, files or unknown,
  change, state and the act it waits on), its skips with their reasons,
  and its parity; at most `follows` per project, its queue within
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

Fuzzy: random runs of source landings, in and out of the paths or with
unknown files, the owner's starts, merges, holds and skips, reds at the
check, and restarts. Always: each landing in the paths is followed or
skipped once, never twice; follow changes land in the source's order;
the lag equals the landings neither followed nor skipped; no follow
change is held for a cancelled dependency.

## 12. Open questions

- **The owner's own hands:** whether a role's session may produce a
  small follow change itself (`../../draft/follows.md`, section 10).
- **Holds and a person owner:** whether the follow tells its owner of each
  hold of its changes, beside the project's escalation role that jig's
  escalation reaches (section 4).
- **The follow task's budget** is given once, at its making; a refill
  each period, as a standing task's, joins with standing roles
  (`../../draft/standing-roles.md`, section 9).

## 13. Later

- **A standing role as owner,** its tool for the owner's acts, and its
  questions passed to it as words (`../../draft/standing-roles.md`,
  sections 2 and 3).
- **A source on another deployment,** or on a forge the deployment doesn't
  keep, read through its public reads alone.
- **A follow started at an old commit,** with the history before it as a
  backlog.
