# Red findings

Provisional, 2026-11-19, from `../draft/after-landing-repairs.md`,
sections 4 to 6. Under kind 4's `wait: landed` (`after-landing.md`,
section 6), every red of a tier following its repository's landing
branch or a named branch is a finding (`findings.md`) blamed on the
landing, and so the goal, that caused it, and kept until a run passes
its tests. A tier following goal branches is unchanged
(`after-landing.md`, 5.4), and under `wait: cleared` a red reaches its
goals or is a red of no goal's, as `after-landing.md`, section 7 says.
What waits is section 12.

## 1. In one page

- **Filed as the verdict is kept** (section 2): a judged run's failures
  that no uncleared red finding of its tier holds make one finding per
  landing that owns some, one for the suspects, and one unblamed for
  those failing before the run's range. Flakes are filed or counted as
  flake findings, never in a red one.
- **Blamed on its record** (section 3): each landing by commit, with its
  members' goals, their milestones and their persons, from the landing
  log; and its strength, owned, suspected or none.
- **Its words** (section 4): its signature, a title saying what is wrong
  and since when, and a body the triager reads the culprit from.
- **Many** (section 5): a many finding names no ids until the next
  enumerated run gives it those no other finding holds.
- **Cleared by a run that passes it** (section 6), whatever its state;
  its repair named in the run's comments, never trusted.
- **Held while uncleared** (section 7): its ids file nothing more, it is
  never evicted, and a filing refused for room waits on the tier's
  record.
- **Switching** a project to `landed` files each red landing's finding
  from its `Verification` row (section 8).
- **Shown on its goals and milestones** (section 10): a goal's page and
  `goal show` list the findings blamed on it and where each stands; a
  milestone counts the uncleared ones its goals owe, and is never held
  for them.

## 2. Filing

- **When:** at each judged run of a landing tier (one following its
  repository's landing branch or a named branch, `after-landing-runs.md`,
  section 1) whose project's kind 4 says `wait: landed`, in the step
  that keeps the verdict, the root filing in that decision. A dispatched
  run and a tip judged by its receipt alike. The run's failed jobs'
  logs are read first, as a red's news reads them (`after-landing.md`,
  section 7), where the run files any red finding.
- **In order,** at that step: an enumerated run's clearing (section 6),
  then its many finding's ids (section 5), then the filings waiting on
  the tier's record (section 7), then the run's own filings below.
- **Held ids:** an id is held where an uncleared red finding of the
  tier holds it, a red finding filed under `landed`, of the tier's
  repository with the tier's name as its source and not cleared,
  whatever its state (section 7); or where a filing waiting on the
  tier's record holds it.
- **The run's filings,** from the tests that failed again at its rerun
  (`after-landing.md`, 5.2, 1), less those held:
  - each landing of the range blamed as owning failures makes that
    landing's finding, with those it owns, strength owned;
  - those any landing is suspected of (`tell_all`, 5.2, 4 there) make
    one finding naming every suspect, strength suspected;
  - the rest, failing before the range and blamed on none of its
    landings (inherited, 5.2, 2 there), make one unblamed finding,
    strength none;
  - owned ones oldest landing first, then the suspected one, then the
    unblamed one; one with no id left is not filed.

  So a run whose failures are all held files nothing; a failure is in
  one finding at most, however many runs fail it; and the first run
  under `landed`, or a tier's baseline, files what was failing before
  it, unblamed.
- **It replaces the red of no goal's** (`after-landing.md`, section 7)
  for such a project's landing tiers: no finding is filed per landing.
- **Kind `red`, severity `high`,** and no area but a follow's owner's
  (`forge/follows.md`, section 6): where a blamed landing brought a
  follow change, its lowest-numbered kept member's follow names it, as
  for a red of no goal's; for a suspected finding the first suspect
  that has one; an unblamed finding has none.
- **Flakes:** a test that failed and passed alone at the rerun is counted
  on the open flake finding of the tier that holds it (kind `flake`,
  the tier's repository and name, any state but `dropped`;
  `findings.md`, section 4), its `seen` one more for each run that
  flakes any of its ids. The run's other flakes make one flake finding:
  severity `low`, no area, its title `K tests flaked in TIER run N`
  (`1 test flaked`), its body `TIER run N on BRANCH at TIP.`, TIP the
  run's tip's twelve digits, `Run: URL` where it has one, then each id,
  one per line, within its bound, and its signature the repository,
  the tier, its ids and an empty first line. A flake never enters a red finding, since
  filing takes only the tests that failed again; and a flake refused
  for room is not filed and does not wait.

## 3. Blame

- **On the record,** `FindingBlame`: its strength, `owned` (bisected,
  or the range's one landing), `suspected` (one of several under
  `tell_all`, or a many finding given ids, section 5) or `none`; and
  each landing it names, oldest first, by commit, with each goal its
  members name, once, in its members' order: the goal, the goal's
  milestone where it has one, and the goal's person where it has one.
  A finding with a blame is one filed under `landed`; any other has
  none.
- **From the landing log** (`after-landing-storage.md`, section 1): a landing's
  members, each a `LoggedChange` with its goal and milestone and, new
  here, its goal's person, title and `Draft:` path. A landing temper
  didn't make, or one the log no longer holds, names no goal: its
  finding names the landing alone.
- **The goal's person** is the one its own questions reach: its
  requester where that is a person, else its milestone's maker or its
  chat's person (`milestones.md`, section 5). It is found as the change
  registers, while its goal lives, and kept on the change's row beside
  its goal and milestone, with the goal's title, the first line of its
  words within 256 bytes, and the path its words' first `Draft: ` line
  names, within 256 bytes; the log takes them from the row.
- **Bounds:** at most `forge.landings_pending` landings, the range's
  oldest, and at most `forge.landing_members` goals each.
- **Moved** by the finding's holder or a project owner with the act
  `blame`, its strength then `moved` (`red-repairs.md`, section 3).

## 4. Its words

- **Its signature** (`findings.md`, section 4): the repository, the tier
  as its source, its ids in byte order, or `many` in their place
  (section 5), and its first line: the last line of the first failed
  job's log end that is not blank, within 512 bytes, empty where no log
  was read.
- **Its title,** within 200 bytes, K its ids' count (`1 test fails`,
  `K tests fail`, or `many tests fail`):
  - one landing with a goal: `K tests fail in TIER since GOAL landed`,
    GOAL the first goal its members name, as its id where its title has
    one (`../client/pages/work.md`, 1, *A goal's id*), else `goal N`;
  - one landing with no goal: `K tests fail in TIER since COMMIT
    landed`, the commit's first twelve digits;
  - several landings: `K tests fail in TIER since one of J landings`;
  - none: `K tests fail in TIER from before run N`, N the run.
- **Its body,** within `findings.md`, section 2's 4,096 bytes, lines in
  this order:
  - `TIER run N on BRANCH: landings FIRST to LAST.`, the range's first
    and last commits' twelve digits; `landing FIRST` for one, `at TIP`
    for none;
  - `Run: URL`, the judged run's link as its status has it, where it
    has one, and `Artifact: NAME`, the tier's report;
  - `Owned by landing COMMIT.`, `Suspected: J landings.` or `Blamed on
    no landing: failing before run N.`;
  - for each landing named, `Landing COMMIT:`, twelve digits, then each
    of its goals, `Goal N: TITLE.`, then `Pull request #P.` (`Pull
    requests #P, #Q.` for several members), `Draft: PATH.` and
    `Person N.` where it has them, each on its own line;
  - `Failing:`, then each id, one per line, or `many tests`;
  - the failed jobs' logs' ends, each headed `End of job NAME's log:`,
    or named with `[no log]`, as a red's news has them
    (`after-landing.md`, section 7).

  The lines before the ids are kept whole while they fit, then
  `and J more landings.`; the ids are kept whole, then `and K more
  tests`; the logs' ends share what is left, each cut from its start.
  The record keeps every id.

## 5. Many

- **A many finding's signature** says `many`, with no ids
  (`after-landing-runs.md`, section 3: past `failing_tests` a run's failures
  are many, blamed by landing alone).
- **Filed by a many run:** a landing blamed as owning many makes its
  many finding, owned; one suspected of many, a many finding naming
  every suspect, suspected; and a many run that blames no landing files
  one unblamed many finding, where no many finding of the tier is
  uncleared.
- **Given ids at the next enumerated run,** after its clearing: the
  oldest uncleared many finding of the tier takes the tests that failed
  again and that no other uncleared red finding of the tier, nor a
  waiting filing, holds. Its signature then holds them, its strength
  turns suspected, since many may hide inherited failures, and its
  title counts them; its body stays as filed. From then on it is a red
  finding like any other. Given none, it is cleared by that run, as is
  every other uncleared many finding of the tier.
- **A many run clears nothing** and gives no ids: it can't say which
  tests failed.

## 6. Cleared by a run that passes it

- **By observation:** an enumerated judged run of the tier clears each
  uncleared red finding of the tier none of whose ids failed again, so
  that those that passed alone count as passed, whatever the finding's
  state. Its record keeps the run's number and the time, `cleared`. It
  never reopens: a later failure of its ids is filed anew.
- **It leaves its holder's list:** a cleared finding is no one's to
  triage. The `findings` subject's and the `finding` tool's `mine`
  filter pass it over, so Needs you does too, and a change of a
  project's areas or triager hands it to no one (`areas.md`, section
  5). Its state stays as triage left it.
- **Its issue,** where it has one (`findings.md`, section 2), is closed
  with a comment, `Cleared by TIER run N.`, and never reopened.
- **Its repair is named, not trusted:** each member of each landing
  that a finding the run clears blames, and that the landing log still
  holds, gets one comment from the run (`after-landing.md`, section 9),
  however many cleared findings blame its landing: `Repaired by #M:
  TIER run N passed the failures blamed on this landing.`, M the pull
  request of the newest landing in the run's range with a member whose
  pull request's body names any of those findings by a line
  `Fixes-finding: n`, n its number in its project, its first such member
  in the log's order; else `Repaired: TIER run N passed the failures
  blamed on this landing.` Each member of the log keeps the findings its body
  names, at most 8, as it is logged. `Fixes #n`, in the home, names the
  finding whose issue is n, so none until findings have issues. That
  name clears nothing by itself, since a fix that lands can leave the
  failure failing, and no landing's clearing closes a red finding filed
  under `landed` (`findings.md`, section 7): only a run that passes it
  does.

## 7. Held, never evicted, waiting for room

- **Uncleared, it holds its ids** in any state, `new`, `routed`,
  `backlog` or `dropped`: a red finding filed under `landed` is open
  until cleared, where any other is open until dropped. A finding
  closed on the forge by another hand is dropped (`findings.md`,
  section 3) and stays uncleared, so a dropped red files nothing more
  until a run passes its tests.
- **Never evicted:** a filing past `forge.findings_per_project` erases
  the project's oldest finding that is cleared, or `routed` or
  `dropped` and not an uncleared red finding filed under `landed`
  (`findings.md`, section 9).
- **Waiting for room:** where no finding can be erased, a red filing is
  refused for room; it then waits on the tier's record whole, its
  title, body, signature, blame and follow, and holds its ids there. At
  most 8 wait per tier; one past them is not kept but counted, its ids
  held by nothing, so a later run files them again.
- **Filed at a later run:** each judged run of the tier, enumerated or
  many, files its waiting filings oldest first while the project's
  findings have room, before its own (section 2).
- **Counted and reported:** the run task's report (`after-landing-storage.md`,
  section 1) ends with `K red filings wait for room in the project's
  findings.` where any wait, and `J red filings were dropped for want
  of room.` where the tier's count is not 0.

## 8. Switching

- **Each red landing's `Verification` row files first:** as the row
  goes with its landing's last change, ended landed and uncleared at
  the start that switched its project to `landed` (`after-landing.md`,
  section 6, *Switching*), each tier whose verdict on it blames the
  landing and that no run repaired since files that landing's finding,
  its strength owned or suspected as the verdict says, its ids those
  blamed on it less those held, or `many`, and its goals from the
  landing log, else from its changes' rows.
- **Its words** are section 4's, its body's first line `TIER run N
  blamed landing COMMIT on BRANCH before its project ended changes at
  landing.`, with no run's link and no logs, which the row doesn't
  keep. It waits for room as section 7 says.
- So its blame isn't lost with the row, and the tier's next run finds
  its failures held.

## 9. The store and the wires

- **`Finding`** (`findings.md`, section 4) gains `blame`, a
  `FindingBlame`, and `cleared`, a `FindingCleared`: the run and the
  wall time. Its signature gains `many`.
- **`LoggedChange`** (`after-landing-storage.md`, section 1) gains its goal's
  `person`, `goal_title` and `draft`, and `fixes`, the findings its
  pull request's body names; the change's row gains the first three.
- **The tier's record** gains its `waiting` filings, at most 8, and
  `unkept`, the count of those not kept.
- **Read from before** (`../protocol/store.md`, sections 2 and 4): each
  is an added field of the registered `temper-store` family with its
  default, so no version moves and no patch comes: a finding with no
  blame, uncleared and not many, so that a red finding filed before
  reads as it was, open until dropped; a member and a change with no
  person, title, draft or fixes; a tier with nothing waiting.
- **No engine wire moves:** filing is a request inside the engine. The
  client's records that show a finding's blame and clearing move in the
  client's next layout, unbridged (section 10).
- **In the code,** the forge's vocabulary
  (`temper-engine-domain-forge-vocabulary`) holds the shapes the
  engine's parts share:
  - `FindingBlame { strength: BlameStrength, landings: Box<[BlamedLanding]> }`,
    `BlameStrength { Owned, Suspected, Unblamed }` (the record's
    `owned`, `suspected` and `none`), `BlamedLanding { commit: Commit,
    goals: Box<[BlamedGoal]> }`, `BlamedGoal { goal: u64, milestone:
    Option<u64>, person: Option<u64> }` and `FindingCleared { run: u64,
    at: Wall }`; `Finding`'s `blame: Option<FindingBlame>` and
    `cleared: Option<FindingCleared>`, `FindingSignature`'s `many:
    bool`, and `Finding::uncleared_red(&self) -> bool`, true for a
    finding of kind `red` with a blame and no `cleared`;
  - `VerdictFiling { project: u32, kind: FindingKind, severity:
    Severity, title: Box<[u8]>, body: Box<[u8]>, signature:
    FindingSignature, blame: Option<FindingBlame>, follow: Option<u32>
    }`, a verdict's filing, red with a blame or flake with none, which
    the forge asks of the root by `Request::FileVerdict { task: u64,
    filing: VerdictFiling }`, filed by the root's
    `findings::file_verdict` through the forge's
    `Domain::prepare_verdict_filing`; a tier's waiting filings are
    `VerdictFiling`s;
  - `ChangeRow` and `LoggedChange` gain `person: Option<u64>`,
    `goal_title: Option<Box<[u8]>>` and `draft: Option<Box<[u8]>>`,
    `LoggedChange` `fixes: Box<[u32]>`, and `TierRow` `waiting:
    Box<[VerdictFiling]>` and `unkept: u32`;
  - for section 10: `FindingsFilter { Triage { state:
    Option<FindingState>, mine: bool }, Goal(u64), Milestone(u64) }`,
    `findings_subject(project: u32, filter: FindingsFilter) ->
    Option<u64>` and `findings_filter(subject: u64, project: u32) ->
    Option<FindingsFilter>`, a goal's key `(1 << 62) | (1 << 41) |
    goal` and a milestone's `(1 << 62) | (1 << 42) | milestone`, a
    number at or past `1 << 41` having none, so that its watch is
    answered as a subject the forge doesn't serve;
    `FindingRow` gains `goal: Option<u64>`, `reason: Option<Box<[u8]>>`
    and `cleared: Option<FindingClearing>`, `FindingClearing { tier:
    Box<[u8]>, run: u64, at: Wall }`, made by `Finding::row`;
    `FindingAnswer` gains `blame: Option<FindingBlame>` and gives up its
    `goal` and `reason` to its row; `MilestoneTotals` gains `reds: u32`.

## 10. Shown on its goals and milestones

From `../draft/after-landing-repairs.md`, section 6. A red finding's
blame says which goals and milestones it is shown on; nothing else is
kept of it, so a blame moved (the draft's section 7) moves where it is
shown, and every count follows at its next read.

- **Blamed on a goal** where a landing its blame names has that goal
  among its goals (section 3), whatever its strength, owned or
  suspected; **on a milestone** where one of those goals' `milestone`
  is it. A finding with no blame, or one whose landings name no goal,
  is blamed on none.
- **The forge's `findings` subject, filtered** by a goal or a milestone
  (`../client/forge/records.md`, *A project's findings*): its rows are
  the findings of its project blamed on it, in every state, cleared or
  not, newest first, at most 128 as any of its rows, with `cut`. Every
  row of the subject carries what says where it stands: the goal it was
  routed to, a drop's reason, and its clearing, the tier (its
  signature's source), the run and the time. A filter by goal or
  milestone takes no state and no `mine`.
- **Its key and marks:** a filtered subject's key holds the goal's or
  the milestone's number alone, since task numbers are the
  deployment's, never reused, and its document is made in its watch's
  project scope. A decision that writes a finding marks the subject of
  each goal and each milestone its blame names, before the write and
  after it, beside the project's ten (`findings.md`, section 9), and
  marks the parts of each such milestone, as a change of its forge row
  does (`../client/forge/parts.md`, 4).
- **A goal's reds:** its page (`../client/pages/goal.md`, section 2) and
  `temper ctl goal show` (`../client/ctl/findings.md`) list the findings
  blamed on it, the uncleared first, each group newest first: each its
  number `P/N`, its title, and where it stands, the first that holds:

  | Where it stands | When |
  |---|---|
  | `cleared by TIER run N` | cleared, whatever its state (section 6) |
  | `waiting for triage` | `new` |
  | `being repaired by goal G` | `routed`, G the goal's number |
  | `in the backlog` | `backlog` |
  | `dropped: REASON` | `dropped`, its reason as kept |

  One pure function of the client's domain says it for both,
  `findings::stands(row: &FindingRow) -> Stands`, and each shell words
  it. The goal's own state is unchanged: done stays done, and the red
  belongs to the record.
- **A milestone's reds owed:** `MilestoneTotals`' `reds`
  (`milestones.md`, section 7), the uncleared red findings of its
  project (`Finding::uncleared_red`) blamed on it, each once, whatever
  their state, a dropped one included, since it still holds its tests
  (section 7). They are counted at each read and each making of its
  part, from the forge's findings, by the forge's
  `Domain::reds_owed(project: u32, milestone: u64) -> u32`, and never
  kept on its row, its ended record or a part's cache. Its headline
  says them on its page, Home's row and Work's card
  (`../client/pages/milestone.md`, section 2), and so do the status
  agent's line (`milestones.md`, section 9) and `temper ctl milestone
  show` (`../client/ctl/milestones.md`, section 4).
- **Not held:** a milestone is complete when its goals ended done
  (`milestones.md`, section 6), whatever reds its goals owe; holding it
  for them is the draft's section 12's.
- **As fresh as the read:** a live milestone's row and its page count
  again as the marks above deliver them. An ended milestone's row on
  Home and Work is read with its ended group, only as that group's mark
  grows (`../client/forge/parts.md`, section 6), so it says what was
  owed at that read; its page reads `Milestone` as it opens.
- **The agents' `finding` tool** keeps its acts and filters: the
  triager's acts on a red are the draft's sections 7 and 8, which move
  the host tools' contract once for them.

## 11. The world

Focused, each failing first, in the forge domain's library tests and
the root's world (`temper-engine-domain-world`):

- filing: a red run of one landing files one owned finding naming its
  goal, milestone and person; bisected over three landings with two
  culprits, two; under `tell_all`, one suspected finding naming every
  suspect; inherited failures no finding holds, one unblamed finding,
  and a run failing only held tests none; the first run under `landed`
  files the earlier failures unblamed; a landing temper didn't make
  gets a finding with no goal; a flake is filed or counted, never in a
  red finding; under `cleared`, today's stories are unchanged;
- many: a many run files a many finding, and the next enumerated run
  gives it only the ids no other finding holds, its blame suspected;
- clearing: a judged run that fails none of a finding's ids clears it,
  at any state, dropped included; a many run clears none; an uncleared
  red finding is never evicted; a filing refused for room waits on the
  tier's record and is filed, with its blame, at a later run; a
  clearing run's comment names the pull request whose body says
  `Fixes-finding: n`, and that name alone clears nothing;
- switching: switching a project to `landed` files each red landing's
  finding blamed on it;
- shown (section 10): the `findings` subject filtered by a goal lists
  the findings blamed on it and no other, owned and suspected alike,
  each row with its routed goal, drop reason and clearing, and by a
  milestone those blamed on its goals; a finding's write marks the
  subjects of the goals and milestones its blame names and those
  milestones' parts; the `Finding` read carries the blame;
  `MilestoneTotals`' `reds` counts a milestone's goals' uncleared red
  findings, each once, a dropped one included and a cleared one not,
  in the read, a live part and an ended one alike, while a complete
  milestone stays complete; the status agent's milestone line ends
  `; R reds owed`.

Fuzzy: random runs, enumerated and many, landings, triage acts,
filings refused for room and restarts keep every failing id in at most
one uncleared red finding of its tier, no uncleared red finding
evicted, and at most 8 filings waiting per tier; and, for section 10,
random filings, blames, clearings and triage acts keep each
milestone's `reds` equal to its uncleared red findings counted from
the record, and each filtered subject's rows equal to the findings
whose blame names its goal or milestone.

The client's side of section 10 is tested where its documents say:
the pages' in the client's domain and view crates
(`../client/pages/goal.md`, section 2; `../client/pages/milestone.md`,
section 2), `temper ctl`'s in its tools' step harness
(`../client/ctl/findings.md`, section 5).

## 12. Later

- **Trailers:** a landing's commits naming a finding by
  `Fixes-finding: n` name its repair once the landing log keeps their
  trailers (`after-landing.md`, section 8, and `after-landing-storage.md`, section 1); until then its
  members' pull request bodies alone do.
- **Sorting and repairs** are `red-repairs.md`'s: the triager's sorts,
  a red's blame moved and its repair goals.
- **Reds owed on an ended milestone's row** counted as they move, once
  a mark of the parts can reach an ended group's row without its mark
  growing (section 10, *As fresh as the read*).
