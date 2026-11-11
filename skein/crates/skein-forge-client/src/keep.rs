//! Bounded keeping-up of live forge resources (forge/client.md, sections 3–4).
//!
//! The working set holds live rows, scan positions, inbox delivery stamps,
//! and observed heads. It never discovers resources from labels or reads
//! task policy. Admission adds a complete watch set; hints advance due
//! reads; answers update durable positions and publish changed facts;
//! departure erases rows after their pending reads settle.
pub(crate) mod answer;
pub(crate) mod echo;
pub(crate) mod poll;
pub(crate) mod restore;
mod since;
use crate::calls::{self, Owner};
use crate::domain::Alarm;
use crate::{
    Cached, Delivery, Domain, Echo, Entry, Fact, Key, Limits, LiveRecord, Position, Priority, RepositoryRecord,
    Request, Resource, Stored, Watch, What,
};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Change, Commit, Error, Op, Read, Repository};
use skein_lib::{Duration, Env, List, Map, Queue, Time, Token, usize_of};
#[derive(Hash, Debug)]
pub(crate) struct Keep {
    live: Map<Resource, Live>,
    repos: Map<Repository, Repo>,
    sequence: u64,
    ready: bool,
    failed: bool,
    fresh_active: bool,
}
#[derive(Hash, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "branch presence since start is independent of watch, writer and hint state"
)]
struct Live {
    record: LiveRecord,
    alarm: u64,
    active: bool,
    fresh: Fresh,
    writer: bool,
    presence: Presence,
    branch_present: bool,
    pushed_by: Option<(Commit, u64)>,
    read: Sent,
    hinted: bool,
    state: State,
    interval: Duration,
    refused: Refusals,
    retry: Duration,
    echo_capacity: u32,
    inbox: Inbox,
}
// Which side of this client's own push the branch read in flight was sent.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Sent {
    Current,
    // A read sent before a push this client reported answers from before
    // that push, so it is read again.
    BeforePush,
}
// Presence is live-only; a restart observes absence again.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Presence {
    Unknown,
    Present,
    Missing,
}
#[derive(Hash, Debug)]
struct Inbox {
    repair_at: Time,
    needs_repair: bool,
    reviews_needed: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum State {
    Idle,
    Due(Phase, Priority),
    Busy(Phase, Priority),
    Waiting(Phase, Priority),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Phase {
    Object,
    Pull,
    Reviews(u32, u32),
    Branch,
    BranchAncestry { before: Commit, after: Commit },
}
// A refusal belongs to a phase kind, not a scan page (forge/client.md, 4.2).
#[derive(Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each phase kind independently retains its refusal")]
struct Refusals {
    object: bool,
    pull: bool,
    reviews: bool,
    branch: bool,
    ancestry: Option<(Commit, Commit)>,
}
impl Refusals {
    const fn new() -> Self {
        Self { object: false, pull: false, reviews: false, branch: false, ancestry: None }
    }
    fn contains(&self, phase: Phase) -> bool {
        match phase {
            Phase::Object => self.object,
            Phase::Pull => self.pull,
            Phase::Reviews(_, _) => self.reviews,
            Phase::Branch => self.branch,
            Phase::BranchAncestry { before, after } => self.ancestry == Some((before, after)),
        }
    }
    fn set(&mut self, phase: Phase, refused: bool) {
        match phase {
            Phase::Object => self.object = refused,
            Phase::Pull => self.pull = refused,
            Phase::Reviews(_, _) => self.reviews = refused,
            Phase::Branch => self.branch = refused,
            Phase::BranchAncestry { before, after } => {
                self.ancestry = if refused { Some((before, after)) } else { None };
            }
        }
    }
}
#[derive(Hash, Debug)]
struct Repo {
    record: RepositoryRecord,
    active: bool,
    dropping: bool,
    fresh: Fresh,
    state: Scan,
    hinted: bool,
    heard: Option<Time>,
    retry: Duration,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Fresh {
    Pending,
    Complete,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Scan {
    Off,
    Idle,
    Due(Progress),
    Busy(Progress),
    Waiting(Progress),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Progress {
    position: crate::paging::Items,
    newest: Time,
    begun: Option<Time>,
    tied: Option<Time>,
    moved: bool,
    bootstrap: bool,
}
impl Progress {
    fn first(mark: Option<Time>) -> Progress {
        let since = mark.unwrap_or(Time::from_nanos(u64::MAX));
        Progress {
            position: crate::paging::Items::first(since),
            newest: since,
            begun: None,
            tied: None,
            moved: false,
            bootstrap: mark.is_none(),
        }
    }
}
impl Keep {
    pub(crate) fn new(l: &Limits) -> Keep {
        Keep {
            live: Map::with_capacity(l.resources),
            repos: Map::with_capacity(l.repositories),
            sequence: 0,
            ready: false,
            failed: false,
            fresh_active: false,
        }
    }
    pub(crate) fn is_ready(&self) -> bool {
        if !self.ready || self.failed {
            return false;
        }
        for (_, row) in &self.live {
            match row.state {
                State::Due(_, _) if row.active => return true,
                State::Due(_, _) | State::Idle | State::Busy(_, _) | State::Waiting(_, _) => {}
            }
        }
        for (_, row) in &self.repos {
            match row.state {
                Scan::Due(_) if row.active && !row.dropping => return true,
                Scan::Off | Scan::Due(_) | Scan::Idle | Scan::Busy(_) | Scan::Waiting(_) => {}
            }
        }
        false
    }
    pub(crate) const fn failed(&self) -> bool {
        self.failed
    }
    pub(crate) fn cached(&self, resource: &Resource) -> Option<&Cached> {
        match self.live.get(resource) {
            Some(row) if row.active => Some(&row.record.cached),
            Some(_) | None => None,
        }
    }
    pub(crate) fn live_keys(&self) -> Box<[Resource]> {
        keys(self)
    }
}
fn valid_resource(r: &Resource, l: &Limits) -> bool {
    match &r.what {
        What::Branch(name) => !name.is_empty() && name.len() <= usize_of(l.op_bytes),
        What::Repository | What::Pull(_) | What::Issue(_) => true,
    }
}
fn first(resource: &Resource) -> Phase {
    match resource.what {
        What::Branch(_) => Phase::Branch,
        What::Pull(_) | What::Issue(_) | What::Repository => Phase::Object,
    }
}
fn live(record: LiveRecord, l: &Limits, alarm: u64) -> Live {
    let phase = first(&record.watch.resource);
    let needs_repair = !record.comment_scanning;
    Live {
        record,
        alarm,
        active: true,
        fresh: Fresh::Complete,
        writer: false,
        presence: Presence::Unknown,
        branch_present: false,
        pushed_by: None,
        read: Sent::Current,
        hinted: false,
        state: State::Due(phase, Priority::Keep),
        interval: l.poll,
        refused: Refusals::new(),
        retry: Duration::ZERO,
        echo_capacity: l.rows,
        inbox: Inbox { repair_at: Time::ZERO, needs_repair, reviews_needed: false },
    }
}
fn repository(record: RepositoryRecord) -> Repo {
    Repo {
        record,
        active: true,
        dropping: false,
        fresh: Fresh::Complete,
        state: Scan::Off,
        hinted: false,
        heard: None,
        retry: Duration::ZERO,
    }
}
fn listed(k: &Keep, repository: Repository) -> bool {
    for (key, row) in &k.live {
        if row.active && key.repository == repository {
            match key.what {
                What::Pull(_) | What::Issue(_) => return true,
                What::Repository | What::Branch(_) => {}
            }
        }
    }
    false
}
fn listings(d: &mut Domain, out: &mut Queue<Request>) {
    for key in repos(&d.keep) {
        let listed = listed(&d.keep, key);
        let row = d.keep.repos.get_mut(&key).expect("repository remains");
        match row.state {
            Scan::Off if listed => row.state = Scan::Due(Progress::first(row.record.mark)),
            Scan::Idle | Scan::Due(_) | Scan::Busy(_) | Scan::Waiting(_) if !listed => {
                d.alarms.cancel(Alarm::Repository(key));
                if scan_busy(row.state) && !calls::cancel_repository(&mut d.calls, key) {
                    row.dropping = true;
                }
                row.state = Scan::Off;
                row.fresh = Fresh::Complete;
                row.hinted = false;
                row.record.mark = None;
                if row.active {
                    out.push(Request::Save { record: Stored::Repository(row.record) });
                }
            }
            Scan::Off | Scan::Idle | Scan::Due(_) | Scan::Busy(_) | Scan::Waiting(_) => {}
        }
    }
}
/// Stretch a timed read while its active repository hears (forge/client.md, 4.1).
pub(crate) fn fallen_back(k: &Keep, l: &Limits, now: Time, repository: Repository, at: Time) -> Time {
    if let Some(row) = k.repos.get(&repository)
        && row.active
        && let Some(heard) = row.heard
    {
        let until = heard.saturating_add(l.fallback);
        if now < until {
            return at.max(until);
        }
    }
    at
}
pub(crate) fn poll_at(k: &Keep, l: &Limits, now: Time, repository: Repository, interval: Duration) -> Time {
    fallen_back(k, l, now, repository, now.saturating_add(interval))
}
fn has(watches: &[Watch], resource: &Resource) -> bool {
    for watch in watches {
        if watch.resource == *resource {
            return true;
        }
    }
    false
}
fn has_repo(watches: &[Watch], repo: Repository) -> bool {
    for watch in watches {
        if watch.resource.repository == repo {
            return true;
        }
    }
    false
}
fn busy(state: State) -> bool {
    match state {
        State::Busy(_, _) => true,
        State::Idle | State::Due(_, _) | State::Waiting(_, _) => false,
    }
}
fn scan_busy(state: Scan) -> bool {
    match state {
        Scan::Busy(_) => true,
        Scan::Off | Scan::Idle | Scan::Due(_) | Scan::Waiting(_) => false,
    }
}
fn admission(k: &Keep, l: &Limits, watches: &[Watch]) -> Result<(), Error> {
    if watches.len() > usize_of(l.resources) {
        return Err(Error::TooLarge(api::Excess::exact(
            api::Bound::Forge,
            u64::from(l.resources),
            skein_lib::len_u64(watches.len()),
        )));
    }
    if !k.ready || k.failed {
        return Err(Error::Busy);
    }
    for (index, watch) in watches.iter().enumerate() {
        if !valid_resource(&watch.resource, l) {
            let excess = match &watch.resource.what {
                What::Branch(name) if name.is_empty() => api::Excess::unnamed(),
                What::Branch(name) => {
                    api::Excess::exact(api::Bound::OpBytes, u64::from(l.op_bytes), skein_lib::len_u64(name.len()))
                }
                What::Repository | What::Pull(_) | What::Issue(_) => api::Excess::unnamed(),
            };
            return Err(Error::TooLarge(excess));
        }
        if watch.participating {
            match watch.resource.what {
                What::Pull(_) | What::Issue(_) => {}
                What::Repository | What::Branch(_) => return Err(Error::Refused),
            }
        }
        for prior in watches.get(..index).expect("prior slice") {
            if prior.resource == watch.resource {
                return Err(Error::Refused);
            }
        }
    }
    let mut resources = u32::try_from(watches.len()).expect("bounded length");
    for (key, row) in &k.live {
        if !has(watches, key) && busy(row.state) {
            resources = resources.saturating_add(1);
        }
    }
    let mut repositories = 0_u32;
    for (index, watch) in watches.iter().enumerate() {
        if !has_repo(watches.get(..index).expect("prior slice"), watch.resource.repository) {
            repositories = repositories.saturating_add(1);
        }
    }
    for (key, row) in &k.repos {
        if !has_repo(watches, *key) && (scan_busy(row.state) || row.dropping) {
            repositories = repositories.saturating_add(1);
        }
    }
    if resources > l.resources || repositories > l.repositories { Err(Error::Busy) } else { Ok(()) }
}
pub(crate) fn replace(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    watches: Box<[Watch]>,
    out: &mut Queue<Request>,
) {
    if let Err(error) = admission(&d.keep, &env.limits, &watches) {
        d.fact(Fact::Refused);
        out.push(Request::Kept { owner, result: Err(error) });
        return;
    }
    remove_live(d, &watches, out);
    remove_repos(d, &watches, out);
    for watch in watches {
        let key = watch.resource.clone();
        match d.keep.live.get_mut(&key) {
            Some(row) => {
                let changed = row.record.watch.participating != watch.participating || !row.active;
                row.active = true;
                row.record.watch = watch;
                if changed {
                    out.push(Request::Save { record: Stored::Live(row.record.clone()) });
                    row.hinted = true;
                }
            }
            None => {
                let record = LiveRecord {
                    watch,
                    position: Position { at: Time::ZERO, comment: 0, review: 0, review_page: 1 },
                    listed: Time::ZERO,
                    comment_after: 0,
                    comment_page: 1,
                    comment_scanning: true,
                    comment_repair: true,
                    review_page: 1,
                    comments: Box::new([]),
                    reviews: Box::new([]),
                    cached: Box::new(Cached { item: None, pull: None, tip: None }),
                    pushed: None,
                    echoes: Box::new([]),
                };
                out.push(Request::Save { record: Stored::Live(record.clone()) });
                d.keep
                    .live
                    .insert(key.clone(), live(record, &env.limits, next_id(&mut d.keep.sequence)))
                    .expect("batch admitted");
            }
        }
        let repo = key.repository;
        match d.keep.repos.get_mut(&repo) {
            Some(row) => {
                if !row.active {
                    out.push(Request::Save { record: Stored::Repository(row.record) });
                }
                row.active = true;
            }
            None => {
                let record = RepositoryRecord { repository: repo, mark: None, clock: Time::ZERO };
                out.push(Request::Save { record: Stored::Repository(record) });
                d.keep.repos.insert(repo, repository(record)).expect("batch admitted");
            }
        }
    }
    listings(d, out);
    out.push(Request::Kept { owner, result: Ok(()) });
}
fn remove_live(d: &mut Domain, watches: &[Watch], out: &mut Queue<Request>) {
    let mut removed = List::with_capacity(d.keep.live.len());
    for key in keys(&d.keep) {
        let row = d.keep.live.get_mut(&key).expect("bounded key remains");
        let key = &key;
        if !has(watches, key) {
            if row.active {
                out.push(Request::Erase { key: Key::Live(key.clone()) });
            }
            row.active = false;
            d.alarms.cancel(Alarm::Resource(row.alarm));
            if busy(row.state) && calls::cancel_resource(&mut d.calls, key) {
                row.state = State::Idle;
            }
            if !busy(row.state) {
                removed.push(key.clone()).expect("bounded removals");
            }
        }
    }
    for key in removed.into_boxed() {
        d.keep.live.remove(&key).expect("removed row exists");
    }
}
fn remove_repos(d: &mut Domain, watches: &[Watch], out: &mut Queue<Request>) {
    let mut removed = List::with_capacity(d.keep.repos.len());
    for key in repos(&d.keep) {
        let row = d.keep.repos.get_mut(&key).expect("bounded key remains");
        let key = &key;
        if !has_repo(watches, *key) {
            if row.active {
                out.push(Request::Erase { key: Key::Repository(*key) });
            }
            row.active = false;
            d.alarms.cancel(Alarm::Repository(*key));
            if scan_busy(row.state) && calls::cancel_repository(&mut d.calls, *key) {
                row.state = Scan::Idle;
            }
            if !scan_busy(row.state) && !row.dropping {
                removed.push(*key).expect("bounded removals");
            }
        }
    }
    for key in removed.into_boxed() {
        d.keep.repos.remove(&key).expect("removed row exists");
    }
}
fn keys(k: &Keep) -> Box<[Resource]> {
    let mut keys = List::with_capacity(k.live.len());
    for (key, _) in &k.live {
        keys.push(key.clone()).expect("bounded live keys");
    }
    keys.into_boxed()
}
fn repos(k: &Keep) -> Box<[Repository]> {
    let mut keys = List::with_capacity(k.repos.len());
    for (key, _) in &k.repos {
        keys.push(*key).expect("bounded repository keys");
    }
    keys.into_boxed()
}
fn next_id(sequence: &mut u64) -> u64 {
    *sequence = sequence.checked_add(1).expect("alarm identities do not exhaust in a process");
    *sequence
}

fn priority(row: &Live) -> Priority {
    match row.state {
        State::Due(_, priority) | State::Busy(_, priority) | State::Waiting(_, priority) => priority,
        State::Idle => unreachable!("a pipeline advances while active"),
    }
}
pub(crate) fn active_resource(k: &Keep, key: &Resource) -> bool {
    match k.live.get(key) {
        Some(row) => row.active,
        None => false,
    }
}
pub(crate) fn active_repository(k: &Keep, key: Repository) -> bool {
    match k.repos.get(&key) {
        Some(row) => row.active && !row.dropping && row.state != Scan::Off,
        None => false,
    }
}

fn valid_participation(watch: &Watch) -> bool {
    match watch.resource.what {
        What::Pull(_) | What::Issue(_) => true,
        What::Repository | What::Branch(_) => !watch.participating,
    }
}

// Keeping failures wait independently of hearing (forge/client.md, 4.2).
fn retry_wait(retry: &mut Duration, l: &Limits) -> Duration {
    *retry = if *retry == Duration::ZERO { l.backoff } else { retry.saturating_mul(2).min(l.slow.max(l.backoff)) };
    *retry
}
fn taken(row: &mut Live, phase: Phase) {
    row.retry = Duration::ZERO;
    row.refused.set(phase, false);
}
