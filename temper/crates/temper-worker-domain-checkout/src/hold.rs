//! Holds: a client's hold on a workspace, from its prepare to its release,
//! and the operations it runs there, one at a time (domain/worker/hosted-runs.md, 1.2
//! steps 2, 7 and 8, and 5). Preparations take permits in arrival order
//! (domain/worker/checkouts.md, Preparations a few at a time).
//!
//! A prepare that passes the entrance holds a workspace ([`crate::cache`]) and
//! readies it, one operation at a time. An evicted workspace may keep its
//! matching clones when reuse is enabled. A first hold looks at each repository
//! on disk when reuse is enabled and none starts saved. A workspace made again
//! is made empty, and each repository cloned into it. Then each is fetched at its
//! starting point and checked out. A base with heads merged in fetches every
//! head and commits each new merge in order before it is ready. Its last
//! commit is the save baseline; its merges are delivered by the first push.
//! A producer's merge start compares its base with the checked-out branch.
//! A contained base resumes as branch or saved work without a merge. A clean
//! producer merge is committed and checked out before ready; its save baseline
//! stays the producer's starting commit. A producer's conflict and other merge
//! starts leave the merge in progress for the run to settle.
//! A base branch the forge does not have is
//! created there from the default branch, for a repository that may be
//! written, and only created: if another party created it meanwhile, it is
//! fetched again and the workspace starts from wherever that party put it.
//!
//! A push commits each writable repository's tree exactly as it is, on the
//! commit it was checked out at or last committed, and pushes the commit to
//! the repository's push branch, as a fast-forward: a branch that moved since
//! the workspace started is left where it is (`Moved`). A design delivery,
//! whose push and landing branches are equal, instead fetches and merges the
//! goal tip first, admits Markdown changes alone and retries a rejected lease
//! three times. Its refusals leave the run going. A repository is
//! pushed only if it has a commit the branch has not had from this hold
//! since the start. Otherwise it is `Unchanged`, unless this hold's last push
//! landed its head there: then it is that landing again, so that a delivery
//! followed by a finish with nothing new is still a delivery. An eligible
//! producer retry compares with the landing pin fetched during preparation:
//! unchanged existing work outside that tip is delivered by a fresh guarded push, after
//! a local ancestry query. This snapshot never changes the checkout start.
//! A checked push first repushes a start covered by the task's own delivery and the fetched push head.
//! An early producer whose snapshot loses a starting head asks its client
//! whether that dependency landed, with no IO in flight while it waits.
//! Catch-up keeps its work locally, moves the landing pin, and merges that tip
//! into the work. A delivery checks again; an unchecked save may push at once.
//! A save does the
//! same to the saved-work branch, for every repository changed since the
//! start, and is `Unchanged` without a change since the start. Pushing
//! is not atomic: each repository is pushed on its own, whatever came of the
//! one before. A push that failed in a way that may have landed all the same
//! (it ran out of time, broke, or lost the forge on the way) is verified: the
//! branch is fetched, and the push landed if the branch is at its commit.
//!
//! The transition table. Every other cell is unreachable by the boundary's
//! contract: one terminal event per operation, and an operation's terminal
//! only one of its own.
//!
//! ```text
//! state       event                     next        requests
//! (none)      prepare, refused          (none)      prepared: refused
//!             prepare, admitted         Preparing   held, io: store, look, make or fetch
//!             ... another makes store   Waiting     held
//!             ... every permit taken    Queued      held
//! Queued      permit returned           Preparing   io: store, look, make or fetch
//!             ... another makes store   Waiting     (nothing)
//!             abort                     Unprepared  prepared: aborted
//!             release                   Closed      prepared: aborted, released
//!             push, save                Queued      pushed or saved: refused, busy
//! Preparing   done, more to do          Preparing   io: the next operation
//!             done, merge checked out   Preparing   io: compare base/head, or merge
//!             done, base contained      Preparing   io: fetch landing or push branch
//!             done, base absent         Preparing   io: merge
//!             done, producer merged     Preparing   io: commit merge with both parents
//!             done, merge committed     Preparing   io: check out merge, clearing its state
//!             done, ready, pack due     Preparing   io: pack the next due repository
//!             done, pack settled        Preparing   io: pack the next due repository, if any
//!             done, no packs due        Ready       prepared: ready or candidate
//!             done, failed              Unprepared  prepared: failed
//!             done, aborting            Unprepared  prepared: aborted
//!             done, releasing           Closed      prepared: aborted, released
//!             abort, release            Preparing   cancel, the first time asked
//! Waiting     store made                Preparing   io: next store, look or make
//!             store failed, oldest      Preparing   io: store
//!             store failed, later       Waiting     (nothing)
//!             abort                     Unprepared  prepared: aborted
//!             release                   Closed      prepared: aborted, released
//! Pushing     done, start head absent   Pushing     dependency landed? (no io in flight)
//!             answer, not landed        Pushing/Ready refused, next repository or pushed
//!             answer, landed            Pushing     io: commit work locally
//!             work committed            Pushing     io: reset HEAD, keep files
//!             work reset                Pushing     io: fetch landing into pin
//!             landing fetched           Pushing     io: compare ancestry
//!             tip contained             Pushing/Ready caught up as below, no merge
//!             tip absent                Pushing     io: merge tip into work
//!             tip merged cleanly        Pushing     io: commit merge (work, tip)
//!             merge committed           Pushing     io: reset HEAD, keep files
//!             caught up, delivery       Pushing/Ready caught up, next repository or pushed
//!             caught up, save           Pushing     io: snapshot commit, then push as save
//!             catch-up conflict         Pushing/Ready conflict, next repository or pushed
//!             waiting, abort/cut        Ready       pushed aborted, no io or cancel
//!             waiting, release          Closed      pushed aborted, released
//!             done, more to do          Pushing     io: the next operation
//!             done, a push that may     Pushing     io: fetch its branch
//!               have landed
//!             done, all pushed          Ready       pushed or saved
//!             done, aborting            Ready       pushed or saved (the rest aborted)
//!             done, releasing           Closed      pushed or saved, released
//!             abort, release            Pushing     (it waits for the operation)
//! Waiting,    push, save                (same)      pushed or saved: refused, busy
//! Preparing,  push, save                (same)      pushed or saved: refused, busy
//! Pushing
//! Ready       push, save                Pushing     io: commit
//!             ... nothing writable      Ready       pushed or saved: all unchanged
//!             ... beyond the limits     Ready       pushed or saved: refused, invalid
//!             abort                     Ready       (nothing is under way)
//!             release                   Closed      released
//! Unprepared  push, save                Unprepared  pushed or saved: refused, busy
//!             abort                     Unprepared  (nothing is under way)
//!             release                   Closed      released
//! Closed      push, save, abort,        Closed      (dropped: the handle is stale)
//!               release
//! Any         answer, not waiting       (same)      (dropped)
//! ```
//!
//! "More to do" follows the steps above. A prepare asked to abort, or to
//! release, cancels its operation in flight, ends aborted once that has
//! ended, whatever its end, and starts nothing more. A push or a save is
//! never cancelled: so that a push that lands is reported landed, the
//! operation in flight runs to its end, which its deadline bounds, and a push
//! that may have landed is verified all the same; then the push or the save
//! ends, reporting the repositories not reached as aborted. Nothing touches
//! the workspace afterwards: the hold runs one operation at a time, and it
//! ends only once that operation has.
//!
//! An operation that broke, ran out of time or was cancelled may have left
//! its repository damaged (git was killed, or failed half way), so the
//! workspace is not trusted again: the next prepare makes it again.
//!
//! A transition's requests account for operations and tell facts in one
//! place ([`sent`]), and what the new state implies (a closed hold is
//! retired, and its workspace idle again) in another ([`follow`]).

mod candidate;
mod catch_up;
mod design;
mod detail;

mod reds;

use alloc::boxed::Box;
use core::mem;

use skein_lib::bytes::{copy_of, find};
use skein_lib::{Duration, Env, Id, List, Queue, Slab, Time, Token, Writer};

use crate::boundary::{
    Conflicts, Failure, Landing, Message, Outcome, Prepared, Refusal, Repository, Request, Spec, Start,
};
use crate::cache::{Cache, Claim, Workspace, count};
use crate::domain::Domain;
use crate::facts::{Cached, Ending, Fact, Facts, Preparation, Tally, Target};
use crate::git::{Commit, Done, Fault, Into, Missing, Op, Place, Want};
use crate::limits::Limits;

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "cut, lane progress, release and mark presence are independent")]
pub struct Hold {
    holding: Holding,
    state: State,
    cut: bool,
    /// The deadline handed to io, retained until its terminal event.
    operation: Option<Time>,
    /// Marks share the workspace's IO lane, but their rewrites do not delay answers.
    marking: bool,
    pending: Queue<(Op, bool)>,
    releasing: bool,
    /// The last queued held mark describes `holding`; only a completion changes its tips.
    marked: bool,
}

#[derive(Debug)]
struct MarkedTip {
    head: Commit,
    merging: Option<Commit>,
    unchecked: bool,
}

impl Hold {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn workspace(&self) -> Token {
        self.holding.workspace.token()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn queued_since(&self) -> Option<Time> {
        match self.state {
            State::Queued { .. } => Some(self.holding.arrived),
            State::Preparing { .. }
            | State::Waiting { .. }
            | State::Ready
            | State::Reds(_)
            | State::Unprepared
            | State::Pushing { .. }
            | State::Keeping { .. }
            | State::Closed => None,
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn is_preparing(&self) -> bool {
        active(&self.state) || self.holding.recovering
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn is_ended(&self) -> bool {
        match self.state {
            State::Closed => self.operation.is_none(),
            State::Queued { .. }
            | State::Preparing { .. }
            | State::Waiting { .. }
            | State::Ready
            | State::Reds(_)
            | State::Unprepared
            | State::Pushing { .. }
            | State::Keeping { .. } => false,
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn operation_deadline(&self) -> Option<Time> {
        self.operation
    }
}

/// What a hold holds in every state.
#[derive(Debug)]
struct Holding {
    /// The client's token, echoed on every record back to it.
    client: Token,
    workspace: Id<Workspace>,
    disk: Token,
    key: Box<[u8]>,
    kept: Option<Box<crate::git::Kept>>,
    saved: Option<Box<[crate::git::KeptRepository]>>,
    preserve: bool,
    recovering: bool,
    recovered: bool,
    /// The project's bound, or `u32::MAX` when only the worker's bound applies.
    rerun_bound: u32,
    original_starts: Box<[(u32, Start)]>,
    arrived: Time,
    admitted: Time,
    stores: Box<[crate::cache::Borrowed]>,
    /// The spec's repositories, in its order.
    repositories: Box<[Repository]>,
    /// Where each repository checked out so far stands, in the same order.
    tips: List<Tips>,
    /// Moved to the client exactly once when preparation succeeds.
    conflicts: List<Conflicts>,
    /// What this prepare fetched, never a previous hold's observations.
    fetched: List<Fetched>,
    candidate: Option<candidate::Stack>,
}

/// The operation's own clock, separate from the prepare and its permit wait.
#[derive(Clone, Copy, Debug)]
struct Timing {
    started: Time,
    timeout: Duration,
}

#[derive(Debug)]
struct Fetched {
    repository: u32,
    want: Want,
    commit: Commit,
}

/// Where a repository stands: the commit it was checked out at, the commit
/// its working tree is on (that one, or the last it committed), and the
/// newest commit its push branch is known to have from this hold (the start,
/// until a push lands); saved starts instead fetch the delivery baseline.
/// `delivered` records whether this hold pushed its head; `unchecked` requires
/// a fresh commit while that head is its last save.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Tips {
    start: Commit,
    head: Commit,
    pushed: Option<Commit>,
    delivered: bool,
    /// Prepare's fetched push head and this task's own delivery both cover the start.
    redeliver: bool,
    unchecked: bool,
    merging: Option<Commit>,
    expected: Option<Commit>,
    landing: Option<Commit>,
    /// The tip this head merged, retained for repeated delivery receipts.
    design_merged: Option<Commit>,
    /// The landing tip merged before this head's unchecked save.
    caught_up: Option<Commit>,
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "diagnostics stay inline with a fixed byte capacity")]
enum State {
    /// Holds a workspace and borrows stores, waiting for a preparation permit.
    Queued {
        cached: Cached,
    },
    Keeping {
        mark: crate::git::Kept,
    },
    /// Preparing the workspace: `step` is in flight.
    Preparing {
        step: PrepareStep,
        asked: Asked,
        timing: Timing,
    },
    /// Waiting for another hold's store, with no operation in flight.
    Waiting {
        repository: u32,
        look: bool,
    },
    /// Prepared, with nothing under way.
    Ready,
    /// Best-effort check files before a deferred push or release.
    Reds(reds::Running),
    /// Its prepare failed or was aborted: it may only be released.
    Unprepared,
    /// Pushing or saving: `step` is in flight, except a dependency question.
    Pushing {
        push: Push,
        step: PushStep,
        asked: Asked,
    },
    /// Terminal: released, holding nothing.
    Closed,
}

/// The operation a preparing hold waits for. A repository is named by its
/// place in the spec.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum PrepareStep {
    Unkeep,
    Mark,
    Marked,
    CompareKept {
        repository: u32,
    },
    /// Best-effort maintenance before ready (domain/worker/checkouts.md, *Kept packed*).
    Pack {
        repository: u32,
    },
    Reds {
        repository: u32,
    },
    Candidate {
        repository: u32,
        step: candidate::Step,
    },
    Store {
        repository: u32,
        look: bool,
    },
    Look {
        repository: u32,
    },
    Make,
    Clone {
        repository: u32,
    },
    /// Its starting point.
    Fetch {
        repository: u32,
    },
    /// The default branch, for a base branch the forge does not have.
    Default {
        repository: u32,
    },
    /// The base branch, at the default branch's `commit`.
    Create {
        repository: u32,
        commit: Commit,
    },
    /// The base branch, which another party created meanwhile.
    Refetch {
        repository: u32,
    },
    CheckOut {
        repository: u32,
        commit: Commit,
        landing: Option<Commit>,
        pushed: Option<Commit>,
    },
    /// Read the delivery branch for a saved baseline or a replaced-head pin; it may not exist yet.
    FetchPush {
        repository: u32,
        ours: Commit,
    },
    /// Pin the delivery branch, or the start when that branch is missing.
    PinReplaced {
        repository: u32,
        ours: Commit,
        pushed: Option<Commit>,
    },
    /// Point the pin at a commit this prepare fetched.
    PointLanding {
        repository: u32,
        ours: Commit,
        commit: Commit,
        merging: Option<Commit>,
        pushed: Option<Commit>,
    },
    /// The pin, followed by the start's remaining preparation or a producer merge.
    FetchLanding {
        repository: u32,
        ours: Commit,
        merging: Option<Commit>,
        pushed: Option<Commit>,
    },
    /// Main's counts, immediately after writing the pin.
    Counts {
        repository: u32,
        ours: Commit,
        then: AfterPin,
    },
    FetchBase {
        repository: u32,
        ours: Commit,
        base: Commit,
        head: Option<u32>,
    },
    PointBase {
        repository: u32,
        ours: Commit,
        base: Commit,
        head: Option<u32>,
    },
    CheckOutMerge {
        repository: u32,
        ours: Commit,
        base: Commit,
    },
    /// A producer's merge start asks whether its branch holds the base (domain/worker/checkouts.md).
    CompareBase {
        repository: u32,
        ours: Commit,
        base: Commit,
    },
    CompareHead {
        repository: u32,
        head: u32,
    },
    Merge {
        repository: u32,
        theirs: Commit,
        head: Option<u32>,
    },
    CommitHead {
        repository: u32,
        head: u32,
    },
    CheckOutHead {
        repository: u32,
        head: u32,
    },
    CommitMerge {
        repository: u32,
        theirs: Commit,
    },
    CheckOutMerged {
        repository: u32,
    },
}

/// The preparation that resumes once the counts ref is refreshed or removed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum AfterPin {
    Landing { commit: Commit, merging: Option<Commit>, pushed: Option<Commit> },
    Replaced { pushed: Option<Commit> },
}

/// A push or a save under way.
#[derive(Debug)]
struct Push {
    to: To,
    message: Message,
    /// What came of each repository so far, in the spec's order.
    landings: List<Landing>,
}

/// Where a push goes: each writable repository's push branch, or its
/// saved-work branch.
#[derive(Debug)]
enum To {
    Unchecked,
    Branches,
    Saved,
}

fn unchecked_head(to: &To) -> bool {
    match to {
        To::Unchecked => true,
        To::Branches | To::Saved => false,
    }
}

/// The operation or dependency answer a pushing hold waits for.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum PushStep {
    Dependency {
        repository: u32,
        head: Commit,
        merging: Option<Commit>,
    },
    CatchUp {
        repository: u32,
        step: catch_up::Step,
    },
    Prefix {
        repository: u32,
        member: u32,
    },
    VerifyPrefix {
        repository: u32,
        member: u32,
        failure: Done,
    },
    Design {
        delivery: design::Delivery,
        step: design::Step,
    },
    CompareLanding {
        repository: u32,
    },
    Commit {
        repository: u32,
    },
    Push {
        repository: u32,
    },
    /// The branch a push that may have landed went to.
    Verify {
        repository: u32,
        failure: Done,
    },
}

/// What the client asked of the operation under way: nothing, to abort it, or
/// to release the hold once it has ended.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Asked {
    Nothing,
    Abort,
    Release,
}

/// What a prepare does once an operation has ended.
enum Next {
    Step(PrepareStep),
    Recover,
    Stores { from: u32, look: bool },
    Ready { pack_from: u32 },
    Failed(Failure),
}

// Entry points, one per event: look the hold up, take its state out, run the
// cell's handler, then tell what happened and follow the new state
// ([`conclude`]).

#[expect(clippy::manual_map, reason = "the step subset boxes the optional recovery mark without a closure")]
pub(crate) fn prepare(domain: &mut Domain, env: &Env<Limits>, client: Token, spec: Spec, out: &mut Queue<Request>) {
    let mark = out.len();
    if !fits(&spec, &env.limits) {
        refuse_prepare(&mut domain.facts, client, Refusal::Invalid, out);
        return;
    }
    let Spec { key, rerun, repositories } = spec;
    let crate::cache::Granted { workspace, cached, stores, token: disk, kept } =
        match domain.cache.hold(key.clone(), &repositories) {
            Ok(holding) => holding,
            Err(refusal) => {
                refuse_prepare(&mut domain.facts, client, refusal, out);
                return;
            }
        };
    domain.facts.push(Fact::Held { client, cached });
    let tips = List::with_capacity(count(repositories.len()));
    let conflicts = List::with_capacity(count(repositories.len()));
    let candidate = candidate::new(&repositories);
    let holding = Holding {
        client,
        workspace,
        disk,
        key,
        kept: match kept {
            Some(mark) => Some(Box::new(mark)),
            None => None,
        },
        saved: None,
        preserve: false,
        recovering: false,
        recovered: false,
        rerun_bound: rerun.unwrap_or(u32::MAX),
        original_starts: Box::new([]),
        arrived: env.now,
        admitted: env.now,
        stores,
        repositories,
        tips,
        conflicts,
        fetched: List::uncapped(),
        candidate,
    };
    let hold = Hold {
        holding,
        state: State::Closed,
        cut: false,
        operation: None,
        marking: false,
        pending: Queue::uncapped(),
        releasing: false,
        marked: false,
    };
    let id = domain.holds.insert(hold).expect("room for a hold for each workspace, and as many released");
    out.push(Request::Held { client, hold: id.token() });
    if domain.active < env.limits.preparations {
        admit(domain, env, id, cached, out);
    } else {
        domain.holds.get_mut(id).expect("inserted above").state = State::Queued { cached };
        domain.queued.push(id);
        domain.facts.push(Fact::Queued { client });
    }
    conclude(domain, env, id, false, out, mark);
}

/// The first step is shared by prepares arriving with a free permit and
/// those admitted from the queue. Its io deadlines start at this step's now.
fn admit(domain: &mut Domain, env: &Env<Limits>, id: Id<Hold>, cached: Cached, out: &mut Queue<Request>) {
    domain.active = domain.active.checked_add(1).expect("one active prepare per held workspace");
    let hold = domain.holds.get_mut(id).expect("an admitted hold lives");
    hold.holding.admitted = env.now;
    hold.state = if hold.holding.kept.is_some() {
        prepare_step(&hold.holding, &domain.cache, id, PrepareStep::Unkeep, env, out)
    } else {
        match cached {
            Cached::Reused | Cached::Kept => {
                prepare_step(&hold.holding, &domain.cache, id, PrepareStep::Fetch { repository: 0 }, env, out)
            }
            Cached::Looking | Cached::Rebuilt | Cached::Evicted | Cached::New => {
                stores_step(&hold.holding, &mut domain.cache, id, 0, cached == Cached::Looking, env, out)
            }
        }
    };
}

/// Store successors are resumed before new admissions, preserving the oldest
/// store waiter's claim when a maker fails. All admissions are in this step.
fn admissions(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    for _ in 0..domain.queued.len() {
        if domain.active >= env.limits.preparations {
            break;
        }
        let id = domain.queued.pop().expect("the queued prepares to admit");
        let cached = match domain.holds.get(id).expect("a queued hold lives").state {
            State::Queued { cached } => cached,
            State::Preparing { .. }
            | State::Waiting { .. }
            | State::Ready
            | State::Reds(_)
            | State::Unprepared
            | State::Pushing { .. }
            | State::Keeping { .. }
            | State::Closed => {
                unreachable!("only queued holds wait for permits")
            }
        };
        let mark = out.len();
        admit(domain, env, id, cached, out);
        conclude(domain, env, id, false, out, mark);
    }
}

pub(crate) fn push(domain: &mut Domain, env: &Env<Limits>, hold: Token, message: Message, out: &mut Queue<Request>) {
    start(domain, env, hold, To::Branches, message, out);
}

pub(crate) fn unchecked(domain: &mut Domain, env: &Env<Limits>, hold: Token, out: &mut Queue<Request>) {
    let message =
        Message { title: copy_of(b"Unchecked: work in progress, saved before a question"), body: Box::new([]) };
    start(domain, env, hold, To::Unchecked, message, out);
}

pub(crate) fn save(domain: &mut Domain, env: &Env<Limits>, hold: Token, message: Message, out: &mut Queue<Request>) {
    start(domain, env, hold, To::Saved, message, out);
}

fn start(domain: &mut Domain, env: &Env<Limits>, hold: Token, to: To, message: Message, out: &mut Queue<Request>) {
    let mark = out.len();
    let Some(id) = addressed(&domain.holds, hold) else {
        return;
    };
    let hold = domain.holds.get_mut(id).expect("addressed above");
    let was_active = active(&hold.state) || hold.holding.recovering;
    let state = mem::replace(&mut hold.state, State::Closed);
    hold.state = match state {
        State::Ready if hold.cut => {
            let push = Push { to, message, landings: List::with_capacity(count(hold.holding.repositories.len())) };
            end_push(&hold.holding, push, Asked::Abort, out)
        }
        State::Ready => begin(&hold.holding, id, to, message, env, out),
        State::Reds(mut running) => {
            if running.push.is_none() {
                running.push = Some((to, message));
            } else {
                answer(hold.holding.client, &to, Outcome::Refused { refusal: Refusal::Busy }, out);
            }
            State::Reds(running)
        }
        busy @ (State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Pushing { .. }
        | State::Unprepared) => {
            answer(hold.holding.client, &to, Outcome::Refused { refusal: Refusal::Busy }, out);
            busy
        }
        State::Keeping { .. } | State::Closed => unreachable!("an addressed hold has not been released"),
    };
    conclude(domain, env, id, was_active, out, mark);
}

pub(crate) fn dependency_landed(
    domain: &mut Domain,
    env: &Env<Limits>,
    hold: Token,
    landed: bool,
    out: &mut Queue<Request>,
) {
    let Some(id) = addressed(&domain.holds, hold) else { return };
    let mark = out.len();
    let hold = domain.holds.get_mut(id).expect("addressed hold");
    let was_active = active(&hold.state) || hold.holding.recovering;
    let state = mem::replace(&mut hold.state, State::Closed);
    hold.state = match state {
        State::Pushing { push, step: PushStep::Dependency { repository, head, merging }, asked } => {
            if landed {
                catch_up::begin(&hold.holding, id, push, repository, head, merging, env, out)
            } else {
                complete_push(&hold.holding, id, push, repository, catch_up::refused(), asked, env, out)
            }
        }
        other @ (State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Pushing { .. }
        | State::Keeping { .. }
        | State::Closed) => other,
    };
    conclude(domain, env, id, was_active, out, mark);
}

pub(crate) fn abort(domain: &mut Domain, env: &Env<Limits>, hold: Token, out: &mut Queue<Request>) {
    ask(domain, env, hold, Asked::Abort, out);
}

/// Unlike Abort, a stop-bound cut cancels pushes and saves too, and is sticky.
pub(crate) fn cut(domain: &mut Domain, env: &Env<Limits>, hold: Token, out: &mut Queue<Request>) {
    let Some(id) = addressed(&domain.holds, hold) else { return };
    let hold = domain.holds.get_mut(id).expect("addressed above");
    if hold.cut {
        return;
    }
    hold.cut = true;
    match &hold.state {
        State::Pushing { step, .. } => {
            if !catch_up::waiting(step) {
                out.push(Request::Cancel { owner: id.token() });
            }
        }
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Keeping { .. }
        | State::Closed => {}
    }
    ask(domain, env, id.token(), Asked::Abort, out);
}

pub(crate) fn release(domain: &mut Domain, env: &Env<Limits>, hold: Token, out: &mut Queue<Request>) {
    ask(domain, env, hold, Asked::Release, out);
}

/// The client asks to abort, or to release: a prepare's operation is
/// cancelled the first time it asks, a push's runs to its end, and a hold
/// with nothing under way is released at once.
fn ask(domain: &mut Domain, env: &Env<Limits>, hold: Token, now: Asked, out: &mut Queue<Request>) {
    let mark = out.len();
    let Some(id) = addressed(&domain.holds, hold) else {
        return;
    };
    let hold = domain.holds.get_mut(id).expect("addressed above");
    let was_active = active(&hold.state) || hold.holding.recovering;
    let state = mem::replace(&mut hold.state, State::Closed);
    hold.state = match state {
        State::Preparing { step, asked, timing } => {
            State::Preparing { step, asked: cancelling(id, asked, now, out), timing }
        }
        State::Reds(mut running) => {
            running.asked = outranking(running.asked, now);
            reds::interrupted(&hold.holding, &running.action, out);
            if let Some(next) = &running.next {
                reds::interrupted(&hold.holding, next, out);
            }
            running.next = Some(crate::git::Reds::Clear);
            running.write = None;
            out.push(Request::Cancel { owner: id.token() });
            State::Reds(running)
        }
        State::Pushing { push, step, asked } => {
            let asked = outranking(asked, now);
            if catch_up::waiting(&step) {
                end_push(&hold.holding, push, asked, out)
            } else {
                State::Pushing { push, step, asked }
            }
        }
        State::Queued { .. } => {
            crate::cache::unqueue(&mut domain.queued, id);
            out.push(Request::Prepared { client: hold.holding.client, prepared: Prepared::Aborted });
            match now {
                Asked::Release => released(hold.holding.client, out),
                Asked::Abort => State::Unprepared,
                Asked::Nothing => unreachable!("a client asks to abort or to release"),
            }
        }
        State::Waiting { repository, look: _ } => {
            domain.cache.unwait(borrowed(&hold.holding, repository).store, id);
            out.push(Request::Prepared { client: hold.holding.client, prepared: Prepared::Aborted });
            match now {
                Asked::Release => released(hold.holding.client, out),
                Asked::Abort => State::Unprepared,
                Asked::Nothing => unreachable!("a client asks to abort or to release"),
            }
        }
        idle @ (State::Ready | State::Unprepared) => match now {
            Asked::Release => released(hold.holding.client, out),
            Asked::Abort => idle,
            Asked::Nothing => unreachable!("a client asks to abort or to release"),
        },
        State::Keeping { .. } | State::Closed => unreachable!("an addressed hold has not been released"),
    };
    conclude(domain, env, id, was_active, out, mark);
    admissions(domain, env, out);
}

fn optional(state: &State) -> bool {
    match state {
        State::Reds(_)
        | State::Preparing { step: PrepareStep::Reds { .. } | PrepareStep::Pack { .. } | PrepareStep::Mark, .. } => {
            true
        }
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Unprepared
        | State::Pushing { .. }
        | State::Keeping { .. }
        | State::Closed => false,
    }
}

/// Terminal for an operation: what the hold does next.
#[expect(clippy::too_many_lines, reason = "each live operation state handles its own terminal")]
pub(crate) fn done(domain: &mut Domain, env: &Env<Limits>, owner: Token, done: Done, out: &mut Queue<Request>) {
    let mark = out.len();
    let done = bounded_done(done, &env.limits);
    let id = Id::from_token(owner);
    let hold = domain.holds.get_mut(id).expect("a hold lives until its operation has ended");
    hold.operation = None;
    if hold.marking {
        domain.facts.push(Fact::Ended { client: hold.holding.client, done: ending(&done) });
        hold.marking = false;
        if hold.cut
            && !hold.pending.is_empty()
            && let State::Pushing { .. } = hold.state
        {
            let State::Pushing { push, asked, .. } = mem::replace(&mut hold.state, State::Closed) else {
                unreachable!("pending push");
            };
            let (_, background) = hold.pending.pop().expect("the unstarted git operation");
            assert!(!background, "the push waits behind this mark");
            hold.state = end_push(&hold.holding, push, asked, out);
        }
        let was_active = active(&hold.state) || hold.holding.recovering;
        conclude(domain, env, id, was_active, out, mark);
        return;
    }
    let previous = if hold.marked { Some(held_tips(&hold.holding)) } else { None };
    let told = ending(&done);
    domain.facts.push(Fact::Ended { client: hold.holding.client, done: told });
    if !optional(&hold.state) && damages(&done) {
        domain.cache.spoil(hold.holding.workspace);
    }
    let was_active = active(&hold.state) || hold.holding.recovering;
    let state = mem::replace(&mut hold.state, State::Closed);
    let recovery_asked = match &state {
        State::Pushing { asked, .. } => *asked,
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Keeping { .. }
        | State::Closed => Asked::Nothing,
    };
    let holding = &mut hold.holding;
    let mut waiters = Queue::uncapped();
    let mut store_succeeded = false;
    hold.state = match state {
        State::Keeping { mark } => {
            if done == Done::Succeeded {
                domain.cache.keep(holding.workspace, mark);
                hold.marked = false;
            }
            released(holding.client, out)
        }

        State::Preparing { step, asked, timing } => {
            let detail = detail::preparation_detail(holding, step, &done, timing, env.now);
            store_succeeded = done == Done::Succeeded;
            let next = prepared(holding, &mut domain.cache, step, done, &env.limits);
            match step {
                PrepareStep::Store { repository, .. } => {
                    waiters = domain.cache.finished(borrowed(holding, repository).store, id, store_succeeded);
                }
                PrepareStep::Unkeep
                | PrepareStep::Mark
                | PrepareStep::Marked
                | PrepareStep::CompareKept { .. }
                | PrepareStep::Pack { .. }
                | PrepareStep::Reds { .. }
                | PrepareStep::Candidate { .. }
                | PrepareStep::Look { .. }
                | PrepareStep::Make
                | PrepareStep::Clone { .. }
                | PrepareStep::Fetch { .. }
                | PrepareStep::Default { .. }
                | PrepareStep::Create { .. }
                | PrepareStep::Refetch { .. }
                | PrepareStep::CheckOut { .. }
                | PrepareStep::FetchPush { .. }
                | PrepareStep::PinReplaced { .. }
                | PrepareStep::PointLanding { .. }
                | PrepareStep::FetchLanding { .. }
                | PrepareStep::Counts { .. }
                | PrepareStep::FetchBase { .. }
                | PrepareStep::PointBase { .. }
                | PrepareStep::CheckOutMerge { .. }
                | PrepareStep::CompareBase { .. }
                | PrepareStep::CompareHead { .. }
                | PrepareStep::Merge { .. }
                | PrepareStep::CommitHead { .. }
                | PrepareStep::CheckOutHead { .. }
                | PrepareStep::CommitMerge { .. }
                | PrepareStep::CheckOutMerged { .. } => {}
            }
            prepare_next(holding, &mut domain.cache, id, next, detail, asked, env, out)
        }
        State::Reds(running) => {
            if hold.cut {
                reds::stopped(holding, running, out)
            } else {
                reds::done(holding, id, running, done, env, out)
            }
        }
        State::Pushing { mut push, step, asked } if hold.cut => {
            // Only a completed push proves a landing without further IO.
            if let PushStep::Push { repository }
            | PushStep::Design { delivery: design::Delivery { repository, .. }, step: design::Step::Push } = step
            {
                let landing = match pushed_to(holding, &push.to, repository, done) {
                    Ok(landing) => landing,
                    Err(failure) => failed_landing(failure),
                };
                push.landings.push(landing).expect("one landing per repository");
            }
            end_push(holding, push, asked, out)
        }
        State::Pushing { push, step, asked } => pushed(holding, id, push, step, asked, done, env, out),
        State::Queued { .. } | State::Waiting { .. } | State::Ready | State::Unprepared | State::Closed => {
            unreachable!("an operation ends only while its hold prepares, pushes or saves")
        }
    };
    for request in out.iter().skip(usize::try_from(mark).expect("queue index")) {
        if let Request::Saved { outcome, .. } = request {
            hold.holding.saved = saved_repositories(&hold.holding, outcome);
        }
    }
    if hold.holding.recovering && (matches_ready(&hold.state) || matches_closed(&hold.state)) {
        let mut retained = Queue::uncapped();
        let length = out.len();
        for index in 0..length {
            let request = out.pop().expect("produced request");
            if index >= mark
                && let Request::Saved { client, .. } = &request
                && *client == hold.holding.client
            {
                continue;
            }
            retained.push(request);
        }
        *out = retained;
        let next = recovered(&mut hold.holding);
        hold.state =
            prepare_next(&mut hold.holding, &mut domain.cache, id, next, Box::new([]), recovery_asked, env, out);
    }
    rewrite_mark(hold, previous, id, env, out);
    conclude(domain, env, id, was_active, out, mark);
    for _ in 0..waiters.len() {
        let waiter = waiters.pop().expect("the waiters to resume");
        let mark = out.len();
        let hold = domain.holds.get_mut(waiter).expect("a queued waiter lives until it is resumed or removed");
        hold.state = match hold.state {
            State::Waiting { repository, look } => {
                let from = if store_succeeded { after(repository) } else { repository };
                stores_step(&hold.holding, &mut domain.cache, waiter, from, look, env, out)
            }
            State::Queued { .. }
            | State::Preparing { .. }
            | State::Ready
            | State::Reds(_)
            | State::Unprepared
            | State::Pushing { .. }
            | State::Keeping { .. }
            | State::Closed => {
                unreachable!("only waiting holds are queued");
            }
        };
        conclude(domain, env, waiter, true, out, mark);
    }
    admissions(domain, env, out);
}

/// The hold a client's handle names, or `None` if it has been released: the
/// handle travelled down while the `Released` travelled up, and is dropped
/// (5.2).
fn addressed(holds: &Slab<Hold>, hold: Token) -> Option<Id<Hold>> {
    let id = Id::from_token(hold);
    match &holds.get(id)?.state {
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Pushing { .. } => Some(id),
        State::Keeping { .. } | State::Closed => None,
    }
}

/// Applied after every transition: what it tells, and what the new state
/// implies. `mark` is where the requests the transition made begin in `out`.
fn conclude(
    domain: &mut Domain,
    env: &Env<Limits>,
    id: Id<Hold>,
    was_active: bool,
    out: &mut Queue<Request>,
    mark: u32,
) {
    let hold = domain.holds.get_mut(id).expect("a hold lives until it is retired");
    if matches_closed(&hold.state) && hold.marked {
        hold.marked = false;
        io(id, Op::Keep { workspace: hold.holding.disk, mark: None }, env, out);
    }
    schedule(hold, id, env, out, mark);
    sent(&mut domain.facts, &mut domain.cache, &hold.holding, out, mark);
    if was_active && !(active(&hold.state) || hold.holding.recovering) {
        domain.active = domain.active.checked_sub(1).expect("a settled prepare returns its permit once");
    }
    follow(&mut domain.holds, &mut domain.cache, id);
}

/// A store waiter keeps its permit even with no operation in flight.
fn active(state: &State) -> bool {
    match state {
        State::Preparing { .. } | State::Waiting { .. } => true,
        State::Queued { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Pushing { .. }
        | State::Keeping { .. }
        | State::Closed => false,
    }
}

fn io(id: Id<Hold>, op: Op, env: &Env<Limits>, out: &mut Queue<Request>) {
    let timeout = operation_timeout(&op, &env.limits);
    out.push(Request::Io { owner: id.token(), op, deadline: env.now.saturating_add(timeout) });
}

/// Account for the operations this transition asked for, before another hold
/// can advance, and tell the facts its requests carry.
fn sent(facts: &mut Facts, cache: &mut Cache, holding: &Holding, out: &Queue<Request>, mark: u32) {
    let client = holding.client;
    let made = usize::try_from(mark).expect("a u32 fits in a usize");
    for request in out.iter().skip(made) {
        let fact = match request {
            Request::DependencyLanded { .. }
            | Request::Held { .. }
            | Request::Results { .. }
            | Request::ResultsEnd { .. } => continue,
            Request::Prepared { client, prepared } => {
                Fact::Prepared { client: *client, prepared: preparation(prepared) }
            }
            Request::Pushed { client, outcome } => pushed_fact(*client, Target::Push, outcome),
            Request::Saved { client, outcome } => pushed_fact(*client, Target::Saved, outcome),
            Request::Released { client } => Fact::Released { client: *client },
            Request::Io { owner: _, op, deadline: _ } => {
                cache.asked(holding.workspace, &holding.stores, op);
                Fact::Started { client, op: op.kind() }
            }
            Request::Cancel { owner: _ } => Fact::Aborting { client },
        };
        facts.push(fact);
    }
}

fn pushed_fact(client: Token, to: Target, outcome: &Outcome) -> Fact {
    match outcome {
        Outcome::Pushed { landings } => Fact::Pushed { client, to, tally: tally(landings) },
        Outcome::Refused { refusal } => Fact::Refused { client, refusal: *refusal },
    }
}

fn ending(done: &Done) -> Ending {
    match done {
        Done::ParentUnreachable { .. } => Ending::Failed { fault: Fault::Refused },
        Done::Paths { files } => Ending::Paths { files: count(files.len()) },
        Done::Looked { holds } => Ending::Looked { holds: *holds },
        Done::IsAncestor { contains } => Ending::IsAncestor { contains: *contains },
        Done::Brings { brings } => Ending::Brings { brings: *brings },
        Done::Results { .. } | Done::Succeeded => Ending::Succeeded,
        Done::Fetched { commit } => Ending::Fetched { commit: *commit },
        Done::Committed { commit } => Ending::Committed { commit: *commit },
        Done::Unchanged => Ending::Unchanged,
        Done::Exists => Ending::Exists,
        Done::Rejected => Ending::Rejected,
        Done::Merged => Ending::Merged,
        Done::Conflicted { files, .. } => Ending::Conflicted { files: count(files.len()) },
        Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Ending::Failed { fault: *fault },
    }
}

fn preparation(prepared: &Prepared) -> Preparation {
    match prepared {
        Prepared::Candidate { workspace, .. } => Preparation::Ready { workspace: *workspace, conflicts: 0 },
        Prepared::Ready { workspace, conflicts } | Prepared::Recovered { workspace, conflicts, .. } => {
            let mut paths = 0_u64;
            for conflict in conflicts {
                paths = paths
                    .checked_add(u64::from(count(conflict.files.len())))
                    .expect("bounded repository/path product fits u64");
            }
            Preparation::Ready { workspace: *workspace, conflicts: paths }
        }
        Prepared::Refused { refusal } => Preparation::Refused { refusal: *refusal },
        Prepared::Failed { failure, .. } => Preparation::Failed { failure: *failure },
        Prepared::Aborted => Preparation::Aborted,
    }
}

fn paths_fit(files: &[Box<[u8]>], limits: &Limits) -> bool {
    let Ok(length) = u32::try_from(files.len()) else {
        return false;
    };
    if length == 0 || length > limits.conflicts {
        return false;
    }
    for (position, file) in files.iter().enumerate() {
        let Ok(length) = u32::try_from(file.len()) else {
            return false;
        };
        if length == 0 || length > limits.path_bytes {
            return false;
        }
        for earlier in files.iter().take(position) {
            if earlier == file {
                return false;
            }
        }
    }
    true
}

fn bounded_done(done: Done, limits: &Limits) -> Done {
    match done {
        Done::Paths { files } => {
            if files.is_empty() || paths_fit(&files, limits) {
                Done::Paths { files }
            } else {
                Done::Failed { fault: Fault::Broken }
            }
        }
        Done::Conflicted { files, cut } => {
            if paths_fit(&files, limits) {
                Done::Conflicted { files, cut }
            } else {
                Done::Failed { fault: Fault::Broken }
            }
        }
        other @ (Done::Results { .. }
        | Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::ParentUnreachable { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Failed { .. }
        | Done::FailedWithOutput { .. }) => other,
    }
}

/// What a hold's state implies, applied after every transition: a closed
/// hold is retired, and its workspace is idle again.
fn follow(holds: &mut Slab<Hold>, cache: &mut Cache, id: Id<Hold>) {
    let hold = holds.get(id).expect("a hold lives until it is retired");
    match hold.state {
        State::Closed => {
            if hold.operation.is_none() && hold.pending.is_empty() {
                cache.release(hold.holding.workspace);
                holds.retire(id);
            }
        }
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Keeping { .. }
        | State::Pushing { .. } => {}
    }
}

// Cell handlers: each takes the source state's data by value and returns the
// target state.

/// Preparing, done: where the prepare goes from the operation `step` that
/// ended so.
#[expect(
    clippy::too_many_lines,
    reason = "the preparation transition table handles every terminal workspace outcome together"
)]
fn prepared(holding: &mut Holding, cache: &mut Cache, step: PrepareStep, done: Done, limits: &Limits) -> Next {
    if step == PrepareStep::Mark {
        return Next::Step(PrepareStep::Marked);
    }
    if let PrepareStep::Reds { repository } = step {
        return reds_prepare(holding, after(repository));
    }
    if let PrepareStep::Candidate { repository, step } = step {
        return candidate::after(holding, repository, step, done);
    }
    match &done {
        Done::Fetched { commit } => remember_fetch(holding, step, *commit),
        Done::Succeeded
        | Done::Failed { .. }
        | Done::FailedWithOutput { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::ParentUnreachable { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Conflicted { .. } => {}
    }
    let last = count(holding.repositories.len()).checked_sub(1).expect("a spec names a repository");
    match step {
        PrepareStep::Unkeep => {
            if done != Done::Succeeded {
                return Next::Failed(Failure::Transient);
            }
            if matches_kept(holding) {
                if holding.kept.as_ref().expect("candidate").state == crate::git::KeptState::Held {
                    restore_tips(holding);
                    Next::Recover
                } else {
                    Next::Step(PrepareStep::CompareKept { repository: 0 })
                }
            } else {
                holding.kept = None;
                Next::Stores { from: 0, look: false }
            }
        }
        PrepareStep::CompareKept { repository } => {
            let commit = match done {
                Done::Fetched { commit } => commit,
                Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => {
                    if holding.recovered {
                        restore_assignment(holding);
                        holding.kept = None;
                        return Next::Stores { from: 0, look: false };
                    }
                    return Next::Failed(failure(repository, fault));
                }
                Done::Succeeded
                | Done::Committed { .. }
                | Done::Unchanged
                | Done::Exists
                | Done::Rejected
                | Done::Results { .. }
                | Done::Paths { .. }
                | Done::ParentUnreachable { .. }
                | Done::Looked { .. }
                | Done::Brings { .. }
                | Done::IsAncestor { .. }
                | Done::Merged
                | Done::Conflicted { .. } => {
                    unreachable!("a kept comparison has fetch terminals")
                }
            };
            let wanted = holding
                .kept
                .as_ref()
                .expect("kept candidate")
                .repositories
                .get(usize::try_from(repository).expect("repository index"))
                .expect("matching shape")
                .commit;
            if commit != wanted {
                restore_assignment(holding);
                holding.kept = None;
                Next::Stores { from: 0, look: false }
            } else if repository < last {
                Next::Step(PrepareStep::CompareKept { repository: after(repository) })
            } else {
                holding.preserve = true;
                holding.original_starts = Box::new([]);
                holding.kept = None;
                let commit = fetched(holding, 0, &start_want(&nth(holding, 0).start)).expect("compared every start");
                fetched_start(holding, 0, commit)
            }
        }

        PrepareStep::Mark | PrepareStep::Marked => unreachable!("mark completes outside repository preparation"),
        PrepareStep::Pack { repository, .. } => Next::Ready { pack_from: after(repository) },
        PrepareStep::Reds { .. } => unreachable!("reds settled above"),
        PrepareStep::Candidate { .. } => unreachable!("candidate preparation was dispatched before the fetch memo"),
        PrepareStep::Store { repository, look } => match done {
            Done::Succeeded => Next::Stores { from: after(repository), look },
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. }
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected => unreachable!("store terminals"),
        },
        PrepareStep::Look { repository } => match done {
            Done::ParentUnreachable { .. } => unreachable!("a look has no snapshot"),
            Done::Results { .. } | Done::Paths { .. } => unreachable!("a look has its own terminals"),
            Done::Looked { holds: true } if repository < last => {
                Next::Step(PrepareStep::Look { repository: after(repository) })
            }
            Done::Looked { holds: true } => {
                cache.cloned(holding.workspace, &holding.repositories, &holding.stores);
                Next::Step(PrepareStep::Fetch { repository: 0 })
            }
            Done::Looked { holds: false } => Next::Step(PrepareStep::Make),
            Done::Failed { fault: _ } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("io ends a look with its own terminals"),
        },
        PrepareStep::Make => match done {
            Done::Succeeded => Next::Step(PrepareStep::Clone { repository: 0 }),
            Done::Failed { fault: _ } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a make with its own terminals")
            }
        },
        PrepareStep::Clone { repository } => match done {
            Done::Succeeded if repository < last => Next::Step(PrepareStep::Clone { repository: after(repository) }),
            Done::Succeeded => {
                cache.cloned(holding.workspace, &holding.repositories, &holding.stores);
                Next::Step(PrepareStep::Fetch { repository: 0 })
            }
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a clone with its own terminals")
            }
        },
        PrepareStep::Fetch { repository } => match done {
            Done::Fetched { commit } => fetched_start(holding, repository, commit),
            Done::Failed { fault: Fault::Missing { missing: Missing::Branch } }
            | Done::FailedWithOutput { fault: Fault::Missing { missing: Missing::Branch }, .. }
                if creates_base(holding, repository) =>
            {
                Next::Step(PrepareStep::Default { repository })
            }
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a fetch with its own terminals")
            }
        },
        PrepareStep::Default { repository } => match done {
            Done::Fetched { commit } => Next::Step(PrepareStep::Create { repository, commit }),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a fetch with its own terminals")
            }
        },
        PrepareStep::Create { repository, commit } => match done {
            Done::Succeeded => fetched_start(holding, repository, commit),
            Done::Exists => Next::Step(PrepareStep::Refetch { repository }),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a creation with its own terminals")
            }
        },
        PrepareStep::Refetch { repository } => match done {
            Done::Fetched { commit } => fetched_start(holding, repository, commit),
            // It was there a moment ago: whoever made it removed it since.
            Done::Failed { fault: Fault::Missing { .. } }
            | Done::FailedWithOutput { fault: Fault::Missing { .. }, .. } => Next::Failed(Failure::Transient),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a fetch with its own terminals")
            }
        },
        PrepareStep::FetchLanding { repository, ours, merging, pushed } => match done {
            Done::Fetched { commit } => Next::Step(PrepareStep::Counts {
                repository,
                ours,
                then: AfterPin::Landing { commit, merging, pushed },
            }),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("a landing fetch has fetch terminals"),
        },
        PrepareStep::PointLanding { repository, ours, commit, merging, pushed } => match done {
            Done::Succeeded => Next::Step(PrepareStep::Counts {
                repository,
                ours,
                then: AfterPin::Landing { commit, merging, pushed },
            }),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("a point has local terminals"),
        },
        PrepareStep::PinReplaced { repository, ours, pushed } => match done {
            Done::Succeeded => {
                Next::Step(PrepareStep::Counts { repository, ours, then: AfterPin::Replaced { pushed } })
            }
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("a replaced-head pin has local terminals"),
        },
        PrepareStep::Counts { repository, ours, then } => match done {
            Done::Fetched { .. } | Done::Succeeded => match then {
                AfterPin::Landing { commit, merging, pushed } => {
                    after_pin(holding, repository, ours, commit, merging, pushed)
                }
                AfterPin::Replaced { pushed } => after_replaced_pin(holding, repository, ours, pushed),
            },
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("counts has fetch and absence terminals"),
        },
        PrepareStep::FetchPush { repository, ours } => match done {
            Done::Fetched { commit } => fetched_push(holding, repository, ours, Some(commit)),
            Done::Failed { fault: Fault::Missing { missing: Missing::Branch } }
            | Done::FailedWithOutput { fault: Fault::Missing { missing: Missing::Branch }, .. }
                if !nth(holding, repository).start.is_saved() || nth(holding, repository).expected.is_none() =>
            {
                fetched_push(holding, repository, ours, None)
            }
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("delivery branch fetch has fetch terminals"),
        },
        PrepareStep::CheckOut { repository, commit, landing, pushed } => match done {
            Done::Succeeded => {
                let tips = Tips {
                    start: commit,
                    head: commit,
                    pushed,
                    delivered: false,
                    redeliver: delivered_start(holding, repository, commit, pushed),
                    unchecked: false,
                    merging: None,
                    design_merged: None,
                    caught_up: None,
                    landing,
                    expected: if nth(holding, repository).start.is_saved() {
                        nth(holding, repository).expected.or(pushed)
                    } else {
                        nth(holding, repository).expected
                    },
                };
                holding.tips.push(tips).expect("room for each repository's tips");
                if repository < last {
                    Next::Step(PrepareStep::Fetch { repository: after(repository) })
                } else {
                    reds_prepare(holding, 0)
                }
            }
            Done::Failed { fault: _ } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a checkout with its own terminals")
            }
        },
        merge @ (PrepareStep::FetchBase { .. }
        | PrepareStep::PointBase { .. }
        | PrepareStep::CheckOutMerge { .. }
        | PrepareStep::Merge { .. }
        | PrepareStep::CompareBase { .. }
        | PrepareStep::CompareHead { .. }
        | PrepareStep::CommitHead { .. }
        | PrepareStep::CheckOutHead { .. }
        | PrepareStep::CommitMerge { .. }
        | PrepareStep::CheckOutMerged { .. }) => prepared_merge(holding, merge, done, limits),
    }
}

#[expect(clippy::too_many_lines, reason = "the merge preparation table lists every step and terminal")]
fn prepared_merge(holding: &mut Holding, step: PrepareStep, done: Done, limits: &Limits) -> Next {
    let last = count(holding.repositories.len()).checked_sub(1).expect("a spec names a repository");
    match step {
        PrepareStep::FetchBase { repository, ours, base, head } => match done {
            Done::Fetched { commit } => {
                assert_eq!(commit, base, "fetching an exact commit returns that commit");
                fetched_merge_base(holding, repository, ours, base, head)
            }
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("io ends a fetch with its own terminals"),
        },
        PrepareStep::PointBase { repository, ours, base, head } => match done {
            Done::Succeeded => fetched_merge_base(holding, repository, ours, base, head),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("a point has local terminals"),
        },
        PrepareStep::CheckOutMerge { repository, ours, base } => match done {
            Done::Succeeded => checked_out_merge(holding, repository, ours, base),
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("io ends a checkout with its own terminals"),
        },
        PrepareStep::Merge { repository, theirs, head } => match done {
            Done::Merged => match head {
                Some(head) => Next::Step(PrepareStep::CommitHead { repository, head }),
                None if nth(holding, repository).landing.is_some() => {
                    Next::Step(PrepareStep::CommitMerge { repository, theirs })
                }
                None => after_merged(holding, repository, last),
            },
            Done::Conflicted { files, .. } => {
                if head.is_some() {
                    return Next::Failed(Failure::Conflicted { repository, head: theirs });
                }
                assert!(paths_fit(&files, limits), "terminal admission bounds retained paths");
                holding.conflicts.push(Conflicts { repository, files }).expect("room for each repository's conflicts");
                after_merged(holding, repository, last)
            }
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Rejected => unreachable!("io ends a merge with its own terminals"),
        },
        PrepareStep::CompareBase { repository, ours, base } => match done {
            Done::IsAncestor { contains: true } => resume_contained(holding, repository, ours),
            Done::IsAncestor { contains: false } => {
                if pin_branch(nth(holding, repository)).is_some() {
                    fetch_landing(holding, repository, ours, Some(base), None)
                } else {
                    begin_merge(holding, repository, ours, Some(base))
                }
            }
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::Conflicted { .. } => unreachable!("ancestry query has its own terminals"),
        },
        PrepareStep::CompareHead { repository, head } => match done {
            Done::IsAncestor { contains: true } => next_head(holding, repository, head, last),
            Done::IsAncestor { contains: false } => Next::Step(PrepareStep::Merge {
                repository,
                theirs: merged_head(holding, repository, head),
                head: Some(head),
            }),
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Merged
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::Conflicted { .. } => unreachable!("ancestry query has its own terminals"),
        },
        PrepareStep::CommitHead { repository, head } => match done {
            Done::Committed { commit } => {
                tips_mut(holding, repository).head = commit;
                Next::Step(PrepareStep::CheckOutHead { repository, head })
            }
            Done::Conflicted { .. } => {
                Next::Failed(Failure::Conflicted { repository, head: merged_head(holding, repository, head) })
            }
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged => unreachable!("explicit merge commit has its own terminals"),
        },
        PrepareStep::CommitMerge { repository, .. } => match done {
            Done::Committed { commit } => {
                let tips = tips_mut(holding, repository);
                tips.head = commit;
                tips.merging = None;
                Next::Step(PrepareStep::CheckOutMerged { repository })
            }
            Done::Conflicted { .. } | Done::Failed { .. } | Done::FailedWithOutput { .. } => {
                Next::Failed(Failure::Transient)
            }
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged => unreachable!("explicit merge commit has its own terminals"),
        },
        PrepareStep::CheckOutMerged { repository } => match done {
            Done::Succeeded => after_merged(holding, repository, last),
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("io ends a checkout with its own terminals"),
        },
        PrepareStep::CheckOutHead { repository, head } => match done {
            Done::Succeeded => next_head(holding, repository, head, last),
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Results { .. }
            | Done::Paths { .. }
            | Done::ParentUnreachable { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("io ends a checkout with its own terminals"),
        },
        PrepareStep::Unkeep
        | PrepareStep::Mark
        | PrepareStep::Marked
        | PrepareStep::CompareKept { .. }
        | PrepareStep::Pack { .. }
        | PrepareStep::Reds { .. }
        | PrepareStep::Candidate { .. }
        | PrepareStep::Store { .. }
        | PrepareStep::Look { .. }
        | PrepareStep::Make
        | PrepareStep::Clone { .. }
        | PrepareStep::Fetch { .. }
        | PrepareStep::Default { .. }
        | PrepareStep::Create { .. }
        | PrepareStep::Refetch { .. }
        | PrepareStep::PointLanding { .. }
        | PrepareStep::FetchLanding { .. }
        | PrepareStep::Counts { .. }
        | PrepareStep::PinReplaced { .. }
        | PrepareStep::CheckOut { .. }
        | PrepareStep::FetchPush { .. } => unreachable!("merge preparation has only its own steps"),
    }
}

fn fetched_merge_base(holding: &Holding, repository: u32, ours: Commit, base: Commit, head: Option<u32>) -> Next {
    if let Some(head) = head {
        let next = after(head);
        if next < count(merged_heads(holding, repository).len()) {
            return fetch_base(holding, repository, ours, merged_head(holding, repository, next), Some(next));
        }
    }
    Next::Step(PrepareStep::CheckOutMerge { repository, ours, base })
}

fn checked_out_merge(holding: &mut Holding, repository: u32, ours: Commit, base: Commit) -> Next {
    let merging = match nth(holding, repository).start {
        Start::Merged { .. } => None,
        Start::Merge { .. } => {
            if nth(holding, repository).landing.is_some() {
                return Next::Step(PrepareStep::CompareBase { repository, ours, base });
            }
            Some(base)
        }
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Prepare { .. }
        | Start::Commit { .. }
        | Start::Saved { .. } => {
            unreachable!("only merge starts check out here")
        }
    };
    begin_merge(holding, repository, ours, merging)
}

fn start_want(start: &Start) -> Want {
    match start {
        Start::Base { branch }
        | Start::Branch { branch }
        | Start::Saved { branch }
        | Start::Merge { branch, .. }
        | Start::Merged { branch, .. } => Want::Branch { branch: copy_of(branch) },
        Start::Commit { commit } => Want::Commit { commit: *commit },
        Start::Prepare { request } => Want::Commit { commit: request.from },
    }
}

fn remember_fetch(holding: &mut Holding, step: PrepareStep, commit: Commit) {
    let (repository, want) = match step {
        PrepareStep::CompareKept { repository }
        | PrepareStep::Fetch { repository }
        | PrepareStep::Refetch { repository } => (repository, start_want(&nth(holding, repository).start)),
        PrepareStep::Default { repository } => (repository, Want::Default),
        PrepareStep::FetchPush { repository, .. } => (
            repository,
            Want::Branch { branch: copy_of(nth(holding, repository).push.as_deref().expect("writable item")) },
        ),
        PrepareStep::FetchLanding { repository, .. } => (repository, pin_want(nth(holding, repository))),
        PrepareStep::FetchBase { repository, base, .. } => (repository, Want::Commit { commit: base }),
        PrepareStep::Pack { .. } | PrepareStep::Counts { .. } => return,
        PrepareStep::Unkeep
        | PrepareStep::Mark
        | PrepareStep::Marked
        | PrepareStep::Reds { .. }
        | PrepareStep::Candidate { .. }
        | PrepareStep::Store { .. }
        | PrepareStep::Look { .. }
        | PrepareStep::Make
        | PrepareStep::Clone { .. }
        | PrepareStep::Create { .. }
        | PrepareStep::CheckOut { .. }
        | PrepareStep::PinReplaced { .. }
        | PrepareStep::PointLanding { .. }
        | PrepareStep::PointBase { .. }
        | PrepareStep::CheckOutMerge { .. }
        | PrepareStep::CompareBase { .. }
        | PrepareStep::CompareHead { .. }
        | PrepareStep::Merge { .. }
        | PrepareStep::CommitHead { .. }
        | PrepareStep::CheckOutHead { .. }
        | PrepareStep::CommitMerge { .. }
        | PrepareStep::CheckOutMerged { .. } => unreachable!("only a fetch returns Fetched"),
    };
    holding.fetched.push(Fetched { repository, want, commit }).expect("prepare observations grow");
}

fn fetched(holding: &Holding, repository: u32, want: &Want) -> Option<Commit> {
    for seen in holding.fetched.as_slice() {
        if seen.repository == repository && (seen.want == *want || *want == Want::Commit { commit: seen.commit }) {
            return Some(seen.commit);
        }
    }
    None
}

fn fetch_push(holding: &Holding, repository: u32, ours: Commit) -> Next {
    let want = Want::Branch { branch: copy_of(nth(holding, repository).push.as_deref().expect("writable item")) };
    match fetched(holding, repository, &want) {
        Some(commit) => fetched_push(holding, repository, ours, Some(commit)),
        None => Next::Step(PrepareStep::FetchPush { repository, ours }),
    }
}

fn pin_want(repository: &Repository) -> Want {
    match repository.pin {
        Some(commit) => Want::Commit { commit },
        None => Want::Branch { branch: copy_of(pin_branch(repository).expect("writable landing")) },
    }
}

fn fetch_landing(
    holding: &Holding,
    repository: u32,
    ours: Commit,
    merging: Option<Commit>,
    pushed: Option<Commit>,
) -> Next {
    let want = pin_want(nth(holding, repository));
    match fetched(holding, repository, &want) {
        Some(commit) => Next::Step(PrepareStep::PointLanding { repository, ours, commit, merging, pushed }),
        None => Next::Step(PrepareStep::FetchLanding { repository, ours, merging, pushed }),
    }
}

fn after_pin(
    holding: &mut Holding,
    repository: u32,
    ours: Commit,
    commit: Commit,
    merging: Option<Commit>,
    pushed: Option<Commit>,
) -> Next {
    if nth(holding, repository).pin.is_some() {
        after_replaced_pin(holding, repository, ours, pushed)
    } else {
        match merging {
            Some(base) => begin_merge(holding, repository, ours, Some(base)),
            None => after_start(holding, repository, ours, Some(commit)),
        }
    }
}

fn fetch_base(holding: &Holding, repository: u32, ours: Commit, base: Commit, head: Option<u32>) -> Next {
    match fetched(holding, repository, &Want::Commit { commit: base }) {
        Some(_) => Next::Step(PrepareStep::PointBase { repository, ours, base, head }),
        None => Next::Step(PrepareStep::FetchBase { repository, ours, base, head }),
    }
}

/// Pin writable items after their start; a producer merge keeps its contained-resume path.
fn fetched_start(holding: &Holding, repository: u32, ours: Commit) -> Next {
    let spec = nth(holding, repository);
    if let Start::Prepare { .. } = spec.start {
        return candidate::first(repository);
    }
    let merge = match spec.start {
        Start::Merge { .. } => true,
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Prepare { .. }
        | Start::Commit { .. }
        | Start::Saved { .. }
        | Start::Merged { .. } => false,
    };
    if spec.push.is_some() && spec.landing.is_none() {
        fetch_push(holding, repository, ours)
    } else if !merge && pin_branch(spec).is_some() {
        fetch_landing(holding, repository, ours, None, None)
    } else {
        after_start(holding, repository, ours, None)
    }
}

fn after_start(holding: &Holding, repository: u32, ours: Commit, pin: Option<Commit>) -> Next {
    let spec = nth(holding, repository);
    match &spec.start {
        Start::Merge { base, .. } => fetch_base(holding, repository, ours, *base, None),
        Start::Merged { heads, .. } => {
            fetch_base(holding, repository, ours, *heads.first().expect("admitted heads are nonempty"), Some(0))
        }
        Start::Saved { .. } if spec.push.is_some() => fetch_push(holding, repository, ours),
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Prepare { .. }
        | Start::Commit { .. }
        | Start::Saved { .. } => {
            let landing = if landing_branch(spec).is_some() { pin } else { None };
            Next::Step(PrepareStep::CheckOut { repository, commit: ours, landing, pushed: Some(ours) })
        }
    }
}

fn fetched_push(holding: &Holding, repository: u32, ours: Commit, pushed: Option<Commit>) -> Next {
    if nth(holding, repository).pin.is_some() {
        fetch_landing(holding, repository, ours, None, pushed)
    } else if nth(holding, repository).landing.is_none() {
        Next::Step(PrepareStep::PinReplaced { repository, ours, pushed })
    } else {
        after_replaced_pin(holding, repository, ours, pushed)
    }
}

fn after_replaced_pin(holding: &Holding, repository: u32, ours: Commit, pushed: Option<Commit>) -> Next {
    match &nth(holding, repository).start {
        Start::Merge { base, .. } => fetch_base(holding, repository, ours, *base, None),
        Start::Merged { heads, .. } => {
            fetch_base(holding, repository, ours, *heads.first().expect("admitted heads are nonempty"), Some(0))
        }
        Start::Saved { .. } => Next::Step(PrepareStep::CheckOut { repository, commit: ours, landing: None, pushed }),
        Start::Base { .. } | Start::Branch { .. } | Start::Prepare { .. } | Start::Commit { .. } => {
            Next::Step(PrepareStep::CheckOut { repository, commit: ours, landing: None, pushed: Some(ours) })
        }
    }
}

/// A supplied landing never grants write authority (temper/docs/design/domain/worker/checkouts.md).
fn pin_branch(repository: &Repository) -> Option<&[u8]> {
    if repository.push.is_some() { repository.landing.as_deref() } else { None }
}

fn resume_contained(holding: &mut Holding, repository: u32, ours: Commit) -> Next {
    let index = usize::try_from(repository).expect("a u32 fits in a usize");
    let spec = holding.repositories.get_mut(index).expect("a repository of the spec");
    let Start::Merge { branch, .. } = &spec.start else {
        unreachable!("only a producer's merge start compares its base")
    };
    if spec.push.as_deref() == Some(&**branch) {
        spec.start = Start::Branch { branch: copy_of(branch) };
        fetched_start(holding, repository, ours)
    } else {
        spec.start = Start::Saved { branch: copy_of(branch) };
        fetched_start(holding, repository, ours)
    }
}

/// Only a fetched push branch at the checkout start can corroborate the task's delivery.
fn delivered_start(holding: &Holding, repository: u32, commit: Commit, pushed: Option<Commit>) -> bool {
    let spec = nth(holding, repository);
    if spec.delivered != Some(commit) || pushed != Some(commit) {
        return false;
    }
    match &spec.push {
        Some(branch) => fetched(holding, repository, &Want::Branch { branch: copy_of(branch) }) == Some(commit),
        None => false,
    }
}

fn begin_merge(holding: &mut Holding, repository: u32, ours: Commit, merging: Option<Commit>) -> Next {
    let tips = Tips {
        start: ours,
        head: ours,
        pushed: Some(ours),
        delivered: false,
        redeliver: false,
        unchecked: false,
        merging,
        design_merged: None,
        caught_up: None,
        landing: None,
        expected: nth(holding, repository).expected,
    };
    holding.tips.push(tips).expect("room for each repository's tips");
    match merging {
        None => Next::Step(PrepareStep::CompareHead { repository, head: 0 }),
        Some(theirs) => Next::Step(PrepareStep::Merge { repository, theirs, head: None }),
    }
}

fn merged_heads(holding: &Holding, repository: u32) -> &[Commit] {
    let Start::Merged { heads, .. } = &nth(holding, repository).start else {
        unreachable!("only a merged start loops over heads")
    };
    heads
}

fn merged_head(holding: &Holding, repository: u32, head: u32) -> Commit {
    *merged_heads(holding, repository).get(usize::try_from(head).expect("bounded index")).expect("admitted head index")
}

fn next_head(holding: &mut Holding, repository: u32, head: u32, last: u32) -> Next {
    let next = after(head);
    if next < count(merged_heads(holding, repository).len()) {
        Next::Step(PrepareStep::CompareHead { repository, head: next })
    } else {
        let tips = tips_mut(holding, repository);
        tips.start = tips.head;
        after_merged(holding, repository, last)
    }
}

/// Full object spelling for the merge message (protocol/channel.md, 4).
fn merge_title(head: Commit) -> Box<[u8]> {
    let length = hex_length(head).checked_add(6).expect("fixed prefix");
    let mut title = Writer::new(length);
    title.put(b"Merge ").expect("measured prefix");
    put_hex(&mut title, head);
    title.finish()
}

fn hex_length(head: Commit) -> usize {
    if head.raw().get(20..).expect("fixed id") == [0; 12] { 40 } else { 64 }
}

fn put_hex(text: &mut Writer, head: Commit) {
    let size = hex_length(head).checked_div(2).expect("two digits per byte");
    for byte in head.raw().get(..size).expect("fixed id") {
        for digit in [byte >> 4_u32, byte & 15] {
            text.put(&[*b"0123456789abcdef".get(usize::from(digit)).expect("hex nibble")]).expect("measured hex");
        }
    }
}

fn after_merged(holding: &Holding, repository: u32, last: u32) -> Next {
    if repository < last {
        Next::Step(PrepareStep::Fetch { repository: after(repository) })
    } else {
        reds_prepare(holding, 0)
    }
}

/// Preparing, done: the next operation, or the prepare's end, which an abort
/// or a release turns into aborted.
#[expect(
    clippy::too_many_arguments,
    reason = "the next store is claimed only after the caller's abort or release is checked"
)]
fn prepare_next(
    holding: &mut Holding,
    cache: &mut Cache,
    id: Id<Hold>,
    mut next: Next,
    detail: Box<[u8]>,
    asked: Asked,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if holding.preserve && asked == Asked::Nothing {
        // At most one fetched-start and one retained-tree transition per repository.
        for _ in 0..holding.repositories.len().saturating_mul(2).saturating_add(1) {
            next = match next {
                Next::Step(step @ PrepareStep::CheckOut { .. }) => {
                    prepared(holding, cache, step, Done::Succeeded, &env.limits)
                }
                Next::Step(PrepareStep::Fetch { repository }) => {
                    let commit = fetched(holding, repository, &start_want(&nth(holding, repository).start))
                        .expect("compared every start");
                    fetched_start(holding, repository, commit)
                }
                other
                @ (Next::Step(_) | Next::Recover | Next::Stores { .. } | Next::Ready { .. } | Next::Failed(_)) => {
                    next = other;
                    break;
                }
            };
        }
    }
    let client = holding.client;
    match asked {
        Asked::Nothing => match next {
            Next::Step(PrepareStep::Marked) => ready(holding, out),
            Next::Step(step) => prepare_step(holding, cache, id, step, env, out),
            Next::Recover => {
                holding.recovering = true;
                begin(holding, id, To::Saved, Message { title: copy_of(crate::SAVED), body: Box::new([]) }, env, out)
            }
            Next::Stores { from, look } => stores_step(holding, cache, id, from, look, env, out),
            Next::Ready { pack_from } => {
                if let Some(step) = next_pack(holding, cache, pack_from) {
                    return prepare_step(holding, cache, id, step, env, out);
                }
                holding.fetched = List::uncapped();
                if can_mark(holding) {
                    prepare_step(holding, cache, id, PrepareStep::Mark, env, out)
                } else {
                    ready(holding, out)
                }
            }
            Next::Failed(failure) => {
                out.push(Request::Prepared { client, prepared: Prepared::Failed { failure, detail } });
                State::Unprepared
            }
        },
        Asked::Abort => {
            out.push(Request::Prepared { client, prepared: Prepared::Aborted });
            State::Unprepared
        }
        Asked::Release => {
            out.push(Request::Prepared { client, prepared: Prepared::Aborted });
            released(client, out)
        }
    }
}

/// Check each repository when it is reached: a previous Pack may already
/// have reset its store, or another hold may since have asked a fetch.
fn next_pack(holding: &Holding, cache: &Cache, from: u32) -> Option<PrepareStep> {
    for repository in from..count(holding.repositories.len()) {
        if cache.store_due(borrowed(holding, repository).store)
            || cache.repository_due(holding.workspace, &nth(holding, repository).name)
        {
            return Some(PrepareStep::Pack { repository });
        }
    }
    None
}

/// Asks io for the prepare's operation `step`.
#[expect(clippy::too_many_lines, reason = "each preparation step builds its operation in one table")]
fn prepare_step(
    holding: &Holding,
    cache: &Cache,
    id: Id<Hold>,
    step: PrepareStep,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let workspace = holding.disk;
    let op = match step {
        PrepareStep::Unkeep => Op::Keep { workspace, mark: None },
        PrepareStep::Mark => Op::Keep { workspace, mark: held_mark(holding, env) },
        PrepareStep::Marked => unreachable!("completed mark has no operation"),
        PrepareStep::Pack { repository } => {
            let borrowed = borrowed(holding, repository);
            let store = if cache.store_due(borrowed.store) { Some(borrowed.token) } else { None };
            Op::Pack { at: place(workspace, nth(holding, repository)), store }
        }
        PrepareStep::Reds { repository } => Op::Reds {
            at: place(workspace, nth(holding, repository)),
            action: crate::git::Reds::Prepare {
                wait_ms: env.limits.reds_read_ms.checked_add(1_000).expect("bounded wait"),
                rerun_processes: holding.rerun_bound.min(env.limits.rerun_processes),
            },
        },
        PrepareStep::Candidate { repository, step } => candidate::op(holding, repository, step),
        PrepareStep::Look { repository } => {
            let store = borrowed(holding, repository).token;
            let repository = nth(holding, repository);
            Op::Look { at: place(workspace, repository), remote: copy_of(&repository.remote), store }
        }
        PrepareStep::Store { repository, .. } => {
            let store = borrowed(holding, repository);
            let spec = nth(holding, repository);
            Op::Store {
                store: store.token,
                aside: store.aside,
                remote: spec.remote.clone(),
                identity: spec.identity,
                account: spec.account,
                git: spec.git.clone(),
            }
        }
        PrepareStep::Make => {
            let mut stores = List::with_capacity(count(holding.stores.len()));
            for borrowed in &holding.stores {
                stores.push(borrowed.token).expect("spec room");
            }
            Op::Make { workspace, stores: stores.into_boxed() }
        }
        PrepareStep::Clone { repository } => {
            let store = borrowed(holding, repository).token;
            let repository = nth(holding, repository);
            let remote = copy_of(&repository.remote);
            Op::Clone { at: place(workspace, repository), remote, store }
        }
        PrepareStep::CompareKept { repository }
        | PrepareStep::Fetch { repository }
        | PrepareStep::Refetch { repository } => {
            let store = borrowed(holding, repository).token;
            let repository = nth(holding, repository);
            let want = start_want(&repository.start);
            let remote = copy_of(&repository.remote);
            let identity = repository.identity;
            let account = repository.account;
            Op::Fetch { store, into: Into::Fetched, at: place(workspace, repository), remote, want, identity, account }
        }
        PrepareStep::Default { repository } => {
            let store = borrowed(holding, repository).token;
            let repository = nth(holding, repository);
            let remote = copy_of(&repository.remote);
            let identity = repository.identity;
            let account = repository.account;
            Op::Fetch {
                store,
                into: Into::Fetched,
                at: place(workspace, repository),
                remote,
                want: Want::Default,
                identity,
                account,
            }
        }
        PrepareStep::Create { repository, commit } => {
            let repository = nth(holding, repository);
            let branch = match &repository.start {
                Start::Base { branch } => copy_of(branch),
                Start::Branch { .. }
                | Start::Prepare { .. }
                | Start::Commit { .. }
                | Start::Saved { .. }
                | Start::Merge { .. }
                | Start::Merged { .. } => {
                    unreachable!("only a base branch is created")
                }
            };
            let remote = copy_of(&repository.remote);
            let identity = repository.identity;
            let account = repository.account;
            Op::Create { at: place(workspace, repository), remote, branch, commit, identity, account }
        }
        PrepareStep::FetchLanding { repository, .. } => {
            let spec = nth(holding, repository);
            Op::Fetch {
                store: borrowed(holding, repository).token,
                into: Into::Landing,
                at: place(workspace, spec),
                remote: copy_of(&spec.remote),
                want: pin_want(spec),
                identity: spec.identity,
                account: spec.account,
            }
        }
        PrepareStep::PointLanding { repository, commit, .. } => {
            Op::Point { at: place(workspace, nth(holding, repository)), into: Into::Landing, commit }
        }
        PrepareStep::PinReplaced { repository, ours, pushed } => Op::Point {
            at: place(workspace, nth(holding, repository)),
            into: Into::Landing,
            commit: pushed.unwrap_or(ours),
        },
        PrepareStep::Counts { repository, .. } => {
            let spec = nth(holding, repository);
            Op::Counts {
                at: place(workspace, spec),
                store: borrowed(holding, repository).token,
                remote: copy_of(&spec.remote),
                identity: spec.identity,
                account: spec.account,
            }
        }
        PrepareStep::FetchPush { repository, .. } => {
            let spec = nth(holding, repository);
            Op::Fetch {
                store: borrowed(holding, repository).token,
                into: Into::Fetched,
                at: place(workspace, spec),
                remote: copy_of(&spec.remote),
                want: Want::Branch { branch: copy_of(spec.push.as_deref().expect("writable item")) },
                identity: spec.identity,
                account: spec.account,
            }
        }
        PrepareStep::CheckOut { repository, commit, .. } => Op::CheckOut {
            at: place(workspace, nth(holding, repository)),
            commit,
            clean: !nth(holding, repository).start.is_saved(),
            identity: nth(holding, repository).identity,
            account: nth(holding, repository).account,
        },
        PrepareStep::PointBase { repository, base, .. } => {
            Op::Point { at: place(workspace, nth(holding, repository)), into: Into::Fetched, commit: base }
        }
        PrepareStep::FetchBase { repository, base, .. } => {
            let store = borrowed(holding, repository).token;
            let repository = nth(holding, repository);
            Op::Fetch {
                store,
                into: Into::Fetched,
                at: place(workspace, repository),
                remote: copy_of(&repository.remote),
                want: Want::Commit { commit: base },
                identity: repository.identity,
                account: repository.account,
            }
        }
        PrepareStep::CheckOutMerge { repository, ours, .. } => Op::CheckOut {
            at: place(workspace, nth(holding, repository)),
            commit: ours,
            clean: true,
            identity: nth(holding, repository).identity,
            account: nth(holding, repository).account,
        },
        PrepareStep::CompareBase { repository, ours, base } => {
            Op::IsAncestor { at: place(workspace, nth(holding, repository)), ancestor: base, commit: ours }
        }
        PrepareStep::CompareHead { repository, head } => Op::IsAncestor {
            at: place(workspace, nth(holding, repository)),
            ancestor: merged_head(holding, repository, head),
            commit: tips(holding, repository).head,
        },
        PrepareStep::Merge { repository, theirs, .. } => Op::Merge {
            at: place(workspace, nth(holding, repository)),
            theirs,
            identity: nth(holding, repository).identity,
            account: nth(holding, repository).account,
        },
        PrepareStep::CommitHead { repository, head } => commit_merge(
            holding,
            workspace,
            repository,
            merged_head(holding, repository, head),
            merge_title(merged_head(holding, repository, head)),
        ),
        PrepareStep::CommitMerge { repository, theirs } => {
            commit_merge(holding, workspace, repository, theirs, merge_title(theirs))
        }
        PrepareStep::CheckOutHead { repository, .. } | PrepareStep::CheckOutMerged { repository } => Op::CheckOut {
            at: place(workspace, nth(holding, repository)),
            commit: tips(holding, repository).head,
            clean: true,
            identity: nth(holding, repository).identity,
            account: nth(holding, repository).account,
        },
    };
    let timing = Timing { started: env.now, timeout: operation_timeout(&op, &env.limits) };
    io(id, op, env, out);
    State::Preparing { step, asked: Asked::Nothing, timing }
}

fn branch_merge_title(branch: &[u8]) -> Box<[u8]> {
    let mut title = Writer::new(b"Merge ".len().checked_add(branch.len()).expect("bounded branch"));
    title.put(b"Merge ").expect("measured prefix");
    title.put(branch).expect("measured branch");
    title.finish()
}

fn commit_merge(holding: &Holding, workspace: Token, repository: u32, theirs: Commit, title: Box<[u8]>) -> Op {
    let spec = nth(holding, repository);
    Op::Commit {
        at: place(workspace, spec),
        parent: tips(holding, repository).head,
        merging: Some(theirs),
        always: false,
        snapshot: None,
        title,
        body: Box::new([]),
        identity: spec.identity,
    }
}

/// Ready, push or save: commits and pushes each writable repository in turn,
/// or refuses what does not fit the limits.
fn begin(
    holding: &Holding,
    id: Id<Hold>,
    to: To,
    message: Message,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if !message_fits(&message, &env.limits) {
        answer(holding.client, &to, Outcome::Refused { refusal: Refusal::Invalid }, out);
        return State::Ready;
    }
    let landings = List::with_capacity(count(holding.repositories.len()));
    push_from(holding, id, Push { to, message, landings }, 0, env, out)
}

/// Pushing, from the repository at `from`: commits the next writable one, a
/// read-only one being unchanged, or, with none left, answers.
fn push_from(
    holding: &Holding,
    id: Id<Hold>,
    mut push: Push,
    from: u32,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    for repository in from..count(holding.repositories.len()) {
        let spec = nth(holding, repository);
        if let Start::Prepare { .. } = spec.start {
            return candidate::push_from(holding, id, push, repository, 0, env, out);
        }
        let writable = match push.to {
            To::Branches | To::Unchecked => spec.push.is_some(),
            To::Saved => spec.push.is_some() && spec.save.is_some(),
        };
        if !writable {
            push.landings.push(Landing::Unchanged).expect("room for each repository's landing");
            continue;
        }
        if design::repository(spec) && unchecked_head(&push.to) {
            push.landings.push(Landing::DesignUnchecked).expect("room for each repository's landing");
            continue;
        }
        let at = place(holding.disk, spec);
        let parent = tips(holding, repository).head;
        let title = copy_of(&push.message.title);
        let body = copy_of(&push.message.body);
        let merging = tips(holding, repository).merging;
        let snapshot = match push.to {
            To::Branches | To::Unchecked => {
                Some(crate::git::Snapshot { remote: spec.remote.clone(), account: spec.account })
            }
            To::Saved => None,
        };
        io(
            id,
            Op::Commit {
                at,
                parent,
                merging,
                always: tips(holding, repository).unchecked,
                title,
                body,
                identity: spec.identity,
                snapshot,
            },
            env,
            out,
        );
        return State::Pushing { push, step: PushStep::Commit { repository }, asked: Asked::Nothing };
    }
    answer(holding.client, &push.to, Outcome::Pushed { landings: push.landings.into_boxed() }, out);
    State::Ready
}

/// Pushing, done: what came of the repository in flight, then the next one,
/// or the end, which an abort or a release brings forward.
#[expect(clippy::too_many_arguments, reason = "a cell handler takes the state's parts and the event's")]
#[expect(clippy::too_many_lines, reason = "the push transition table lists each step and terminal together")]
fn pushed(
    holding: &mut Holding,
    id: Id<Hold>,
    push: Push,
    step: PushStep,
    asked: Asked,
    done: Done,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let (repository, landing) = match step {
        PushStep::Dependency { .. } => unreachable!("a dependency answer has no IO terminal"),
        PushStep::CatchUp { repository, step } => {
            return catch_up::pushed(holding, id, push, repository, step, asked, done, env, out);
        }
        PushStep::Prefix { repository, member } => {
            return candidate::pushed(holding, id, push, repository, member, asked, done, None, env, out);
        }
        PushStep::VerifyPrefix { repository, member, failure } => {
            return candidate::pushed(holding, id, push, repository, member, asked, done, Some(failure), env, out);
        }
        PushStep::Design { delivery, step } => {
            return design::pushed(holding, id, push, delivery, step, asked, done, env, out);
        }
        PushStep::Commit { repository } => match asked {
            Asked::Nothing => match committed(holding, repository, done, &push.to) {
                Ok(()) if design::checked(nth(holding, repository), &push.to) => {
                    return design::begin(holding, id, push, repository, env, out);
                }
                Ok(()) => match unpushed(holding, &push.to, repository) {
                    Some(commit) => return push_commit(holding, id, push, repository, commit, env, out),
                    None => {
                        let prior = landed_before(holding, &push.to, repository);
                        let current = tips_mut(holding, repository);
                        let redeliver = match push.to {
                            To::Branches => current.redeliver && current.head == current.start,
                            To::Unchecked | To::Saved => false,
                        };
                        if prior == Landing::Unchanged && redeliver {
                            if current.expected.is_none() {
                                current.expected = Some(current.start);
                            }
                            let head = current.head;
                            return push_commit(holding, id, push, repository, head, env, out);
                        }
                        let comparison = match push.to {
                            To::Branches | To::Unchecked => tips(holding, repository).landing,
                            To::Saved => None,
                        };
                        if prior == Landing::Unchanged
                            && let Some(landing) = comparison
                        {
                            let ancestor = tips(holding, repository).head;
                            io(
                                id,
                                Op::IsAncestor {
                                    at: place(holding.disk, nth(holding, repository)),
                                    ancestor,
                                    commit: landing,
                                },
                                env,
                                out,
                            );
                            return State::Pushing {
                                push,
                                step: PushStep::CompareLanding { repository },
                                asked: Asked::Nothing,
                            };
                        }
                        (repository, prior)
                    }
                },
                Err(Done::ParentUnreachable { head, merging, parent }) => {
                    if catch_up::eligible(holding, &push.to, repository, parent) {
                        out.push(Request::DependencyLanded { client: holding.client, head: parent });
                        return State::Pushing {
                            push,
                            step: PushStep::Dependency { repository, head, merging },
                            asked,
                        };
                    }
                    (repository, catch_up::refused())
                }
                Err(failure) => (repository, failed_landing(failure)),
            },
            // Committed or not, nothing is pushed.
            Asked::Abort | Asked::Release => match committed(holding, repository, done, &push.to) {
                Ok(())
                | Err(
                    Done::Failed { fault: Fault::Cancelled } | Done::FailedWithOutput { fault: Fault::Cancelled, .. },
                ) => (repository, Landing::Aborted),
                Err(failure) => (repository, failed_landing(failure)),
            },
        },
        PushStep::CompareLanding { repository } => {
            let landing = match done {
                Done::IsAncestor { contains: false } if asked == Asked::Nothing => {
                    let tips = tips_mut(holding, repository);
                    // Freeze the fetched start as the lease only on this retry path.
                    if tips.expected.is_none() {
                        tips.expected = Some(tips.start);
                    }
                    let head = tips.head;
                    return push_commit(holding, id, push, repository, head, env, out);
                }
                Done::IsAncestor { .. } => {
                    if asked == Asked::Nothing {
                        Landing::Unchanged
                    } else {
                        Landing::Aborted
                    }
                }
                failure @ (Done::Failed { .. } | Done::FailedWithOutput { .. }) => failed_landing(failure),
                Done::Succeeded
                | Done::Fetched { .. }
                | Done::Committed { .. }
                | Done::Unchanged
                | Done::Exists
                | Done::Rejected
                | Done::Merged
                | Done::Results { .. }
                | Done::Paths { .. }
                | Done::ParentUnreachable { .. }
                | Done::Looked { .. }
                | Done::Brings { .. }
                | Done::Conflicted { .. } => unreachable!("an ancestry query has its own terminals"),
            };
            (repository, landing)
        }
        PushStep::Push { repository } => match pushed_to(holding, &push.to, repository, done) {
            Ok(landing) => (repository, landing),
            Err(failure) => return verify(holding, id, push, repository, asked, failure, env, out),
        },
        PushStep::Verify { repository, failure } => {
            (repository, verified(holding, &push.to, repository, done, failure))
        }
    };
    complete_push(holding, id, push, repository, landing, asked, env, out)
}

#[expect(clippy::too_many_arguments, reason = "a push completion takes the state and the repository's landing")]
fn complete_push(
    holding: &Holding,
    id: Id<Hold>,
    mut push: Push,
    repository: u32,
    landing: Landing,
    asked: Asked,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    push.landings.push(landing).expect("room for each repository's landing");
    match asked {
        Asked::Nothing => push_from(holding, id, push, after(repository), env, out),
        Asked::Abort | Asked::Release => end_push(holding, push, asked, out),
    }
}

/// Pushing, a commit done: the repository's new head, if it committed, or
/// why it failed.
#[expect(clippy::result_large_err, reason = "the fixed diagnostic terminal is returned without allocation")]
fn committed(holding: &mut Holding, repository: u32, done: Done, to: &To) -> Result<(), Done> {
    match done {
        Done::Committed { commit } => {
            let tips = tips_mut(holding, repository);
            if tips.head != commit && tips.merging.is_none() {
                tips.design_merged = None;
                tips.caught_up = None;
            }
            tips.head = commit;
            tips.unchecked = unchecked_head(to);
            tips.merging = None;
            Ok(())
        }
        Done::Unchanged => {
            assert!(tips(holding, repository).merging.is_none(), "an explicit merge is never unchanged");
            Ok(())
        }
        failure @ (Done::ParentUnreachable { .. }
        | Done::Failed { .. }
        | Done::FailedWithOutput { .. }
        | Done::Conflicted { .. }) => Err(failure),
        Done::Succeeded
        | Done::Fetched { .. }
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged => {
            unreachable!("io ends a commit with its own terminals")
        }
    }
}

/// The head of the repository at `repository`, if it is a commit the branch
/// `to` names has not had from this hold: one since the last push that landed
/// on its push branch, or since the start for its saved-work branch.
fn unpushed(holding: &Holding, to: &To, repository: u32) -> Option<Commit> {
    let tips = tips(holding, repository);
    let base = match to {
        To::Branches | To::Unchecked => {
            // A saved tree is unfinished work even when a prior mid-run delivery
            // already put it on the target. Establish this hold's landing with a
            // guarded push before reporting it; subsequent finishes reuse it.
            if nth(holding, repository).start.is_saved() && !tips.delivered {
                return Some(tips.head);
            }
            tips.pushed
        }
        To::Saved => {
            if holding.recovering {
                None
            } else {
                Some(tips.start)
            }
        }
    };
    if Some(tips.head) == base { None } else { Some(tips.head) }
}

/// A repository with nothing to push: the landing of this hold's last push to
/// its push branch, if that push landed the head, or `Unchanged`.
fn landed_before(holding: &Holding, to: &To, repository: u32) -> Landing {
    let tips = tips(holding, repository);
    match to {
        To::Branches | To::Unchecked => match tips.pushed {
            Some(commit) if commit == tips.head && tips.delivered => design::landed(&tips, to),
            Some(_) | None => Landing::Unchanged,
        },
        To::Saved => Landing::Unchanged,
    }
}

/// Asks io to push `commit`, the head of the repository at `repository`, to
/// the branch `push` goes to.
fn push_commit(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    commit: Commit,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let spec = nth(holding, repository);
    let branch = branch_of(spec, &push.to);
    let at = place(holding.disk, spec);
    let remote = copy_of(&spec.remote);
    let identity = spec.identity;
    let account = spec.account;
    let expected = match push.to {
        To::Branches | To::Unchecked => tips(holding, repository).expected,
        To::Saved => None,
    };
    let op = Op::Push { at, remote, commit, branch, expected, identity, account };
    io(id, op, env, out);
    State::Pushing { push, step: PushStep::Push { repository }, asked: Asked::Nothing }
}

/// Pushing, a push done: what came of it, or `None` if it may have landed
/// all the same, to be verified. A commit that landed on its push branch is
/// the base of the next push.
#[expect(
    clippy::result_large_err,
    reason = "the ambiguous owned terminal is retained for verification without allocation"
)]
fn pushed_to(holding: &mut Holding, to: &To, repository: u32, done: Done) -> Result<Landing, Done> {
    match done {
        Done::Succeeded => Ok(land(holding, to, repository)),
        Done::Rejected | Done::Exists => Ok(Landing::Moved),
        failure @ (Done::Failed { fault: Fault::TimedOut | Fault::Broken | Fault::Unreachable | Fault::Cancelled }
        | Done::FailedWithOutput {
            fault: Fault::TimedOut | Fault::Broken | Fault::Unreachable | Fault::Cancelled,
            ..
        }) => Err(failure),
        failure @ (Done::Failed { fault: Fault::Missing { .. } | Fault::Refused }
        | Done::FailedWithOutput { fault: Fault::Missing { .. } | Fault::Refused, .. }) => Ok(failed_landing(failure)),
        Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::ParentUnreachable { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Conflicted { .. } => {
            unreachable!("io ends a push with its own terminals")
        }
    }
}

/// Asks io for the branch a push that may have landed went to, whether the
/// client asked to abort or not: one more operation, bounded as any.
#[expect(clippy::too_many_arguments, reason = "a cell handler takes the fields it touches")]
fn verify(
    holding: &Holding,
    id: Id<Hold>,
    push: Push,
    repository: u32,
    asked: Asked,
    failure: Done,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let spec = nth(holding, repository);
    let want = Want::Branch { branch: branch_of(spec, &push.to) };
    let at = place(holding.disk, spec);
    let remote = copy_of(&spec.remote);
    let identity = spec.identity;
    let account = spec.account;
    io(
        id,
        Op::Fetch {
            store: borrowed(holding, repository).token,
            into: Into::Fetched,
            at,
            remote,
            want,
            identity,
            account,
        },
        env,
        out,
    );
    State::Pushing { push, step: PushStep::Verify { repository, failure }, asked }
}

/// Pushing, a verification done: the push landed if its branch is at the
/// commit it pushed.
fn verified(holding: &mut Holding, to: &To, repository: u32, done: Done, failure: Done) -> Landing {
    match done {
        Done::Fetched { commit } if commit == tips(holding, repository).head => land(holding, to, repository),
        Done::Fetched { .. } | Done::Failed { .. } | Done::FailedWithOutput { .. } => failed_landing(failure),
        Done::Succeeded
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::ParentUnreachable { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Conflicted { .. } => {
            unreachable!("io ends a fetch with its own terminals")
        }
    }
}

/// The head of the repository at `repository` landed on the branch `to`
/// names.
fn land(holding: &mut Holding, to: &To, repository: u32) -> Landing {
    let tips = tips_mut(holding, repository);
    match to {
        To::Branches | To::Unchecked => {
            tips.pushed = Some(tips.head);
            tips.delivered = true;
            tips.unchecked = unchecked_head(to);
            if tips.expected.is_some() {
                tips.expected = Some(tips.head);
            }
        }
        To::Saved => {}
    }
    match to {
        To::Unchecked => match tips.caught_up {
            Some(tip) => Landing::CaughtUpSaved { commit: tips.head, tip },
            None => design::landed(tips, to),
        },
        To::Branches | To::Saved => design::landed(tips, to),
    }
}

/// Pushing, aborted or released once the operation in flight has ended: the
/// repositories not reached are aborted, and the client is answered.
fn end_push(holding: &Holding, mut push: Push, asked: Asked, out: &mut Queue<Request>) -> State {
    for _ in push.landings.len()..count(holding.repositories.len()) {
        push.landings.push(Landing::Aborted).expect("room for each repository's landing");
    }
    answer(holding.client, &push.to, Outcome::Pushed { landings: push.landings.into_boxed() }, out);
    match asked {
        Asked::Abort => State::Ready,
        Asked::Release => released(holding.client, out),
        Asked::Nothing => unreachable!("a push ends early only when asked to"),
    }
}

/// What the client has asked of a prepare once it asks `now`, having asked
/// `before`: the operation in flight is cancelled the first time.
fn cancelling(id: Id<Hold>, before: Asked, now: Asked, out: &mut Queue<Request>) -> Asked {
    match before {
        Asked::Nothing => out.push(Request::Cancel { owner: id.token() }),
        Asked::Abort | Asked::Release => {}
    }
    outranking(before, now)
}

/// What the client has asked once it asks `now`, having asked `before`: a
/// release outranks an abort.
const fn outranking(before: Asked, now: Asked) -> Asked {
    match before {
        Asked::Nothing | Asked::Abort => now,
        Asked::Release => Asked::Release,
    }
}

fn released(client: Token, out: &mut Queue<Request>) -> State {
    out.push(Request::Released { client });
    State::Closed
}

fn answer(client: Token, to: &To, outcome: Outcome, out: &mut Queue<Request>) {
    match to {
        To::Branches | To::Unchecked => out.push(Request::Pushed { client, outcome }),
        To::Saved => out.push(Request::Saved { client, outcome }),
    }
}

fn refuse_prepare(facts: &mut Facts, client: Token, refusal: Refusal, out: &mut Queue<Request>) {
    facts.push(Fact::Refused { client, refusal });
    out.push(Request::Prepared { client, prepared: Prepared::Refused { refusal } });
}

fn operation_timeout(op: &Op, limits: &Limits) -> Duration {
    if op.is_remote() { limits.remote_timeout } else { limits.local_timeout }
}

/// Whether an operation that ended so may have left its repository damaged:
/// git was killed, or failed on the worker's side.
const fn damages(done: &Done) -> bool {
    match done {
        Done::Failed { fault: Fault::Broken | Fault::TimedOut | Fault::Cancelled }
        | Done::FailedWithOutput { fault: Fault::Broken | Fault::TimedOut | Fault::Cancelled, .. } => true,
        Done::Failed { fault: Fault::Missing { .. } | Fault::Refused | Fault::Unreachable }
        | Done::FailedWithOutput { fault: Fault::Missing { .. } | Fault::Refused | Fault::Unreachable, .. }
        | Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::ParentUnreachable { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Conflicted { .. } => false,
    }
}

/// Why a prepare failed, from an operation's fault for the repository at
/// `repository`.
const fn failure(repository: u32, fault: Fault) -> Failure {
    match fault {
        Fault::Missing { missing } => Failure::Missing { repository, missing },
        Fault::Refused => Failure::Refused { repository },
        Fault::Unreachable | Fault::Broken | Fault::TimedOut | Fault::Cancelled => Failure::Transient,
    }
}

/// What a failed invocation said, retaining its diagnostic output.
fn failed_landing(done: Done) -> Landing {
    match done {
        Done::ParentUnreachable { .. } => catch_up::refused(),
        Done::Conflicted { files, .. } => Landing::Conflicted { files },
        Done::Failed { fault } => Landing::Explained { fault, diagnostic: crate::git::PushDiagnostic::empty() },
        Done::FailedWithOutput { fault, diagnostic } => Landing::Explained { fault, diagnostic },
        Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Results { .. }
        | Done::Paths { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged => unreachable!("only a failed invocation is explained"),
    }
}

fn tally(landings: &[Landing]) -> Tally {
    let mut tally = Tally { landed: 0, moved: 0, failed: 0, refused: 0, unchanged: 0, aborted: 0 };
    for landing in landings {
        let counted = match landing {
            Landing::CaughtUpSaved { .. }
            | Landing::Landed { .. }
            | Landing::Prefixes { .. }
            | Landing::DesignMerged { .. } => &mut tally.landed,
            Landing::Moved => &mut tally.moved,
            Landing::CaughtUp { .. }
            | Landing::CaughtUpConflict { .. }
            | Landing::DesignConflict { .. }
            | Landing::DesignCode { .. }
            | Landing::DesignMoving
            | Landing::DesignUnchecked
            | Landing::Conflicted { .. }
            | Landing::Refused
            | Landing::Explained { fault: Fault::Refused, .. } => &mut tally.refused,
            Landing::Failed | Landing::Explained { .. } => &mut tally.failed,
            Landing::Unchanged => &mut tally.unchanged,
            Landing::Aborted => &mut tally.aborted,
        };
        *counted = counted.saturating_add(1);
    }
    tally
}

/// Whether `spec` fits the limits: a workstream, and between one and
/// `Limits::repositories` repositories, each in a directory of its own named
/// by one safe path component, and every name, remote, branch and identity at
/// most `Limits::name_bytes` and none empty.
fn fits(spec: &Spec, limits: &Limits) -> bool {
    let Ok(repositories) = u32::try_from(spec.repositories.len()) else {
        return false;
    };
    if !named(&spec.key, limits) || !(1..=limits.repositories).contains(&repositories) {
        return false;
    }
    for (place, repository) in spec.repositories.iter().enumerate() {
        let start = match &repository.start {
            Start::Base { branch } | Start::Branch { branch } | Start::Saved { branch } => named(branch, limits),
            Start::Prepare { request } => {
                limits.conflicts > 0
                    && limits.path_bytes > 0
                    && repositories == 1
                    && (repository.push.is_some() || !repository.prefixes.is_empty())
                    && repository.save.is_none()
                    && repository.expected.is_none()
                    && repository.landing.is_none()
                    && candidate::fits(repository, request, limits)
            }
            Start::Commit { .. } => true,
            Start::Merged { branch, heads } => {
                named(branch, limits)
                    && (1..=64).contains(&heads.len())
                    && limits.conflicts > 0
                    && limits.path_bytes > 0
            }
            Start::Merge { branch, .. } => named(branch, limits) && limits.conflicts > 0 && limits.path_bytes > 0,
        };
        let push = match &repository.push {
            Some(branch) => named(branch, limits),
            None => true,
        };
        let save = match &repository.save {
            Some(branch) => named(branch, limits),
            None => true,
        };
        let directory = named(&repository.name, limits) && component(&repository.name);
        let reached = named(&repository.remote, limits);
        let landing = match &repository.landing {
            Some(branch) => named(branch, limits),
            None => true,
        };
        if !directory
            || !reached
            || !start
            || !push
            || !landing
            || !save
            || (!repository.prefixes.is_empty() && !matches_prepare(&repository.start))
            || (repository.expected.is_some() && repository.push.is_none())
        {
            return false;
        }
        for earlier in spec.repositories.iter().take(place) {
            if earlier.name == repository.name {
                return false;
            }
        }
    }
    true
}

/// Whether `name` is one safe path component: not `.` or `..`, without `/`
/// or NUL, and not a git directory (`.git` in any ASCII case).
fn component(name: &[u8]) -> bool {
    let dots = name == b"." || name == b"..";
    let separated = find(name, b"/").is_some() || find(name, b"\0").is_some();
    !dots && !separated && !name.eq_ignore_ascii_case(b".git")
}

fn matches_prepare(start: &Start) -> bool {
    match start {
        Start::Prepare { .. } => true,
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Commit { .. }
        | Start::Saved { .. }
        | Start::Merge { .. }
        | Start::Merged { .. } => false,
    }
}

/// Whether a push's or a save's message fits the limits:
/// a title, and at most `Limits::message_bytes` with the body.
fn message_fits(message: &Message, limits: &Limits) -> bool {
    let Some(bytes) = message.title.len().checked_add(message.body.len()) else {
        return false;
    };
    let within = match u32::try_from(bytes) {
        Ok(bytes) => bytes <= limits.message_bytes,
        Err(_) => false,
    };
    within && !message.title.is_empty()
}

fn named(bytes: &[u8], limits: &Limits) -> bool {
    let within = match u32::try_from(bytes.len()) {
        Ok(len) => len <= limits.name_bytes,
        Err(_) => false,
    };
    within && !bytes.is_empty()
}

/// Ineligible supplied comparisons are ignored; they can never grant a write.
fn landing_branch(repository: &Repository) -> Option<&[u8]> {
    match &repository.start {
        Start::Branch { branch } => match &repository.push {
            Some(push) if branch == push => repository.landing.as_deref(),
            Some(_) | None => None,
        },
        Start::Base { .. }
        | Start::Saved { .. }
        | Start::Prepare { .. }
        | Start::Commit { .. }
        | Start::Merge { .. }
        | Start::Merged { .. } => None,
    }
}

/// Whether the repository at `repository` starts from a base branch that is
/// created if the forge does not have it: only for one that may be written.
fn creates_base(holding: &Holding, repository: u32) -> bool {
    let spec = nth(holding, repository);
    let base = match spec.start {
        Start::Base { .. } => true,
        Start::Branch { .. }
        | Start::Prepare { .. }
        | Start::Commit { .. }
        | Start::Saved { .. }
        | Start::Merge { .. }
        | Start::Merged { .. } => false,
    };
    base && spec.push.is_some()
}

fn nth(holding: &Holding, repository: u32) -> &Repository {
    let index = usize::try_from(repository).expect("a u32 fits in a usize");
    holding.repositories.get(index).expect("a repository of the spec")
}

fn tips(holding: &Holding, repository: u32) -> Tips {
    *holding.tips.get(repository).expect("a repository checked out")
}

fn tips_mut(holding: &mut Holding, repository: u32) -> &mut Tips {
    holding.tips.get_mut(repository).expect("a repository checked out")
}

fn place(workspace: Token, repository: &Repository) -> Place {
    Place { workspace, repository: copy_of(&repository.name), git: repository.git.clone() }
}

/// The branch a push to `to` of `repository` goes to.
fn branch_of(repository: &Repository, to: &To) -> Box<[u8]> {
    match to {
        To::Branches | To::Unchecked => {
            copy_of(repository.push.as_deref().expect("only a writable repository is pushed"))
        }
        To::Saved => copy_of(repository.save.as_deref().expect("only a repository with a save destination is saved")),
    }
}

fn after(repository: u32) -> u32 {
    repository.checked_add(1).expect("fewer repositories than a u32 counts")
}

fn borrowed(holding: &Holding, repository: u32) -> &crate::cache::Borrowed {
    holding.stores.get(usize::try_from(repository).expect("bounded index")).expect("spec store")
}
fn stores_step(
    holding: &Holding,
    cache: &mut Cache,
    id: Id<Hold>,
    from: u32,
    look: bool,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    for repository in from..count(holding.stores.len()) {
        match cache.claim(borrowed(holding, repository).store, id) {
            Claim::Made => {}
            Claim::Make => return prepare_step(holding, cache, id, PrepareStep::Store { repository, look }, env, out),
            Claim::Waiting => return State::Waiting { repository, look },
        }
    }
    let step = if look { PrepareStep::Look { repository: 0 } } else { PrepareStep::Make };
    prepare_step(holding, cache, id, step, env, out)
}

fn reds_prepare(holding: &Holding, from: u32) -> Next {
    for repository in from..count(holding.repositories.len()) {
        if nth(holding, repository).push.is_some() {
            return Next::Step(PrepareStep::Reds { repository });
        }
    }
    Next::Ready { pack_from: 0 }
}
pub(crate) fn reds(
    domain: &mut Domain,
    env: &Env<Limits>,
    token: Token,
    action: crate::git::Reds,
    out: &mut Queue<Request>,
) {
    let Some(id) = addressed(&domain.holds, token) else { return };
    let mark = out.len();
    let hold = domain.holds.get_mut(id).expect("addressed hold");
    let was_active = active(&hold.state) || hold.holding.recovering;
    if hold.cut {
        reds::interrupted(&hold.holding, &action, out);
        return;
    }
    let state = mem::replace(&mut hold.state, State::Closed);
    hold.state = reds::update(&hold.holding, id, state, action, env, out);
    conclude(domain, env, id, was_active, out, mark);
}

/// Keep only a completed save: each unchanged repository names its start.
fn saved_repositories(holding: &Holding, outcome: &Outcome) -> Option<Box<[crate::git::KeptRepository]>> {
    let landings = match outcome {
        Outcome::Pushed { landings } => landings,
        Outcome::Refused { .. } => return None,
    };
    let mut repositories = List::with_capacity(count(holding.repositories.len()));
    for ((spec, tips), landing) in holding.repositories.iter().zip(holding.tips.as_slice()).zip(landings.iter()) {
        let commit = match landing {
            Landing::Landed { commit } | Landing::DesignMerged { commit, .. } => *commit,
            Landing::Unchanged => tips.start,
            Landing::CaughtUpSaved { .. }
            | Landing::Prefixes { .. }
            | Landing::CaughtUp { .. }
            | Landing::CaughtUpConflict { .. }
            | Landing::Moved
            | Landing::Conflicted { .. }
            | Landing::DesignConflict { .. }
            | Landing::DesignCode { .. }
            | Landing::Explained { .. }
            | Landing::Failed
            | Landing::Refused
            | Landing::DesignMoving
            | Landing::DesignUnchecked
            | Landing::Aborted => return None,
        };
        repositories
            .push(crate::git::KeptRepository {
                name: spec.name.clone(),
                remote: spec.remote.clone(),
                commit,
                merging: None,
                unchecked: false,
            })
            .expect("repository capacity");
    }
    if repositories.len() != count(holding.repositories.len()) {
        return None;
    }
    Some(repositories.into_boxed())
}

pub(crate) fn keep(domain: &mut Domain, env: &Env<Limits>, token: Token, out: &mut Queue<Request>) {
    let from = out.len();
    let Some(id) = addressed(&domain.holds, token) else {
        return;
    };
    let hold = domain.holds.get_mut(id).expect("addressed hold");
    let Some(repositories) = hold.holding.saved.take() else {
        release(domain, env, token, out);
        return;
    };
    assert!(matches_ready(&hold.state), "keep follows the settled save");
    let mark = crate::git::Kept {
        workspace: hold.holding.disk,
        key: hold.holding.key.clone(),
        state: crate::git::KeptState::Kept,
        repositories,
        when: env.wall.as_nanos(),
    };
    io(id, Op::Keep { workspace: mark.workspace, mark: Some(mark.clone()) }, env, out);
    hold.state = State::Keeping { mark };
    conclude(domain, env, id, false, out, from);
}

fn matches_ready(state: &State) -> bool {
    match state {
        State::Ready => true,
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Reds(_)
        | State::Unprepared
        | State::Pushing { .. }
        | State::Keeping { .. }
        | State::Closed => false,
    }
}

/// One IO lane per held workspace keeps rewrites, subsequent git operations and
/// removal in order. Only the first mark participates in preparation.
fn schedule(hold: &mut Hold, id: Id<Hold>, env: &Env<Limits>, out: &mut Queue<Request>, mark: u32) {
    let mut next = if hold.operation.is_none() { hold.pending.pop() } else { None };
    let mut produced = Queue::uncapped();
    let length = out.len();
    for index in 0..length {
        let request = out.pop().expect("the requests to route");
        if index < mark {
            produced.push(request);
            continue;
        }
        match request {
            Request::Io { owner, op, deadline: _ } if owner == id.token() => {
                let background = if let Op::Keep { mark, .. } = &op {
                    match mark {
                        Some(mark) if mark.state == crate::git::KeptState::Held => hold.marked = true,
                        None => hold.marked = false,
                        Some(_) => {}
                    }
                    match hold.state {
                        State::Preparing { step: PrepareStep::Mark | PrepareStep::Unkeep, .. }
                        | State::Keeping { .. } => false,
                        State::Queued { .. }
                        | State::Preparing { .. }
                        | State::Waiting { .. }
                        | State::Ready
                        | State::Reds(_)
                        | State::Unprepared
                        | State::Pushing { .. }
                        | State::Closed => true,
                    }
                } else {
                    false
                };
                if hold.operation.is_none() && next.is_none() {
                    next = Some((op, background));
                } else {
                    hold.pending.push((op, background));
                }
            }
            Request::Released { client } if client == hold.holding.client => hold.releasing = true,
            other @ (Request::Io { .. }
            | Request::Released { .. }
            | Request::Held { .. }
            | Request::Prepared { .. }
            | Request::Pushed { .. }
            | Request::Saved { .. }
            | Request::Cancel { .. }
            | Request::DependencyLanded { .. }
            | Request::Results { .. }
            | Request::ResultsEnd { .. }) => produced.push(other),
        }
    }
    if hold.pending.is_empty() {
        // An idle lane need not retain capacity for full git operations.
        hold.pending = Queue::uncapped();
    }
    if let Some((op, background)) = next {
        let timeout = operation_timeout(&op, &env.limits);
        let deadline = env.now.saturating_add(timeout);
        if let State::Preparing { timing, .. } = &mut hold.state {
            *timing = Timing { started: env.now, timeout };
        }
        hold.operation = Some(deadline);
        hold.marking = background;
        produced.push(Request::Io { owner: id.token(), op, deadline });
    }
    if hold.releasing && hold.operation.is_none() && hold.pending.is_empty() {
        produced.push(Request::Released { client: hold.holding.client });
        hold.releasing = false;
    }
    *out = produced;
}

fn matches_closed(state: &State) -> bool {
    match state {
        State::Closed => true,
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Reds(_)
        | State::Unprepared
        | State::Pushing { .. }
        | State::Keeping { .. } => false,
    }
}

fn can_mark(holding: &Holding) -> bool {
    let mut saving = false;
    for spec in &holding.repositories {
        saving |= spec.push.is_some() && spec.save.is_some();
    }
    holding.tips.len() == count(holding.repositories.len()) && saving
}

/// Keep the comparison only while a completion can change the tips, not for the
/// hold's lifetime. Names, remotes and repository order do not change while marked.
fn held_tips(holding: &Holding) -> Box<[MarkedTip]> {
    let mut tips = List::with_capacity(holding.tips.len());
    for current in holding.tips.as_slice() {
        tips.push(MarkedTip { head: current.head, merging: current.merging, unchecked: current.unchecked })
            .expect("one tip per marked repository");
    }
    tips.into_boxed()
}

fn rewrite_mark(
    hold: &Hold,
    previous: Option<Box<[MarkedTip]>>,
    id: Id<Hold>,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) {
    let changed = match previous {
        Some(previous) => !mark_matches(&hold.holding, &previous),
        None => false,
    };
    if hold.marked && changed && !hold.holding.recovering && can_mark(&hold.holding) {
        let current = held_mark(&hold.holding, env).expect("a complete saving hold has a mark");
        io(id, Op::Keep { workspace: current.workspace, mark: Some(current) }, env, out);
    }
}

/// Compare only the changing fields, in the hold's fixed repository order.
fn mark_matches(holding: &Holding, marked: &[MarkedTip]) -> bool {
    if marked.len() != holding.repositories.len() {
        return false;
    }
    for (tips, saved) in holding.tips.as_slice().iter().zip(marked.iter()) {
        if tips.head != saved.head || tips.merging != saved.merging || tips.unchecked != saved.unchecked {
            return false;
        }
    }
    true
}

fn held_mark(holding: &Holding, env: &Env<Limits>) -> Option<crate::git::Kept> {
    if !can_mark(holding) {
        return None;
    }
    let mut repositories = List::with_capacity(holding.tips.len());
    for (spec, tips) in holding.repositories.iter().zip(holding.tips.as_slice()) {
        repositories
            .push(crate::git::KeptRepository {
                name: spec.name.clone(),
                remote: spec.remote.clone(),
                commit: tips.head,
                merging: tips.merging,
                unchecked: tips.unchecked,
            })
            .expect("one mark per repository");
    }
    Some(crate::git::Kept {
        workspace: holding.disk,
        key: holding.key.clone(),
        state: crate::git::KeptState::Held,
        repositories: repositories.into_boxed(),
        when: env.wall.as_nanos(),
    })
}

fn ready(holding: &mut Holding, out: &mut Queue<Request>) -> State {
    let conflicts = mem::replace(&mut holding.conflicts, List::with_capacity(0)).into_boxed();
    let prepared = if holding.recovered {
        Prepared::Recovered { workspace: holding.disk, conflicts, saved: holding.saved.clone().expect("taken-up save") }
    } else {
        match &holding.candidate {
            Some(stack) => Prepared::Candidate { workspace: holding.disk, outcome: candidate::result(stack) },
            None => Prepared::Ready { workspace: holding.disk, conflicts },
        }
    };
    out.push(Request::Prepared { client: holding.client, prepared });
    State::Ready
}

fn recovered(holding: &mut Holding) -> Next {
    holding.recovering = false;
    holding.tips = List::with_capacity(count(holding.repositories.len()));
    let Some(repositories) = holding.saved.clone() else {
        holding.kept = None;
        return Next::Stores { from: 0, look: false };
    };
    let mut original_starts = List::uncapped();
    for (index, spec) in holding.repositories.iter_mut().enumerate() {
        if spec.push.is_some() {
            let start = Start::Saved { branch: spec.save.clone().expect("matched save destination") };
            let original = mem::replace(&mut spec.start, start);
            original_starts.push((count(index), original)).expect("writable repository");
        }
    }
    holding.original_starts = original_starts.into_boxed();
    let mark = holding.kept.as_mut().expect("recovering a held mark");
    mark.state = crate::git::KeptState::Kept;
    mark.repositories = repositories;
    holding.recovered = true;
    Next::Step(PrepareStep::CompareKept { repository: 0 })
}

fn restore_assignment(holding: &mut Holding) {
    let originals = mem::replace(&mut holding.original_starts, Box::new([]));
    for (index, start) in originals {
        holding
            .repositories
            .get_mut(usize::try_from(index).expect("repository index"))
            .expect("original repository")
            .start = start;
    }
    holding.recovered = false;
    holding.saved = None;
    holding.tips = List::with_capacity(count(holding.repositories.len()));
    holding.fetched = List::uncapped();
}

fn restore_tips(holding: &mut Holding) {
    let mark = holding.kept.as_ref().expect("matched held mark");
    for repository in &mark.repositories {
        holding
            .tips
            .push(Tips {
                start: repository.commit,
                head: repository.commit,
                merging: repository.merging,
                unchecked: repository.unchecked,
                pushed: None,
                delivered: false,
                redeliver: false,
                expected: None,
                landing: None,
                design_merged: None,
                caught_up: None,
            })
            .expect("matched repositories");
    }
}

fn matches_kept(holding: &Holding) -> bool {
    let Some(mark) = &holding.kept else {
        return false;
    };
    if mark.key != holding.key || mark.repositories.len() != holding.repositories.len() {
        return false;
    }
    for (saved, spec) in mark.repositories.iter().zip(holding.repositories.iter()) {
        let accepts = match mark.state {
            crate::git::KeptState::Kept => spec.start.is_saved(),
            crate::git::KeptState::Held => spec.push.is_none() || spec.save.is_some(),
        };
        if saved.name != spec.name || saved.remote != spec.remote || !accepts {
            return false;
        }
    }
    true
}
