# reference-m8: contracts

The shapes that cross tasks, binding for every spec that cites them. A later batch adds
sections; a released section changes only by a numbered revision.

## 1. The pure verdict policy: `temper-engine-domain-forge-tiers`

A new pure crate beside `temper-engine-domain-forge-landing`, depending on `skein-lib` alone,
`no_std` with `alloc`: it keeps no state and knows no forge call. The forge connector's top
(`temper-engine-domain-forge`) owns the records, reads reports and receipts, saves each
decision before dispatching its part and passes each completion with that committed run. It
serves `temper/docs/design/domain/after-landing.md`, sections 2 to 5, per tier: N5's branch
is the top's (which tip a tier's record follows), never this crate's.

Its public surface, as the old crate has it (`notes/old/temper-tiers-crate.diff`), names and
signatures unchanged:

```rust
pub type Commit = [u8; 32];
pub type Key = [u8; 32];            // a tier's key at a commit (landing.md, 6.1), the top's
pub type TestId = Box<[u8]>;        // at most Limits::test_id_bytes

pub struct Limits { pub failing_tests: u32, pub test_id_bytes: u32 }
pub struct Settings { pub retries: u32, pub suspects: Suspects }
pub enum Suspects { Bisect, TellAll }
pub enum Format { Junit, Lines }
pub enum Failures { Tests(Box<[TestId]>), Many }       // Tests: sorted, unique; empty is none
pub struct Unreadable;
pub enum Outcome { Passed, Failed(Failures), Fault }    // Failed never holds Tests(empty)
pub enum Receipt { Passed, Failed(Failures) }
pub struct Record {                                      // store family 25's policy part
    pub sequence: u64, pub judged: Option<Judged>, pub failing: Failures,
    pub flakes: Box<[Flake]>, pub run: Option<Run>,
}
pub struct Judged { pub tip: Commit, pub key: Key }
pub struct Flake { pub test: TestId, pub count: u32 }
pub struct Run {
    pub number: u64, pub tip: Commit, pub key: Key,
    pub landings: Box<[Landing]>,                        // oldest first; empty for the baseline
    pub part: Part, pub faults: u32,
}
pub struct Landing { pub commit: Commit, pub key: Key }
pub enum Part {
    Whole,
    Rerun { failed: Box<[TestId]> },
    Probe { landing: u32, tests: Selection, finding: Finding },
}
pub enum Selection { Tests(Box<[TestId]>), All }
pub struct Finding { pub failing: Failures, pub flakes: Box<[TestId]>, pub blamed: Box<[Blame]> }
pub enum Blame { Cleared, Owns(Failures), Suspect(Failures) }
pub struct Verdict {
    pub number: u64, pub tip: Commit, pub key: Key, pub failing: Failures,
    pub landings: Box<[Blame]>,                          // one per landing of the range, oldest first
    pub flakes: Box<[TestId]>,
}
pub enum Next { Dispatch(Run), Judged(Verdict), Held(Run) }

pub fn report(format: Format, file: &[u8], limits: &Limits) -> Result<Failures, Unreadable>;
pub fn failed(report: Option<Result<Failures, Unreadable>>) -> Outcome;
pub fn due(record: &Record, tip: Commit) -> bool;
pub fn start(record: &Record, tip: Commit, key: Key, landings: Box<[Landing]>,
             receipt: Option<Receipt>, settings: &Settings) -> Next;
pub fn step(record: &Record, run: &Run, outcome: Outcome, settings: &Settings) -> Next;
pub fn judged(record: &Record, verdict: &Verdict, limits: &Limits) -> Record;
pub fn repaired(blame: &Blame, failing: &Failures) -> bool;
```

Every type derives `Clone, PartialEq, Eq, Debug` (and `Copy` where the old crate has it).
`Settings::retries` is the project's `tier_retries`; `Held(run)` is a fault past it, and a
person's release dispatches `run` again with `faults` 0.

The policy's decisions, as the old mission settled them: probes go oldest first, linearly,
the first landing whose commit fails a test owns it, and the last candidate owns what is left
with no probe; a landing whose key equals the one before it is skipped; `Many` (past
`failing_tests`, or an id past `test_id_bytes`) is never rerun, probes then run the whole
tier, and a `Many` landing is repaired by any judged run whose failures are by test; new
flakes past `failing_tests` on the record are not kept; a failed receipt at the tip counts
as a rerun that failed again.

## 2. A change lives to cleared: `temper-engine-domain-forge-change`

`temper/docs/design/domain/after-landing.md`, section 6; `temper/docs/design/protocol/store.md`,
section 2. `Landed` keeps its name and now means landed and verifying; three states join after
the last existing variant (in the store's `ChangeLive`, after `no_change`, so no existing
variant's number moves).

```rust
pub enum State {
    // ... every existing variant, in place ...
    Landed { merge: Commit, since: Wall },   // `since`: when the top first saw the merge
    // ...
    Red { merge: Commit, since: Wall },       // a tier's verdict blames its landing
    Cleared { merge: Commit },                // every tier following its branch cleared it
    Uncleared { merge: Commit },              // past the bound with no verdict, or cancelled verifying
}

/// What the top reads of the change's landing's verification, every step.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Verification {
    /// No tier follows the branch it landed on (or it has not landed): cleared as it lands.
    Unfollowed,
    /// Some tier following its branch has not judged it yet.
    Verifying,
    /// A tier's verdict blames it and no later run has passed those failures.
    Red,
    /// Every tier following its branch has cleared it.
    Cleared,
}

pub struct Facts {
    // ... every existing field ...
    pub verification: Verification,
    /// The project's `verification_bound` (configuration.md, 3: `verification_bound_ms`).
    pub verification_bound: skein_lib::Duration,
}
```

The steps (`domain.rs`):

- `Pull::Merged { commit }` on a change not yet landed: `Landed { merge, since: now }`; then, in
  the same step, `Unfollowed` finishes at once, `Decision::Finish { merge }`, the state staying
  `Landed` (so a change no tier follows ends exactly as today); `Cleared` goes to `Cleared` and
  finishes; `Red` goes to `Red` and waits; `Verifying` waits, `Decision::Wait { until: Some(since
  + verification_bound) }`.
- `Landed` (verifying) on a later step: as above by `facts.verification`; past
  `since + verification_bound` with `Verifying`, `Uncleared` and `Decision::Finish { merge }`.
- `Red`: `Cleared` finishes at `Cleared`; otherwise waits with no deadline (the bound counts only
  while no verdict has come).
- Cancelled while `Landed` (verifying) or `Red`: `Uncleared`, `Decision::Finish { merge }`
  (after-landing.md 6: cancelling a verifying change ends it landed and uncleared).
- `terminal()`: `Cleared` and `Uncleared` finish, as `Landed` does today; `Landed` and `Red` are
  no longer terminal by state alone: they step by `facts.verification` as above, so a `Landed`
  change stepped again with `Unfollowed` finishes again, exactly as today.
- The top words a finished change's result from its final state: `Landed` as today; `Cleared`
  and `Uncleared` as section 4's words say.

## 3. Tiers in the configuration

`temper/docs/design/protocol/configuration.md`, section 3; `temper/docs/design/domain/landing.md`,
13.1; `temper/docs/design/domain/after-landing.md`, sections 2 and 11.

```rust
// temper-engine-domain, src/translate.rs (the configuration's typed form)
pub struct ProjectLanding { pub by: Way, pub repositories: Box<[RepositoryChecks]>, pub after: AfterLandingCheck }
pub struct RepositoryChecks { pub repository: Repository, pub gates: Box<[GateCheck]>, pub tiers: Box<[TierCheck]> }
pub struct TierCheck {
    pub name: Box<[u8]>, pub workflow: Box<[u8]>, pub ignored: Box<[Box<[u8]>]>,
    pub artifact: Box<[u8]>, pub format: temper_engine_domain_forge_tiers::Format,
    pub branch: Option<Box<[u8]>>,           // absent: the repository's landing branch
}
pub struct AfterLandingCheck { pub tier_retries: u8, pub verification_bound_ms: u64 }  // defaults 2, 7_200_000

// temper-engine-domain-forge, src/boundary.rs
pub struct Tier {
    pub name: Box<[u8]>, pub workflow: Box<[u8]>, pub ignored: Box<[Box<[u8]>]>,
    pub artifact: Box<[u8]>, pub format: temper_engine_domain_forge_tiers::Format,
    pub branch: Option<Box<[u8]>>,
}
pub struct RepositoryGates { pub repository: Repository, pub gates: Box<[landing::Gate]>, pub tiers: Box<[Tier]> }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AfterLanding { pub verdict: temper_engine_domain_forge_tiers::Settings, pub bound: skein_lib::Duration }

impl Domain {
    /// grows one argument; kept per project beside its way and gates
    pub fn project_landing(&mut self, project: u32, way: change::Way,
                           gates: Box<[RepositoryGates]>, after: AfterLanding) -> bool;
    /// read by the forge's tiers and the root's change facts
    pub fn after_landing(&self, project: u32) -> Option<AfterLanding>;
}

// temper-engine-domain-forge, src/limits.rs: one nested field on `Limits`
pub struct TierLimits { pub tiers: u32, pub failing_tests: u32, pub test_id_bytes: u32 }
// `Limits { ..., pub tiers: TierLimits }`; the small profile 2, 16, 256; limit names
// `forge.tiers.tiers`, `forge.tiers.failing_tests`, `forge.tiers.test_id_bytes`.
```

`AfterLanding::verdict` is `Settings { retries: tier_retries, suspects: Suspects::Bisect }`: the
configuration names no `suspects` in M8. A tier's name is a gate's (letters, digits, `-`, `_`),
at most 32 bytes, unique among its repository's gates and tiers; an `artifact` is non-empty,
at most a gate name's bytes, with no `/`; a repository's tiers at most `forge.tiers.tiers`.

## 4. The forge's tier surface: `temper-engine-domain-forge`

`temper/docs/design/domain/after-landing.md`, sections 3, 6 and 10. A tier run is a procedure
task of the forge's, `tier`, code 4 (`AFTER_LANDING_CODE` in
`temper/crates/temper-engine-domain/src/route/forge_route.rs`), made by the root when the forge
asks, mirroring the landing task's plumbing (`Event::Landing`, `StepLanding`, `VetoLanding`,
`Request::LandingDecision`, `describe_landing_effect`).

```rust
// boundary.rs
pub enum Request { /* ... */
    /// Make the tier's run task, number `number`, at `tip` (the root answers TierTask or TierRefused).
    TierRun { repository: Repository, tier: Box<[u8]>, number: u64, tip: Commit },
    /// The tier task's step: wait, hold saying why, or end done with the verdict's words.
    TierDecision { task: u64, decision: TierDecision, entry: Option<u64> },
}
pub enum TierDecision { Wait { until: Option<Wall> }, Held { words: Box<[u8]> }, Done { words: Box<[u8]> } }
pub enum Event { /* ... */
    TierTask { repository: Repository, tier: Box<[u8]>, number: u64, task: u64 },
    TierRefused { repository: Repository, tier: Box<[u8]>, number: u64 },
    StepTier { task: u64, entry: u64, heard: TierHeard },
    VetoTier { task: u64, entry: u64 },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TierHeard { pub released: bool, pub cancelled: bool }

// purpose.rs: tag 8, 30 bytes with the tag
pub enum EffectPurpose { /* ... */
    Tier { repository: Repository, tier: u16, run: u64, operation: u8, part: u16 },
}
// operation: 0 the run's branch, 1 the dispatch, 2 a rerun, 3 a probe's branch,
// 4 a probe's dispatch, 5 a branch's deletion, 6 the status, 7 a comment (6 and 7 unused in M8)

// the store families (store.md, 2): 28 Tier, 29 Verification
pub enum Stored { /* ... */ Tier(TierRow), Verification(Verification) }
pub enum Key { /* ... */
    Tier { repository: Repository, tier: Box<[u8]> },
    Verification { repository: Repository, commit: Commit },
}
pub struct TierRow {
    pub repository: Repository,
    pub tier: Box<[u8]>,
    pub record: temper_engine_domain_forge_tiers::Record,
    pub task: Option<u64>,
    // the run's operational state (pending tips, baseline, branches, flights, held fault):
    // the tiers-forge task's, which may add fields here and in the store's record.
}
pub struct Verification {
    pub repository: Repository,
    pub commit: Commit,
    pub changes: Box<[u64]>,                    // the change tasks it landed, oldest first
    pub tiers: Box<[TierVerification]>,         // the tiers following its branch at landing
}
pub struct TierVerification {
    pub tier: Box<[u8]>,
    pub verdict: Option<TierVerdict>,           // None while verifying
    pub repaired: Option<Commit>,               // the landing whose run passed its failures (5.3)
}
pub struct TierVerdict { pub run: u64, pub blame: temper_engine_domain_forge_tiers::Blame }
pub struct Receipt { /* ... */ pub failures: Option<temper_engine_domain_forge_tiers::Failures> }

impl Domain {
    pub fn tier_task(&self, task: u64) -> Option<&TierRow>;
    pub fn describe_tier_effect(&self, task: u64, entry: u64)
        -> Option<(api::Repository, crate::What, crate::AgentEffect)>;
    /// The verification of a change task's landing, for its facts (section 2).
    pub fn change_verification(&self, task: u64) -> temper_engine_domain_forge_change::Verification;
}
```

The words: a tier task's words name the tier and the run, `Tier NAME run N`; held, `Tier NAME run
N held after K runner faults, last: KIND`; done, `Tier NAME run N: green, cleared L landings` or
`Tier NAME run N: red, F tests failing, blamed on L landings`. A change's result at cleared adds
a line after today's `Landed:` line: `Cleared by TIER run N[, TIER run M].`; uncleared, `Not
cleared: no verdict within the verification bound.` (or `... : cancelled while verifying.`);
repaired after red, `Cleared by TIER run N, which passed the failures blamed on it.`

Revision 1 (2026-11-02, an answer to `panel-steps`): a repair keeps its run, so the panel and
the result can name it (`temper/docs/design/client/forge.md`, 2.1). `TierVerification::repaired`
becomes `Option<TierRepair>`, `pub struct TierRepair { pub run: u64, pub tip: Commit }` (`Clone,
Copy, PartialEq, Eq, Debug`); in the store, `record TierRepair { run: u64 tip: fixed 32 }` and
`repaired: option TierRepair`, inside v11. `panel-steps` makes the change.

## 5. skein's reads

As designed, no shape of this plan's: `skein/docs/design/forge.md`, 3.2, 3.3 and 5.2
(`Read::Artifact { run, name, max_bytes }` answering `File`; `Read::Commits { before, after,
page }` answering `Answer::Commits { commits, more }`, oldest first), and
`skein/docs/design/fake-forge.md`, 2.3 (a workflow's `fails`, `flaky`, `report`, `select`;
`Cue::Tests`). The bounded inflater is private to `skein-forge`: no crate gains a dependency
(forge.md, 2.1 holds). GitHub's dialect answers both reads `Refused`. temper persists neither
answer.

## 6. The tier task's deadline

Added to section 4's surface (2026-11-02, an answer to `tiers-root`): the forge keeps a tier
task's wait as it keeps a landing's (`wait_landing`, `offered_landing`), so the root never
reuses a change's or a landing's bounded deadline map.

```rust
// temper-engine-domain-forge, src/tiers.rs
impl Domain {
    /// `TierDecision::Wait`'s `until`, else a poll from now (as `wait_landing`); woken by
    /// `Request::WakeProcedure { task }` from `fire`.
    pub fn wait_tier(&mut self, env: &Env<Limits>, task: u64, until: Option<Wall>);
    /// The tier task was stepped: its deadline, if any, is spent.
    pub fn offered_tier(&mut self, task: u64);
}
```

The root calls them where the landing route calls the landing's: `offered_tier` as it starts or
steps a tier task (`StartProcedure`, `WakeProcedure`, a release), `wait_tier` on
`TierDecision::Wait`. The forge (`tiers-runs`) keeps the deadlines in a map of its own, sized to
the tier tasks that can be live at once (one per configured tier), counts them in
`next_wall_deadline` and fires them in `fire`, and drops a task's deadline when it ends or is
restored, as the landing's `due` is.
