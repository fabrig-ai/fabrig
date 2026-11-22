# Red repairs

Provisional, 2026-11-19, from `../draft/after-landing-repairs.md`,
sections 7 and 8. A red finding (`red-findings.md`) is the project's
triager's (`areas.md`, section 6) to sort: a routine one gets a repair
goal with no one asked, one that needs a product decision is asked of
the culprit's person, a flake is dropped and wrong blame is moved. A
repair goal is made by one act for the findings it settles, at most
`repair_goals` live at once and `repair_attempts` a finding, and every
goal a red finding is routed to is judged at its end by the tier that
filed it. What waits is section 11.

## 1. In one page

- **The triager sorts** (section 2): routine, a product decision, a
  flake, or blamed wrongly, in its charter's words; code sorts nothing.
- **`blame`** (section 3) moves a finding's blame to a landing the
  landing log holds on the tier's branch, or to none, with why; its
  strength becomes `moved`.
- **`repair`** (section 4) makes one goal for one or more red findings
  of one repository, of kind 4's `repairs.goal` shape, as the triager's
  delegate, in one decision that routes each finding to it and counts
  its attempt; refused as 4.2 lists.
- **Judged at its end** (section 5): every goal a red finding is routed
  to, a repair goal or any other, is judged by its finding's tier. A
  finding the run passes is cleared; one it still fails goes back to its
  holder `new`, with its tests and why; a cancelled goal's go back at
  once. A repair goal past `repair_deadline` ends failed.
- **Then a person** (section 6): past `repair_attempts`, or after a
  cancel, `repair` refuses the finding, and the triager asks.
- **Kind 4's `repairs`** (section 7) turns it on, per project.
- **The stock** (section 9): the classes' routine-repair item at
  Decide, and charter 14's sorts.

## 2. The triager sorts each red

- **It holds each red** filed with no area (`areas.md`, section 5) and
  hears it on the project's findings topic. It reads the finding's body
  (`red-findings.md`, section 4: the failures, their logs' ends, the
  blame, each culprit goal's title, pull request, `Draft:` path and
  person), and the culprit's words and draft where it needs more, then
  sorts each finding, or each group one act settles (`areas.md`, section
  6): the findings of one culprit where it can be, those with no culprit
  together.
- **Routine,** a bug with a clear repair within what the culprit was
  asked to do: a repair goal (section 4), with no one asked. It first
  calls `decided`, kind `findings`, `under` the project's classes item
  whose words are "a repair goal for a routine red after landing", so
  that its person reads the choice under Decided for you. The stock's
  tables hold that item at Decide (section 9). A project whose classes
  lack it answers Ask, and the triager asks its person as for any
  finding.
- **A product decision,** where the red shows what the culprit's
  request didn't foresee: two wants at odds, a test and the request
  disagreeing, or a repair that would change what a person sees beyond
  the request.
  - It is asked of the culprit goal's person, `Person N` in the
    finding's body, with a person task as `need ask` makes one
    (`../client/ctl/tools.md`, section 3): the question in one sentence,
    then each choice on its own line, the recommended one first.
  - It is asked of the triager's own person where its delegate to that
    person is refused, or where the finding names no person.
  - The triager carries the answer out with the finding's acts: a
    repair goal whose words carry the decision, the backlog or a drop.
    A repair goal's own question that needs a product decision goes the
    same way.
- **A flake,** a failure of load or of the machine that failed again
  alone: dropped, saying so. It stays uncleared, so it files nothing
  more until a run passes it (`red-findings.md`, section 7).
- **Blamed wrongly:** moved with `blame` (section 3), then sorted as
  above. Suspected blame is confirmed or moved before any repair: the
  charter says so; `repair` doesn't refuse it.
- **A culprit still live,** a goal whose changes land straight into
  main, may have its red routed to it (`findings.md`, section 6), as
  news would have reached it; it is judged at its end (section 5).

## 3. Moving blame

- **`blame`,** an act of the `finding` tool and of `temper ctl finding
  blame`, by the finding's holder or a project owner (`areas.md`,
  section 7), on a red finding with a blame (one filed under `wait:
  landed`), uncleared, in any state.
- **To a landing:** a commit that the landing log (`after-landing.md`,
  section 10) holds as a landing of the finding's repository on the
  branch its tier follows. The finding's blame then names that landing
  alone, with its members' goals, milestones and persons as filing
  takes them (`red-findings.md`, section 3).
- **To none:** no landing.
- **Either way** its strength becomes `moved`, and the move's author
  (the acting task or party) and its why, 1 to 200 bytes in a line, are
  kept on the blame. Its title, body, signature, state and holder stay
  as they were: the record's blame is what goal and milestone reads
  count (`../draft/after-landing-repairs.md`, section 6). A later move
  replaces it whole.
- **Refused,** in words: `finding P/N has no blame: it is no red filed
  under landed`; `finding P/N is cleared`; `landing COMMIT is not one
  the landing log holds on BRANCH`; `finding P/N's tier TIER has no
  followed branch`, for a move to a landing when the repository's
  settings today name no branch that tier follows (the branch is read
  from the settings alone, never from past tier runs or the forge; a
  move to none still holds); and as any act is, `finding P/N is not
  yours: its owner is HOLDER`.
- **A many finding** given ids later keeps a moved blame as moved
  (`red-findings.md`, section 5 turns only filed blame suspected).

## 4. Repair goals

### 4.1 The act

- **`repair`,** an act of the `finding` tool and of `temper ctl finding
  repair FINDING... TEXT`, by the findings' holder or a project owner:
  one to 16 findings of one project and the triager's words, 1 to
  4,096 bytes.
- **One decision:** the root checks the act (4.2), makes the goal
  (4.3) and, in the same decision, writes each finding: `routed`, its
  `goal` the new goal, and its repair (section 8): the goal, its
  attempts one more, not cancelled, its deadline the goal's. Each
  finding's goal hears it as a route's news (`findings.md`, section 6).
  A restart replays the act's answer, through the tool call's key or
  ctl's `--key`, and makes no second goal.
- **Its answer** names the goal: `Repair goal G made for findings P/N,
  P/M.`

### 4.2 Refused

Refused, naming why, in these words, checked in this order:

- `repairs are off in project P`, where kind 4 has no `repairs`, and
  `repairs are off in project P: FIELD`, where a start found them
  incomplete (section 7), FIELD the first it found: `goal.charter`,
  `goal.llm` or `tasks.spec_bytes`;
- `no triager stands in project P`, where kind 15 resolves to no
  standing task (`areas.md`, section 4);
- `finding P/N is not yours: its owner is HOLDER`, as any act is;
- `finding P/N is not an uncleared red filed under landed`, for a
  finding of another kind, one with no blame, or one cleared;
- `finding P/N is STATE, not new`;
- `findings P/N and P/M are in two repositories`;
- `ask before repairing finding P/N again`, where a finding's attempts
  reached `repair_attempts`, or its last goal was cancelled;
- `project P has its K repair goals live: goal G, goal H`, where
  `repair_goals` are live (section 6);
- `the triager's period has L left, short of the repair goal's budget
  B`, L what its numbers leave (budget less spent, spent below and
  reserved), since money waits for its person (`agent/standing.md`,
  section 5);
- a making jig refuses (a model, the delegation's room or depth), in
  its refusal's words.

### 4.3 The goal

- **Its shape** is kind 4's `repairs.goal` (section 7): an agent of its
  `charter`, its `llm` (none: the charter's default), its tracked
  `priority` and its `budget`. Its label and flags are the deployment's
  goal template's (template 2, as the `goal` tool takes them,
  `route/goal.rs`), ended by its closer, procedure 6 of the forge's
  connector, as template 4's goal is; it holds its own branch as a goal
  `goal start` makes does (`forge/branches.md`, section 2;
  `GOAL_BRANCH_SYMBOL`), so it lands as any goal does, ending at
  landing.
- **The triager's delegate,** whoever acted: made by the root for the
  standing triager, funded from its period's budget (`Funder::Task`),
  its authority the triager's narrowed as the `goal` tool narrows a
  goal's (forge read and change grants, no role kinds, one level
  shallower), its contract a report within `tasks.result_bytes`. The
  goal's questions and result reach the triager as its own delegate's;
  each result wakes it, and it starts what waits.
- **Made as jig makes a root-made goal** (jig's
  `domain/authority.md`, 6.3): a number allocated as a follow's task
  is, then `RepairSeed::goal`, in the decision that takes the act.
- **Its deadline** is `repair_deadline` after its making, kept on its
  findings' repair. The forge wakes at it (as it wakes for a change's
  wait, `next_wall_deadline`) and asks the root to end the goal failed,
  once, through jig's failing by the application (jig's
  `domain/tasks/lifecycle.md`, section 13), its reason `past its repair
  deadline of H hours`. jig's own deadline is not set on it, so jig
  never holds it for one.

### 4.4 Its words

Within `tasks.spec_bytes`, in this order:

```
Repair K tests failing in TIER

WORDS

Repository: REPOSITORY. Tier: TIER.
Name each finding a change repairs in its pull request's body, by its line:
Fixes-finding: N
Fixes-finding: M

Finding P/N: TITLE
Attempt A of R.
Still failing after goal G: K of its tests.
ID
BODY
```

- **TIER,** in the title line and the repository's line, the findings'
  distinct tiers, oldest finding's first, joined by `, `: one
  repository's findings may be of several tiers (4.2 refuses only two
  repositories).
- **The title line,** at most 60 characters: K the distinct ids of the
  findings (`1 test`; `many tests` where a finding's signature says
  `many`); where the line would pass 60, TIER is cut and ends with
  `…`; the repository's line and each finding are never cut for it.
- **WORDS,** the act's, whole: the triager's sort and, where a person
  decided, the answer.
- **Each finding,** oldest first: its number and title, its attempt
  (this goal's, of `repair_attempts`), where it was tried before the
  goal before and the tests still failing then, one id a line, then its
  body.
- **Cut** to the bound from the last finding back, whole findings
  dropped, then `and more: finding P/N, finding P/M` naming the rest;
  the lines before the findings are never cut, since the act's words
  and 16 `Fixes-finding` lines fit within the bound (repairs are off
  where `tasks.spec_bytes` is below 8,192, section 7).

## 5. Judged at its end

### 5.1 What is judged

- **Any goal a red is routed to,** by `repair`, by `route` (the
  culprit's live goal included) or by a goal started on a person's
  answer: at its end, each uncleared red finding filed under `landed`
  that is `routed` to it is judged. A finding routed by `route` gains a
  repair record then, its attempts 0, so a routed goal counts in no
  cap.
- **The root hears every task's end** (jig's `Ask::Release`, with its
  ending), and asks the forge to judge where a finding is routed to it.

### 5.2 By which run

- **Ended done or failed, having landed:** its last landing is the
  newest the landing log holds of the findings' repository whose members
  name the goal. It is judged by the first judged run of the tier whose
  range holds that landing, as the log's verdict of that tier on it
  marks it. Where that tier already judged it, at once, by the tier's
  last judged run.
- **Ended without landing:** at once, by the tier's last judged run,
  since nothing it did reached the branch.
- **Its landing not held,** the log having let it go, or on another
  branch than the tier's: by the tier's first enumerated judged run
  that starts after the goal's end.
- **Told apart by a kept change:** the log alone can't tell a landing
  it let go from none. A goal with no landing in the log is judged as
  one whose landing the log let go when the forge still keeps a change
  of it landed, and as ended without landing when it keeps none.
- **A many run judges nothing:** a finding waiting on a landing that a
  many run covers, or judged at once while the tier's last run was
  many, waits for the tier's next enumerated judged run.
- **A held tier judges at its release,** by its last judged run then; a
  tier that has judged no run waits for its first.
- **Kept on the finding's repair** while it waits (section 8):
  `Landing(commit)`, `After(run)` or `Release`.

### 5.3 What it finds

- **Cleared:** a finding none of whose ids the run failed again is
  cleared by it, as `red-findings.md`, section 6 clears one, at any
  state. A run that judges a waiting finding has cleared it first where
  it can, so only those still failing are left.
- **Back to its holder:** each finding the run still fails goes back
  `new`, its goal none, its holder's again by the rule (`areas.md`,
  section 4), with its tests still failing on its repair and why kept as
  a hand-off's is, its hander none: `repair goal G ended and TIER run M
  still fails K of its tests` (`goal G ended…` for a goal `repair`
  didn't make; `TIER's last run` where the tier's record keeps no run
  number, one from before it kept one). Its holder hears it as a
  hand-off (`areas.md`, section 5), the news's third line `Back: WHY`.
- **Cancelled:** a cancelled goal's findings go back at once, the same
  way, with why `repair goal G was cancelled` (`goal G was cancelled`),
  and their repair marks the cancel.

## 6. Then a person, and the cap

- **Then a person:** past `repair_attempts`, or after a cancel,
  `repair` refuses the finding (4.2). The triager then asks, as for a
  product decision (section 2), or backlogs or drops it. A goal it
  starts on the answer, with `goal` and `route`, is judged at its end
  as a repair goal is, and counts in no cap.
- **The cap:** a repair goal is live from its making to its end; the
  live ones are the distinct goals the project's findings' repairs name
  with a deadline kept, at most `repair_goals`. The forge clears a
  goal's deadline from its findings at its end.

## 7. Kind 4's `repairs`

- **A member of kind 4** (`landing-settings.md`, 1.1), none by default,
  taken at the next start as the kind's others are:

  | Member | Default | Bounds |
  |---|---|---|
  | `repair_goals` | 2 | 1 to 16 |
  | `repair_attempts` | 2 | 1 to 8 |
  | `repair_deadline_ms` | 86,400,000 (24 hours) | one hour to 7 days |
  | `goal` | none: required | its members all required but `llm` |

  `goal`'s members: `charter`, a charter number; `llm`, the `llm` form
  a template takes (`../../help/reference/settings-fields.md`, *llm*),
  or `null` for the charter's default; `priority`, a tracked priority,
  `u32`; `budget`, the goal's spend, above the rules'
  `minimum_run_spend`.
- **`llm` is kept as written,** by the catalogue's names, in the
  setting, the store and the client's wire: the root resolves it to
  the catalogue's numbers when it judges a proposal and at each start
  (below), and the act reads the resolved one. Nothing renumbers an
  application kind's value when the catalogue changes, as jig does its
  templates', so a kept number could name another model after a
  redeploy; a kept name is resolved or found gone. A proposal and a
  rendering need no catalogue: the entrances that decode kind 4 hold
  none.
- **In a settings file** as kind 4's other members, `{"wait":
  "landed", "repairs": {"repair_goals": 2, "goal": {"charter": 5,
  "llm": null, "priority": 0, "budget": 2000000}}}`, absent members at
  their defaults; rendered omitting defaults.
- **Judged when proposed:** refused without `wait: landed` (`repairs:
  needs wait landed`); a member out of bounds; `goal` missing or a
  member of it missing; a charter the deployment lacks or that runs no
  agent (every charter jig holds is an agent's, `Executor::Agent`'s, so
  a charter held is one that runs an agent); an `llm` whose names the
  deployment's catalogue or the charter lacks.
- **Checked at each start** complete again, against the deployment's
  charters, models and limits as they then stand: a charter or model
  gone, or `tasks.spec_bytes` below 8,192, turns the project's repairs
  off until a save makes them whole and a start takes it. The start
  goes on: it refuses nothing for it. Its report is `repair`'s
  refusal, naming the project and the field (4.2), since a start has
  no report of its own for a setting it can't honour. The root checks
  it where the forge's records are restored (`route.rs`'s
  `ForgeRestored`), and the forge holds which projects are off, in
  memory, until the next start.
- **Findings on:** the draft also refuses `repairs` without findings
  on; kind 4's `findings` member isn't built (`landing-settings.md`,
  1.1), and red findings file under `wait: landed` alone, so the check
  waits for that member.

## 8. The store and the wires

- **`Finding`** (`findings.md`, section 2) gains `repair`, a
  `FindingRepair`, none until a goal is routed it; `FindingBlame`
  gains `moved`, a `BlameMove`, and `BlameStrength` the variant
  `Moved`. Each is an added field of the registered `temper-store`
  family with its default (none), so no version moves and no patch
  comes (`../protocol/store.md`, sections 2 and 4); a finding filed
  before reads as unrepaired and unmoved. `ProjectLanding`'s `repairs`
  is an added field, none.
- **In the code,** the forge's vocabulary
  (`temper-engine-domain-forge-vocabulary`) holds:
  - `FindingRepair { goal: u64, made: bool, attempts: u8, cancelled:
    bool, failing: Box<[Box<[u8]>]>, until: Option<Wall>, judging:
    Option<RepairJudging> }`: the last goal routed it, whether `repair`
    made it, the goals `repair` made for it, whether its last goal was
    cancelled, its ids still failing at its last judging (at most the
    signature's 64), a live repair goal's deadline, and what its judging
    waits for;
  - `RepairJudging { Landing(Commit), After(u64), Release }`;
  - `BlameMove { by: Filer, why: Box<[u8]> }` and `FindingBlame`'s
    `moved: Option<BlameMove>`;
  - the setting as written: `RepairSettings { goals: u8, attempts: u8,
    deadline_ms: u64, goal: RepairGoalSettings }`,
    `RepairGoalSettings { charter: u32, llm: Option<NamedLlm>,
    priority: u32, budget: u64 }`, `NamedLlm { account: Option<u32>,
    main: NamedModel, subagents: Box<[NamedModel]>, later:
    Option<NamedLater> }`, `NamedLater { main: NamedModel, subagents:
    Box<[NamedModel]> }` and `NamedModel { model: Box<[u8]>, effort:
    Option<Box<[u8]>> }`, a template's file form's members, and
    `ProjectLanding`'s `repairs: Option<RepairSettings>`; in the store
    and the client's wire, records of the same members;
  - the setting resolved at a start (section 7): `Repairs { goals: u8,
    attempts: u8, deadline_ms: u64, goal: RepairGoal }`, `RepairGoal {
    charter: u32, llm: Option<LlmChoice>, priority: u32, budget: u64
    }`, `LlmChoice` jig's (`jig_domain_tasks`), in the forge's memory
    only, which `Domain::project_repairs(project) -> Result<&Repairs,
    Box<[u8]>>` answers, its `Err` 4.2's first refusal's words;
  - `Triage`'s variants `Repair(RepairAct)` and `Blame(BlameAct)`,
    `RepairAct { more: Box<[u32]>, words: Box<[u8]> }` (the findings
    past `TriageFinding`'s `number`) and `BlameAct { landing:
    Option<Commit>, why: Box<[u8]> }`; `FindingDone`'s `Repaired(u64)`
    and `Blamed`, and `TriageFindingAnswer`'s `Repaired(u64)`, the goal
    a repair made, beside `Triaged`. New refusals are
    `FindingRefusal::Words` in 4.2's and section 3's words. The stored
    finding reply (`FindingResult`) appends `repaired` and `blamed`.
    `FindingReply` appends `more: Box<[u32]>` (at most 15) and
    `landing: Option<Commit>`, in the store defaulted empty and none:
    the call's `RepairAct.more` and `BlameAct.landing`, set for
    `repair` and `blame` whatever the result and empty and none for
    every other act, so that the tool's answer is worded from its reply
    alone, on a restart too.
- **The tier's record** (`temper-engine-domain-forge-tiers`' `Record`)
  keeps its last judged run's number where it keeps none today, an
  added field, none in a record from before, for judging's why (5.3).
- **The finding tool** (`agent/tools.md`): its `act` gains `repair`
  and `blame`, and its members `numbers` (an array of 1 to 16 finding
  numbers, for `repair`), `words` (for `repair`) and `commit` (40 or 64
  hex digits, for `blame`, absent for none; `why` as `hand` takes it).
  The enumeration's growth moves the host tools' contract to its next
  version, as `findings` moved `decided`'s, the previous version's
  adapter handing its `finding` calls on unchanged.
- **ctl** (`../client/ctl/tools.md`): `finding repair FINDING...
  TEXT --key KEY` and `finding blame FINDING (--landing COMMIT |
  --none) TEXT --key KEY`, FINDING as `P/N`, all of one project, COMMIT
  40 or 64 hex digits, TEXT the act's words or the blame's why. They
  send `TriageFinding` with the act and print `repair goal G made for
  P/N, P/M`, or `blamed: P/N on landing COMMIT` (its first twelve
  digits) or `blamed: P/N on no landing`; a refusal prints its words,
  exit 2.
- **The client's wire** (`../client/forge/records.md`): `Triage` gains
  `repair` and `blame`, and the triage answer the goal a repair made,
  in its next layout, unbridged (`../protocol/wires.md`, section 9).
- **The worker** sees a repair goal as any goal.

## 9. The stock

- **The classes:** `Classes::stock` (`temper-protocol-reaches`), for
  Agents review and Draft first alike, gains an eighth item after
  `findings` at Ask: kind `findings`, at Decide, its words "a repair
  goal for a routine red after landing" (`forge/attention.md`, section
  6). A `decided` naming it by `under` 8 is decided; any other
  `findings` choice still asks.
- **Charter 14's instructions**
  (`../../../crates/temper-engine-service-stock/src/stock/triager.md`)
  gain section 2's sorts, one a finding or a group, the acts `repair`
  and `blame`, and what to do when `repair` refuses.

## 10. The world

Focused, each failing first, in the forge domain's library tests and
the root's world (`temper-engine-domain-world`):

- settings: kind 4's `repairs` round-trips a settings file with its
  defaults; is refused without `wait: landed`, without `goal`, and with
  an unknown charter or model; a start with a charter gone turns repairs
  off;
- repair: `repair` makes one goal of `repairs.goal`'s shape for two
  findings, its title line within 60 characters, its words naming each
  finding by `Fixes-finding: n`, each finding routed with attempt 1;
  refused at `repair_goals` live, past `repair_attempts`, after a
  cancel, across two repositories and with `repairs` off; a restart
  mid-decision makes the goal once;
- judging: a repair goal that landed, whose run still fails its tests,
  sends its finding back `new` with its why; one ended without landing
  is judged at once; a cancelled one's findings go back at once; a goal
  past `repair_deadline` ends failed and is judged; a finding routed by
  `route` is judged at its goal's end;
- blame: `blame` moves a finding's blame to another landing or to
  none, its strength `moved`, its mover and why kept;
- stock: the Agents review and Draft first tables hold the
  routine-repair item at Decide, and charter 14 names the sorts.

Fuzzy: random runs, landings, goal ends, restarts, triage acts and
blame moves keep every failing id in at most one uncleared red finding
of its tier, live repair goals within `repair_goals`, and each
finding's attempts within `repair_attempts`.

## 11. Later

- **The draft's Later** (`../draft/after-landing-repairs.md`, section
  12): repairs in the culprit's milestone, a revert as a red's option,
  a repair goal's own tier run.
- **Findings on:** `repairs` refused without kind 4's `findings` on,
  once that member is built.
- **Pages:** repair and blame acts on the findings page; until then,
  ctl and the agents' tool.
