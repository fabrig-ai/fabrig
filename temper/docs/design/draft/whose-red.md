# whose-red: one way to judge a red, one release rule, and temper's own writes

Draft, 2026-11-25, on the user's word: the fourth of the five drafts the user accepted on 2026-11-20, from the review of goal execution on the reference deployment (its group on the change lifecycle). It builds on [bounded-waits.md](bounded-waits.md), where every wait is a record with a bound and every hold names its cause and its way out (8), and on [landing-liveness.md](landing-liveness.md), whose landings settle what they hold. Its reasons are in [whose-red.notes.md](whose-red.notes.md), its execution in [whose-red.execution.md](whose-red.execution.md).

## 1. In one page

A change decides each step by comparing one fact it remembers (a head, a pin, a pull number, a verdict) with one fresh read. A mismatch becomes a hold or a resend, and a red becomes a repair. Each incident added an exception where it was seen: three fixes for a forge view behind temper's own writes, separate paths for a flake, a repair that found no failing test, a repair that changed nothing and a gate with no report, and a hold type of fourteen arms, each released in its own way. A goal's close check that refuses runs three more times in full before anyone hears of it.

This draft gives temper one rule for each of these:

1. **One judgement of a check's outcome** (3). Every check, at a delivery, a change's CI, a landing's gate or a goal's close, ends passed, refused by the change's own failures, or not judged: the runner failed, or the check broke. Code judges it, from what each site can show of a verdict and from its run's facts. Only a refusal asks a repair or reaches the goal. A check that didn't judge is tried again within its own allowance and never spends a repair. A project says whether its checks can flake at all, and fabrig's can't.
2. **A check that refused holds at once** (4). A close command gets a typed refusal. A refusal finishes the job with its verdict, and the close holds at once with the failures. Only a check that broke is tried again.
3. **A repair that changes nothing** (5), at CI and at a landing alike: one automatic check of the same content, then a hold that says the red stands though the repair changed nothing.
4. **One release rule** (6). A change holds with one record: a cause, its words and its way out. A release settles what is owed, forgets the stale reads and verdicts of the held step, reads afresh and re-enters where the facts say. A released judgement runs again only when its inputs have changed, or when its decider asks for a fresh measurement.
5. **A record of temper's own writes** (7). Every write temper sends to a branch or a pull, a goal's run delivering its design included, is admitted into a record before it reaches the forge, with what it changes from and to, and settled when it's read back. Drift is a change no record explains. A forge view behind the record is a bounded wait for the caller that needs it, not a hold.

What it leaves to the other drafts is in section 9.

## 2. Where it starts

| What happened | Why it went on | Section |
|---|---|---|
| Goals 15519 and 15520's close checks refused on test failures on 11-25, and each ran three more times in full before holding: about 23 and 8 minutes of a worker, the goal hearing that much later | Any nonzero exit is the close command's failure (`../protocol/tooling.md`, 4.2), which jig tries three more times. pre-PR's replay cache (b025ef123f) repeats a budget or format refusal, but not a test failure, and a repeat still spends a try | 4 |
| pre-PR exits 1 for a refusal and for an exception in its own code alike, and `temper/check-results` reads `refused` for both | The check's outer contract has two outcomes, not three | 3.1 |
| A goal's full pre-PR can run a failing test's process again and pass it as a flake, up to the bound in its checkout's `temper/rerun` (`checkout.rerun_processes`, stock 4, where a worker writes it). fabrig's checks run only tests that replay, so such a pass would hide a bug. It never fired for fabrig: the CI runner's checkouts have no marker, so the bound is 0, and close mode never reruns. The reference deployment's figure was set to 0 on 11-25 as a guard | Whether a project's checks can flake is a deployment figure and a marker, not the project's statement | 3.3 |
| A plain no-change landing repair holds `Failed`, and its release asks a fresh repair (P67). A no-change CI repair reruns and can ask more repairs up to the allowance of 80 (F22). A repair's flake or no-test verdict holds until a release lets it regate once (F64, F127) | Each found its own path | 5 |
| A landing gate with no report blamed the change (P116). Fixed for a configured report that's missing (F126), but a report that names no failure still asks a repair (P37) | The runner's fault is judged at one site of four | 3 |
| Goal 13861 held for branch drift on 11-23, four minutes after its own run delivered `04b27280` onto its branch, a merge of the expected `45fdb415`. It was released by hand | A design writer's delivery is not recorded as temper's write when it's pushed: it's taken only through a later fresh read, and only a pair of heads observed in order is kept beside a branch hold (F108) | 7 |
| Three fixes for a late forge view read as drift: a lagging pull head (F6), a lagging pull base (F63) and temper's own later commit (F108) | Each at its own call site | 7 |
| The change's hold has fourteen arms, each released in its own way, and blocked changes were cancelled and cut again by hand (F2) | A release restores the held step and runs the same comparison again | 6 |

## 3. One judgement of a check's outcome

### 3.1 What a check can show

- **A record,** where a check promises one: passed or refused, the tree and the base it judged, and each failure by its key and class, as pre-PR's `pre-pr-failures 1` record does (`tools/pre-pr/design.md`, 6), or a gate's configured report (`../domain/landing-proof.md`, 1.2).
- **What the run shows,** where it promises none: a failed status, a failed workflow step, a failed job with its log (`../domain/landing-proof.md`, 2.3). These stay as they are judged today: a failure with no fact of the runner's is the change's.
- **Candidate diagnostics are the change's:** a compiler error, a lint, a test failure, an exceeded count, wherever they show, a helper the check builds from the candidate's tree included. **Errors of the check or its machine are not:** an exception in the checker's own code, a signal, a lost process, a full disk, a network or forge read that failed, a deadline.
- **A record's promise binds its check.** A check that promises a record and ends without one, or with a refusal that names no failure, didn't judge the change. pre-PR gains a third outcome for its own errors (4.1), so `refused` means the change was judged.

### 3.2 The classes

Code judges each outcome once, by one function, the same at every site (3.4):

| Class | From | What follows |
|---|---|---|
| passed | a passed verdict; inherited failures, starved tests without an observed failure, and, where the project's checks can flake, flakes, each named | the next step; a starved test is a warning (`skein/docs/foundation/testing-strategy.md`, 8.1) |
| refused | a verdict naming the change's own failures (new, not inherited), or a run's candidate diagnostics where no record is promised (3.1) | the site's own action (3.4): a repair, the run told, or the close held (4) |
| the runner's | the run's facts: a setup or completion failure, no failed step, a cancel temper didn't ask for, a lost run, a killed process, a full disk, a run stopped at its wall with nothing observed, an empty or missing report from a run its facts show interrupted | tried again within the runner's allowance, never a repair; past it, a hold naming the runner and the fact |
| broke | the check's own error (3.1), or a promised record missing or naming no failure from a run that wasn't interrupted | tried again within the run's own allowance; past it, a hold naming the check, its words and that the change wasn't judged |

- **A verified red stands.** When one job of a check refused and another failed for the runner, the check is refused: the runner's fault never hides a red already shown.
- **Not the change's, never repaired.** The runner's and broke never spend the change's repairs and never reach a repair's brief as its red.
- **An agent's verdict** on a repair it made (a flake, no failing test) is no class: section 5 reads it.
- **Advisory gates** are judged the same and hold nothing, as today (`../domain/landing-proof.md`, 2.2).

### 3.3 Whether a project's checks can flake

- **A project setting**, `checks.flakes`: `none`, or `rerun N`, N at most the deployment's `checkout.rerun_processes`. Stock: `rerun` at that figure, as today.
- **With `none`:** every checkout temper prepares or dispatches for the project's checks, on a worker or a CI runner, has `temper-rerun 1 0` (`../domain/forge/reds.md`, 5.2). The landing gate runs no known-flake rerun (`../domain/landing-regate.md`, 2), and no flake finding is filed for the project. A test that fails is the change's red, or a repair shows it isn't (5). A test that passes alone after failing beside others is still red, its log naming the tests before it, as today.
- **The after-landing tiers keep their one rerun:** a project's `flakes` doesn't reach them. A red tier runs its failed tests again and calls those that pass a flake, kept on the tier's record (`../domain/after-landing.md`, 5.2), whatever the setting says (the drafter, 2026-11-26).
- **fabrig's is `none`** (the user, 2026-11-25: pre-PR "is deterministic and shouldn't flake", and the landing gate's known-flake rerun goes with it): its checks run only tests that replay, which own their clock, seeds and kernel (`tools/tiers/layers.md`, 6.1). A test that fails and then passes is a determinism bug, to be fixed where it lies.

### 3.4 At each site

| Site | Refused | The runner's or broke |
|---|---|---|
| A delivery's checks | the run gets the refusal and its record, as today | the run is told the check didn't judge, and why, and may deliver again. Today every nonzero exit is a refusal (smith's `domain/run-delivery.md`, 1.1) |
| A change's CI | a counted repair, as today | a rerun within the runner's allowance (`forge.change_policy.reruns`), then a hold |
| A landing's gate | the red unit's repairs, as today | the gate's retries (`gate_retries`), then a hold on the landing (`../domain/landing-proof.md`, 2.4) |
| A goal's close | the close holds at once (4) | the job is tried again in its class, then held |

## 4. A check that refused holds at once

### 4.1 The close command's typed refusal

The close command (`../protocol/tooling.md`, 4.2) keeps its exits, and its exit 3 gains a meaning. The worker already reads `result.json` after exit 0 or 3, as merge's exit 3 finishes with a result and delivers nothing (4.1 there):

- **Exit 0,** as today: settled, and what it staged committed and pushed. Nothing staged is success with no change.
- **Exit 3, refused:** `result.json` is `{"refused": {"tree": T, "base": B, "failures": [...], "more": N}}`, its failures cut to the kind's result bound as `merge`'s are. Nothing is committed or pushed. The job finishes, spending no try, and the close change holds at once with the refusal (4.2).
- **Any other exit,** a signal, the job's deadline, or a result missing or past its bound: the check broke. The job fails in the run's own class and is tried again within its tries, then held, its words naming the check, not the change.
- **Asked for at both ends.** `request.json` stays version 1, since a command of the previous build refuses any other. The engine asks for the refusal only of a worker whose opening says it can carry it, and that worker sets `TEMPER_CLOSE_REFUSAL=3` in the command's environment. A command that doesn't see it exits as today, so a refusal never reaches a build that would read a finished job with nothing pushed as a close with nothing to settle.
- **Checked against its attempt.** The engine takes a finished close as a refusal only when it asked for one, its result is `refused`, and its tree is the one that attempt checked. As settlement it takes only `settled`. Any other finished result is the command's failure.

In fabrig, `.temper/checked-close` settles, then runs pre-PR's close check. With the variable set, a refusal is written into `result.json` from pre-PR's close record (`goal-close-pre-pr 1`) and exits 3, and pre-PR's own error exits 2. Without it, both exit 1, as today.

### 4.2 Its hold and its release

- **The hold** is the close change's refusal (6.1). Its words name the first failures and the record, and it escalates to the goal at once, as a held close does today (`../domain/forge/change/repairs-releases.md`, 1).
- **Its release** checks again only when what the close checked has changed: the goal's branch tip, its landing branch's tip, or the published counts the check judged against (`tools/pre-pr/design.md`, 4), each recorded with the refusal. Landed work supersedes a held close, as today (`../domain/forge/branches-closer.md`, 1). Its decider may ask for a fresh measurement instead (6.3).
- **A close that broke** has no verdict to keep: its release runs it again, unchanged inputs or not.
- **The replay cache retires.** With no try after a refusal, pre-PR's cache of hard refusals (`tools/pre-pr/design.md`, 4) has nothing left to replay.

## 5. A repair that changes nothing

- **One rule, at CI and at a landing.** A repair that leaves the change's head where it was, whatever its verdict (no change, a flake, no failing test), is followed by one check of the same content: a rerun of the change's CI job, or a regate at its landing's key, spending no repair.
- **Its identity is kept.** The change stores the check it repairs (its run, job attempt or receipt key), the measurement asked in its place and whether it has been taken. A restart finds it so. Only a measurement taken after the repair counts: an older receipt, or a key a new landing made at the same head, does not.
- **Where the same content can't be checked again,** a CI job the forge can't rerun among them, it holds at once with words saying so.
- **Green, it goes on.** The repair's verdict and words are kept with the change.
- **Red again at the same content,** it holds: "Its red at H stands: repair R changed nothing, and the same content failed again: KEYS." The red may be the change's, a regression the repair didn't fix, or not. The goal or a person decides, by a repair asked afresh, an amendment, a fresh measurement (6.3) or a cancel.
- **What it replaces:** F22's reruns up to the repairs allowance, P67's `Failed` hold and its fresh repair, and F64's and F127's hold before their one regate. A repair that moves the head is a delivery, as today.

## 6. One release rule

### 6.1 One hold, with a cause

A held change keeps one record: the step it held at, its cause, its words, its *since* and its way out (`bounded-waits.md`, 8). The fourteen arms become causes of four kinds:

| Kind | Causes |
|---|---|
| A bound spent | repairs, resolutions, updates, questions, authority, a wait past its bound |
| The forge disagrees | the branch moved (7) or is missing; the pull closed or retargeted |
| A judgement | a person rejected its gate; its close refused (4); its red stands after a repair that changed nothing (5) |
| A failure | a delegate or an effect failed for good |

### 6.2 What a release does

One function, for every cause, taken once per hold revision and fenced against reads made before it, as releases are today (`../domain/forge/change/repairs-releases.md`, 1):

1. **Settle what is owed first:** a delegate's durable result, an accepted decision, an effect in flight (an uncertain write is read back before anything else), a landing's ask or its uncertain merge (`landing-liveness.md`, 5.2). These are kept, never forgotten.
2. **Forget what is stale:** the held step's reads, pins and verdicts, except a held judgement's verdict with the inputs it judged, and the spent allowance of the cause alone. The change's owned pull, its identity and the record of its writes (7) are kept.
3. **Read afresh:** the branch, its base and the pull, through the record of temper's writes, so a view behind them waits rather than holds again.
4. **Re-enter at the earliest step the fresh facts support:** producing with no branch, opening with no pull, checking at a new head, gating at the same head with its asks cleared, as the change's steps run them (`../domain/forge/change/README.md`, 2).

The engine's own releases keep their triggers (a green read at the head ends a CI hold, a delegate's end ends its stall) and go through the same function. A change blocked at any step is moved on by its release, so a cancel and a fresh cut (F2) are never needed to unblock one.

### 6.3 Judgements and fresh measurements

- **A judgement released with nothing changed holds again at once,** with words saying so: a refused close whose inputs (4.2) are the same, a red that stands at the same content (5). A person's rejection of a gate stays held until the person lifts it: no engine release or forge read ends it, nor does a hold a party placed.
- **A fresh measurement** of the same content is an act of its own, `task measure TASK --hold REV`, distinct from `task release --again`, which jig keeps for an uncertain effect. Whoever may decide the hold's escalation may take it, the goal for its own changes' holds among them. It is revisioned like a release, and one accepted act takes one measurement.
- **A release resets only its cause's allowance.** A released repairs hold may ask repairs again; a release for a moved branch leaves the repairs taken. Questions are never reset by a release, as today.

## 7. A record of temper's own writes

### 7.1 The record

- **Per branch and per pull,** temper records every write it makes: a push, an update, a merge, an open, a reopen, a retarget, a close of a pull, a deletion of a branch, a slot writer's or a merge job's push, a close's push, and a goal's run delivering its design or saving its work through the worker. Each entry holds the operation, its before state and its after state (a branch's head from and to; a pull's state, head and base from and to), who sent it, its order on its resource, and its state: admitted, sent, confirmed, refused or uncertain.
- **Admitted before it reaches the forge.** A write temper sends through its forge client is admitted in the decision that asks for it. A write a worker pushes itself (a design delivery, a save, a job's push) waits until the engine has durably admitted its destination, its expected head, its commit and its attempt, and is pushed only then. A write whose after state only the forge makes (a merge commit, an update's result) is admitted as an intent with its before state, and its after state is filled in at readback.
- **Settled by readback.** A write whose answer is lost stays uncertain until a read settles it. An uncertain write is never dropped.

### 7.2 Drift is a change no record explains

- **A fresh read is explained** when it is the after state of the newest settled entry on its resource, or the after state of an entry still admitted, sent or uncertain, in order. A pull closed or retargeted is explained only by temper's own close or retarget.
- **A view behind the record** (a read that is the before state of a confirmed entry, or an older head of the same history) waits for the caller that needs the fresh fact, read again until it catches up, within that caller's wait bound (`bounded-waits.md`, 4). A caller that doesn't need the fact doesn't wait for it.
- **Drift** is a change no entry explains: another actor's push, a move backwards, a reset to an older commit of temper's after a newer one was confirmed, a pull closed or retargeted by someone else. It holds as today, its words naming the expected state, the observed one and the nearest entry.
- **Can't tell.** Where the forge can't tell a stale view from another actor's write (a read of an older commit of temper's own, with no actor evidence), the caller reads again within its bound, then holds with words saying the read may be stale or another's write.

### 7.3 Its room

- **Admission waits for room.** The record has a figure of its own per resource. A write that finds it full waits before it's sent, a `room` wait (`bounded-waits.md`, 3.2), and names the figure. Only confirmed entries are ever evicted, oldest first. An admitted, sent or uncertain entry is never evicted.

### 7.4 What it replaces

The pair of heads F108 keeps beside a branch hold, the lagging-head exceptions of F6 and F63, and the writer flag's ancestry exceptions on a kept branch all become reads of the record. Goal 13861's case reads so: the run's delivery of `04b27280` is admitted before it's pushed, with `45fdb415` as its before state, so whichever read next sees `04b27280` finds it explained.

## 8. Across restarts and builds

- **Stored:** the hold's cause, a repair's measurement awaiting its check with its identity (5), the inputs a held judgement judged (4.2), and the write record (7.1) are stored with the change, its branch and its pull, so a start finds them as they were.
- **The previous build's holds** convert by arm to the causes of 6.1. `Flake` and `NoTest` convert to a repair that changed nothing with its one check still owed, so their release regates once, as today. An arm the previous build held and this build would explain (drift the record now explains, for one) still holds once, and its release follows 6.2.
- **The previous build's branches** have no write record. A start takes each branch's expected head as its first confirmed entry. F108's pair converts as an entry still to compare: its waiting head is compared by ancestry, as today, before any drift is judged. A writer the previous build kept as live or lost converts to an uncertain entry, read back before anything on its branch is decided. Nothing held is released and nothing new is read as drift.
- **The previous build's receipts:** a retained failed receipt whose report names no failure is judged again by 3.2 before it's reused.
- **A close's exit 3** is read as a refusal only by an engine that asks for one, of a worker that can carry it (4.1). The previous build asks for none, so its closes refuse by exit 1 and keep today's tries. A goal's branch made before the command learned the variable refuses by exit 1 too, until it merges its base.

## 9. Out

- **Provider failures, accounts and funding,** classified as limited, refused, out of credits or a model not served: the fifth draft. A run's model failures are not checks.
- **One capacity model:** here a starved run is judged as the testing strategy says; how much room a gate gets is the review's capacity part, for a later draft.
- **One structured record of what a check found, for every brief:** this draft needs only what each site can show of a verdict (3.1). What a repair's brief reads of it stays as it is.
- **Waits and holds' words:** [bounded-waits.md](bounded-waits.md). **A landing's members and endings:** [landing-liveness.md](landing-liveness.md).

## 10. Tests

- **The judgement, at each site:** focused tests that each class comes from its facts, at a delivery, a change's CI, a landing gate and a close. A record's refusal and a run's candidate diagnostics are the change's. A checker's error is broke, and an interrupted run the runner's. Neither asks a repair. A red shown beside a runner's fault stands. A starved test alone passes with its warning. P37's scenario (`a_starved_gate_is_the_runners`) passes.
- **A refused close holds at once:** a close exiting 3 when asked holds with its failures and no retry, and the goal hears at once. One exiting 1 or past its deadline is tried again, then held naming the check. An exit 3 not asked for, or with another tree, is the command's failure. A command without the variable exits as today. pre-PR's own tests: an exception in its code is broke, not refused.
- **`checks.flakes none`:** every checkout's rerun marker reads 0, a CI runner's included, the landing gate runs no known-flake rerun, and no flake finding is filed.
- **A repair that changes nothing,** at CI and at a landing: one check of the same content, taken after the repair and kept across a restart. Green goes on; red holds with the red standing. An older receipt never counts. P67's (`a_no_change_landing_repair_regates_once`) and P58's (`a_repair_whose_tests_pass_regates_by_itself`) scenarios pass.
- **One release rule:** for each cause, a release from each step settles what is owed first, re-enters where 6.2 says, and resets only its cause's allowance. A judgement released with nothing changed holds again, and a measurement takes one fresh check. A person's rejection survives the forge agreeing. A fuzzy property in the change policy's sweeps: from any held step, a release followed by the forge agreeing converges without a person, unless a judgement holds it.
- **Temper's own writes:** a fuzzy sweep of writes and reads in any order, with views lagging, answers lost and restarts between. Temper's own writes are never drift, and every outside write is: a push, a reset to an old commit of temper's, a pull closed or retargeted. A lagging view ends at its caller's bound, and an uncertain write is never evicted. Goal 13861's shape, a run's delivery merging the expected head, is a scenario of its own.
- **The previous build:** each old hold arm converts to its cause, `Flake` and `NoTest` keeping their one regate. A branch with F108's pair compares it first, and a live or lost writer is read back first.

## 11. Later

- A check's verdict read by every repair's brief, paged, with the other checks' records.
- A project's flake setting learned from its findings, rather than stated.
