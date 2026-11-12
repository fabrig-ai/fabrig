//! Holds: a client's hold on a workspace, from its prepare to its release,
//! and the operations it runs there, one at a time (domain/worker/README.md, 4.2
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
//!             done, all checked out     Ready       prepared: ready
//!             done, failed              Unprepared  prepared: failed
//!             done, aborting            Unprepared  prepared: aborted
//!             done, releasing           Closed      prepared: aborted, released
//!             abort, release            Preparing   cancel, the first time asked
//! Waiting     store made                Preparing   io: next store, look or make
//!             store failed, oldest      Preparing   io: store
//!             store failed, later       Waiting     (nothing)
//!             abort                     Unprepared  prepared: aborted
//!             release                   Closed      prepared: aborted, released
//! Pushing     done, more to do          Pushing     io: the next operation
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
//! What a transition tells as facts is derived from the requests it made, in
//! one place ([`tell`]), and what the new state implies (a closed hold is
//! retired, and its workspace idle again) in another ([`follow`]).

mod candidate;
mod design;

use alloc::boxed::Box;
use core::mem;

use skein_lib::bytes::{copy_of, find};
use skein_lib::{Env, Id, List, Queue, Slab, Token, Writer};

use crate::boundary::{
    Conflicts, Failure, Landing, Message, Outcome, Prepared, Refusal, Repository, Request, Spec, Start,
};
use crate::cache::{Cache, Claim, Workspace, count};
use crate::domain::Domain;
use crate::facts::{Cached, Ending, Fact, Facts, Preparation, Tally, Target};
use crate::git::{Commit, Done, Fault, Into, Missing, Op, Place, Want};
use crate::limits::Limits;

#[derive(Debug)]
pub(crate) struct Hold {
    holding: Holding,
    state: State,
}

/// What a hold holds in every state.
#[derive(Debug)]
struct Holding {
    /// The client's token, echoed on every record back to it.
    client: Token,
    workspace: Id<Workspace>,
    stores: Box<[crate::cache::Borrowed]>,
    /// The spec's repositories, in its order.
    repositories: Box<[Repository]>,
    /// Where each repository checked out so far stands, in the same order.
    tips: List<Tips>,
    /// Moved to the client exactly once when preparation succeeds.
    conflicts: List<Conflicts>,
    /// What this prepare fetched, never a previous hold's observations.
    fetched: List<Fetched>,
    candidate: Option<crate::Candidate>,
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
    unchecked: bool,
    merging: Option<Commit>,
    expected: Option<Commit>,
    landing: Option<Commit>,
    /// The tip this head merged, retained for repeated delivery receipts.
    design_merged: Option<Commit>,
}

#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "diagnostics stay inline with a fixed byte capacity")]
enum State {
    /// Holds a workspace and borrows stores, waiting for a preparation permit.
    Queued { cached: Cached },
    /// Preparing the workspace: `step` is in flight.
    Preparing { step: PrepareStep, asked: Asked },
    /// Waiting for another hold's store, with no operation in flight.
    Waiting { repository: u32, look: bool },
    /// Prepared, with nothing under way.
    Ready,
    /// Its prepare failed or was aborted: it may only be released.
    Unprepared,
    /// Pushing or saving: `step` is in flight.
    Pushing { push: Push, step: PushStep, asked: Asked },
    /// Terminal: released, holding nothing.
    Closed,
}

/// The operation a preparing hold waits for. A repository is named by its
/// place in the spec.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum PrepareStep {
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
    },
    /// The pin, followed by the start's remaining preparation or a producer merge.
    FetchLanding {
        repository: u32,
        ours: Commit,
        merging: Option<Commit>,
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

/// The operation a pushing hold waits for.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "diagnostics stay inline with a fixed byte capacity")]
enum PushStep {
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
    Stores { from: u32, look: bool },
    Ready,
    Failed(Failure),
}

// Entry points, one per event: look the hold up, take its state out, run the
// cell's handler, then tell what happened and follow the new state
// ([`conclude`]).

pub(crate) fn prepare(domain: &mut Domain, env: &Env<Limits>, client: Token, spec: Spec, out: &mut Queue<Request>) {
    let mark = out.len();
    if !fits(&spec, &env.limits) {
        refuse_prepare(&mut domain.facts, client, Refusal::Invalid, out);
        return;
    }
    let Spec { key, repositories } = spec;
    let crate::cache::Granted { workspace, cached, stores } = match domain.cache.hold(key, &repositories) {
        Ok(holding) => holding,
        Err(refusal) => {
            refuse_prepare(&mut domain.facts, client, refusal, out);
            return;
        }
    };
    domain.facts.push(Fact::Held { client, cached });
    let tips = List::with_capacity(count(repositories.len()));
    let conflicts = List::with_capacity(count(repositories.len()));
    let holding = Holding {
        client,
        workspace,
        stores,
        repositories,
        tips,
        conflicts,
        fetched: List::uncapped(),
        candidate: None,
    };
    let hold = Hold { holding, state: State::Closed };
    let id = domain.holds.insert(hold).expect("room for a hold for each workspace, and as many released");
    out.push(Request::Held { client, hold: id.token() });
    if domain.active < env.limits.preparations {
        admit(domain, env, id, cached, out);
    } else {
        domain.holds.get_mut(id).expect("inserted above").state = State::Queued { cached };
        domain.queued.push(id);
        domain.facts.push(Fact::Queued { client });
    }
    conclude(domain, id, false, out, mark);
}

/// The first step is shared by prepares arriving with a free permit and
/// those admitted from the queue. Its io deadlines start at this step's now.
fn admit(domain: &mut Domain, env: &Env<Limits>, id: Id<Hold>, cached: Cached, out: &mut Queue<Request>) {
    domain.active = domain.active.checked_add(1).expect("one active prepare per held workspace");
    let hold = domain.holds.get_mut(id).expect("an admitted hold lives");
    hold.state = match cached {
        Cached::Reused | Cached::Kept => {
            prepare_step(&hold.holding, id, PrepareStep::Fetch { repository: 0 }, env, out)
        }
        Cached::Looking | Cached::Rebuilt | Cached::Evicted | Cached::New => {
            stores_step(&hold.holding, &mut domain.cache, id, 0, cached == Cached::Looking, env, out)
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
            | State::Unprepared
            | State::Pushing { .. }
            | State::Closed => {
                unreachable!("only queued holds wait for permits")
            }
        };
        let mark = out.len();
        admit(domain, env, id, cached, out);
        conclude(domain, id, false, out, mark);
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
    let was_active = active(&hold.state);
    let state = mem::replace(&mut hold.state, State::Closed);
    hold.state = match state {
        State::Ready => begin(&hold.holding, id, to, message, env, out),
        busy @ (State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Pushing { .. }
        | State::Unprepared) => {
            answer(hold.holding.client, &to, Outcome::Refused { refusal: Refusal::Busy }, out);
            busy
        }
        State::Closed => unreachable!("an addressed hold has not been released"),
    };
    conclude(domain, id, was_active, out, mark);
}

pub(crate) fn abort(domain: &mut Domain, env: &Env<Limits>, hold: Token, out: &mut Queue<Request>) {
    ask(domain, env, hold, Asked::Abort, out);
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
    let was_active = active(&hold.state);
    let state = mem::replace(&mut hold.state, State::Closed);
    hold.state = match state {
        State::Preparing { step, asked } => State::Preparing { step, asked: cancelling(id, asked, now, out) },
        State::Pushing { push, step, asked } => State::Pushing { push, step, asked: outranking(asked, now) },
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
        State::Closed => unreachable!("an addressed hold has not been released"),
    };
    conclude(domain, id, was_active, out, mark);
    admissions(domain, env, out);
}

/// Terminal for an operation: what the hold does next.
pub(crate) fn done(domain: &mut Domain, env: &Env<Limits>, owner: Token, done: Done, out: &mut Queue<Request>) {
    let mark = out.len();
    let done = bounded_done(done, &env.limits);
    let id = Id::from_token(owner);
    let hold = domain.holds.get_mut(id).expect("a hold lives until its operation has ended");
    let told = ending(&done);
    domain.facts.push(Fact::Ended { client: hold.holding.client, done: told });
    if damages(&done) {
        domain.cache.spoil(hold.holding.workspace);
    }
    let was_active = active(&hold.state);
    let state = mem::replace(&mut hold.state, State::Closed);
    let holding = &mut hold.holding;
    let mut waiters = Queue::uncapped();
    let mut store_succeeded = false;
    hold.state = match state {
        State::Preparing { step, asked } => {
            let detail = preparation_detail(holding, step, &done);
            store_succeeded = done == Done::Succeeded;
            let next = prepared(holding, &mut domain.cache, step, done, &env.limits);
            match step {
                PrepareStep::Store { repository, .. } => {
                    waiters = domain.cache.finished(borrowed(holding, repository).store, id, store_succeeded);
                }
                PrepareStep::Candidate { .. }
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
        State::Pushing { push, step, asked } => pushed(holding, id, push, step, asked, done, env, out),
        State::Queued { .. } | State::Waiting { .. } | State::Ready | State::Unprepared | State::Closed => {
            unreachable!("an operation ends only while its hold prepares, pushes or saves")
        }
    };
    conclude(domain, id, was_active, out, mark);
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
            | State::Unprepared
            | State::Pushing { .. }
            | State::Closed => {
                unreachable!("only waiting holds are queued");
            }
        };
        conclude(domain, waiter, true, out, mark);
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
        | State::Unprepared
        | State::Pushing { .. } => Some(id),
        State::Closed => None,
    }
}

/// Applied after every transition: what it tells, and what the new state
/// implies. `mark` is where the requests the transition made begin in `out`.
fn conclude(domain: &mut Domain, id: Id<Hold>, was_active: bool, out: &Queue<Request>, mark: u32) {
    let hold = domain.holds.get(id).expect("a hold lives until it is retired");
    tell(&mut domain.facts, hold.holding.client, out, mark);
    if was_active && !active(&hold.state) {
        domain.active = domain.active.checked_sub(1).expect("a settled prepare returns its permit once");
    }
    follow(&mut domain.holds, &mut domain.cache, id);
}

/// A store waiter keeps its permit even with no operation in flight.
fn active(state: &State) -> bool {
    match state {
        State::Preparing { .. } | State::Waiting { .. } => true,
        State::Queued { .. } | State::Ready | State::Unprepared | State::Pushing { .. } | State::Closed => false,
    }
}

/// What a transition tells, derived from the requests it made: a fact for
/// each but `Held`, which the prepare tells with how the cache found its
/// workspace.
fn tell(facts: &mut Facts, client: Token, out: &Queue<Request>, mark: u32) {
    let made = usize::try_from(mark).expect("a u32 fits in a usize");
    for request in out.iter().skip(made) {
        let fact = match request {
            Request::Held { .. } => continue,
            Request::Prepared { client, prepared } => {
                Fact::Prepared { client: *client, prepared: preparation(prepared) }
            }
            Request::Pushed { client, outcome } => pushed_fact(*client, Target::Push, outcome),
            Request::Saved { client, outcome } => pushed_fact(*client, Target::Saved, outcome),
            Request::Released { client } => Fact::Released { client: *client },
            Request::Io { owner: _, op, deadline: _ } => Fact::Started { client, op: op.kind() },
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
        Done::Paths { files } => Ending::Paths { files: count(files.len()) },
        Done::Looked { holds } => Ending::Looked { holds: *holds },
        Done::IsAncestor { contains } => Ending::IsAncestor { contains: *contains },
        Done::Brings { brings } => Ending::Brings { brings: *brings },
        Done::Succeeded => Ending::Succeeded,
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
        Prepared::Ready { workspace, conflicts } => {
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
        other @ (Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
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
            cache.release(hold.holding.workspace);
            holds.retire(id);
        }
        State::Queued { .. }
        | State::Preparing { .. }
        | State::Waiting { .. }
        | State::Ready
        | State::Unprepared
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
        | Done::Paths { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged
        | Done::Conflicted { .. } => {}
    }
    let last = count(holding.repositories.len()).checked_sub(1).expect("a spec names a repository");
    match step {
        PrepareStep::Candidate { .. } => unreachable!("candidate preparation was dispatched before the fetch memo"),
        PrepareStep::Store { repository, look } => match done {
            Done::Succeeded => Next::Stores { from: after(repository), look },
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Paths { .. }
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
            Done::Paths { .. } => unreachable!("a look has its own terminals"),
            Done::Looked { holds: true } if repository < last => {
                Next::Step(PrepareStep::Look { repository: after(repository) })
            }
            Done::Looked { holds: true } => {
                cache.cloned(holding.workspace, &holding.repositories, &holding.stores);
                Next::Step(PrepareStep::Fetch { repository: 0 })
            }
            Done::Looked { holds: false } => Next::Step(PrepareStep::Make),
            Done::Failed { fault: _ } | Done::FailedWithOutput { fault: _, .. } => Next::Failed(Failure::Transient),
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
            Done::Failed { fault: _ } | Done::FailedWithOutput { fault: _, .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => {
                unreachable!("io ends a fetch with its own terminals")
            }
        },
        PrepareStep::FetchLanding { repository, ours, merging } => match done {
            Done::Fetched { commit } => match merging {
                Some(base) => begin_merge(holding, repository, ours, Some(base)),
                None => after_start(holding, repository, ours, Some(commit)),
            },
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Succeeded
            | Done::Paths { .. }
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
        PrepareStep::PointLanding { repository, ours, commit, merging } => match done {
            Done::Succeeded => match merging {
                Some(base) => begin_merge(holding, repository, ours, Some(base)),
                None => after_start(holding, repository, ours, Some(commit)),
            },
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Paths { .. }
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
            Done::Succeeded => after_replaced_pin(holding, repository, ours, pushed),
            Done::Failed { fault } | Done::FailedWithOutput { fault, .. } => Next::Failed(failure(repository, fault)),
            Done::Fetched { .. }
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
                    unchecked: false,
                    merging: None,
                    design_merged: None,
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
                    Next::Ready
                }
            }
            Done::Failed { fault: _ } | Done::FailedWithOutput { fault: _, .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
                None => after_merged(repository, last),
            },
            Done::Conflicted { files, .. } => {
                if head.is_some() {
                    return Next::Failed(Failure::Conflicted { repository, head: theirs });
                }
                assert!(paths_fit(&files, limits), "terminal admission bounds retained paths");
                holding.conflicts.push(Conflicts { repository, files }).expect("room for each repository's conflicts");
                after_merged(repository, last)
            }
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Succeeded
            | Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Rejected => unreachable!("io ends a merge with its own terminals"),
        },
        PrepareStep::CompareBase { repository, ours, base } => match done {
            Done::IsAncestor { contains: true } => resume_contained(holding, repository, ours),
            Done::IsAncestor { contains: false } => {
                if pin_branch(nth(holding, repository)).is_some() {
                    fetch_landing(holding, repository, ours, Some(base))
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
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
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged => unreachable!("explicit merge commit has its own terminals"),
        },
        PrepareStep::CheckOutMerged { repository } => match done {
            Done::Succeeded => after_merged(repository, last),
            Done::Failed { .. } | Done::FailedWithOutput { .. } => Next::Failed(Failure::Transient),
            Done::Fetched { .. }
            | Done::Committed { .. }
            | Done::Unchanged
            | Done::Exists
            | Done::Rejected
            | Done::Paths { .. }
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
            | Done::Paths { .. }
            | Done::Looked { .. }
            | Done::Brings { .. }
            | Done::IsAncestor { .. }
            | Done::Merged
            | Done::Conflicted { .. } => unreachable!("io ends a checkout with its own terminals"),
        },
        PrepareStep::Candidate { .. }
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
        Start::Prepare { request } => Want::Commit { commit: request.base },
    }
}

fn remember_fetch(holding: &mut Holding, step: PrepareStep, commit: Commit) {
    let (repository, want) = match step {
        PrepareStep::Fetch { repository } | PrepareStep::Refetch { repository } => {
            (repository, start_want(&nth(holding, repository).start))
        }
        PrepareStep::Default { repository } => (repository, Want::Default),
        PrepareStep::FetchPush { repository, .. } => (
            repository,
            Want::Branch { branch: copy_of(nth(holding, repository).push.as_deref().expect("writable item")) },
        ),
        PrepareStep::FetchLanding { repository, .. } => (
            repository,
            Want::Branch { branch: copy_of(pin_branch(nth(holding, repository)).expect("writable landing")) },
        ),
        PrepareStep::FetchBase { repository, base, .. } => (repository, Want::Commit { commit: base }),
        PrepareStep::Candidate { .. }
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

fn fetch_landing(holding: &Holding, repository: u32, ours: Commit, merging: Option<Commit>) -> Next {
    let want = Want::Branch { branch: copy_of(pin_branch(nth(holding, repository)).expect("writable landing")) };
    match fetched(holding, repository, &want) {
        Some(commit) => Next::Step(PrepareStep::PointLanding { repository, ours, commit, merging }),
        None => Next::Step(PrepareStep::FetchLanding { repository, ours, merging }),
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
        fetch_landing(holding, repository, ours, None)
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
    if nth(holding, repository).landing.is_none() {
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

fn begin_merge(holding: &mut Holding, repository: u32, ours: Commit, merging: Option<Commit>) -> Next {
    let tips = Tips {
        start: ours,
        head: ours,
        pushed: Some(ours),
        delivered: false,
        unchecked: false,
        merging,
        design_merged: None,
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
        after_merged(repository, last)
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

fn conflicted_head_detail(holding: &Holding, repository: u32, head: Commit) -> Box<[u8]> {
    let branch = match &nth(holding, repository).start {
        Start::Merged { branch, .. } => branch,
        Start::Base { .. }
        | Start::Branch { .. }
        | Start::Commit { .. }
        | Start::Saved { .. }
        | Start::Merge { .. }
        | Start::Prepare { .. } => {
            unreachable!("only a merged start refuses a head's conflict")
        }
    };
    let prefix = b"merge: Conflicted: head ";
    let reason = b" does not merge into ";
    let length = prefix
        .len()
        .checked_add(hex_length(head))
        .expect("fixed head")
        .checked_add(reason.len())
        .expect("fixed reason")
        .checked_add(branch.len())
        .expect("admitted branch");
    let mut text = Writer::new(length);
    text.put(prefix).expect("measured prefix");
    put_hex(&mut text, head);
    text.put(reason).expect("measured reason");
    text.put(branch).expect("measured branch");
    text.finish()
}

fn after_merged(repository: u32, last: u32) -> Next {
    if repository < last { Next::Step(PrepareStep::Fetch { repository: after(repository) }) } else { Next::Ready }
}

/// The named step and git's fault, outside the checkout's facts (domain/worker/jobs.md).
fn preparation_detail(holding: &Holding, step: PrepareStep, done: &Done) -> Box<[u8]> {
    let (fault, output, conflicted) = match done {
        Done::Failed { fault } => (fault.name(), b"".as_slice(), false),
        Done::FailedWithOutput { fault, diagnostic } => (fault.name(), diagnostic.output(), false),
        Done::Conflicted { .. } => (b"Conflicted".as_slice(), b"".as_slice(), true),
        Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
        | Done::Paths { .. }
        | Done::Looked { .. }
        | Done::Brings { .. }
        | Done::IsAncestor { .. }
        | Done::Merged => {
            return Box::new([]);
        }
    };
    let step: &[u8] = match step {
        PrepareStep::Candidate { step, .. } => match step {
            candidate::Step::Head => b"fetch member",
            candidate::Step::Upstream => b"fetch upstream",
            candidate::Step::Checkout => b"check out base",
            candidate::Step::Merge => b"merge member",
            candidate::Step::Brings => b"compare upstream",
            candidate::Step::Commit { .. } => b"commit candidate",
        },
        PrepareStep::Store { .. } => b"store",
        PrepareStep::Look { .. } => b"look",
        PrepareStep::Make => b"make",
        PrepareStep::Clone { .. } => b"clone",
        PrepareStep::Fetch { .. } => b"fetch",
        PrepareStep::Default { .. } => b"fetch default",
        PrepareStep::Create { .. } => b"create base",
        PrepareStep::Refetch { .. } => b"refetch base",
        PrepareStep::CheckOut { .. } => b"check out",
        PrepareStep::FetchPush { .. } => b"fetch push",
        PrepareStep::FetchLanding { .. } => b"fetch landing",
        PrepareStep::PointLanding { .. } | PrepareStep::PinReplaced { .. } => b"point landing",
        PrepareStep::FetchBase { .. } => b"fetch base",
        PrepareStep::PointBase { .. } => b"point base",
        PrepareStep::CheckOutMerge { .. } => b"check out merge",
        PrepareStep::CompareBase { .. } => b"compare base",
        PrepareStep::CompareHead { .. } => b"compare head",
        PrepareStep::Merge { repository, theirs, head: Some(_) } if conflicted => {
            return conflicted_head_detail(holding, repository, theirs);
        }
        PrepareStep::Merge { .. } => b"merge",
        PrepareStep::CommitHead { .. } => b"commit head",
        PrepareStep::CheckOutHead { .. } => b"check out head",
        PrepareStep::CommitMerge { .. } => b"commit merge",
        PrepareStep::CheckOutMerged { .. } => b"check out merged",
    };
    let separator = if output.is_empty() { b"".as_slice() } else { b": ".as_slice() };
    let size = step
        .len()
        .checked_add(2)
        .expect("fixed step")
        .checked_add(fault.len())
        .expect("fixed fault")
        .checked_add(separator.len())
        .expect("fixed separator")
        .checked_add(output.len())
        .expect("bounded git diagnostic");
    let mut text = Writer::new(size);
    for part in [step, b": ", fault, separator, output] {
        text.put(part).expect("measured diagnostic");
    }
    text.finish()
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
    next: Next,
    detail: Box<[u8]>,
    asked: Asked,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let client = holding.client;
    match asked {
        Asked::Nothing => match next {
            Next::Step(step) => prepare_step(holding, id, step, env, out),
            Next::Stores { from, look } => stores_step(holding, cache, id, from, look, env, out),
            Next::Ready => {
                holding.fetched = List::uncapped();
                let conflicts = mem::replace(&mut holding.conflicts, List::with_capacity(0));
                let prepared = match holding.candidate.take() {
                    Some(outcome) => Prepared::Candidate { workspace: holding.workspace.token(), outcome },
                    None => Prepared::Ready { workspace: holding.workspace.token(), conflicts: conflicts.into_boxed() },
                };
                out.push(Request::Prepared { client, prepared });
                State::Ready
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

/// Asks io for the prepare's operation `step`.
#[expect(clippy::too_many_lines, reason = "each preparation step builds its operation in one table")]
fn prepare_step(
    holding: &Holding,
    id: Id<Hold>,
    step: PrepareStep,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    let workspace = holding.workspace.token();
    let op = match step {
        PrepareStep::Candidate { repository, step } => candidate::op(holding, repository, step),
        PrepareStep::Look { repository } => {
            let repository = nth(holding, repository);
            Op::Look { at: place(workspace, repository), remote: copy_of(&repository.remote) }
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
        PrepareStep::Fetch { repository } | PrepareStep::Refetch { repository } => {
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
                want: Want::Branch { branch: copy_of(pin_branch(spec).expect("writable item names its landing")) },
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
        PrepareStep::CommitHead { repository, head } => {
            commit_merge(holding, workspace, repository, merged_head(holding, repository, head))
        }
        PrepareStep::CommitMerge { repository, theirs } => commit_merge(holding, workspace, repository, theirs),
        PrepareStep::CheckOutHead { repository, .. } | PrepareStep::CheckOutMerged { repository } => Op::CheckOut {
            at: place(workspace, nth(holding, repository)),
            commit: tips(holding, repository).head,
            clean: true,
            identity: nth(holding, repository).identity,
            account: nth(holding, repository).account,
        },
    };
    io(id, op, env, out);
    State::Preparing { step, asked: Asked::Nothing }
}

fn commit_merge(holding: &Holding, workspace: Token, repository: u32, theirs: Commit) -> Op {
    let spec = nth(holding, repository);
    Op::Commit {
        at: place(workspace, spec),
        parent: tips(holding, repository).head,
        merging: Some(theirs),
        always: false,
        snapshot: None,
        title: merge_title(theirs),
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
        if let Start::Prepare { .. } = spec.start {
            let commit = tips(holding, repository).head;
            return push_commit(holding, id, push, repository, commit, env, out);
        }
        let at = place(holding.workspace.token(), spec);
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
                                    at: place(holding.workspace.token(), nth(holding, repository)),
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
                | Done::Paths { .. }
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
        failure @ (Done::Failed { .. } | Done::FailedWithOutput { .. } | Done::Conflicted { .. }) => Err(failure),
        Done::Succeeded
        | Done::Fetched { .. }
        | Done::Exists
        | Done::Rejected
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
        To::Saved => Some(tips.start),
    };
    if Some(tips.head) == base { None } else { Some(tips.head) }
}

/// A repository with nothing to push: the landing of this hold's last push to
/// its push branch, if that push landed the head, or `Unchanged`.
fn landed_before(holding: &Holding, to: &To, repository: u32) -> Landing {
    let tips = tips(holding, repository);
    match to {
        To::Branches | To::Unchecked => match tips.pushed {
            Some(commit) if commit == tips.head && tips.delivered => design::landed(tips, to),
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
    let at = place(holding.workspace.token(), spec);
    let remote = copy_of(&spec.remote);
    let identity = spec.identity;
    let account = spec.account;
    let expected = match push.to {
        To::Branches | To::Unchecked => tips(holding, repository).expected,
        To::Saved => None,
    };
    let op = if let Start::Prepare { .. } = spec.start {
        Op::Create { at, remote, commit, branch, identity, account }
    } else {
        Op::Push { at, remote, commit, branch, expected, identity, account }
    };
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
        | Done::Paths { .. }
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
    let at = place(holding.workspace.token(), spec);
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
        | Done::Paths { .. }
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
    design::landed(*tips, to)
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

fn io(id: Id<Hold>, op: Op, env: &Env<Limits>, out: &mut Queue<Request>) {
    let timeout = if op.is_remote() { env.limits.remote_timeout } else { env.limits.local_timeout };
    out.push(Request::Io { owner: id.token(), op, deadline: env.now.saturating_add(timeout) });
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
        | Done::Paths { .. }
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
        Done::Conflicted { files, .. } => Landing::Conflicted { files },
        Done::Failed { fault } => Landing::Explained { fault, diagnostic: crate::git::PushDiagnostic::empty() },
        Done::FailedWithOutput { fault, diagnostic } => Landing::Explained { fault, diagnostic },
        Done::Succeeded
        | Done::Fetched { .. }
        | Done::Committed { .. }
        | Done::Unchanged
        | Done::Exists
        | Done::Rejected
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
            Landing::Landed { .. } | Landing::DesignMerged { .. } => &mut tally.landed,
            Landing::Moved => &mut tally.moved,
            Landing::DesignConflict { .. }
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
                    && repository.push.is_some()
                    && repository.save.is_none()
                    && repository.expected.is_none()
                    && repository.landing.is_none()
                    && message_fits(&request.message, limits)
                    && message_fits(&request.squash_message, limits)
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
            Claim::Make => return prepare_step(holding, id, PrepareStep::Store { repository, look }, env, out),
            Claim::Waiting => return State::Waiting { repository, look },
        }
    }
    let step = if look { PrepareStep::Look { repository: 0 } } else { PrepareStep::Make };
    prepare_step(holding, id, step, env, out)
}
