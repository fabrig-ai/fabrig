# reference-m9: contracts

The shapes that cross this plan's tasks, binding for every spec that cites them. The design is
`temper/docs/design/domain/after-landing.md`, sections 2, 3, 5.3, 5.4, 6, 7 and 10;
`temper/docs/design/domain/agent.md`, 4.3.1 (`repairs`); and
`temper/docs/design/protocol/configuration.md`, section 3 (a tier's `branch` and `span_ms`) and
13.9 (the reference's tiers and charter 5's triage). Paths are from the repository root; a
name in `code` is exact.

## 1. The store (task `store-surface`)

`temper/crates/temper-protocol-store/schema/store-v11.schema`, fields appended at each record's
end, the version staying 11 (`temper/docs/design/protocol/store.md`, 4: until the first
release):

```
record ChangeRow { … report: bytes 1048576 repairs: option fixed 32 }
record TierRepair { run: u64 tip: fixed 32 by: option fixed 32 }
record TierGoal { goal: u64 branch: bytes 4096 tip: option fixed 32 ran: option fixed 32 failing: Failures started: option u64 }
record TierRow { … received: bool goals: list 65536 TierGoal goal: option u64 started: option u64 }
```

`TierGoal` is declared before `TierRow`. `started` takes the encoding
`TierFlight.start` takes. The forge's mirrors (`temper/crates/temper-engine-domain-forge/src/boundary.rs`),
each new field last:

```rust
pub struct ChangeRow { …, pub repairs: Option<skein_forge::api::Commit> }
pub struct TierRepair { pub run: u64, pub tip: skein_forge::api::Commit, pub by: Option<skein_forge::api::Commit> }
/// A goal branch a tier following goal branches follows (temper/docs/design/domain/after-landing.md, 3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TierGoal {
    /// The goal task holding the branch.
    pub goal: u64,
    /// `temper/<root>/g<goal>`, as the hold names it.
    pub branch: Box<[u8]>,
    /// The branch's newest tip seen since the goal's hold made it.
    pub tip: Option<skein_forge::api::Commit>,
    /// The tip its last run tested.
    pub ran: Option<skein_forge::api::Commit>,
    /// What its last run failed again after its rerun; empty when green.
    pub failing: temper_engine_domain_forge_tiers::Failures,
    /// When its last run started.
    pub started: Option<skein_lib::Time>,
}
pub struct TierRow { …, pub goals: Box<[TierGoal]>, pub goal: Option<u64>, pub started: Option<skein_lib::Time> }
```

- `ChangeRow.repairs`: the red landing's commit the change names (section 2); `None` repairs
  nothing.
- `TierRepair.by`: the repairing landing's commit; `None` when the run alone cleared it.
- `TierRow.goals`: sorted by `goal`, unique, empty for a tier following one branch, at most the
  forge's `holds`. `TierRow.goal`: the goal whose branch the run in flight is for, `None` for a
  tier following one branch. `TierRow.started`: when the last run of a tier following one
  branch started, for its span.
- Every new field is `None` or empty where nothing sets it yet.

The read the root checks `repairs` with, in
`temper/crates/temper-engine-domain-forge/src/verification.rs`:

```rust
/// What the forge knows of a landing a change names as the red it repairs (temper/docs/design/domain/agent.md, 4.3.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LandingRed {
    /// No live `Verification` row of temper's has this commit in the repository.
    Unknown,
    /// Its changes landed into another branch than the one asked.
    Elsewhere { branch: Box<[u8]> },
    /// No tier holds it red: no verdict `Owns` or `Suspect` without its `repaired`.
    NotRed,
    Red,
}
impl Domain {
    pub fn landing_red(&self, repository: skein_forge::api::Repository, commit: &skein_forge::api::Commit, branch: &[u8]) -> LandingRed;
}
```

A landing's branch is its changes' `ChangeRow.base` (its first live change's).

## 2. `change`'s `repairs` (task `repairs-call`)

- **Declared** in `PLAN_CHANGE` (`temper/crates/temper-engine-smith/src/calls.rs`), in each change's
  object (alone and in `changes`), beside `after`:
  `"repairs":{"type":"string","description":"The red landing this change repairs: its merge commit's 40 or 64 hexadecimal digits, as the red's news names it.","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"}`.
- **Carried:** `temper_engine_domain::ChangeCall` (`temper/crates/temper-engine-domain/src/boundary.rs`)
  gains `pub repairs: Option<skein_forge::api::Commit>`, last; `ChangeCall::words` sets `None`.
  The digits read into the commit's fixed form as every other commit text temper reads (40
  digits fill its first 20 bytes).
- **A plan member:** like `landing` and `after`, only a run holding jig's control family names
  it (the plan predicate in `forge_route::change_batch`). The refusal's words list it
  (`change's landing, branch, name, after, repairs and changes are a plan's: …`); the two
  assertions of those words in `temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs`
  follow them, and `repairs-call` may edit them for that alone.
- **Checked** in `change_batch` once the change's repository and landing branch are resolved,
  with `domain.forge.landing_red(repository, &commit, &base)`; a refusal refuses the call, its
  words exactly:
  - `Unknown`: `repairs: HEX is no live landing of temper's in OWNER/NAME.`
  - `Elsewhere { branch }`: `repairs: HEX landed into BRANCH, not BASE.`
  - `NotRed`: `repairs: HEX is red in no tier.`
  HEX is the digits as given; OWNER/NAME the repository's; BASE the change's landing branch.
- **Kept:** `chat_change` adds the delegate spec's `Parameter::Bytes { name: 12, value: HEX }`;
  `start_change` reads parameter 12 into `ChangeRow.repairs` (digits that are not 40 or 64
  lower-case hexadecimal fail the start as any malformed parameter does).

## 3. A repair's clearing (task `repairs-verdict`)

- `verification::judged`: where `policy::repaired(&prior.blame, &verdict.failing)` sets the
  tier's `TierRepair { run, tip, by }`, `by` is the commit of the newest landing in the run's
  range (its `landings`, oldest first) one of whose `Verification.changes` has
  `ChangeRow.repairs == Some(the red landing's commit)`; `None` when none has.
- The red landing's change's result words (`change_verification_words`,
  `temper/crates/temper-engine-domain/src/route/forge_route.rs`): where a tier's `repaired.by`
  is `Some`, its line is `Cleared by NAME run N, which passed the failures blamed on it;
  repaired by HEX12.`, HEX12 the repairing commit's first twelve digits; with `by: None` the
  line stays as it is.

## 4. A tier's settings (task `goal-tier-config`)

- The forge's `Tier` (`boundary.rs`) replaces `branch: Option<Box<[u8]>>` with
  `follows: Follows`, and gains `span: Option<skein_lib::Duration>`, last:

  ```rust
  /// What a tier follows (temper/docs/design/domain/after-landing.md, 2).
  #[derive(Clone, PartialEq, Eq, Debug)]
  pub enum Follows {
      /// The repository's landing branch, its adoption's default.
      Landing,
      /// One branch by its name on the forge.
      Branch(Box<[u8]>),
      /// The goals' own branches of the repository; judges no landing.
      Goals,
  }
  ```
- `TierCheck` (`temper/crates/temper-engine-domain/src/translate.rs`) replaces `branch` with
  `follows: temper_engine_domain_forge::Follows` and gains `span_ms: u64`, last (0: none);
  translation maps 0 to `None`.
- The document (`temper/crates/temper-service/src/startup/application/landing.rs`, `tier`):
  `"branch"` absent → `Landing`; exactly `"goal"` → `Goals`; another non-empty string →
  `Branch`; empty → refused as today. `"span_ms"`: a u64, absent → 0.
- `tiers::definition`: the bytes for `Landing` and `Branch` stay exactly as today (tier keys
  and receipts keep); `Goals` writes a marker of its own, so its key differs from both. The span
  is not in the key.
- Inert until `goal-tier-forge`: `tiers::followed` answers `None` for `Goals`, and
  `verification::landed` counts no `Goals` tier, so a change into a goal's branch still ends
  `Unfollowed`, at landed.

## 5. A red's news (tasks `red-news-surface`, `red-news-tell`)

The forge's `News` (`boundary.rs`) gains a variant, its types beside it:

```rust
pub enum News { …, Red(RedNews) }
/// A red tier run told to a goal (temper/docs/design/domain/after-landing.md, 7).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RedNews {
    pub tier: Box<[u8]>,
    /// The run's number in the tier's sequence.
    pub run: u64,
    /// The forge run's link, once observed.
    pub url: Option<Box<[u8]>>,
    pub on: RedOn,
    /// Each failed job of the run's last failed part, in the run's order.
    pub logs: Box<[JobLog]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RedOn {
    /// A red after landing: the landing, its branch, the failures blamed on it.
    Landing { commit: skein_forge::api::Commit, branch: Box<[u8]>, failing: temper_engine_domain_forge_tiers::Failures },
    /// A red of a tier following goal branches: the branch, the run's tip, its tests new and still failing.
    Goal { branch: Box<[u8]>, tip: skein_forge::api::Commit, new: temper_engine_domain_forge_tiers::Failures, still: temper_engine_domain_forge_tiers::Failures },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JobLog {
    pub name: Box<[u8]>,
    /// The log's end, cut to its share; `None` when it could not be read.
    pub end: Option<Box<[u8]>>,
}
```

**The words** (`news_words`, `forge_route.rs`), lines joined by `\n`; HEX the commit's digits
whole, HEX12 its first twelve; K and J counts, or `many` for `Failures::Many`:

```
Red after landing: tier NAME run N blames landing HEX on BRANCH: K failing.
<each test id, one per line; or the one line "many">
Run: URL                                  (only with a url)
End of job NAME's log:                    (each job with an end)
<its end>
End of job NAME's log: [no log]           (each job without)
```

```
Red on BRANCH: tier NAME run N at HEX12: K new, J still failing.
new: <id>                                 (each new test; "new: many" for Many)
still failing: <id>                       (each still failing; "still failing: many")
Run: URL
<the jobs' logs as above>
```

Cut to the root's `limit` (`tasks.message_bytes`): the header and the ids first, whole lines
only; then `Run:`; then the logs, sharing the room left equally, each keeping its end (cut from
its start). What does not fit is left out, never a broken line but a log's first.

**Delivery:** `Request::News { task, subscription, topic, news: News::Red(..), class }`, through
a subscription the goal already has to `Topic::Landings { repository, branch }`:
- a red after landing: for each change of the red landing's `Verification.changes`, the
  subscriber of the landing branch's topic whose `goal_tasks` holds that change; one news per
  subscriber task, `Class::Wakes`;
- a red of a tier following goal branches: the subscriber of the goal branch's topic whose
  `task` is the goal holding the branch; `Class::Wakes` when `new` is non-empty or `Many`,
  `Class::Kept` otherwise;
- no such subscriber: nothing is told.

**The logs:** for each failed job attempt of the run's last failed part (its rerun's when the
rerun failed again), `skein_forge::api::Read::Job { attempt, max_bytes }`, `max_bytes` the
forge's `brief_bytes` divided by the number of failed jobs (at least 1); a failed or empty read
is `end: None`. The news is emitted in the step that keeps the verdict, after the reads have
answered; a restart before then reads them again.

## 6. g's tier (task `goal-tier-forge`)

- **Followed:** for each `Goals` tier of a repository, each name of that repository held by a
  live goal whose name `items::goal_branch` parses (`d.holds`) is watched; the hold's release
  ends it. A move of it seen after the hold makes or updates the goal's `TierGoal` (`tip`).
- **Due:** an entry with `tip != ran` whose `started` plus the tier's span has passed (or no
  span, or never started). With no run in flight, the due entry with the oldest `started`
  (`None` first, then the lowest `goal`) runs: `TierRow.goal = Some(goal)`, `started = now`,
  the run at its `tip`, its landings empty, no receipt taken (`p::start(.., receipt: None, ..)`)
  and none made. A move within the span waits on the forge's deadline (`tiers::deadline`,
  `tiers::fire`) until it has passed. The row's `record.sequence` numbers every run.
- **Finish:** no `verification::judged`. The failures that failed again are the verdict's
  `failing`; `new` are those the entry's `failing` lacks, `still` those it has (`Many` against
  `Many` is still failing; any other `Many` is new). Red (any) tells the goal (section 5);
  green tells nothing. Then `ran = tip`, `failing` = the verdict's failures, `goal = None`.
- **A goal's close:** its hold released drops its `TierGoal`; a run in flight for it ends with
  its words and tells nothing.
- **A tier following one branch** keeps `TierRow.started` and waits its span the same way.
- **Words:** the run task's specification (`tiers/words.rs`, `tier_specification`) adds
  `Goal branch: BRANCH` after its repository line, for a goal's run alone.
