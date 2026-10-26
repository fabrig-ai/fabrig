# Session 06: what Forgejo settles after it delivers

A change that reads its pull request just after a move must not take "not mergeable" for a
conflict: Forgejo answers `mergeable` false while it checks (domain/forge.md, 5.2). 06.1 makes the
fake forge's hooks Forgejo's (a job's end only, with its run; a body for the key; a checking phase)
and lets the connector world turn them into hints. 06.2 builds `settle`: a window of
`forge.client.settle_ms` from a move, inside which a pull not mergeable, or a hinted job's CI not
yet written, is the change's own wait, never a stall, read again at `hinted`, doubling, to the
window's end; past it, not mergeable is a conflict. The hinted story (temper 11.2) depends on it.

## Read first

- Design: `domain/forge.md` 5.2 ("What Forgejo settles", "The window is its own wait"), 8.2 (item 6), 8.4, 16; `forge.md` 5 (the table), 13
  (last bullet); `setup.md` 10; `configuration.md` 7 (row `forge.client.settle_ms`); `domain/testing.md` 4.2.
- `decisions.md` 5 (30000; the stall stays 30 s), 13, 15, 16; `findings/webhooks.md` 2 and 4; `forge-and-tiers.md`
  1 (`mergeable` is a bool on the wire: `document.rs:151`, `reading.rs:244`); temper 04.1's
  `tools/forgejo-conformance/facts.txt` (the timing pairs).
- skein: `programming-model.md` 6.3, 9, 10.3 (the subset binds `temper-engine-domain-forge` and its `-change`
  crate: no closure, `matches!`, `todo!` outside sketches); `testing-strategy.md` 2.2, 5, 7. temper `workflow.md` 2.

## What the design asks

- **A window** (domain/forge.md 5.2): within `settle` (above 0) of a move of a pull's head or base, known by
  a hint, a read or temper's own push, a pull reading not mergeable is read again at `hinted`, doubling, until
  it reads mergeable or the window has passed; a commit a job's hint named whose CI reads pending, the same.
- **Its own wait** (5.2, 8.2 item 6): the change waits to the window's end at most, the stall does not count
  that wait, and past it not mergeable is a conflict. A move long before leaves no window open.
- **Hooks as Forgejo's** (forge.md 5): a job's end only, never a commit status; a body carries the key.

## Choices this plan makes

- **The change procedure settles, not the keep** (5.2): a change names its branch and base only
  (`forge_route.rs:1579`), nothing creates `Topic::Pull`, and each step reads the pull afresh (`domain.rs:1750`).
  `settle` is a client limit by name, read by the forge domain as it reads `client.inbox` (`domain.rs:1824`).
- **The window is `[start, start + settle)`** in wall time. `start` is the later of the change's own `since`
  (its head's push, persisted in its row; a release and a repair's or resolution's request also set it,
  the change crate's `domain.rs:124,549,578`, which only makes a conflict believed late) and the last move a hint or a
  base read told of. So a head pushed long before leaves no window, and a first read opens none by itself.
  Domain/forge.md 5.2 states the start: the change keeps its own push over a restart and not what hints and
  reads told, so after a restart only its own push opens a window. What they told is a volatile
  `Map<u64, Settle>` by task in the domain and its `worst_case`: no `Stored` row, no codec.
- **Doubling from the start:** the next read is as far from `start` as now is, at least `hinted`, no later
  than the end: 1, 1, 2, 4, 8 s. No counter, so extra wakes (any hint wakes every change, `domain.rs:791`)
  re-time it.
- **Sources.** A hint names a move (`Branch` or `BranchMoved` of the change's branch or base) or a job's end
  (`Commit` of its head); a read names a move (the pull's base commit differing from the last read's). A pull
  `Item` hint is no move. CI `none` counts as pending for a hinted job (its status is written after).
- **The check is a scenario action** (`forge::check`): the world's scripted moves run at `Time::ZERO`
  (`world.rs:278-281`, `335-339`), so a span counted from a move would be over before the world's clock;
  the world calls it after a move, with its own clock.
- **`translate::hint`** is the one mapping of a fake hook to a hint, for the forge world (`tests/engine/forge`)
  and the conformance world, which repeats it now (`systems.rs:224-245`). The client world
  (`tests/engine/forge/client/src/world.rs`: `hooked: false` `:125`, `Hook { .. } => {}` `:467`, `:645`) keeps
  ignoring hooks, unchanged. The helpers of `simulation.rs` (1830 lines) that
  `settle.rs` needs move to the world crate's `src/changes.rs`.

## Rules that bind it

- **Limits** are the shipped profile's: `temper_service::limits::profile(Small)` (`limits.rs:247`) gives
  `settle` 30 s and `stall` 30 s, and the worlds' consts (`world.rs:8`, `client/src/world.rs:8`) carry them.
  No test names another value.
- Files at most 800 lines (`simulation.rs` takes no new test); a test at most 40.

## Increments

### 6.1 The fake forge's hooks as Forgejo's
Scope: hooks per forge.md 5; a checking phase; the world forwards hooks as hints.
Needs: temper 04.1 (the paired timing facts).
Files: `testing/temper-fake-forge-domain/src/{api,boundary,hooks,ci,git,issues,pulls,store,wiki,scenario,
domain,lib}.rs` (`ci.rs:221` raises the status hook, `pulls.rs:347` computes `mergeable`, `store.rs:82` is
`Pull`); `tests/engine/forge/client/src/translate.rs`; `tests/engine/forge/src/{world,lib,changes}.rs`;
`tests/engine/forge/tests/{hooks,simulation}.rs`; `tests/conformance/src/systems.rs`;
`testing/temper-fake-forge-domain/src/tests/next.rs`. No `Config` field: its six literals stay (fake `tests.rs:46`,
`:1576`, `tests/engine/forge/src/world.rs:563`, `client/src/world.rs:658`, `tests/fake-forge/tests/memory.rs:42`,
`tests/worker/checkout/src/forge.rs:60`). Hotspot with 05.1: the protocol layer matches `Request::Hook { .. }` (`server.rs:140`,
`service.rs:94`); whichever of 05.1 and 06.1 lands second adapts 05.1's rendering to the reshaped `Hook`.
Users: `Request::Hook` with named fields only at fake `tests.rs:313,340,365`; `Change::Status` at
`tests.rs:1418,1800,1828,1829` and `systems.rs:234`, `Wiki` at `tests.rs:1421,1803` and `systems.rs:238`; the
harness's hooks read by tuple position (`hook.0`, `.1`, `.3`) at fake `tests/next.rs:32,33`, reshaped with it.
Shape: sketches A to D. `Change::Status` becomes `Job`, `Wiki` goes (the table subscribes none). A job hook
is raised only when CI (`env.limits.ci`) reports `Passed` or `Failed`: `ci::reported` calls `domain::changed`
with `Hook::job` (the `JobRef` from `actions::reference`, `actions.rs:91`), else pushes the observation alone,
as `wiki.rs:45,71` do once `Wiki` goes (they call `domain::changed(.., Hook::wiki(user))` now). Issue hooks carry the issue's body, comment hooks the comment's, review hooks the
review's (`Hook::body`), pull hooks none. `Pull.checked: Time` (forge time, zero at open): `pulls::pull`
answers `mergeable` false while `domain.clock < checked`. **The order follows 04.1's job pair**
(the ready check runs `grep -E '^(holds|fails) delivery-job-end' tools/forgejo-conformance/facts.txt`, two lines): `delivery-job-end-follows-its-commit-status`:
`ci::report` (`ci.rs:77`) sets the status, then raises the hook; `…-precedes-…`: it raises the hook, and a
new `Alarm::Status` (`domain.rs:191`, `fire` `:419`) sets the status `LAG` (1 s, a const of `ci.rs`) later.
**The phase is always checking** (forge.md 9), whichever of 04.1's pull pair holds: `git::moved` (`git.rs:351`) starts
the check of each `on_branch` pull (`store.rs:235`), for `CHECK` (2 s, a const of `pulls.rs`), as `scenario::check`
does by hand. `World::hooked` sets `hooked: true` (`world.rs:132`); `take_forge` (`:514`) and
`external` (`:358`) pass each `Request::Hook` to `translate::hint`, queue the hint to the top and keep it.
Test shape: fake `tests.rs` (harness `:249-258` keeps each hook whole, `heard` `:495`, `heard_all` `:504`;
`setup()` `:147`, `CALM`): updated `a_subscriber_hears_of_each_change_by_webhook` (`:1401`; `Status` 2 becomes
`Job` 1 per context, and its totals `heard.len()` and `tally().hooks`, 10, at `:1431-1433`),
`observations_and_webhooks_follow_branches_pull_requests_and_the_wiki` (`:1720`; its `Status` 3 and `Wiki` 2
counts) and `webhooks_name_the_branch_and_commit_a_push_or_status_is_about` (`:1818`); new
`a_job_ending_is_heard_with_its_run_job_and_commit` (asserts the order the fact names),
`a_pending_report_and_a_persons_status_are_not_heard`, `an_issue_a_comment_and_a_review_are_heard_with_their_bodies`,
`a_pull_reads_not_mergeable_until_the_check_span_passes` (starts its check as the fact says). World, new
`hooks.rs` (`World` `world.rs:104`, `fake_config` `:562`, `translate::keyed` `translate.rs:21`): the reference,
and `an_issues_body_key_reaches_the_connector_in_its_hint`. Ceilings: fake tests under 20 ms, world under 0.5 s.
```rust
// tests/engine/forge/tests/hooks.rs
use temper_engine_domain_forge_client::api as client;
use temper_engine_forge_world::{World, translate};

#[test]
fn a_jobs_end_reaches_the_connector_as_a_hint_for_its_commit() {
    let mut world = World::hooked(23);
    world.adopt();
    world.produce(b"temper/51");
    world.run_for(1);
    let head = translate::commit(world.branch(b"temper/51"));
    let ended = world.hints().iter().filter(|hint| hint.change == client::Change::Commit(head)).count();
    assert_eq!(ended, 1, "one job end for the head's one context, none for its pending report");
}
```
Proof: world, `cargo nextest run -p temper-engine-forge-world --test hooks`: the fake CI's pass of
`temper/51` reaches the connector as one `Commit(head)` hint, and an issue's key as `Item(n)` with `Some(key)`.
Bound 0.5 s per test. Highest layer that reads these hooks: the fake's protocol layer sends them signed only
in 05.1, and 05.2 receives them.
Don't: sign, serialise or send a hook; teach the fake a marker; add a `Config` field; change `Check` (Questions 1).
Size: at most 150 lines of production code (about 30 more if `…-precedes-…` holds: within tolerance).
Removes: `Change::Status`, `Change::Wiki`, `Hook::status`, `Hook::wiki`; 22 lines of `systems.rs:224-245`.

### 6.2 `settle`
Scope: `forge.client.settle_ms` and the window of domain/forge.md 5.2 and 8.2 for the change procedure: inside
it a pull read not mergeable is `Pending` and a hinted job's pending CI waits too, both the change's own wait
to the window's end, never a stall; past it, not mergeable is a conflict.
Needs: 6.1 (`World::check_pull`, `World::hooked`, `changes.rs`).
Files: `crates/temper-engine-domain-forge-client/src/limits.rs` (`settle` after `slow` `:40`; `worst_case` `:47`
refuses zero, `:60-69`); a line `settle: Duration::from_secs(30)` after `slow` in each of the ten literals
`git grep -n 'hinted: Duration::from_secs' -- crates testing tests tools` lists (protocol-forge's
`tests/fixtures.rs` is shared with 05.2; the profile is `temper-service/src/limits.rs:81`, with `limits/names.rs:107`:
both hotspots with client-api's 05.1). `crates/temper-engine-domain-forge-change/src/`: `boundary.rs` (`Facts` `:172-187`), `domain.rs`
(new `settling`, used at `:327` and `:491`; `waiting` `:174` stays), `tests.rs` (`facts()` `:34`).
`crates/temper-engine-domain-forge/src/`: new `settle.rs`; `lib.rs` (`mod settle`); `domain.rs` (field `:127`
and `:331`; `Event::Hint` `:791`; `finish_step` `:1909`, the call before `change_facts` `:1943`, the decision
`:1970`; `change_facts` `:2011-2029`); `limits.rs:141` (`worst_case` gains the map).
Shape: sketch E. Change crate: `Facts` gains `pub settling: Option<Wall>` as its last member ("the end of the
window Forgejo is still settling in, none when no window is open"); `settling(change, facts, now, limits) ->
Stepped` replaces `waiting` at the two waits on what Forgejo settles (`first`'s `Pending` mergeability,
`checking`'s pending CI): before `facts.settling` it is `Wait { until: Some(end) }`, never a stall; past it or
with none, `waiting`. Forge domain: `observe` runs in `finish_step` before `change_facts`, from `pending.pull`,
`pending.ci` and the row's `change.since`; `change_facts` reads `settle::window` once: a not-mergeable pull is
`Pending` while it is some, else `Failed` (replacing the arm at `:2027`), and `settling` is its end; `cap`
brings the `Wait` to the doubling read before `ChangeDecision` is emitted; `hinted` runs from `Event::Hint`
with the changes it wakes. A closed or merged pull is never moving.
Test shape: change crate `tests.rs` (`LIMITS` `:15` stall 30 s, `decide` `:55`, `at`): unit
`a_pending_wait_inside_the_settle_window_ends_with_it_and_never_stalls` (now 40 s, stall due at 30, window to 50:
`Wait` to 50) and `a_pending_wait_past_the_settle_window_stalls_as_any_wait`. World, new
`tests/engine/forge/tests/settle.rs`, over `changes::{change_row, change_step, reviewed_change}` (moved from
`simulation.rs:140,315,334,368`, with `change_step_with_release` `:324`, which `change_step` calls;
`change_step_task` is `pub`, as `simulation.rs:645-662` call it; `tests/engine/protocol-forge/tests/protocol.rs:63-66`
keeps its own copy), `World::new` (`world.rs:104`), `slow_calls(ZERO)` (`:222`), `adopt`, `produce`,
`status`, `open_pull`, `restart` (`:226`), `writes`, and the hint of `simulation.rs:1600`; each gives the change
`since` = `world.wall()`, and waits are seconds from the read, `t0` the first. Tests: the reference; then
`a_pull_not_mergeable_past_settle_is_taken_as_conflicted` (check 40 s: waits 1, 1, 2, 4, 8, 14, then
`Delegate(Resolve { base })` at `t0` + 30), `a_hint_naming_the_base_opens_the_window_again` (check 40 s: waits 1,
1, 2, then a hint at `t0` + 3 and 1), `a_job_end_hint_makes_pending_ci_wait_a_hinted_read` (`Wait` to `t0` + 1, then
`Ready` once the status is written), `a_head_pushed_long_before_leaves_no_window_open` (60 s on, then `Resolve`
at once), `a_restart_keeps_the_window_of_the_changes_own_push` (waits 1, 1, restart, then `max(1, now - t0)`, not 1). Client
`limits_reject_incoherent_budgets_and_account_for_capacity` (`tests.rs:460`) gains a zero `settle`; service
`an_override_sets_the_field_its_dotted_name_names` (`src/tests/limits.rs:36`) gains `forge.client.settle_ms`, and
`the_small_profile_gives_settle_thirty_seconds` pins 30 s. Ceilings: unit under 20 ms; world under 0.5 s.
```rust
// tests/engine/forge/tests/settle.rs
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::changes::{change_row, change_step};
use temper_engine_forge_world::{World, translate};
use temper_fake_forge_domain::api::Check;

/// First in its queue, head pushed now, on a pull Forgejo starts checking now.
fn first_on_a_checked_pull(mut world: World, span: Duration) -> World {
    world.adopt();
    world.slow_calls(Duration::ZERO);
    world.produce(b"temper/51");
    world.status(b"temper/51", Check::Passed);
    let pull = world.open_pull(b"temper/51", b"main");
    world.check_pull(pull, span);
    let head = translate::commit(world.branch(b"temper/51"));
    let mut row = change_row();
    row.pull = Some(pull);
    row.change.since = world.wall();
    row.change.state = change::State::First { head, ready_since: world.wall() };
    row.change.last_head = Some(head);
    row.change.owns_turn = true;
    world.event(top::Event::Change { row });
    world
}

#[test]
fn a_pull_forgejo_is_checking_waits_by_doubling_reads_then_merges() {
    let mut world = first_on_a_checked_pull(World::new(5), Duration::from_secs(5));
    let mut asked = Vec::new();
    for entry in 1..=4 {
        let at = world.wall();
        let decision = change_step(&mut world, entry, change::Status::Passed, Box::new([]));
        let change::Decision::Wait { until: Some(until) } = decision else { panic!("{decision:?}, not a wait") };
        asked.push((until.as_nanos() - at.as_nanos()) / 1_000_000_000);
        world.run_to(until);
    }
    assert_eq!(asked, [1, 1, 2, 4], "seconds from each read: hinted, then doubling");
    let merged = change_step(&mut world, 5, change::Status::Passed, Box::new([]));
    assert!(matches!(merged, change::Decision::Effect(change::Effect::Merge { .. })), "{merged:?}");
    assert_eq!(world.writes(), 1, "never resolved");
}
```
Proof:
- unit, `cargo nextest run -p temper-engine-domain-forge-change -E 'test(/settle_window/)'`: a pending
  mergeability waits to the window's end at second 50 though the stall fell due at 30, and stalls once the
  window has ended. Bound 20 ms per test.
- world, `cargo nextest run -p temper-engine-forge-world --test settle -E
  'test(=a_pull_forgejo_is_checking_waits_by_doubling_reads_then_merges)'`: a change on a pull Forgejo checks for
  5 s asks to wait 1, 1, 2 and 4 s, then merges, and no resolution is delegated. Bound 0.5 s. World is the
  highest layer that runs the change procedure; the process tier's hinted story (11.2) connects it to Forgejo.

Don't: touch the client crate beyond `Limits`, or the change crate beyond `Facts.settling` and `settling`; persist
the window; read `Cached.pull.mergeable` for a decision; count a pull `Item` hint as a move; use `std::time`.
Size: at most 200 lines of production code.
Removes: nothing; the arm of `change_facts` at `:2027` and two calls to `waiting` are replaced.

## Interface sketches

A: the fake's vocabulary and boundary, 6.1.
```rust
// testing/temper-fake-forge-domain/src/api.rs (beside `Check` and `JobRef`)
/// What a webhook says changed. Those about an item name it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Change {
    /// An issue opened, closed, reopened or labelled; the hook carries its body.
    Issue,
    /// A pull request opened, closed, reopened, labelled, merged or its head moved; no body.
    Pull,
    /// A comment posted, edited or deleted; the hook carries its body.
    Comment,
    /// A review submitted; the hook carries its body.
    Review,
    /// A job ended, passed or failed; `Ended` names it. Never a person's status.
    Job,
    /// A branch moved, made or deleted.
    Push,
}

/// A job's end as its delivery names it (forge.md 5): the run's job, the commit, the verdict.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ended {
    pub job: JobRef,
    pub commit: u64,
    pub state: Check,
}
```
```rust
// testing/temper-fake-forge-domain/src/boundary.rs
use alloc::boxed::Box;
use skein_lib::ReplyTo;

use crate::api::{Answer, Change, Ended, Error};

/// domain -> protocol
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    /// Terminal for `Call`: its answer, exactly one per call.
    Reply { to: ReplyTo, result: Result<Answer, Error> },
    /// A webhook, answered by nothing: `body` is the item's, comment's or review's text; `ended` is set for
    /// `Change::Job`, whose `commit` is then none.
    Hook {
        repository: Box<[u8]>,
        change: Change,
        number: Option<u64>,
        branch: Option<Box<[u8]>>,
        commit: Option<u64>,
        by: Option<u64>,
        body: Option<Box<[u8]>>,
        ended: Option<Ended>,
    },
}
```
B and C: the scenario action and the one mapping of a hook to a hint, 6.1.
```rust
// testing/temper-fake-forge-domain/src/scenario.rs
use skein_lib::{Duration, Env};

use crate::domain::{Config, Domain};

/// Forgejo starts checking pull request `number` now: it reads not mergeable until `span` of the forge's
/// time has passed.
pub fn check(domain: &mut Domain, env: &Env<Config>, repository: &[u8], number: u64, span: Duration) {
    todo!()
}
```
```rust
// tests/engine/forge/client/src/translate.rs
use temper_engine_domain_forge_client::api as client;
use temper_fake_forge_domain::Request;

/// The hint the component makes of a hook (forge.md 5): a push `BranchMoved` (`Branch` if deleted), a job's
/// end `Commit`, an issue, comment or review `Item` with its body's key, a pull `Item` with none.
#[must_use]
pub fn hint(repository: client::Repository, request: &Request) -> Option<client::Hint> {
    todo!()
}
```
D: the world, 6.1 (in `impl World`).
```rust
// tests/engine/forge/src/world.rs
use skein_lib::{Duration, Wall};
use temper_engine_domain_forge_client as client;

impl World {
    /// As `new`, with the repository's hooks heard; `hints` are the hints they made, in order.
    #[must_use]
    pub fn hooked(seed: u64) -> World { todo!() }
    #[must_use]
    pub fn hints(&self) -> &[client::api::Hint] { todo!() }
    #[must_use]
    pub fn wall(&self) -> Wall { todo!() }
    /// Run until the wall clock reaches `until`; nothing if it has.
    pub fn run_to(&mut self, until: Wall) { todo!() }
    /// A person opens an issue with `body`; its number.
    pub fn issue(&mut self, body: &[u8]) -> u64 { todo!() }
    /// Forgejo starts checking pull `number` now, for `span` (`forge::check`).
    pub fn check_pull(&mut self, number: u64, span: Duration) { todo!() }
}
```
E: the window, 6.2. `fold` is the rule; `observe` derives its arguments from the reads and the row.
```rust
// crates/temper-engine-domain-forge/src/settle.rs
use skein_lib::{Duration, Env, Wall};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_client as client;

use crate::{ChangeRow, Domain, Limits};

type Commit = client::api::Commit;

/// What hints and reads told of one change: volatile, one row per change task. `since`: when they last
/// told of a move or a job's end; `job`: a job's end was hinted, so CI is waited on; `base`: the pull's
/// base as the last read found it; `open`: the window's start and end, while something is unsettled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Settle {
    since: Option<Wall>,
    job: bool,
    base: Option<Commit>,
    open: Option<(Wall, Wall)>,
}

impl Settle {
    /// A row nothing has told of yet (the domain's rows start here; no `Default`).
    pub(crate) const NEW: Settle = Settle { since: None, job: false, base: None, open: None };

    /// The row after one step's reads. `own` is the change's `since`: the window starts at the later of it
    /// and `since`. `moving`: the pull is open, not mergeable; `ci_pending`: CI reads pending or none.
    pub(crate) fn fold(mut self, now: Wall, own: Wall, settle: Duration, moving: bool, ci_pending: bool, base: Option<Commit>) -> Settle {
        if self.base.is_some() && self.base != base {
            self.since = Some(now);
        }
        self.base = base;
        let unsettled = moving || (self.job && ci_pending);
        if !unsettled {
            self.since = None;
            self.job = false;
        }
        let start = own.max(self.since.unwrap_or(own));
        let end = Wall::from_nanos(start.as_nanos().saturating_add(settle.as_nanos()));
        self.open = if unsettled && now < end { Some((start, end)) } else { None };
        self
    }
}

/// Open the window of each change in `tasks` the hint names: `Branch`/`BranchMoved` of its branch or base
/// is a move, `Commit` of its head a job's end (sets `job`); others do nothing.
pub(crate) fn hinted(d: &mut Domain, env: &Env<Limits>, hint: &client::api::Hint, tasks: &[u64]) { todo!() }

/// Fold one step's fresh reads into the change's row (`fold`), before its facts are made.
pub(crate) fn observe(d: &mut Domain, env: &Env<Limits>, row: &ChangeRow, pull: Option<&client::api::Pull>, ci: change::Status) {
    todo!()
}

/// The change's window as its last step left it, while something is unsettled in it.
pub(crate) fn window(d: &Domain, task: u64) -> Option<(Wall, Wall)> { todo!() }

/// A timed `Wait` with the window open comes sooner: at `now + max(hinted, now - start)`, no later than
/// the end and than its own `until`. Other decisions pass.
pub(crate) fn cap(d: &Domain, env: &Env<Limits>, task: u64, decision: change::Decision) -> change::Decision {
    todo!()
}
```

## Questions to raise when reached

1. **Job ends the fake lacks:** `workflow_job_cancelled` and `_skipped` (forge.md 5); `Check` has neither.
   Left out.

## Done when

- `cargo nextest run -p temper-fake-forge-domain -p temper-engine-forge-world
  -p temper-engine-domain-forge-client -p temper-engine-domain-forge -p temper-engine-domain-forge-change
  -p temper-service -p temper-conformance-world` passes (the change crate's two unit tests among them).
- `git grep -n -e 'Change::Status' -e 'Change::Wiki' -e 'Hook::status' -e 'Hook::wiki' -- crates testing tests tools`
  prints nothing and exits 1 (13 lines at the base).
- `git grep -n 'settle::window' -- crates/temper-engine-domain-forge/src/domain.rs` prints one line (nothing
  and exit 1 at the base).
- `git grep -c 'forge.client.settle_ms' -- crates/temper-service/src/limits/names.rs` prints
  `crates/temper-service/src/limits/names.rs:1` (nothing at the base).
- `scripts/check.sh` exits 0; this session has no share of the run's suite aim (decisions.md 13).
