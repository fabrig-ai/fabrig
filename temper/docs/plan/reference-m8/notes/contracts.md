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
