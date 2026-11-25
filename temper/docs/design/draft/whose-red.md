# whose-red: one way to judge a red, one release rule, and temper's own writes

Draft, 2026-11-25, on the user's word: the fourth of the five drafts the user accepted on 2026-11-20, from the review of goal execution on the reference deployment (its group on the change lifecycle). It builds on [bounded-waits.md](bounded-waits.md), where every wait is a record with a bound and every hold names its cause and its way out (8), and on [landing-liveness.md](landing-liveness.md), whose landings settle what they hold. Its reasons are in [whose-red.notes.md](whose-red.notes.md), its execution in [whose-red.execution.md](whose-red.execution.md).

## 1. In one page

A change decides each step by comparing one fact it remembers (a head, a pin, a pull number, a verdict) with one fresh read. A mismatch becomes a hold or a resend, and a red becomes a repair. Each incident added an exception where it was seen: three fixes for a forge view behind temper's own writes, separate paths for a flake, a repair that found no failing test, a repair that changed nothing and a gate with no report, and a hold type of fourteen arms, each released in its own way. A goal's close check that refuses runs three more times in full before anyone hears of it.

This draft gives temper one rule for each of these:

1. **One judgement of a check's outcome** (3). Every check, at a delivery, a change's CI, a landing's gate or a goal's close, ends passed, refused by the change's own failures, or not the change's: the runner's, or the check's own. Code judges it, from the check's verdict record and its run's facts. Only a refusal asks a repair or reaches the goal. What isn't the change's is tried again within its own allowance and never spends a repair. A project says whether its checks can flake at all, and fabrig's can't.
2. **A check that refused holds at once** (4). A close command says by its exit whether it ran to a verdict. A refusal finishes the job with its verdict, and the close holds at once with the failures. Only a check that broke is tried again.
3. **A repair that changes nothing** (5), at CI and at a landing alike: one automatic check of the same content, then a hold naming the red as not the change's to repair.
4. **One release rule** (6). A change holds with one record: a cause, its words and its way out. A release drops what the held step remembered, reads afresh and re-enters at the earliest step the fresh facts support. A released check runs again only when what it checks has changed, or when the person releasing asks for it.
5. **A record of temper's own writes** (7). Every write temper sends to a branch or a pull, a goal's run delivering its design included, is recorded before it's sent and settled when it's answered. Drift is a move no record explains. A forge view behind the record is a bounded wait, not a hold.

What it leaves to the other drafts is in section 9.

## 2. Where it starts

| What happened | Why it went on | Section |
|---|---|---|
| Goals 15519 and 15520's close checks refused on test failures on 11-25, and each ran three more times in full before holding: about 23 and 8 minutes of a worker, the goal hearing that much later | Any exit but 0 is the close job's own failure, which jig tries three more times. pre-PR's replay cache (b025ef123f) repeats a budget or format refusal, but not a test failure, and still spends a try | 4 |
| pre-PR exits 1 for a refusal and for an exception in its own code alike, and `temper/check-results` reads `refused` for both | The check's outer contract has two outcomes, not three | 3.1 |
| pre-PR runs a failing test's process again and passes it as a flake when it passes, at most `checkout.rerun_processes` of them (stock 4). fabrig's checks run only tests that replay, so such a pass hides a bug. The figure was set to 0 on the reference deployment by hand on 11-25 | Whether a project's checks can flake is a deployment figure, not the project's statement | 3.3 |
| A plain no-change landing repair holds `Failed`, and its release asks a fresh repair (P67). A no-change CI repair reruns and can ask more repairs up to the allowance of 80 (F22). A repair's flake or no-test verdict holds until a person releases it for one regate (F64, F127) | Each found its own path | 5 |
| A landing gate with no report of its own blamed the change (P116). Fixed for a configured report that's missing (F126), but an empty report still asks a repair (P37) | The runner's fault is judged at one site of four | 3 |
| Goal 13861 held for branch drift on 11-23, two minutes after its own run delivered `04b27280` onto its branch, merging the expected `45fdb415`. It was released by hand | A design writer's delivery is not recorded as temper's write: only a pair of heads observed in order is (F108) | 7 |
| Three fixes for a late forge view read as drift: a lagging pull head (F6), a lagging pull base (F63) and temper's own later commit (F108) | Each at its own call site | 7 |
| The change's hold has fourteen arms, each released in its own way, and blocked changes were cancelled and cut again by hand (F2) | A release restores the held step and runs the same comparison again | 6 |

## 3. One judgement of a check's outcome

### 3.1 A check's verdict is a record

- **A check that ran to its verdict writes it**: passed or refused, the tree and the base it judged, and each failure by its key and class, as pre-PR's `pre-pr-failures 1` record does today (`tools/pre-pr/design.md`, 6). A gate's report is its record (`../domain/landing-proof.md`, 1.2).
- **A check that broke writes no refusal.** An exception in the checker's own code, a helper it couldn't build, an input it couldn't read: it ends broke, with its words, never refused. A refusal says the change is red; a broken check says nothing about the change.
- **Three outcomes, by exit:** 0 passed, a refusal's own exit with its record, and every other exit, a signal, a deadline, or a refusal's exit without its record, broke. Section 4.1 gives a close command's; a delivery's and a gate's keep their channels and gain the third outcome.

### 3.2 The classes

Code judges each outcome once, by one function, the same at every site (3.4):

| Class | From | What follows |
|---|---|---|
| passed | a passed record; inherited failures and, where the project's checks can flake, flakes named in it | the next step |
| refused | a refused record whose failures are the change's own: new, not inherited | the site's own action (3.4): a repair, the run told, or the close held (4) |
| the runner's | the run's facts: a setup or completion failure, no failed step, a cancel temper didn't ask for, a lost or starved run, a gate whose report is missing or names no failure, a killed process, a full disk | tried again within the runner's allowance, never a repair; past it, a hold naming the runner and the fact |
| the check's own | a broke outcome (3.1) | tried again within the run's own allowance; past it, a hold naming the check, its words and that the change wasn't judged |

A red is the change's only when a record names its failures. The runner's and the check's own never spend the change's repairs, and never reach a repair's brief as the change's red. An agent's verdict on a repair it made (a flake, no failing test) is not a class: it is what section 5 reads.

### 3.3 Whether a project's checks can flake

- **A project setting**, `checks.flakes`: `none`, or `rerun N`, N at most the deployment's `checkout.rerun_processes`. Stock: `rerun` at that figure, as today.
- **With `none`:** temper writes `temper-rerun 1 0` for the project's checks (`../domain/forge/reds.md`, 5.2), the landing gate runs no known-flake rerun (`../domain/landing-regate.md`, 2), and no flake finding is filed for the project. A test that fails is the change's red, or a repair shows it isn't (5).
- **fabrig's is `none`** (the user, 2026-11-25: pre-PR "is deterministic and shouldn't flake"): its checks run only tests that replay, which own their clock, seeds and kernel (`tools/tiers/layers.md`, 6.1). A test that fails and then passes is a determinism bug, to be fixed where it lies.

### 3.4 At each site

| Site | Refused | Not the change's |
|---|---|---|
| A delivery's checks | the run gets the refusal and its record, as today | the run is told the check broke, or the runner failed, and may deliver again; nothing is refused for the change |
| A change's CI | a counted repair, as today | a rerun within the runner's or the check's allowance (`forge.change_policy.reruns`), then a hold |
| A landing's gate | the red unit's repairs, as today | the gate's retries (`gate_retries`), then a hold on the landing (`../domain/landing-proof.md`, 2.4) |
| A goal's close | the close holds at once (4) | the job is tried again in its class, then held |

## 4. A check that refused holds at once

### 4.1 The close command's exits

The close command (`../protocol/tooling.md`, 4.2) gains a third exit, as merge's exit 3 already finishes with a result and delivers nothing (4.1 there):

- **Exit 0,** as today: settled, committed and pushed.
- **Exit 3, refused:** `result.json` holds the refusal, `{"refused": {"tree": T, "base": B, "failures": [...], "more": N}}`, its failures cut to the kind's result bound as `merge`'s are. Nothing is committed or pushed. The job finishes, spending no try, and the close change holds at once with the refusal (4.2).
- **Any other exit,** a signal, the job's deadline, or a result missing or past its bound: the check broke. The job fails in the run's own class and is tried again within its tries, then held, its words naming the check, not the change.

- **Only when asked for:** the close's `request.json` says `"version": 2` when the engine reads exit 3 so. A command given version 1 exits as today, so a refusal never reaches a build that would read a finished job with nothing pushed as a close with nothing to settle (8).

In fabrig, `.temper/checked-close` settles, then runs pre-PR's close check. On a refusal, pre-PR writes it into `result.json` from its close record (`goal-close-pre-pr 1`) and exits 3. Its own failure exits 2.

### 4.2 Its hold and its release

- **The hold** is the close change's refusal (6.1): its words name the first failures and the record, and it escalates to the goal at once, as a held close does today (`../domain/forge/change/repairs-releases.md`, 1).
- **Its release** checks again only when what the close checks has changed: the goal's branch tip or its landing branch's tip (ff44375285). Landed work supersedes a held close, as today (`../domain/forge/branches.md`, 2).
- **The replay cache retires.** With no try after a refusal, pre-PR's cache of hard refusals (`tools/pre-pr/design.md`, 4) has nothing left to replay.

## 5. A repair that changes nothing

- **One rule, at CI and at a landing.** A repair that leaves the change's head where it was, whatever its verdict (no change, a flake, no failing test), is followed by one check of the same content: a rerun of the change's CI, or a regate of its landing's key. It spends no repair.
- **Green, it goes on.** The repair's verdict and words are kept with the change.
- **Red again at the same content,** it holds, *not the change's to repair*: "Its red at H is not its to repair: repair R changed nothing, and the same content failed again: KEYS." Its way out is a person's or the goal's: a release that measures again (6.2), an amendment, or a cancel.
- **What it replaces:** F22's reruns up to the repairs allowance, P67's `Failed` hold and its fresh repair, and F64's and F127's hold before their one regate. A repair that moves the head is a delivery, as today.

## 6. One release rule

### 6.1 One hold, with a cause

A held change keeps one record: the step it held at, its cause, its words, its *since* and its way out (`bounded-waits.md`, 8). The fourteen arms become causes of four kinds:

| Kind | Causes |
|---|---|
| A bound spent | repairs, resolutions, updates, questions, authority, a wait past its bound |
| The forge disagrees | the branch moved (7) or is missing; the pull closed or retargeted |
| A judgement | a person rejected its gate; its close refused (4); its red is not its to repair (5) |
| A failure | a delegate or an effect failed for good |

### 6.2 What a release does

One function, for every cause:

1. **Forget the held step:** its outcomes, pins and verdicts, and the spent allowance of the cause alone.
2. **Read afresh:** the branch, its base and the pull, through the record of temper's writes (7), so a view behind them waits rather than holds again.
3. **Re-enter at the earliest step the fresh facts support:** producing with no branch, opening with no pull, checking at a new head, gating at the same head with its asks cleared, as the change's steps run them (`../domain/forge/change/README.md`, 2).
4. **A check runs again only when what it checks has changed** since the verdict it held on. A person releasing may ask for a fresh measurement of the same content instead, and the hold's words say so.

The engine's own releases keep their triggers (a green read at the head ends a CI hold, a delegate's end ends its stall) and go through the same function. A change blocked at any step is moved on by its release, so a cancel and a fresh cut (F2) are never needed to unblock one.

### 6.3 What it keeps

A release resets only its cause's allowance: a released repairs hold may ask repairs again, but a release for a moved branch leaves the repairs taken. Questions are never reset by a release, as today. A judgement released without a change to its inputs and without a request to measure again holds again at once, with words saying so, as a released close does today.

## 7. A record of temper's own writes

### 7.1 The record

- **Per branch and per pull,** temper records every write it sends: a push, an update, a merge, an open, a reopen, a retarget, a slot writer's or a merge job's push, a close's push, and a goal's run delivering its design or saving its work through the worker. Each entry holds the write's precondition (the head or base it expects), the commit it writes, who sent it and its state: sent, confirmed, refused or uncertain.
- **Recorded before it's sent, settled when it's answered,** in the same decision as the effect it records. A write whose answer is lost stays uncertain until a read settles it.
- **Bounded.** An entry is kept until a read at or past its commit confirms it, and the record has a figure of its own per branch, which names its setting when full. The oldest confirmed entries go first.

### 7.2 Drift is a move no record explains

- **A fresh read is explained** when its head is an entry's commit, or reached from the expected head by entries' commits alone, sent or settled.
- **A view behind the record,** a pull head or base older than a confirmed write, is a wait inside the forge client, read again until it catches up, with a bound of its own (`bounded-waits.md`, 4): the same for every caller, at opening, at an update and at a release.
- **Drift** is a head no entry explains: a push by another actor, a move backwards, or a commit temper never sent. It holds as today, its words naming the expected head, the observed one and the nearest entry.

### 7.3 What it replaces

The pair of heads F108 keeps beside a branch hold, the lagging-head exceptions of F6 and F63, and the writer flag's ancestry exceptions on a kept branch all become reads of the record. Goal 13861's case reads so: the run's delivery of `04b27280` enters the record when the worker pushes it, with `45fdb415` as its precondition, so the read that saw `04b27280` finds it explained.

## 8. Across restarts and builds

- **Stored:** the hold's cause, a repair's no-change verdict awaiting its check (5), and the write record (7.1) are stored with the change, its branch and its pull, so a start finds them as they were.
- **The previous build's holds** convert by arm to the causes of 6.1, each arm to one cause. An arm the previous build held and this build would explain (drift the record now explains, for one) still holds once, and its release follows 6.2.
- **The previous build's branches** have no write record. A start takes each branch's expected head, and F108's pair where one is kept, as its first confirmed entry, so nothing held is released and nothing new is read as drift.
- **A close's exit 3** is read only by an engine that asks for it (4.1). The previous build sends version 1, so its closes refuse by exit 1 and keep today's tries. A goal's branch made before the command learned version 2 exits 1 for every refusal, until it merges its base.

## 9. Out

- **Provider failures, accounts and funding,** classified as limited, refused, out of credits or a model not served: the fifth draft. A run's model failures are not checks.
- **One capacity model:** here a starved run is the runner's; how much room a gate gets, and when a run is starved, is the review's capacity part, for a later draft.
- **One structured record of what a check found, for every brief:** this draft needs only the verdict records it names (3.1). What a repair's brief reads of them stays as it is.
- **Waits and holds' words:** [bounded-waits.md](bounded-waits.md). **A landing's members and endings:** [landing-liveness.md](landing-liveness.md).

## 10. Tests

- **The judgement, at each site:** focused tests that each class comes from its facts, at a delivery, a change's CI, a landing gate and a close: a refused record is the change's, a broke outcome the check's, a missing or empty report the runner's, and neither of the last two asks a repair. P37's scenario (`a_starved_gate_is_the_runners`) passes.
- **A refused close holds at once:** a close exiting 3 holds with its failures and no retry, the goal hearing at once; one exiting 1 or past its deadline is tried again, then held naming the check. pre-PR's own tests: an exception in its code exits broke, not refused, and writes no refused record.
- **`checks.flakes none`:** the rerun marker reads 0, the landing gate runs no known-flake rerun, and no flake finding is filed.
- **A repair that changes nothing,** at CI and at a landing: one check of the same content, green going on, red holding *not the change's to repair*. P67's (`a_no_change_landing_repair_regates_once`) and P58's (`a_repair_whose_tests_pass_regates_by_itself`) scenarios pass.
- **One release rule:** for each cause, a release from each step re-enters where 6.2 says, resets only its cause's allowance, and a judgement released with nothing changed holds again at once. A fuzzy property in the change policy's sweeps: from any held step, a release followed by the forge agreeing converges without a person.
- **Temper's own writes:** a fuzzy sweep of writes and reads in any order, views lagging, answers lost and restarts between: temper's own writes are never drift, every outside write is, and a lagging view always ends at its bound. Goal 13861's shape (a run's delivery merging the expected head, read before any other) is a scenario of its own.
- **The previous build:** each old hold arm converts to its cause, and a branch with F108's pair starts with it as its record.

## 11. Later

- A check's verdict record read by every repair's brief, paged, with the other checks' records.
- A project's flake setting learned from its findings, rather than stated.
