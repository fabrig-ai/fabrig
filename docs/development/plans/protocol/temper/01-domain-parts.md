# Session 01: an undecodable call relayed, a failed preparation reported, the worker's parts by tag

The domain changes the protocol layer needs before it can carry temper's
parts and smith's calls (protocol.md, section 5, "Kept"). The engine's
root relays the error the routing rendered for a host call that never
decoded, with no commit and no record, where today it drops it and the
run waits forever. The root's `Parts` gains `preparation`, a permanent
failure to prepare named by its repository's tag, which the root reports
as a fact of the attempt, in a bounded queue of its own facts. The
worker's root names each repository of its saved work and of a failed
preparation by tag, with whether it started from saved work, so the
engine's routing, which keeps no state, can read them; the composed
system world drops the bookkeeping it kept in their place. Three
increments, each proven in the engine's or the worker's worlds.

## Read first

- `docs/design/llm.md`, section 4 (a call that does not decode).
- `docs/design/domain/engine.md`, 7.3 (the last paragraph) and section 8
  ("A failed preparation names its repository and why").
- `docs/design/channel.md`, section 5 (the records up, "By tag, never by
  place", "Into the engine's `Parts`") and section 7 (the translations).
- `docs/design/domain/worker.md`, 4.1, 4.3 and section 9 ("temper's parts
  on jig's channel").
- `findings/temper-engine.md`, 2(c), 2(d) and section 6;
  `findings/temper-worker.md`, section 2 and Findings 1, 2.
- `decisions.md`, 3 (the design fix, "Design: close the gaps the protocol plan's code audit found").
- The code: `crates/temper-engine-domain/src/route/host_route.rs` (whole,
  187 lines); `crates/temper-engine-domain/src/{boundary.rs:270-311,
  link.rs:14-257, domain.rs:30-66, 320-336, limits.rs:212-310}`;
  `crates/temper-worker-domain/src/{wire.rs:90-166, workspace.rs:55-180,
  320-420, translate.rs:60-103, limits.rs:125-152}`;
  `tests/engine/domain/src/{protocol.rs, direct.rs}`;
  `tests/engine/domain/tests/{preparation.rs, root_routes.rs:4011-4110}`;
  `tests/worker/domain/src/whole.rs:266-450`;
  `tests/system/src/composed/link.rs` (whole, 273 lines) and
  `tests/system/src/composed.rs:575-690`.

## What the design asks

- **An undecodable call decides nothing** (domain/engine.md, 7.3): "The
  protocol layer, which decodes every call the fleet has fenced ...,
  answers one it cannot decode at once, as an error naming what is
  wrong; the root relays that answer to the run with no commit and no
  record, as it relays a call it could not serve, and the run may ask
  again with a corrected input under a new name." llm.md, 4: the routing
  "hands it to the root as `WorkerInput::RenderedHostCall { to, answer
  }`, with the answer marked an error".
- **A failed preparation is a fact of the attempt** (domain/engine.md,
  8): "the worker sends the repository's tag and the reason ... as
  temper's part beside the answer ..., and the root reports them as a
  fact of the attempt: its task, its attempt, the repository's tag and
  the reason. ... Whether the task tries again follows the answer's own
  class, transient or permanent, as for any failure."
- **`Parts.preparation`** (channel.md, 5): "from `missing`, the
  repository's tag and what the forge lacks; from `refused`, its tag and
  that the forge refused the identity."
- **By tag, never by place** (channel.md, 5): "The worker's root keeps
  each run's items, tags and starts included, until the engine has its
  answer, and names each repository of `Saved` and `Preparation` by its
  tag, with whether it started from saved work." The records:
  `SavedRepository { tag: u32  from_saved: bool  landing: Landing }`,
  `Preparation { transient, missing: Missed, refused: Tag }`.
- **The worker's routing sends a preparation for a permanent failure
  only** (channel.md, 5; domain/worker.md, 9): the root keeps
  `Transient` in its own vocabulary; 09.1's routing drops it.

## Choices this plan makes

- **The root's facts are a bounded queue the loop drains,** as the
  worker's root's are (`crates/temper-worker-domain/src/{facts.rs,
  domain.rs:196-209}`): `Domain::pop_fact` and `Domain::facts_lost`,
  `limits::FACTS` (32) facts kept, the next dropped and counted. Not a
  `Request` variant: a fact decides nothing, so it takes no room in the
  journal's door and no world's exhaustive match of `Request` changes.
  `Domain::drain_facts` keeps discarding the core's children's facts
  only. The protocol layer (08) drains the root's facts after each
  iteration.
- **The root reports the fact when the fleet fences the answer**
  (`link::result_event`, once per fenced answer), whatever the answer's
  ending; the answer's class alone decides the retry, as today.
- **The engine world's protocol stand renders an error for a name the
  world registers as undecodable** (`Protocol::undecodable`), not for
  every unknown name: `root_routes.rs:4011-4038` (`typed_call`) sends an
  unknown name and needs the stand to hand the `Decode` back (`Err`),
  which it keeps doing.
- **The worker's root keeps `from_saved` beside `tags` in `Items`,** one
  `bool` per repository in the assignment's order, set at `stage` from
  `wire::Start::Saved`.
- **The system world translates a preparation into `Parts` now**
  (`link::parts` takes it), as channel.md, 5 says the engine's routing
  will; 18.3 replaces the whole translation by the service's.

## Rules that bind it

- "**Large files.** ... An increment adds at most 40 lines to each
  (`git diff --numstat origin/main...HEAD -- <file>`, its first column);
  its new code goes in a new module beside it" (README, "Rules"). Here:
  `tests/engine/domain/tests/root_routes.rs` and `forge_routes.rs`.
- "no type beside an existing one for the same thing: change the type in
  place, and its users in the same increment" and "no field or predicate
  that its counterpart across a boundary lacks" (code-organization.md,
  section 6).
- "**No thread in a test**" and "a unit test 20 ms; a focused world test
  0.5 s" (code-organization.md, section 5).

## Increments

### 01.1 The root relays an undecodable call's rendered error

Scope: domain/engine.md, 7.3; llm.md, section 4. In `host_route::rendered`
(`host_route.rs:111-142`), a flight whose `key` is `None` (no
`DecodedHostCall` came for it) no longer returns at :115: the root relays
the answer to the run as `decoded`'s invalid branch does (:66-80): a
`Payload::SettledCall(SettledCall { serial: 0, name: flight.name, tool:
flight.tool, answer })` in `domain.payloads`, then
`Work::Fleet(fleet::Event::Relayed { to, answer: id.token() })`. No
commit, no `core::Event::SettledCall`, no record. The answer relayed is
`SettledAnswer::Host { error: true, body }`, `body` the routing's when
the `SettledCall` is `valid` under `core_limits(&domain.limits)`, and
`b"invalid call"` when it is not or when the answer is a `Delivery`. The
engine world's stand gains `Protocol::undecodable` (below), whose name's
`Decode` it answers with `WorkerInput::RenderedHostCall { to, answer:
core::SettledAnswer::Host { error: true, body: b"unknown tool" } }`.
Needs: nothing beyond the session's start.
Files: `crates/temper-engine-domain/src/route/host_route.rs`;
`tests/engine/domain/src/protocol.rs` (the stand: an `undecodable:
BTreeSet<Box<[u8]>>` beside `calls`, the method, the `Decode` arm at
:172-179); `tests/engine/domain/tests/host_calls.rs` (new).
Shape: no type changes. `host_route::rendered(domain: &mut Domain, to:
ReplyTo, answer: SettledAnswer)` keeps its signature; its no-key branch
is one private function `undecoded(domain: &mut Domain, to: ReplyTo,
flight: Flight, answer: SettledAnswer)` in the same file. World:
`Protocol::undecodable(&mut self, channel: Token, task: u64, attempt:
u64, name: u64) -> engine::Event`, the hub's `LinkUp::Call` for `name`
(its bytes `name.to_be_bytes()`, tool `b"unknown"`, as `Protocol::call`
builds one at protocol.rs:123-139).
Test shape: `tests/engine/domain/tests/host_calls.rs` (new: root_routes.rs
is a large file), on `direct::Driver` (`tests/engine/domain/src/direct.rs:39`,
`Driver::chat` at :208). Tests:
`an_undecodable_call_is_answered_as_an_error_with_no_commit`,
`an_undecodable_call_leaves_no_record_for_the_resumed_run` (park the run
with `driver.answer(.., tasks::End::Parked, engine::Parts::none())`, then
`driver.say(task, 93)` as root_routes.rs:4083-4086 does; the resumed
assignment's `settled` and `answered` are empty),
`a_corrected_call_under_a_new_name_is_decided_once` (after name 31, a
`driver.call(.., 32, engine::Call { completion: 2, position: 0, tool:
engine::Tool::Unavailable })` is answered `CallAnswer::Unavailable` once,
`driver.answered(32)`). Under 0.5 s each. Reference test, in full:

```rust
//! A host call that does not decode is answered at once as an error, with
//! no commit and no record (domain/engine.md, 7.3; llm.md, 4).
use jig_domain_host as host;
use skein_lib::Token;
use temper_engine_domain::Delivery;
use temper_engine_domain_world::direct::Driver;

#[test]
fn an_undecodable_call_is_answered_as_an_error_with_no_commit() {
    let (mut driver, run) = Driver::chat();
    let (applied, rows) = (driver.store.applied, driver.store.rows.clone());
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    assert_eq!(driver.store.applied, applied);
    assert_eq!(driver.store.rows, rows);
    let relayed: Vec<&host::Relayed> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Link { down: host::LinkDown::Relayed { call, answer, .. }, .. } = delivery
                && call.as_ref() == 31_u64.to_be_bytes()
            {
                Some(answer)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(relayed, [&host::Relayed::Answer { error: true, text: Box::from(&b"unknown tool"[..]) }]);
}
```

Proof: world, `cargo nextest run -p temper-engine-domain-world -E
'binary(host_calls)'`, run once: a call the routing could not decode
reaches its run as an error and the store applied nothing for it; the
resumed run is told nothing of it; a corrected call is decided once (the
three tests above); under 2 s.
Don't: commit, record or settle anything for an undecoded call; change
`decoded` or `render`; make the stand answer an unregistered name.
Size: at most 60 lines.
Removes: nothing.

### 01.2 `Parts.preparation`, reported as a fact of the attempt

Scope: domain/engine.md, section 8; channel.md, 5 ("Into the engine's
`Parts`"). `Parts` (`boundary.rs:283-298`) gains `preparation:
Option<Preparation>`, with `Preparation { Missing { tag, missing:
Missing }, Refused { tag } }` and `Missing { Repository, Branch, Commit }`
beside it (the interface sketch). The root's own facts: `Fact::
PreparationFailed { task, attempt, preparation }`, a `Queue<Fact>` of
`limits::FACTS` (32) in `Domain`, a `u64` count of those dropped,
`Domain::pop_fact` and `Domain::facts_lost`, the queue's room in
`limits::worst_case` (`Queue::<Fact>::worst_case(FACTS)`).
`link::result_event` (`link.rs:204-242`) destructures `Parts { saved,
pushed, preparation }` and, when the parts are within their bounds and
`preparation` is `Some`, queues the fact for the answer's task and
attempt. `parts_within` is unchanged (one value has no bound of its own).
Needs: nothing beyond the session's start.
Files: `crates/temper-engine-domain/src/{boundary.rs, link.rs, domain.rs,
limits.rs, lib.rs}` (the exports: `Fact`, `FACTS`); the users of `Parts`'
literals, each given `preparation: None` or, where both other fields are
empty, `engine::Parts::none()`:
`tests/engine/domain/tests/forge_routes.rs` (21 literals at 873, 930,
991, 1027, 1082, 1105, 1138, 1162, 1216, 1279, 1331, 1489, 1544, 1592,
1694, 1754, 1849, 1917, 1964, 2209, 2263; a large file: 21 lines),
`tests/engine/domain/tests/root_routes.rs:3937` (large file: 1 line),
`tests/engine/domain/tests/preparation.rs:56`,
`tests/system/src/composed/link.rs:90, 258, 271`,
`tests/conformance/src/link.rs:94`. Shared hotspots: `boundary.rs`,
`forge_routes.rs`.
Shape: as the interface sketch (`s01.rs`, module `engine`); `Parts::none()`
gives `preparation: None`. `Domain::pop_fact(&mut self) -> Option<Fact>`,
`Domain::facts_lost(&self) -> u64`.
Test shape: `tests/engine/domain/tests/preparation.rs` (65 lines today),
on `direct::Driver::chat` and `protocol::up` (`tests/engine/domain/src/
protocol.rs:97`). Tests:
`a_permanent_preparation_failure_is_reported_as_a_fact_of_its_attempt`,
`an_answer_without_a_preparation_reports_no_fact` (a parked answer with
`Parts::none()`: `pop_fact` is `None`),
`a_failed_preparation_retries_by_its_answers_class` (the reference's
answer: `task.tries.permanent == 1`, as `driver.task(..)` reads it at
preparation.rs:63). Under 0.5 s each. Reference test, in full (add
`use jig_domain_host as host;`):

```rust
// A permanent preparation failure names its repository beside the answer;
// the root reports it as a fact of the attempt (domain/engine.md, 8).
#[test]
fn a_permanent_preparation_failure_is_reported_as_a_fact_of_its_attempt() {
    let (mut driver, run) = Driver::chat();
    let ending = host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    };
    let answer = host::Answer { read: None, refused: Box::new([]), turns: 0, spent: 1, ending };
    let preparation = engine::Preparation::Missing { tag: 3, missing: engine::Missing::Branch };
    let parts = engine::Parts { saved: None, pushed: Box::new([]), preparation: Some(preparation) };
    let up = host::LinkUp::Answer { run: Token::new(run.task), attempt: Token::new(run.attempt), answer };
    driver.send(protocol::up(Token::new(7), up, parts));
    driver.settle();
    let fact = engine::Fact::PreparationFailed { task: run.task, attempt: run.attempt, preparation };
    assert_eq!(driver.root.pop_fact(), Some(fact));
    assert_eq!(driver.root.pop_fact(), None);
}
```

Proof: world, `cargo nextest run -p temper-engine-domain-world -E
'binary(preparation)'`, run once: a permanent failure's repository and
reason come out of the root as one fact naming the task and attempt, an
answer without one gives none, and the retry follows the class (the
three tests above, and the two already there); under 2 s.
Don't: put the preparation in a commit, a record or a task's state;
change the retry by it; add a `Request` variant; grow `forge_routes.rs`
or `root_routes.rs` by more than the literals above.
Size: at most 120 lines.
Removes: nothing.

### 01.3 The worker's saved work and failed preparation by tag

Scope: channel.md, 5; domain/worker.md, section 9. `wire::Work.saved`
becomes `Option<Box<[SavedRepository]>>` with `SavedRepository { tag,
from_saved, landing }`; `wire::Preparation` names a tag:
`{ Transient, Missing { tag, missing }, Refused { tag } }`. The worker's
root: `Items` (`workspace.rs:66-75`) keeps `from_saved: Box<[bool]>`
beside `tags`, set in `stage` (:86-140) from each repository's
`wire::Start::Saved`; `prepared` (:349-360) maps the checkout's failure's
place to its tag with `translate::failure(failure, &items.tags)`
(`translate.rs:67-75`); `wrote` (:417) builds each save's
`SavedRepository` from `tags`, `from_saved` and the landing at the same
place; `parts` (:143-175) is unchanged but for the types.
`limits::workspace_memory` (`limits.rs:143-146`) prices a saved
repository as `size_of::<wire::SavedRepository>()` and five bytes per
repository for its tag and flag (four today). The composed system world
(`tests/system/src/composed/link.rs`) drops `Items` (:28) and its
bookkeeping: `link::assignment` returns `(host::Assignment,
wire::Workspace)`; `link::parts(work: wire::Work, preparation:
Option<wire::Preparation>) -> engine::Parts` reads each saved
repository's tag and `from_saved` from the record, and maps `Missing`
and `Refused` into `engine::Preparation` (01.2's) and `Transient` to
`None`; `composed.rs` drops its `items` field (:90, :254, :584) and
passes the answer's `preparation` (:665, :679).
Needs: 01.2's `engine::Preparation` (for the system world's `parts`).
Files: `crates/temper-worker-domain/src/{wire.rs, workspace.rs,
translate.rs, limits.rs}`; users of `wire::Work.saved` and
`wire::Preparation`: `tests/worker/domain/tests/whole.rs:37` (its expected
`Missing { tag: 1, missing: wire::Missing::Branch }`: the repository at
place 0 has tag 1, `tests/worker/domain/src/whole.rs:284-285`),
`tests/worker/domain/src/whole.rs:407-411`,
`tests/worker/domain/src/runtime.rs:315` (unchanged in meaning; it
compiles as it is), `tests/system/src/composed/link.rs:24-102, 250-273`,
`tests/system/src/composed.rs:90, 254, 582-584, 665-680`. Shared
hotspots: `tests/system/src/composed.rs` (789 lines: this increment
removes more lines than it adds, and leaves it under 800).
Shape: as the interface sketch (`s01.rs`, module `wire`);
`translate::failure(failure: checkout::Failure, tags: &[u32]) ->
wire::Preparation`; `link::assignment(&engine::Assignment) ->
(host::Assignment, wire::Workspace)`; `link::parts(wire::Work,
Option<wire::Preparation>) -> engine::Parts`.
Test shape: the whole worker's world, `tests/worker/domain/tests/whole.rs`,
with a new `pub fn saved(seed: u64, start: wire::Start) -> wire::Work` in
`tests/worker/domain/src/whole.rs` (a world whose forge also has a branch
`saved` at its first commit, assigned at `start`, run through
`Case::ContactPast`'s save as `World::run` does at :384-424; the
answer's work returned); and the system world's unit tests in
`tests/system/src/composed/link.rs`. Tests:
`a_save_says_whether_each_repository_started_from_saved_work` (a table:
rows `(wire::Start::Base { branch: b"main" }, false)` and
`(wire::Start::Saved { branch: b"saved" }, true)`, each row checking
`saved == [SavedRepository { tag: 1, from_saved, landing: Landed { .. } }]`
by pattern and naming the failing row);
`a_branch_the_forge_lacks_fails_the_prepare_naming_the_repository` (kept,
its expected tag 1); in link.rs:
`the_heads_a_run_landed_reach_the_engine_as_its_pushed_heads` (kept,
`parts(work, None)`), `a_save_names_each_repository_with_saved_work_after_it`
(kept, its input as four `SavedRepository` rows with the tags and starts
the old `items` gave), `a_permanent_preparation_reaches_the_engine_by_its_tag`,
`a_transient_preparation_reaches_the_engine_as_no_part`. Whole-world
tests under 0.5 s each; link.rs's under 20 ms. Reference test, in full
(link.rs's `mod tests`):

```rust
#[test]
fn a_permanent_preparation_reaches_the_engine_by_its_tag() {
    let work = wire::Work { landed: Box::new([]), saved: None };
    let preparation = wire::Preparation::Missing { tag: 4, missing: wire::Missing::Commit };
    let parts = parts(work, Some(preparation));
    let expected = engine::Preparation::Missing { tag: 4, missing: engine::Missing::Commit };
    assert_eq!(parts, engine::Parts { saved: None, pushed: Box::new([]), preparation: Some(expected) });
}
```

Proof: world, `cargo nextest run -p temper-worker-domain-world -p
temper-system-world -E 'binary(whole) | test(/composed::link::tests::/)'`,
run once: a save names each repository by its tag and says whether it
started from saved work, a failed prepare names its repository's tag, and
the system world's translation carries both into the engine's `Parts`
with no state of its own (the tests above); under 5 s.
Don't: send a `Transient` preparation's tag anywhere; keep tags or starts
in the system world; change `wire::Landed` or the landed part.
Size: at most 150 lines.
Removes: the system world's `link::Items` and its use in `composed.rs`.

## Interface sketches

The same text as `scratch/plan-sketches/src/s01.rs`, which compiles.
`engine` is the root's boundary as 01.2 leaves it (owner 01.2; 08's
engine `parts` translation fills it field for field); `wire` is the
worker's as 01.3 leaves it (owner 01.3; 03.1's records mirror it, 09.1's
translations read it). Nothing here has a lifetime of its own: a `Parts`
rides one answer, a `Fact` lives in the queue until drained or dropped.

```rust
// temper_engine_domain (crates/temper-engine-domain/src/boundary.rs): the
// root's parts as 01.2 leaves them, and the root's own fact. Owner: 01.2.
// Users: 08's engine `parts` translation (field for field), 18.3.
pub mod engine {
    use alloc::boxed::Box;
    use temper_engine_domain_forge as forge;

    /// temper's parts beside a hub answer (channel.md, 5): the engine's
    /// routing makes it from the worker's `Left`, `Saved` and `Preparation`
    /// records; the root reads it once the fleet has fenced the answer.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Parts {
        /// The tags with saved work after this terminal, when the worker
        /// saved; absent keeps the previous set.
        pub saved: Option<Box<[u32]>>,
        /// The last head each repository's pushes landed, by tag.
        pub pushed: Box<[forge::Pushed]>,
        /// A permanent failure to prepare, naming its repository by tag;
        /// none for every other answer.
        pub preparation: Option<Preparation>,
    }

    impl Parts {
        /// A record that carries no application part.
        #[must_use]
        pub fn none() -> Parts {
            unreachable!("sketch")
        }
    }

    /// Why a repository could not be prepared, permanently: channel.md 5's
    /// `Preparation` but `transient`, which the worker's routing never sends.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Preparation {
        /// The forge lacks what the assignment names for repository `tag`.
        Missing { tag: u32, missing: Missing },
        /// The forge refused the identity of repository `tag`.
        Refused { tag: u32 },
    }

    /// What the forge lacks (channel.md 5's `Missing`).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Missing {
        Repository,
        /// A branch to start from.
        Branch,
        Commit,
    }

    /// What the root tells whoever watches it (domain/engine.md, 8). The
    /// root queues it in the step that fenced the answer; the loop drains it
    /// with `Domain::pop_fact`; past `FACTS` queued, one is dropped and
    /// counted in `Domain::facts_lost`. Nothing the root decides reads it.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Fact {
        /// A fenced answer said its workspace could not be prepared,
        /// permanently: its task, its attempt, the repository and why.
        PreparationFailed { task: u64, attempt: u64, preparation: Preparation },
    }

    /// How many of the root's own facts wait to be drained, as `VIEWS.facts`.
    pub const FACTS: u32 = 32;

    // impl temper_engine_domain::Domain {
    //     /// The oldest of the root's own facts not drained yet.
    //     pub fn pop_fact(&mut self) -> Option<Fact>;
    //     /// How many of the root's own facts were dropped for want of room.
    //     pub fn facts_lost(&self) -> u64;
    // }
    // `drain_facts` keeps discarding the core's children's facts only.
}

// temper_worker_domain::wire (crates/temper-worker-domain/src/wire.rs): what
// a run left, by tag, as 01.3 leaves it. Owner: 01.3. Users: 03.1's records
// (field for field), 09.1's worker translations, 18.3.
pub mod wire {
    use alloc::boxed::Box;
    use temper_worker_domain::wire::{Landed, Landing, Missing};

    /// What a run left on the forge (channel.md, 5): the repositories its
    /// pushes landed in, by tag, ascending; and its save, if one was made,
    /// each repository's outcome in the assignment's order.
    #[derive(PartialEq, Eq, Hash, Debug)]
    pub struct Work {
        pub landed: Box<[Landed]>,
        pub saved: Option<Box<[SavedRepository]>>,
    }

    /// One repository's outcome of a save, named by its tag, with whether
    /// its checkout started from the task's saved work. Made by the worker's
    /// root from the items it keeps until the engine has the answer.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct SavedRepository {
        pub tag: u32,
        pub from_saved: bool,
        pub landing: Landing,
    }

    /// Why a workspace could not be prepared, naming the repository by tag.
    /// The worker's routing (09.1) sends `Missing` and `Refused` as temper's
    /// part beside a permanent failure, and drops `Transient`.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Preparation {
        /// A retry may work: the forge unreachable, the worker's side, time,
        /// or the workstream's checkout held by another run.
        Transient,
        /// The forge lacks what the assignment names for repository `tag`.
        Missing { tag: u32, missing: Missing },
        /// The forge refused the identity of repository `tag`.
        Refused { tag: u32 },
    }

    // crates/temper-worker-domain/src/translate.rs: a checkout's failure,
    // its repository's place mapped to the tag the items keep for it.
    // pub(crate) fn failure(failure: checkout::Failure, tags: &[u32]) -> wire::Preparation;
}
```

The engine world's stand (ordinary Rust, not compiled), in
`tests/engine/domain/src/protocol.rs`:

```rust
impl Protocol {
    /// A run's call that the world's protocol cannot decode, named `name`:
    /// its `Decode` is answered with a rendered error, as the routing
    /// answers a call `temper_engine_smith::call` refuses (llm.md, 4).
    pub fn undecodable(&mut self, channel: Token, task: u64, attempt: u64, name: u64) -> engine::Event;
}
```

## Questions to raise when reached

- **A rendered answer for an undecoded call that is not marked an
  error.** llm.md, 4 has the routing mark it; the root, as 01.1 writes
  it, relays it as an error whatever the routing marked. Default: the
  root marks it an error.
- **The root's facts' sink.** domain/engine.md, 8 says the root reports
  the fact; who reads it (a log line, a count) is the protocol layer's
  (08). Default: the root queues and counts; 08 drains it.

## Done when

- The three increments' tests pass: `cargo nextest run -p
  temper-engine-domain-world -p temper-worker-domain-world -p
  temper-system-world -E 'binary(host_calls) | binary(preparation) |
  binary(whole) | test(/composed::link::tests::/)'` passes.
- The system world keeps no tags of its own: `grep -nw Items
  tests/system/src/composed/link.rs tests/system/src/composed.rs` prints
  nothing.
- The large files grew within their bound: `for c in $(git log
  --format=%h --grep='^Step: temper-01\.' origin/main); do git show
  --numstat --format= $c -- tests/engine/domain/tests/root_routes.rs
  tests/engine/domain/tests/forge_routes.rs; done | awk '$1 > 40'` prints
  nothing.
- The gate passed on this session's last landing: `git -C ~/src/rust/temper
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-01\.' origin/main | head -1` names
  `scripts/check.sh passed`.
