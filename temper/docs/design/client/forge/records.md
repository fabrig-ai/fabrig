# The forge's records

`temper-client-protocol-wire`, generated from one schema (jig's
`protocol/client/wire.md`, section 1, and `jig/docs/design/protocol/client/README.md`, section 8):

## 1. The family
- **`FORGE`:** the forge connector's number, 1, which is the family's
  owner number on the wire, under which its records travel as bytes
  (jig's `protocol/client/README.md`, section 8), and the engine's one
  configured connector (`engine.connectors`,
  `../../protocol/configuration/README.md`, section 2).
- **`Owner`:** temper's owners, one enum the engine's root and the
  client root share, with the functions from an owner to its number and
  back (jig's `client/domain/root.md`, section 15): `Forge`, `FORGE`'s
  number; `Pages`, temper's pages, 2; and `Runner`, the agents' tools'
  runner (`../ctl/README.md`, section 4), 3: numbers no connector takes, never
  reused. `Pages` and `Runner` ask only for jig's operations, reads and
  subjects, so no request, read or subject of their own reaches the
  engine.
- **`CLIENT_VERSION`:** the version of every record below, the number
  the schema's `family` line names. It moves with the family's layout,
  at the landing that changes it: a request, an outcome, a read, a
  subject or a field added to the family (the FORGE requests of
  `../engine.md`, the picker's ids in `Adopt`, a change's panel) changes
  its schema and generated records, and moves its
  version by one, its layouts' list taking the new version's line;
  `PUBLIC_VERSION` does the same for the visitors' records (jig's
  `protocol/client/versions.md`, section 1).
  The engine serves exactly one version (jig's `protocol/client/versions.md`, section 1): a client of another
  is out of date for the forge (`family.md`, 4), and its kept adoption is dropped
  as kept work at start (`../line.md`, 6; jig's `protocol/client/small-store.md`, section 1).
## 2. Requests

Moved to [`records-requests.md`](records-requests.md), section 1.

## 3. Deployment reads

Moved to [`records-deployment.md`](records-deployment.md), section 1.

## 4. A change’s panel
- **A change's panel, the subject `change`** (`served.md`, 2): of a
  project's scope, its window unused, naming the change task's number
  (u64). On the wire it is `Watching`'s `change`, `Watching` naming
  which of the family's subjects a watch is, as `Reading` names a read.
  Its document, `ChangePanel`:
  - the task (u64); the repository's forge's number (u16), its owner
    and name; the change's branch and its landing branch (byte strings,
    each bounded as `Adopt`'s); its head: none before its branch is
    pushed, or a commit (32 bytes, as the forge connector keeps one);
  - its pull request: none yet, or its number (u64) and its address on
    the forge (a byte string of at most 2048 bytes, empty for a forge
    with no address: GitHub's `web`, Forgejo's `git` base, none for a
    fake, `served.md`, 2);
  - its step: taking, appended after the step's last variant, while a
    change opened from a pushed branch takes it
    (`../../domain/forge/change/opened.md`, 4); producing, opening,
    recreating, reopening, checking, gating, queued (its place, from 1 (u32), and the tasks ahead of it,
    nearest first, at most 64 (u64 each)), first, updating, resolving,
    repairing (its reason: CI, including a failed landing test gate,
    a gate by its number (u64), a semantic
    conflict, or a person's request for changes on the forge,
    `../../domain/forge/change/gates.md`, 1), landing, or landed (its merge commit, 32 bytes); verifying,
    red and cleared also carry the merge and at most 32 tier strings of at
    most 8192 bytes each. A tier string names its run and every full failing
    ID owned by or suspected of that landing, or explicitly `many tests`.
    The bound holds the 16 admitted IDs of up to 256 bytes each, their
    separators and tier/run words; IDs are never cut to fit the wire;
  - its hold: none, or why: stalled; past its repairs, its resolutions
    or its updates; its branch moved; its branch missing; its pull
    request closed; its pull request retargeted; a delegate or effect
    failed; a person rejected a gate. A held change's step is the one it
    was held at;
  - its wait: none, or why it waits, `ChangeWait`: for room in the
    forge's outbox, `outbox_room` (`../../domain/forge/room.md`, 7);
  - the task its step runs: none, or its number (u64);
  - CI at its head: unknown, pending, passed or failed;
  - its gates, at most 64, each with its number (u64), its kind (review,
    approval or check), whether it blocks (bool), and its verdict at the
    head: none, pending, approved, or changes asked;
  - its repairs, its resolutions and its updates, each used of its
    limit (u32 each);
  - its questions (`../../domain/forge/change/questions.md`, 1): its route, none
    or up; the questions taken, used of its limit (u32 each); and its
    open questions, at most 8 in the order taken, each with its asker's
    task (u64) and the question's number there (u32), the first line of
    its words (at most 256 bytes, cut on a UTF-8 boundary and marked),
    and where it stands: taken, asked
    of the change's requester, or answered and held, with its blocker,
    a dependency's task (u64) or message room, and since when, an
    instant, each none where the change's record keeps none
    (`../../domain/forge/change/notices.md`, 2): a field of the held
    variant, so the family's version moves. A change past its bound
    shows the question that held it among them, as its hold says;
  - its source (`../../domain/forge/change/opened.md`, 6), `source`,
    appended after the record's last field: none for a change with no
    source, or `PanelSource`: the source `branch` (bytes, bounded as
    `Adopt`'s), the `at` named (an option of a commit, 32 bytes) and the
    `kept` head (an option of a commit, none before taking selects it).
## 5. The deployment’s forges

Moved to [`records-deployment.md`](records-deployment.md), section 2.

## 6. A person’s state
- **A person's Away switch, the subject `away`:** of the party's own scope,
  with no fields, its window unused; on the wire `Watching`'s `away`, one
  subject for the deployment and each watcher's own rows, as `dismissed`
  (below). Its document, `Aways`: `rows`, at most 1,024, each
  `AwayRow { project, since, until, landed, off }`. *The client's forge
  side watches it once,* for the session, as it does `dismissed`, a watch
  refused or ended kept as such, with no rows: then the switch draws off and
  its act is not offered.
- **A person's devices, the subject `devices`:** of the party's own
  scope, with no fields, its window unused; on the wire `Watching`'s
  `devices`, after its last variant, one subject for the deployment and
  each watcher's own rows, as `away`. Its document, `Devices`
  (`../../domain/forge/attention.md`, 7.1): `rows`, at most 8, each
  `DeviceRow { number, label, at, endpoint }`, in number order. *The
  client watches it* while Telling you is open (`../pages/told.md`), and
  `temper ctl push list` reads it (`../ctl/tools-reads.md`, 1).
- **What was decided for a person, the subject `decided`:** of the project's
  scope, on the wire `Watching`'s `decided`, `DecidedSubject { project }`, its
  window unused. Its document, `Decisions` (`../../domain/forge/attention.md`, 5):
  `project`; `rows`, at most 128, each `Decision { number, task, at, question,
  took, why, overturned, reach }`, `took` the option's words; `asked`, at most
  128, each `Asked { number, task, at, reach }`, a call that was answered ask;
  each list the asking party's alone, newest last; and `cut`, whether either
  left older rows out. `reach` is a `Reach` (`../../protocol/configuration/settings-kinds.md`,
  1): the kind, the class, the reason, the item used, the sets touched, the
  patterns matched and whether Away was on. *The client watches it* per
  project while a page that shows it is open, at most `DECIDED_PROJECTS`, 4,
  at once (`../pages/home.md`, 1, *Decided for you*), and decodes each reach by the kit's
  `reach_from`.
## 7. Findings
- **A project's findings, the subject `findings`:** of the project's
  scope, for a party with a role there; on the wire `Watching`'s
  `findings`, `FindingsSubject { project, filter }`, `filter` a
  `FindingsFilter`: `triage`, `FindingsTriage { state, mine }`, `state`
  an option of `new`, `routed`, `backlog` or `dropped`, `mine` whether
  only the reader's (`../../domain/areas.md`, section 5); or `goal`, a
  `FindingsGoal { goal }`, or `milestone`, a `FindingsMilestone {
  milestone }`, a u64 each, the findings whose blame names that goal or a
  goal of that milestone, in every state
  (`../../domain/red-findings.md`, section 10). An enum's case carries a
  record, as `HolderTask` does, hence the two of one field. The root makes its
  document, since holders read the core (`../../domain/areas.md`,
  section 4). Its document, `Findings`: `project`; `rows`, at most 128,
  newest first, each `FindingRow { number, kind, severity, area, state,
  title, holder, at, why, hander, goal, reason, cleared }`, `holder` a
  `Holder`, `at` when it was
  filed, `why` the last hand-off's, option, and `hander` who made that
  hand-off, a task or a party as `filer` is, option, present with `why`
  (`../../domain/findings.md`, 2; `../pages/needs-you.md`, 1); `goal`,
  option, the task it was routed to; `reason`, option, a drop's, text
  of 400; `cleared`, option, a `FindingClearing { tier, run, at }`, the
  tier, its signature's source, bytes of 64, the run that cleared it
  and when (`../../domain/red-findings.md`, section 6); `areas`, at most 16, each
  `AreaRow { name, words, owner, holder }`, `owner` an `AreaOwner` (`../../protocol/configuration/settings-kinds.md`, 1) and
  `holder` an option of `Holder`, none where its owner does not stand;
  `triager`, its owner, option, and its holder; and `cut`, whether rows
  were left out. Marked at each committed decision that writes one of
  the project's findings or saves its kind 12 or 13, and a goal's or a
  milestone's filter where the written finding's blame names it, before
  the write or after (`../../domain/red-findings.md`, section 10); a
  holder that stops standing shows at the next mark. *Needs you watches
  it* with `triage`, `mine` and `state` `new`, for each project the
  person is in (`../pages/needs-you.md`, 1); *a goal's page* with
  `goal` (`../pages/goal.md`, section 2).
- **One finding, the read `Finding`:** `FindingRead { project, number }`,
  in the project's scope by a party with a role there. Its answer,
  `FindingAnswer`: its `FindingRow`, its `body`, its `filer`, a task or a
  party, and its `blame`, option, a `FindingBlame { strength, landings
  }`, `strength` a `BlameStrength`, `owned`, `suspected` or `unblamed`
  (the record's `none`), `landings`, at most 256, oldest first, each
  `BlamedLanding { commit, goals }`, `commit` fixed 32 as `BlamedGate`'s
  commits, `goals`, at most 64, each `BlamedGoal { goal, milestone,
  person }`, the last two options (`../../domain/red-findings.md`,
  section 3); or `no_finding`. Its routed goal and drop's reason are its
  row's.
- **A milestone's totals, the read `Milestone`:** `MilestoneRead {
  project, task }`, in the project's scope by a party with a role
  there. Its answer, `MilestoneAnswer`: `totals`, a `MilestoneTotals`,
  `task`, `done`, `failed`, `closed` and `live` (u32 each), `ended`,
  option, a wall, `last_landing`, option, a `MilestoneLanding`, `at`, a
  wall, `goal` and `change`, `spent` (u64), and `reds` (u32), its goals'
  uncleared red findings (`../../domain/red-findings.md`, section 10);
  or `not_milestone` (`../../domain/milestones.md`, section 7). A
  milestone's part carries the same `MilestoneTotals` (`parts.md`, 3).
  It moves `CLIENT_VERSION` by one.
- **Reds on goals and milestones** (`../../domain/red-findings.md`,
  section 10): the `findings` subject's `filter`, `FindingRow`'s
  `goal`, `reason` and `cleared`, `FindingAnswer`'s `blame` and
  `MilestoneTotals`' `reds` come together in the client's next layout,
  which moves `CLIENT_VERSION` by one, with no bridge
  (`../../draft/redeploys.md`, section 8).
## 8. Dismissed facts
- **A person's facts, the subject `dismissed`:** of the party's own
  scope (jig's `client/domain/contract/watches.md`, 1.5), with no fields, its
  window unused; on the wire `Watching`'s `dismissed`. One subject for
  the deployment: the core names each watcher's party, and each
  watcher gets its own party's document (`../../domain/forge/attention.md`, 1). Its
  document, `Dismissals`: `dismissed`, the party's rows in their key's
  order, at most `DISMISSED` (8,192), each a `PersonFact`: `ToBegin {
  project }`; `GoalFrom { project, chat, goal, at }`, that the person started
  goal `goal` from chat `chat`'s document at the client's clock's `at`; or
  `KeepAsking { project, subject, at }`, Keep asking pressed on a need of
  that project for the item of `subject`, a `ClassSubject`, at the client's
  clock's `at` (`../../domain/forge/attention.md`, 1). A start whose
  `forge.judge_projects` is past an eighth of that bound is refused, a party
  holding at most one `ToBegin` a project (`../../protocol/configuration/limits.md`
  ). A party's rows are bounded by `DISMISSED` too, and all rows by
  `limits.dismissed()`: over either, `Dismiss` answers too large, and the link
  alone is lost: the goal has started, and the client draws no refusal for it. A
  fact reaches the watch of every client of the party's once it
  commits. *The client's forge side watches it once,* for the session, as
  it does `forges`, from when `me` is first delivered while the link
  is live, for every party: a watch refused or ended is kept as such,
  with no rows, and not opened again until the next sign-in. Its rows are
  the forge side's facts, which Home reads to compute To begin (`../pages/README.md`).
## 9. Changes
- **A project's changes, the subject `changes`:** of the project's scope
  (jig's `client/domain/contract/watches.md`, 1.5), for a party with a role
  there, its window unused; on the wire `Watching`'s `changes`,
  `ChangesSubject`: `project`. `Changes`' page watches it while open. Its
  document, `ChangesPanel`:
  - `project`;
  - `landing`, the branch to name while nothing is queued: the newest
    landing's, else the landing branch of the project's home repository
    (`../../domain/forge/adoption.md`, 8), else of its first, else empty;
  - `queues`, at most 32, by repository then branch, one per landing branch
    with ready changes, a change owning its queue, or red (`README.md`, section 3), each a
    `ChangesQueue`: `repository`, `owner/name`; `branch`; `red`; and
    `changes`, in order (`../../domain/forge/queues.md`), each a
    `ChangesQueued`: `task`, `title` (at most 256 bytes), `ready_since`
    and, on a queue's first row, `checks_since`, when the step it is at
    began, both nanoseconds since the epoch;
  - `more`, the rows past `forge.changes_rows` (32 in `small`), counted
    across the queues, which fill in order; the changes queued are the
    rows listed and `more`;
  - `catching_up`, none, or while the forge of the project's home
    repository (`../../domain/forge/adoption.md`, 8), else of its first,
    catches up after a restart, its `CatchingUp`, as `ForgeRow`'s;
  - `landed`, at most `forge.changes_landed` (8 in `small`), newest first,
    across the project's repositories, a row for each change of each
    landing, each a `ChangesLanded`: `task`, `title` and `at`, the landing
    log's (`../../domain/after-landing-storage.md`, section 1); `commit`, the
    landing's, 32 bytes; `default_branch`, whether the branch the
    landing moved is its repository's landing branch, the main one
    (`../../domain/forge/adoption.md`, 8), whatever the forge's default
    branch; and `signed`, at
    most 8, the landing log's sign-offs of the change, in its order, each
    a `ChangesSigned`: `party`; `head`, the commit their review named, 32
    bytes; and `at`, when they reviewed, nanoseconds since the epoch
    (`../../domain/forge/change/gates.md`, 2, *Sign-offs kept at
    landing*);
  - `landed_more`, what `landed` cut;
  - `landed_default`, the rows of `landed`'s kind whose `default_branch` is
    set, at most `forge.changes_landed`, newest first, so that landings
    into goal branches never push them out (`../pages/work.md`, 1, *Work*);
    a landing with no change, a push from outside temper, gives no row;
  - `landed_default_more`, what `landed_default` cut.

  *Where each row came from* (`../../draft/page-round-trips.md`, section
  4), from version 29: `ChangesQueued`, the in-flight `TrainLanding` (of
  its `member`) and `ChangesLanded` each end with `goal`, `chat` and
  `milestone`, each an option of `ChangeFrom { task: u64, title: bytes
  256 }`, the task's number and its chip's title: its goal, the change
  task's nearest tracked ancestor, as its task document's `goal` names it;
  the chat it came from, the root of its lineage where that root is on the
  charter of the project's chat template (`pages.md`, *Changes*); and its
  milestone, its goal's requester where that is a milestone
  (`../../domain/milestones.md`, section 7). The engine's root knows them
  from the change's task: from the core's live tasks, held memory-only by
  change task, at most 4,096, the least recently used dropped first; for
  a change whose task or an ancestor has ended and isn't held, it loads
  their ended rows, as an ended task read loads, and marks the subject
  when they arrive. Until then, and where a load fails, the row names
  none. `ChangesSigned` ends with `name`, an option of at most 256 bytes:
  the party as the project's people name it, its member's name or, where
  that is empty, its login (`../pages/team.md`, 1); none where the
  project's document doesn't list the party. Names are made with the
  document, so a member renamed shows at the subject's next mark.

  It is marked as a change enters, leaves or moves in a queue, as the step
  of the change that owns one changes, as a branch's red changes and at each
  merge.
## 10. Landed drafts
- **A project's landed drafts, the subject `drafts`:** of the project's
  scope, for a party with a role there, its window unused
  (`../../domain/forge/drafts.md`, 3); on the wire `Watching`'s `drafts`,
  `DraftsSubject`: `project`, appended after the family's last subject.
  Work watches it while open, as it does `changes` (`../pages/work.md`,
  1). Its document, `DraftsPanel`: `project`; and `drafts`, at most 64,
  newest `at` first, then the lower path, each a `LandedDraft`:
  - `forge`, u16; `owner` and `name`, its repository's, at most 1024
    bytes each; `path`, at most 1024 bytes;
  - `branch`, at most 1024 bytes, and `commit`, at most 64 bytes, the
    commit's id as the forge names it;
  - `title`, at most 200 bytes, empty with none;
  - `writer`, the party, u64; `at`, nanoseconds since the epoch;
  - `chat` and `goal`, optional u64s; `ready`, bool.

  It is marked at each row written or erased.
## 11. What changed
- **What changed since the person last looked, the subject `status`:** of
  the project's scope, for a party with a role there, its window unused;
  on the wire `Watching`'s `status`, `StatusSubject`: `project`. `Home`
  watches it while open. Its document, `ProjectStatus`, is each watcher's
  own, the core naming the watcher's party (`../../domain/forge/attention.md`, 2):
  - `project`;
  - `landed`, u64, the changes landed into every landing branch of the
    project; `last_landed`, the time of the latest, option;
  - `brief`, option, `StatusBrief`: `sentence` (at most 200 bytes), `lede`
    (at most 600), `at`, and `landed`, the count it covers: the
    watcher's own brief;
  - `unpaid`, bool, appended after `branch`: whether the watcher's last
    status run found nothing to pay for it (`../../domain/forge/attention.md`,
    3, *When nothing pays*). Its field moves the client wire's version,
    unbridged (`../../draft/redeploys.md`);
  - `previous`, option, the party's last look of the sitting before; and
    `seen`, u64, the `landed` count then;
  - `queued`, u32, what the subject `changes` counts; `red`, whether the
    home repository's landing branch is red; and `branch`, its name, at
    most 1024 bytes, empty with no home.

  Times are nanoseconds since the epoch. It is marked with the subject
  `changes`, and at a look and a brief.
## 12. Approvals
- **The changes waiting on a person's review, the subject `approvals`:** of
  the party's own scope, with no fields, its window unused; on the wire
  `Watching`'s `approvals`. One subject for the deployment: the core names
  each watcher's party, and each watcher gets its own rows, which the root
  chooses by the watcher's role (`../../domain/forge/change/questions.md`, 2). Its document,
  `Approvals`: `rows`, at most 32, oldest wait first, each an `ApprovalRow`:
  - `task`, the change's; `project`; `goal`, option, the goal it was made under;
  - `forge`, u16; `kind`, `Kind` (`forgejo` or `github`); `owner` and `name`,
    at most 1024 bytes each; `pull`, the pull request's number; `address`, at
    most 2048 bytes, its files page on the forge's web address
    (`README.md`, 4.1), empty for none;
  - `title`, at most 256 bytes; `since`, when the change began waiting at
    its head;
  - the approval it waits on: `scope`, 0 for the deployment's rules or the
    project's number; `rule`; `role`; `required`; `counted`, the people
    counted so far;
  - `sets`, at most 6, each an `ApprovalSet`: `number`, u8, and `name`, at
    most 40 bytes, the sets of the approval's rule the change touches;
  - `whole`, whether its reviews were read whole;
  - `yours`, `ApprovalYours`: `counts`, `no_account`, `too_few` or
    `other_role`, whether the watcher's review would count and, if not, why;
  - who can still sign (`../../domain/forge/change/questions.md`, 2, *Who
    can still sign*): `signed`, at most 8, the people counted at the head;
    `can_sign`, at most 8, the people who can still give the next
    sign-off, the watcher first where among them; and `can_sign_more`,
    u32, how many more can past those. Each is an `ApprovalPerson`:
    `person`, u64, the deployment's person number, and `name`, at most 64
    bytes, empty where none is known;
  - `size`, option, last, `ApprovalSize`: the pull request's size at the
    head its reviews were read at, as the forge counts it
    (`../../domain/forge/change/questions.md`, 2): `files`, `additions`
    and `deletions`, u32 each, a count past 4294967295 kept at it; and
    `cut`, bool, whether one was; none where the forge gave none;

  then `more`, the rows past the bound. Times are nanoseconds since the
  epoch. Every client of the party's watches it once, for the session (`../pages/needs-you.md`, 1, *What the client
  keeps*). The engine fills `kind` and `address` from the
  forge's configuration as it does a change's panel's pull request's
  (`served.md`, 2).
## 13. Ways and views
- **A path, `BranchPath`** (`../../domain/forge/change/branch-path.md`): `repository`, option,
  `owner/name`, none for a project with no repository; `branch`; `stops`,
  at most 24, in order, each a `Stop`:
  - `actor`, `Actor`: `person`, `agent`, `code` or `bar`;
  - `step`, `PathStep`: `ask`, `draft`, `plan`, `produce`, `produce_many`,
    `check`, `review`, `repair`, `approve`, `conflict`, `resolve`, `merge`,
    `catch_up`, `prepare`, `prove`, `turn`, `verify`, `branch`, `design`
    or `close_out`, the last two a goal's coordinator's
    (`../../domain/forge/change/branch-path.md`);
  - `words`, at most 64 bytes, as the path draws them ("you ask",
    "produce ×n", "an owner approves", "main");
  - `needed`, whether the stop is made only when needed, drawn outlined;
    `over`, at most 16 bytes, the small words over its own ("tests,
    review", "into goal"), empty for none (`../../domain/forge/change/branch-path.md`,
    *Drawn apart*);
  - `opens`, `Opens`: `nothing`, `charter` (`charter`), `rule` (`kind`,
    `scope`, `number`), `gate` (`number`, and `charter`, its reviews'
    charter, the role its card opens), `tier` (`name`), `queue` or
    `turn`;
  - `card`, `Card`: `who`, at most 128 bytes, as "an agent · holds a
    change until it approves", or a role's "For each chat: talks with
    you and asks for the changes"
    (`../../domain/forge/change/branch-path.md`, *The project's roles*); `name`, at most 64, as "the reviewer",
    "owner", "CI" or "main"; `facts`, at most 4, each a `Fact`, `label`
    at most 32 bytes and `value` at most 184, as "Model" and "Opus `../pages/accounts-words.md`, 1 ·
    high", "Can", "Asked to": the engine makes every value at most 128
    bytes but an agent's "Asked to", its instructions' first sentence
    in quotes, at most 184 (`../../domain/forge/change/branch-path.md`), so that the worst
    ways document and the worst view answer still fit one decision's
    delivery in the smallest profile.
- **A project's ways, the subject `ways`:** of the project's scope, for a
  party with a role there, its window unused; on the wire `Watching`'s
  `ways`, `WaysSubject`: `project`. The root makes its document, since
  roles, charters and models are the core's (`../../domain/forge/change/branch-path.md`).
  Its document, `WaysPanel` (`../../protocol/configuration/settings.md`, 8):
  - `project`; `ways`, at most 4, the stock's in their order, each a
    `StockWay`: `name`, at most 64 bytes; `headline`, at most 120, How
    work runs' heading while the project runs it; `line`, at most 256; `path`, its
    path over the project; `items`, at most 10, each a `ProposalItem`, what
    "Use this" sends; `delegation`, whether the project's policy lacks the
    delegation the way needs (13.10 there); `roles`, at most 4, the
    standing roles it brings (`../../protocol/configuration/settings.md`,
    8), each a `WayRole`: `charter`, u32; `words`, at most 480 bytes;
    `budget`, u64, a week's; and `running`, whether the project runs a
    live standing role by that charter, which "Use this" starts where
    not;
  - `current`, option, the index of the way the project runs; `own`, the
    project's own path; `words`, option, the project's words, kind 10, a
    `ProjectWords`, `headline` at most 120 bytes and `lede` at most 480
    (`../../protocol/configuration/settings-kinds.md`, 1); `rules`, a `PathRules`,
    the project's rules by path (`../../domain/forge/change/branch-path.md`,
    *Rules by path*): `rows`, at most 9, each a `PathRule`, `paths` at most
    256 bytes, `review`, `sign_off` and `lands` at most 64 each, an empty
    `review` or `sign_off` for none; and `note`, at most 160, empty for
    none; no rows where the project holds no rule by path; `may_change`,
    whether the reader may change the project's settings, an owner of it.

  Marked at each committed change of settings, at each adoption into the
  project or removal from it, and at a change of any person's role in the
  project, since how many hold a role words its sign-offs. A
  path's bounds are set so that the worst document the root makes, the
  stock's ways and the project's own path, each at 24 stops of the
  largest agent's card, with its rules by path at every bound, fits one
  decision's delivery in the smallest profile.
- **A view, the read `View`:** `ViewRead`: `project` and `address`, at most
  512 bytes, a `view` block's line (`../pages/chat/conversation.md`, 3), read in the project's scope by a
  party with a role there. Its answer, `ViewAnswer`, holds a `View`: `way`,
  a `WayView`, `name`, `line`, `path` and `current`; `path`, a `BranchPath`;
  `proposal`, a `ProposalView`, `before` and `after`; `charter`, a
  `CharterView`, `charter` and its `card`; `rule`, a `RuleView`, `kind`,
  `number` and its `card`; `roles`, a `RolesView`, `roles`, at most 16,
  each a `CharterView` (`../../domain/forge/change/branch-path.md`, *The project's
  roles*); or `not_drawn`, `words`, at
  most 512 bytes, saying in a line why it is not drawn.
- **A proposal's items, the read `ProposalItems`:** its answer,
  `ProposalItemsAnswer`, holds the items and `owned_approval`, whether one of
  them is a landing rule with an enforcing approval covering a writable
  repository on an owned Forgejo (`../../domain/forge/change/questions.md`, 2), for its card's
  words (`../pages/settings.md`, 2).
## 14. The other records and bounds
- **The forge's documents' records:** its other reads, subjects and
  documents, and its other requests, beside `Adopt` and `SetUp`
  (`served.md`).
- **Bounds:** the schema's ceilings bound every record at both ends,
  and the generated worst cases price each. Within them, the client
  bounds its fields by its own limits (`family.md`, 4), and the engine's forge
  connector refuses what is past its own as too large (`family.md`, 6), so a
  record a client can make always decodes.

## 15. Open questions

None.
