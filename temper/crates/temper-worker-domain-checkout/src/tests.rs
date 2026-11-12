//! Feed the domain events, inspect the requests that come out.

use alloc::boxed::Box;

use skein_lib::bytes::copy_of;
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall, Writer};

use crate::git::{Commit, Done, Fault, Into, Kind, Missing, Op, Place, Want};
use crate::{
    Cached, Domain, Event, Fact, Failure, Landing, Limits, Message, Outcome, Prepared, Refusal, Repository, Request,
    Spec, Start, Tally, Target, step, valid,
};

const LIMITS: Limits = Limits {
    workspaces: 2,
    preparations: 2,
    repositories: 2,
    name_bytes: 16,
    message_bytes: 64,
    conflicts: 0,
    path_bytes: 0,
    remote_timeout: Duration::from_secs(60),
    local_timeout: Duration::from_secs(10),
    facts: 64,
};

const NOW: Time = Time::from_nanos(1_000);

const MERGES: Limits = Limits { conflicts: 4, path_bytes: 32, ..LIMITS };

fn merge_spec() -> Spec {
    let mut repo = repository(b"a", true);
    repo.start = Start::Merge { branch: bytes(b"main"), base: commit(7) };
    repo.expected = Some(commit(1));
    Spec { key: bytes(b"merge"), repositories: Box::new([repo]) }
}

/// The domain, its environment and its output queue.
struct Harness {
    domain: Domain,
    env: Env<Limits>,
    out: Queue<Request>,
}

impl Harness {
    fn new(limits: Limits) -> Harness {
        assert!(valid(&limits), "the test limits are valid");
        Harness {
            domain: Domain::new(&limits),
            env: Env { now: NOW, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
        }
    }

    fn reusing(limits: Limits) -> Harness {
        let mut harness = Self::new(limits);
        harness.domain.reuse_workspaces();
        harness
    }

    /// Steps the domain with `event`, takes what it emitted, then reclaims,
    /// as the loop would at the end of the iteration.
    fn step(&mut self, event: Event) -> List<Request> {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let mut requests = List::uncapped();
        while let Some(request) = self.out.pop() {
            requests.push(request).expect("the request collection can grow");
        }
        self.domain.reclaim();
        requests
    }

    /// Steps the domain with `event`, which emits exactly one request.
    fn one(&mut self, event: Event) -> Request {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let request = self.out.pop().expect("one request");
        assert!(self.out.is_empty(), "only one: {:?} after {request:?}", self.out);
        self.domain.reclaim();
        request
    }

    /// Steps the domain with `event`, which emits nothing.
    fn none(&mut self, event: Event) {
        let requests = self.step(event);
        assert!(requests.is_empty(), "no request: {requests:?}");
    }

    /// Ends the operation in flight for `hold` with `done`, which asks for the
    /// next operation: returns it.
    fn next(&mut self, hold: Token, done: Done) -> Op {
        io(self.one(Event::Done { owner: hold, done }), hold)
    }

    /// Finish a replaced-head read and its pin, returning the start's next operation.
    fn replaced(&mut self, hold: Token, head: Commit) -> Op {
        let pin = self.next(hold, Done::Fetched { commit: head });
        let Op::Point { into: Into::Landing, commit, .. } = pin else { panic!("prepare pins the replaced head") };
        assert_eq!(commit, head);
        self.next(hold, Done::Succeeded)
    }

    /// Prepares `spec` for `client`: returns the hold and its first
    /// operation.
    fn begin(&mut self, client: u64, spec: Spec) -> (Token, Op) {
        step(&mut self.domain, &self.env, Event::Prepare { client: Token::new(client), spec }, &mut self.out);
        let Some(Request::Held { client: held, hold }) = self.out.pop() else {
            panic!("the prepare is admitted");
        };
        assert_eq!(held, Token::new(client), "the client's token is echoed");
        let op = io(self.out.pop().expect("and its first operation asked for"), hold);
        assert!(self.out.is_empty(), "nothing else");
        self.domain.reclaim();
        (hold, op)
    }

    /// Workspace cases start after making any stores their spec needs.
    fn prepare(&mut self, client: u64, spec: Spec) -> (Token, Op) {
        let (hold, mut op) = self.begin(client, spec);
        while let Op::Store { .. } = op {
            op = self.next(hold, Done::Succeeded);
        }
        (hold, op)
    }

    /// Prepares `spec` for `client`, io doing everything asked of it, each
    /// repository's starting point at `commit(place + 1)`: returns the hold.
    fn ready(&mut self, client: u64, spec: Spec) -> Token {
        let (hold, mut op) = self.prepare(client, spec);
        for _ in 0..16_u32 {
            let done = match &op {
                Op::Look { .. } => Done::Looked { holds: false },
                Op::Fetch { want: Want::Commit { commit }, .. } => Done::Fetched { commit: *commit },
                Op::Fetch { at, .. } => Done::Fetched { commit: start_of(at) },
                Op::Merge { .. } => Done::Merged,
                Op::Store { .. }
                | Op::Point { .. }
                | Op::Make { .. }
                | Op::Clone { .. }
                | Op::Create { .. }
                | Op::CheckOut { .. }
                | Op::Push { .. } => Done::Succeeded,
                Op::DiffPaths { .. } | Op::IsAncestor { .. } | Op::Commit { .. } => {
                    panic!("a prepare commits or compares nothing")
                }
            };
            match self.one(Event::Done { owner: hold, done }) {
                Request::Io { owner, op: next, deadline: _ } => {
                    assert_eq!(owner, hold, "an operation is the hold's");
                    op = next;
                }
                Request::Prepared { client: to, prepared: Prepared::Ready { .. } } => {
                    assert_eq!(to, Token::new(client), "the client's token is echoed");
                    return hold;
                }
                other @ (Request::Held { .. }
                | Request::Prepared { .. }
                | Request::Pushed { .. }
                | Request::Saved { .. }
                | Request::Released { .. }
                | Request::Cancel { .. }) => panic!("the prepare goes on: {other:?}"),
            }
        }
        panic!("the prepare ends");
    }

    /// The facts told so far.
    fn facts(&mut self) -> List<Fact> {
        let mut facts = List::with_capacity(LIMITS.facts);
        while let Some(fact) = self.domain.pop_fact() {
            facts.push(fact).expect("room for the facts");
        }
        facts
    }
}

/// The operation `request` asks for, for `hold`.
fn io(request: Request, hold: Token) -> Op {
    let Request::Io { owner, op, deadline: _ } = request else {
        panic!("an operation is asked for: {request:?}");
    };
    assert_eq!(owner, hold, "an operation is the hold's");
    op
}

fn commit(n: u8) -> Commit {
    Commit::new([n; 32])
}

/// Where the test's io starts a repository: `commit(1)` for `a`, `commit(2)`
/// for `b`.
fn start_of(at: &Place) -> Commit {
    match &*at.repository {
        b"a" => commit(1),
        b"b" => commit(2),
        other => panic!("a repository of the tests: {other:?}"),
    }
}

fn bytes(text: &[u8]) -> Box<[u8]> {
    copy_of(text)
}

/// A repository starting from the branch `main`, pushed back to it if
/// `writable`.
fn repository(name: &[u8], writable: bool) -> Repository {
    let push = if writable { Some(bytes(b"main")) } else { None };
    Repository {
        save: Some(bytes(b"saved/1")),
        git: None,
        name: bytes(name),
        remote: remote(name),
        start: Start::Branch { branch: bytes(b"main") },
        identity: 0,
        account: None,
        push,
        expected: None,
        landing: None,
    }
}

/// Where the forge has the repository `name`.
fn remote(name: &[u8]) -> Box<[u8]> {
    let mut remote = Writer::new(name.len().checked_add(6).expect("a short name"));
    remote.put(b"forge:").expect("room for the prefix");
    remote.put(name).expect("room for the name");
    remote.finish()
}

fn store_token(name: &[u8]) -> Token {
    let hash = skein_lib::sha256::digest(&remote(name));
    Token::new(u64::from_be_bytes(hash[..8].try_into().expect("eight bytes")))
}

fn spec(key: &[u8], repositories: Box<[Repository]>) -> Spec {
    Spec { key: bytes(key), repositories }
}

/// The workstream `key` with the writable `a` and the read-only `b`.
fn two(key: &[u8]) -> Spec {
    spec(key, Box::new([repository(b"a", true), repository(b"b", false)]))
}

/// The workstream `key` with the writable `a` only.
fn one(key: &[u8]) -> Spec {
    spec(key, Box::new([repository(b"a", true)]))
}

/// A queued prepare whose store is independent of the active maker's.
fn other_store(key: &[u8]) -> Spec {
    spec(key, Box::new([repository(b"b", true)]))
}

/// Drive the one-repository fixture from Make to its final `CheckOut`.
fn checkout_one(h: &mut Harness, hold: Token) -> Token {
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Clone);
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Fetch);
    let op = h.replaced(hold, commit(1));
    assert_eq!(op.kind(), Kind::CheckOut);
    workspace_of(&op)
}

fn message() -> Message {
    Message { title: bytes(b"Fix it"), body: bytes(b"Because.") }
}

fn failed(fault: Fault) -> Done {
    Done::Failed { fault }
}

fn place(workspace: Token, repository: &[u8]) -> Place {
    Place { git: None, workspace, repository: bytes(repository) }
}

fn want_main() -> Want {
    Want::Branch { branch: bytes(b"main") }
}

fn prepared(request: Request) -> Prepared {
    let Request::Prepared { client: _, prepared } = request else {
        panic!("the prepare ends: {request:?}");
    };
    prepared
}

fn landings(request: Request) -> Box<[Landing]> {
    match request {
        Request::Pushed { client: _, outcome: Outcome::Pushed { landings } }
        | Request::Saved { client: _, outcome: Outcome::Pushed { landings } } => landings,
        other @ (Request::Held { .. }
        | Request::Prepared { .. }
        | Request::Pushed { .. }
        | Request::Saved { .. }
        | Request::Released { .. }
        | Request::Io { .. }
        | Request::Cancel { .. }) => panic!("the push ends: {other:?}"),
    }
}

fn refused(request: Request) -> Refusal {
    match request {
        Request::Pushed { client: _, outcome: Outcome::Refused { refusal } }
        | Request::Saved { client: _, outcome: Outcome::Refused { refusal } }
        | Request::Prepared { client: _, prepared: Prepared::Refused { refusal } } => refusal,
        other @ (Request::Held { .. }
        | Request::Prepared { .. }
        | Request::Pushed { .. }
        | Request::Saved { .. }
        | Request::Released { .. }
        | Request::Io { .. }
        | Request::Cancel { .. }) => panic!("refused: {other:?}"),
    }
}

/// The workspace a held workspace's operations name.
fn workspace_of(op: &Op) -> Token {
    match op {
        Op::Store { .. } => panic!("a store has no workspace"),
        Op::Make { workspace, .. } => *workspace,
        Op::Clone { at, .. }
        | Op::Point { at, .. }
        | Op::Fetch { at, .. }
        | Op::Create { at, .. }
        | Op::CheckOut { at, .. }
        | Op::Look { at, .. }
        | Op::DiffPaths { at, .. }
        | Op::IsAncestor { at, .. }
        | Op::Merge { at, .. }
        | Op::Commit { at, .. }
        | Op::Push { at, .. } => at.workspace,
    }
}

#[test]
fn a_new_workspace_is_made_then_each_repository_cloned_fetched_and_checked_out() {
    let mut h = Harness::new(LIMITS);
    let (hold, op) = h.begin(7, two(b"issue-1"));
    assert_eq!(
        op,
        Op::Store {
            store: store_token(b"a"),
            aside: None,
            remote: remote(b"a"),
            identity: 0,
            account: None,
            git: None
        }
    );
    assert_eq!(
        h.next(hold, Done::Succeeded),
        Op::Store {
            store: store_token(b"b"),
            aside: None,
            remote: remote(b"b"),
            identity: 0,
            account: None,
            git: None
        }
    );
    let op = h.next(hold, Done::Succeeded);
    let Op::Make { workspace, ref stores } = op else { panic!("a new workspace is made: {op:?}") };
    assert_eq!(stores.as_ref(), &[store_token(b"a"), store_token(b"b")]);
    let identity = 0;
    let clone_a = Op::Clone { at: place(workspace, b"a"), remote: remote(b"a"), store: store_token(b"a") };
    assert_eq!(h.next(hold, Done::Succeeded), clone_a);
    let clone_b = Op::Clone { at: place(workspace, b"b"), remote: remote(b"b"), store: store_token(b"b") };
    assert_eq!(h.next(hold, Done::Succeeded), clone_b);
    let fetch_a = Op::Fetch {
        store: store_token(b"a"),
        into: Into::Fetched,
        at: place(workspace, b"a"),
        remote: remote(b"a"),
        want: want_main(),
        identity,
        account: None,
    };
    assert_eq!(h.next(hold, Done::Succeeded), fetch_a);
    let check_out_a =
        Op::CheckOut { at: place(workspace, b"a"), commit: commit(1), clean: true, identity: 0, account: None };
    assert_eq!(h.replaced(hold, commit(1)), check_out_a);
    let fetch_b = Op::Fetch {
        store: store_token(b"b"),
        into: Into::Fetched,
        at: place(workspace, b"b"),
        remote: remote(b"b"),
        want: want_main(),
        identity,
        account: None,
    };
    assert_eq!(h.next(hold, Done::Succeeded), fetch_b);
    let check_out_b =
        Op::CheckOut { at: place(workspace, b"b"), commit: commit(2), clean: true, identity: 0, account: None };
    assert_eq!(h.next(hold, Done::Fetched { commit: commit(2) }), check_out_b);
    let ready = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    assert_eq!(
        ready,
        Request::Prepared { client: Token::new(7), prepared: Prepared::Ready { workspace, conflicts: Box::new([]) } }
    );
    assert_eq!((h.domain.workspaces(), h.domain.idle(), h.domain.holds()), (1, 0, 1));
}

#[test]
fn every_operation_has_a_deadline_by_where_it_runs() {
    let local = NOW.saturating_add(LIMITS.local_timeout);
    let remote = NOW.saturating_add(LIMITS.remote_timeout);
    let mut h = Harness::new(LIMITS);
    let requests = h.step(Event::Prepare { client: Token::new(1), spec: one(b"w") });
    let Some(Request::Io { owner, op: Op::Store { .. }, deadline }) = requests.get(1) else {
        panic!("a new workspace is made");
    };
    assert_eq!(*deadline, remote, "Store reaches the forge");
    let owner = *owner;
    h.next(owner, Done::Succeeded);
    let Request::Io { deadline, op: Op::Clone { .. }, .. } = h.one(Event::Done { owner, done: Done::Succeeded }) else {
        panic!("a clone follows");
    };
    assert_eq!(deadline, local, "a clone is local");
    let Request::Io { deadline, op: Op::Fetch { .. }, .. } = h.one(Event::Done { owner, done: Done::Succeeded }) else {
        panic!("fetch follows clone")
    };
    assert_eq!(deadline, remote, "fetch reaches the forge");
    let Request::Io { deadline, op: Op::Point { .. }, .. } =
        h.one(Event::Done { owner, done: Done::Fetched { commit: commit(1) } })
    else {
        panic!("pin points at the fetched commit")
    };
    assert_eq!(deadline, local, "point is local");
    let Request::Io { deadline, op, .. } = h.one(Event::Done { owner, done: Done::Succeeded }) else {
        panic!("checkout follows pin")
    };
    assert_eq!(op.kind(), Kind::CheckOut);
    assert!(op.is_remote());
    assert_eq!(deadline, remote, "checkout may fetch contents");
    let mut h = Harness::new(MERGES);
    let hold = begin_merged(&mut h, &[commit(7)]);
    let Request::Io { deadline, op, .. } =
        h.one(Event::Done { owner: hold, done: Done::IsAncestor { contains: false } })
    else {
        panic!("merge follows ancestry")
    };
    assert_eq!(op.kind(), Kind::Merge);
    assert!(op.is_remote());
    assert_eq!(deadline, remote, "merge may fetch contents");
}

#[test]
fn a_cached_workspace_is_reused_for_its_workstream_and_fetched_afresh() {
    let mut h = Harness::new(LIMITS);
    let first = h.ready(1, two(b"issue-1"));
    assert_eq!(h.one(Event::Release { hold: first }), Request::Released { client: Token::new(1) });
    assert_eq!(h.domain.idle(), 1, "the workspace is idle in the cache");
    h.facts();
    let (hold, op) = h.prepare(2, two(b"issue-1"));
    let Op::Fetch { at, want, .. } = op else { panic!("a reused workspace is fetched at once: {op:?}") };
    assert_eq!((&*at.repository, want), (&b"a"[..], want_main()));
    assert_ne!(hold, first, "a new hold, under a name of its own");
    let facts = h.facts();
    assert_eq!(facts.get(0), Some(&Fact::Held { client: Token::new(2), cached: Cached::Reused }));
    assert_eq!(h.domain.workspaces(), 1, "the same workspace");
}

#[test]
fn a_workspace_holding_other_repositories_is_rebuilt() {
    let mut h = Harness::new(LIMITS);
    let first = h.ready(1, two(b"issue-1"));
    h.one(Event::Release { hold: first });
    h.facts();
    let (_, op) = h.prepare(2, one(b"issue-1"));
    assert_eq!(op.kind(), Kind::Make, "made again, empty");
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(2), cached: Cached::Rebuilt }));
}

#[test]
fn the_least_recently_used_idle_workspace_is_evicted_when_the_cache_is_full() {
    let mut h = Harness::new(LIMITS);
    let a = h.ready(1, one(b"a-work"));
    let b = h.ready(2, one(b"b-work"));
    assert_eq!((h.domain.workstream(0), h.domain.workstream(1)), (Some(&b"a-work"[..]), Some(&b"b-work"[..])));
    // Every workspace is held: nothing to evict.
    let refused_full = h.one(Event::Prepare { client: Token::new(3), spec: one(b"c-work") });
    assert_eq!(refused(refused_full), Refusal::Full);
    // b is released first, so it is the least recently used.
    h.one(Event::Release { hold: b });
    h.one(Event::Release { hold: a });
    h.facts();
    let (_, op) = h.prepare(3, one(b"c-work"));
    let Op::Make { workspace, .. } = op else { panic!("the evicted workspace is made again: {op:?}") };
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(3), cached: Cached::Evicted }));
    // c's is being built: it is not listed until it is cloned.
    assert_eq!((h.domain.workstream(0), h.domain.workstream(1)), (Some(&b"a-work"[..]), None));
    // a is still cached, and reused.
    let (_, op) = h.prepare(4, one(b"a-work"));
    assert_eq!(op.kind(), Kind::Fetch, "a's workspace is still cached");
    assert_ne!(workspace_of(&op), workspace, "and it is not the one evicted");
    assert_eq!(h.domain.workspaces(), h.domain.idle()), (2, 0);
}

/// A full cache reassigns its sole workspace without changing its token.
fn evict(h: &mut Harness, old: Spec, new: Spec, cached: Cached, kind: Kind) {
    let first = h.ready(1, old);
    h.one(Event::Release { hold: first });
    h.facts();
    let (_, op) = h.prepare(2, new);
    assert_eq!(workspace_of(&op), Token::new(0), "eviction retains the workspace's token");
    assert_eq!(op.kind(), kind);
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(2), cached }));
}

#[test]
fn an_evicted_matching_clone_is_kept_when_reuse_is_enabled() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    evict(&mut h, one(b"old"), one(b"new"), Cached::Kept, Kind::Fetch);
    assert_eq!(h.domain.workstream(0), Some(&b"new"[..]), "the kept clone belongs to the new workstream at once");
    assert_eq!(h.domain.workstream(1), None);
}

#[test]
fn an_evicted_matching_clone_is_made_again_by_default() {
    let mut h = Harness::new(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    evict(&mut h, one(b"old"), one(b"new"), Cached::Evicted, Kind::Make);
}

#[test]
fn an_evicted_clone_from_another_remote_is_made_again() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    let repo = Repository { remote: remote(b"b"), ..repository(b"a", true) };
    evict(&mut h, one(b"old"), spec(b"new", Box::new([repo])), Cached::Evicted, Kind::Make);
}

#[test]
fn an_evicted_clone_in_another_directory_is_made_again() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    let repo = Repository { remote: remote(b"a"), ..repository(b"b", true) };
    evict(&mut h, one(b"old"), spec(b"new", Box::new([repo])), Cached::Evicted, Kind::Make);
}

#[test]
fn an_evicted_workspace_needing_one_more_repository_is_made_again() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    evict(&mut h, one(b"old"), two(b"new"), Cached::Evicted, Kind::Make);
}

#[test]
fn an_evicted_workspace_needing_one_fewer_repository_is_made_again() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    evict(&mut h, two(b"old"), one(b"new"), Cached::Evicted, Kind::Make);
}

#[test]
fn an_evicted_clone_starting_saved_is_made_again() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    let repo = Repository { start: Start::Saved { branch: bytes(b"saved/1") }, ..repository(b"a", true) };
    evict(&mut h, one(b"old"), spec(b"new", Box::new([repo])), Cached::Evicted, Kind::Make);
}

#[test]
fn an_evicted_workspace_spoiled_by_a_broken_operation_is_made_again() {
    let mut h = Harness::reusing(Limits { workspaces: 1, preparations: 1, ..LIMITS });
    let first = h.ready(1, one(b"old"));
    h.one(Event::Release { hold: first });
    let (hold, op) = h.prepare(2, one(b"old"));
    let workspace = workspace_of(&op);
    h.one(Event::Done { owner: hold, done: Done::Failed { fault: Fault::Broken } });
    h.one(Event::Release { hold });
    h.facts();
    let (_, op) = h.prepare(3, one(b"new"));
    assert_eq!(op, Op::Make { workspace, stores: Box::new([store_token(b"a")]) });
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(3), cached: Cached::Evicted }));
}

#[test]
fn reuse_considers_only_the_least_recently_used_idle_workspace() {
    let mut h = Harness::reusing(LIMITS);
    let oldest = h.ready(1, two(b"oldest"));
    let matching = h.ready(2, one(b"matching"));
    h.one(Event::Release { hold: oldest });
    h.one(Event::Release { hold: matching });
    h.facts();
    let (_, op) = h.prepare(3, one(b"new"));
    assert_eq!(op, Op::Make { workspace: Token::new(0), stores: Box::new([store_token(b"a")]) });
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(3), cached: Cached::Evicted }));
    let (_, op) = h.prepare(4, one(b"matching"));
    assert_eq!(op.kind(), Kind::Fetch, "the newer matching workspace stays with its workstream");
}

#[test]
fn a_held_workstream_is_busy() {
    let mut h = Harness::new(LIMITS);
    h.prepare(1, one(b"w"));
    let busy = h.one(Event::Prepare { client: Token::new(2), spec: one(b"w") });
    assert_eq!(
        busy,
        Request::Prepared { client: Token::new(2), prepared: Prepared::Refused { refusal: Refusal::Busy } }
    );
    assert_eq!(h.facts().last(), Some(&Fact::Refused { client: Token::new(2), refusal: Refusal::Busy }));
}

#[test]
fn a_spec_beyond_the_limits_is_refused_at_the_entrance() {
    let long = [b'x'; 17];
    let invalid = [
        spec(b"w", Box::new([])),
        spec(b"w", Box::new([repository(b"a", true), repository(b"b", true), repository(b"c", true)])),
        spec(b"w", Box::new([repository(b"a", true), repository(b"a", false)])),
        spec(b"", Box::new([repository(b"a", true)])),
        spec(&long, Box::new([repository(b"a", true)])),
        spec(b"w", Box::new([repository(&long, true)])),
        spec(b"w", Box::new([repository(b"", true)])),
        spec(b"w", Box::new([Repository { push: Some(bytes(b"")), ..repository(b"a", true) }])),
        spec(b"w", Box::new([Repository { start: Start::Base { branch: bytes(&long) }, ..repository(b"a", true) }])),
        spec(b"w", Box::new([Repository { remote: bytes(b""), ..repository(b"a", true) }])),
        spec(b"w", Box::new([Repository { remote: bytes(&long), ..repository(b"a", true) }])),
        // Each repository's directory is one safe path component.
        spec(b"w", Box::new([repository(b".", true)])),
        spec(b"w", Box::new([repository(b"..", true)])),
        spec(b"w", Box::new([repository(b"a/b", true)])),
        spec(b"w", Box::new([repository(b"/", true)])),
        spec(b"w", Box::new([repository(b"a\0", true)])),
        spec(b"w", Box::new([repository(b".git", true)])),
        spec(b"w", Box::new([repository(b".GiT", true)])),
    ];
    let mut h = Harness::new(LIMITS);
    for spec in invalid {
        let request = h.one(Event::Prepare { client: Token::new(1), spec });
        assert_eq!(refused(request), Refusal::Invalid);
    }
    assert_eq!((h.domain.workspaces(), h.domain.holds()), (0, 0), "nothing is held");
    // At the limits, it is admitted; and a name may look like a git
    // directory's or have dots, so long as it is not one.
    let at = Repository { identity: 0, remote: bytes(&long[..16]), ..repository(&long[..16], true) };
    h.prepare(1, spec(&long[..16], Box::new([at, repository(b".gitx", false)])));
    h.prepare(2, spec(b"v", Box::new([repository(b"...", true), repository(b".git.", false)])));
}

#[test]
fn a_missing_base_branch_is_created_from_the_default_branch() {
    let mut h = Harness::new(LIMITS);
    let base = Repository { start: Start::Base { branch: bytes(b"temper/1") }, ..repository(b"a", true) };
    let (hold, _) = h.prepare(1, spec(b"w", Box::new([base])));
    h.next(hold, Done::Succeeded);
    let fetch = h.next(hold, Done::Succeeded);
    let Op::Fetch { at, want: Want::Branch { branch }, .. } = fetch else { panic!("the base branch is fetched") };
    assert_eq!(&*branch, b"temper/1");
    let default = h.next(hold, failed(Fault::Missing { missing: Missing::Branch }));
    let Op::Fetch { want: Want::Default, .. } = default else { panic!("then the default branch: {default:?}") };
    let create = h.next(hold, Done::Fetched { commit: commit(9) });
    let expected = Op::Create {
        at: at.clone(),
        remote: remote(b"a"),
        branch: bytes(b"temper/1"),
        commit: commit(9),
        identity: 0,
        account: None,
    };
    assert_eq!(create, expected, "created at the default branch's tip");
    let Op::Fetch { into: Into::Fetched, .. } = h.next(hold, Done::Succeeded) else {
        panic!("read the push branch before pinning it")
    };
    let check_out = h.replaced(hold, commit(1));
    assert_eq!(check_out, Op::CheckOut { at, commit: commit(9), clean: true, identity: 0, account: None });
    let ready = prepared(h.one(Event::Done { owner: hold, done: Done::Succeeded }));
    let Prepared::Ready { .. } = ready else { panic!("ready: {ready:?}") };
}

#[test]
fn a_base_branch_created_meanwhile_is_fetched_again_and_not_moved() {
    let mut h = Harness::new(LIMITS);
    let base = Repository { start: Start::Base { branch: bytes(b"temper/1") }, ..repository(b"a", true) };
    let (hold, _) = h.prepare(1, spec(b"w", Box::new([base])));
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    h.next(hold, failed(Fault::Missing { missing: Missing::Branch }));
    h.next(hold, Done::Fetched { commit: commit(9) });
    let refetch = h.next(hold, Done::Exists);
    let Op::Fetch { want: Want::Branch { branch }, .. } = refetch else { panic!("fetched again: {refetch:?}") };
    assert_eq!(&*branch, b"temper/1");
    h.next(hold, Done::Fetched { commit: commit(5) });
    let check_out = h.replaced(hold, commit(1));
    let Op::CheckOut { commit: at, .. } = check_out else { panic!("checked out: {check_out:?}") };
    assert_eq!(at, commit(5), "where the other party put it");
    // Gone again by the time it is fetched: transient.
    let mut h = Harness::new(LIMITS);
    let base = Repository { start: Start::Base { branch: bytes(b"temper/1") }, ..repository(b"a", true) };
    let (hold, _) = h.prepare(1, spec(b"w", Box::new([base])));
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    h.next(hold, failed(Fault::Missing { missing: Missing::Branch }));
    h.next(hold, Done::Fetched { commit: commit(9) });
    h.next(hold, Done::Exists);
    let gone = h.one(Event::Done { owner: hold, done: failed(Fault::Missing { missing: Missing::Branch }) });
    assert_eq!(
        prepared(gone),
        Prepared::Failed { failure: Failure::Transient, detail: bytes(b"refetch base: Missing(Branch)") }
    );
}

#[test]
fn a_commit_start_is_fetched_by_its_hash() {
    let mut h = Harness::new(LIMITS);
    let exact = Repository { start: Start::Commit { commit: commit(4) }, ..repository(b"a", false) };
    let (hold, _) = h.prepare(1, spec(b"w", Box::new([exact])));
    h.next(hold, Done::Succeeded);
    let fetch = h.next(hold, Done::Succeeded);
    let Op::Fetch { want: Want::Commit { commit: wanted }, .. } = fetch else { panic!("by its hash: {fetch:?}") };
    assert_eq!(wanted, commit(4));
}

#[test]
fn preparation_failures_are_typed() {
    let cases = [
        (
            2,
            failed(Fault::Missing { missing: Missing::Repository }),
            Failure::Missing { repository: 1, missing: Missing::Repository },
            b"clone: Missing(Repository)".as_slice(),
        ),
        (2, failed(Fault::Unreachable), Failure::Transient, b"clone: Unreachable".as_slice()),
        (2, failed(Fault::Refused), Failure::Refused { repository: 1 }, b"clone: Refused".as_slice()),
        (
            3,
            failed(Fault::Missing { missing: Missing::Branch }),
            Failure::Missing { repository: 0, missing: Missing::Branch },
            b"fetch: Missing(Branch)".as_slice(),
        ),
        (3, failed(Fault::TimedOut), Failure::Transient, b"fetch: TimedOut".as_slice()),
        (0, failed(Fault::Broken), Failure::Transient, b"make: Broken".as_slice()),
        (4, failed(Fault::Broken), Failure::Transient, b"point landing: Broken".as_slice()),
        (5, failed(Fault::Broken), Failure::Transient, b"check out: Broken".as_slice()),
        (
            3,
            Done::FailedWithOutput {
                fault: Fault::Unreachable,
                diagnostic: crate::git::PushDiagnostic::new(b"fatal: unreachable", 0),
            },
            Failure::Transient,
            b"fetch: Unreachable: fatal: unreachable".as_slice(),
        ),
    ];
    for (after, fault, failure, detail) in cases {
        let mut h = Harness::new(LIMITS);
        let (hold, _) = h.prepare(1, two(b"w"));
        // Make, clone a, clone b, fetch a, point its pin, check out a.
        let answers =
            [Done::Succeeded, Done::Succeeded, Done::Succeeded, Done::Fetched { commit: commit(1) }, Done::Succeeded];
        for done in answers.into_iter().take(after) {
            h.next(hold, done);
        }
        let end = h.one(Event::Done { owner: hold, done: fault });
        assert_eq!(prepared(end), Prepared::Failed { failure, detail: bytes(detail) }, "after {after} operations");
        // It may only be released.
        assert_eq!(refused(h.one(Event::Push { hold, message: message() })), Refusal::Busy);
        h.none(Event::Abort { hold });
        assert_eq!(h.one(Event::Release { hold }), Request::Released { client: Token::new(1) });
    }
}

#[test]
fn a_build_that_failed_is_rebuilt_and_a_fetch_that_failed_is_not() {
    let mut h = Harness::new(LIMITS);
    let (hold, _) = h.prepare(1, one(b"w"));
    h.next(hold, Done::Succeeded);
    h.one(Event::Done { owner: hold, done: failed(Fault::Unreachable) });
    h.one(Event::Release { hold });
    let (hold, op) = h.prepare(2, one(b"w"));
    assert_eq!(op.kind(), Kind::Make, "what a failed clone left is not known");
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    h.one(Event::Done { owner: hold, done: failed(Fault::Unreachable) });
    h.one(Event::Release { hold });
    let (_, op) = h.prepare(3, one(b"w"));
    assert_eq!(op.kind(), Kind::Fetch, "a failed fetch leaves the clones");
}

#[test]
fn a_push_commits_the_tree_on_its_start_and_pushes_it_to_the_push_branch() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, two(b"w"));
    let commit_op = io(h.one(Event::Push { hold, message: message() }), hold);
    let Op::Commit { at, parent, title, body, identity, merging: None, always: false, snapshot } = commit_op else {
        panic!("a commit: {commit_op:?}")
    };
    assert_eq!(snapshot, Some(crate::git::Snapshot { remote: remote(b"a"), account: None }));
    assert_eq!((&*at.repository, parent), (&b"a"[..], commit(1)), "the writable one, on its start");
    assert_eq!(&*title, &*body, identity), (&b"Fix it"[..], &b"Because."[..], 0);
    let push = h.next(hold, Done::Committed { commit: commit(11) });
    let expected = Op::Push {
        at,
        remote: remote(b"a"),
        commit: commit(11),
        branch: bytes(b"main"),
        expected: None,
        identity: 0,
        account: None,
    };
    assert_eq!(push, expected);
    let end = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    assert_eq!(
        end,
        Request::Pushed {
            client: Token::new(1),
            outcome: Outcome::Pushed {
                landings: Box::new([Landing::Landed { commit: commit(11) }, Landing::Unchanged])
            },
        }
    );
    let told = Tally { landed: 1, moved: 0, failed: 0, refused: 0, unchanged: 1, aborted: 0 };
    assert_eq!(h.facts().last(), Some(&Fact::Pushed { client: Token::new(1), to: Target::Push, tally: told }));
}

#[test]
fn a_push_builds_on_what_the_last_one_landed() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Committed { commit: commit(11) });
    h.one(Event::Done { owner: hold, done: Done::Succeeded });
    // The next commits on the one that landed.
    let commit_op = io(h.one(Event::Push { hold, message: message() }), hold);
    let Op::Commit { parent, .. } = commit_op else { panic!("a commit: {commit_op:?}") };
    assert_eq!(parent, commit(11));
    // With nothing new, nothing is pushed: the landing stands, as a delivery
    // followed by a finish needs.
    let end = h.one(Event::Done { owner: hold, done: Done::Unchanged });
    assert_eq!(&*landings(end), &[Landing::Landed { commit: commit(11) }]);
}

#[test]
fn a_new_hold_delivers_the_existing_change_without_another_edit() {
    let mut h = Harness::new(LIMITS);
    let first = h.ready(1, one(b"w"));
    h.one(Event::Push { hold: first, message: message() });
    h.next(first, Done::Committed { commit: commit(11) });
    assert_eq!(
        &*landings(h.one(Event::Done { owner: first, done: Done::Succeeded })),
        &[Landing::Landed { commit: commit(11) }]
    );
    h.one(Event::Release { hold: first });
    let mut retry = repository(b"a", true);
    retry.landing = Some(bytes(b"base"));
    let (second, fetch) = h.prepare(2, spec(b"w", Box::new([retry])));
    assert_ne!(first, second, "a distinct hold");
    assert_eq!(fetch.kind(), Kind::Fetch);
    let base = h.next(second, Done::Fetched { commit: commit(11) });
    let Op::Fetch { want: Want::Branch { branch }, .. } = base else { panic!("landing fetch") };
    assert_eq!(&*branch, b"base");
    h.next(second, Done::Fetched { commit: commit(1) });
    let Prepared::Ready { .. } = prepared(h.one(Event::Done { owner: second, done: Done::Succeeded })) else {
        panic!("second hold ready")
    };
    h.one(Event::Push { hold: second, message: message() });
    compares(h.next(second, Done::Unchanged), commit(11), commit(1));
    retry_push(h.next(second, Done::IsAncestor { contains: false }), commit(11), commit(11));
    assert_eq!(
        &*landings(h.one(Event::Done { owner: second, done: Done::Succeeded })),
        &[Landing::Landed { commit: commit(11) }]
    );
}

fn compares(op: Op, expected_ancestor: Commit, expected_commit: Commit) {
    let Op::IsAncestor { ancestor, commit, .. } = op else { panic!("ancestry query") };
    assert_eq!((ancestor, commit), (expected_ancestor, expected_commit));
}

fn retry_push(op: Op, head: Commit, expected: Commit) {
    let Op::Push { commit, expected: old, .. } = op else { panic!("guarded push") };
    assert_eq!((commit, old), (head, Some(expected)));
}

fn retry_repository() -> Repository {
    Repository { landing: Some(bytes(b"base")), ..repository(b"a", true) }
}

fn pin_starts() -> [Start; 5] {
    [
        Start::Base { branch: bytes(b"main") },
        Start::Merged { branch: bytes(b"main"), heads: Box::new([commit(2)]) },
        Start::Saved { branch: bytes(b"saved/1") },
        Start::Branch { branch: bytes(b"main") },
        Start::Merge { branch: bytes(b"main"), base: commit(2) },
    ]
}

/// Drive preparation only; the observed operations, not facts, count reference writes.
fn pin_ready(h: &mut Harness, client: u64, repo: Repository, tip: Commit) -> (Token, List<Op>) {
    let (hold, mut op) = h.prepare(client, spec(b"w", Box::new([repo])));
    let mut seen = List::uncapped();
    for _ in 0..32_u32 {
        let done = match &op {
            Op::Fetch { into: Into::Landing, .. } => Done::Fetched { commit: tip },
            Op::Fetch { want: Want::Commit { commit }, .. } => Done::Fetched { commit: *commit },
            Op::Fetch { .. } => Done::Fetched { commit: commit(1) },
            Op::DiffPaths { .. } => panic!("this prepare lists no delivery paths"),
            Op::IsAncestor { .. } => Done::IsAncestor { contains: false },
            Op::Merge { .. } => Done::Merged,
            Op::Commit { .. } => Done::Committed { commit: commit(3) },
            Op::Point { .. }
            | Op::Store { .. }
            | Op::Make { .. }
            | Op::Look { .. }
            | Op::Clone { .. }
            | Op::CheckOut { .. }
            | Op::Create { .. }
            | Op::Push { .. } => Done::Succeeded,
        };
        seen.push(op).expect("uncapped observations");
        match h.one(Event::Done { owner: hold, done }) {
            Request::Io { op: next, .. } => op = next,
            Request::Prepared { prepared: Prepared::Ready { .. }, .. } => return (hold, seen),
            other @ (Request::Held { .. }
            | Request::Prepared { .. }
            | Request::Pushed { .. }
            | Request::Saved { .. }
            | Request::Released { .. }
            | Request::Cancel { .. }) => panic!("preparation progresses: {other:?}"),
        }
    }
    panic!("preparation settles");
}

#[test]
fn a_goal_points_its_landing_at_its_only_fetch_of_g() {
    let mut h = Harness::new(LIMITS);
    let repo = Repository {
        start: Start::Branch { branch: bytes(b"g") },
        push: Some(bytes(b"g")),
        landing: Some(bytes(b"g")),
        ..repository(b"a", true)
    };
    let (hold, ops) = pin_ready(&mut h, 1, repo, commit(8));
    let [
        Op::Make { .. },
        Op::Clone { .. },
        Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, at, .. },
        Op::Point { into: Into::Landing, commit: pin, at: pointed },
        Op::CheckOut { commit: start, clean: true, .. },
    ] = ops.as_slice()
    else {
        panic!("one fetch, then a local pin and checkout: {ops:?}")
    };
    assert_eq!(branch.as_ref(), b"g");
    assert_eq!((*pin, *start), (commit(1), commit(1)));
    assert_eq!(at, pointed);
    // Delivery can fetch g again, but cannot move the hold's landing reference.
    let op = io(h.one(Event::Push { hold, message: message() }), hold);
    assert_eq!(op.kind(), Kind::Commit);
    let Op::Fetch { into: Into::Fetched, .. } = h.next(hold, Done::Committed { commit: commit(2) }) else {
        panic!("delivery leaves the pin frozen")
    };
}

#[test]
fn a_repair_uses_one_fetch_for_start_baseline_and_pin() {
    let mut h = Harness::new(LIMITS);
    let repo = Repository {
        start: Start::Branch { branch: bytes(b"p") },
        push: Some(bytes(b"p")),
        expected: Some(commit(1)),
        ..repository(b"a", true)
    };
    let (hold, ops) = pin_ready(&mut h, 1, repo, commit(8));
    let [
        Op::Make { .. },
        Op::Clone { .. },
        Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, .. },
        Op::Point { into: Into::Landing, commit: pin, .. },
        Op::CheckOut { commit: start, .. },
    ] = ops.as_slice()
    else {
        panic!("start, baseline and pin share one fetch: {ops:?}")
    };
    assert_eq!(branch.as_ref(), b"p");
    assert_eq!((*pin, *start), (commit(1), commit(1)));
    h.one(Event::Push { hold, message: message() });
    assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })), &[Landing::Unchanged]);
    h.one(Event::Push { hold, message: message() });
    retry_push(h.next(hold, Done::Committed { commit: commit(2) }), commit(2), commit(1));
}

#[test]
fn a_missing_push_branch_pins_the_fetched_start_locally() {
    let mut h = Harness::new(LIMITS);
    let repo = Repository { push: Some(bytes(b"new")), ..repository(b"a", true) };
    let (hold, op) = h.prepare(1, spec(b"w", Box::new([repo])));
    assert_eq!(op.kind(), Kind::Make);
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Clone);
    let Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, .. } = h.next(hold, Done::Succeeded) else {
        panic!("start fetch")
    };
    assert_eq!(branch.as_ref(), b"main");
    let Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, .. } =
        h.next(hold, Done::Fetched { commit: commit(1) })
    else {
        panic!("read the distinct push branch")
    };
    assert_eq!(branch.as_ref(), b"new");
    let Op::Point { into: Into::Landing, commit: pin, .. } =
        h.next(hold, failed(Fault::Missing { missing: Missing::Branch }))
    else {
        panic!("pin without fetching the start's commit again")
    };
    assert_eq!(pin, commit(1));
    let Op::CheckOut { commit: start, .. } = h.next(hold, Done::Succeeded) else { panic!("checkout follows pin") };
    assert_eq!(start, commit(1));
}

#[test]
fn a_merge_base_already_fetched_as_the_start_needs_no_fetch() {
    let mut h = Harness::new(MERGES);
    let repo = Repository { start: Start::Merge { branch: bytes(b"main"), base: commit(1) }, ..repository(b"a", true) };
    let (_, ops) = pin_ready(&mut h, 1, repo, commit(8));
    let [
        Op::Make { .. },
        Op::Clone { .. },
        Op::Fetch { into: Into::Fetched, .. },
        Op::Point { into: Into::Landing, .. },
        Op::Point { into: Into::Fetched, commit: base, .. },
        Op::CheckOut { .. },
        Op::Merge { theirs, .. },
    ] = ops.as_slice()
    else {
        panic!("the fetched commit is available as the merge base: {ops:?}")
    };
    assert_eq!((*base, *theirs), (commit(1), commit(1)));
}

#[test]
fn merged_heads_reuse_commits_this_prepare_fetched() {
    let mut h = Harness::new(MERGES);
    let repo = Repository {
        start: Start::Merged { branch: bytes(b"main"), heads: Box::new([commit(1), commit(2), commit(2)]) },
        ..repository(b"a", true)
    };
    let (_, ops) = pin_ready(&mut h, 1, repo, commit(8));
    let [
        Op::Make { .. },
        Op::Clone { .. },
        Op::Fetch { want: Want::Branch { .. }, .. },
        Op::Point { into: Into::Landing, .. },
        Op::Point { into: Into::Fetched, commit: first, .. },
        Op::Fetch { want: Want::Commit { commit: head }, .. },
        Op::Point { into: Into::Fetched, commit: repeated, .. },
        Op::CheckOut { commit: start, .. },
        rest @ ..,
    ] = ops.as_slice()
    else {
        panic!("only the unseen head is fetched: {ops:?}")
    };
    assert_eq!((*first, *head, *repeated, *start), (commit(1), commit(2), commit(2), commit(1)));
    for op in rest {
        assert_ne!(op.kind(), Kind::Fetch, "no repeated head fetch");
    }
}

fn landing_fetch(op: Op) {
    let Op::Fetch { into: Into::Landing, want: Want::Branch { branch }, .. } = op else { panic!("landing fetch") };
    assert_eq!(&*branch, b"base");
}

fn pin_count(ops: &List<Op>) -> usize {
    let mut count = 0_usize;
    let mut fetched = false;
    for op in ops {
        match op {
            Op::Fetch { into, .. } => {
                if !fetched {
                    assert_eq!(*into, Into::Fetched, "start before pin");
                }
                fetched = true;
                if *into == Into::Landing {
                    count = count.checked_add(1).expect("bounded prepare");
                }
            }
            Op::Point { into: Into::Landing, .. } => {
                assert!(fetched, "start before pin");
                count = count.checked_add(1).expect("bounded prepare");
            }
            Op::Make { .. }
            | Op::Store { .. }
            | Op::Look { .. }
            | Op::Clone { .. }
            | Op::Point { .. }
            | Op::Create { .. }
            | Op::CheckOut { .. }
            | Op::IsAncestor { .. }
            | Op::DiffPaths { .. }
            | Op::Merge { .. }
            | Op::Commit { .. }
            | Op::Push { .. } => {}
        }
    }
    assert!(fetched, "start fetched");
    count
}

#[test]
fn landing_comparison_is_frozen_per_hold_and_only_a_push_can_deliver_it() {
    for kind in 0..5 {
        for supplied in [false, true] {
            let mut h = Harness::new(MERGES);
            for client in [1, 2] {
                let start = pin_starts().into_iter().nth(kind).expect("start kind");
                let repo = Repository {
                    start,
                    landing: if supplied { Some(bytes(b"base")) } else { None },
                    ..repository(b"a", true)
                };
                let (hold, ops) = pin_ready(&mut h, client, repo, commit(u8::try_from(client + 7).expect("small tip")));
                assert_eq!(pin_count(&ops), 1, "start kind {kind}");
                // A real edit takes the ordinary push and ambiguous verification path.
                h.one(Event::Push { hold, message: message() });
                assert_eq!(h.next(hold, Done::Committed { commit: commit(11) }).kind(), Kind::Push);
                let verify = h.next(hold, failed(Fault::TimedOut));
                let Op::Fetch { into: Into::Fetched, .. } = verify else { panic!("verification cannot move pin") };
                assert_eq!(
                    &*landings(h.one(Event::Done { owner: hold, done: Done::Fetched { commit: commit(11) } })),
                    &[Landing::Landed { commit: commit(11) }]
                );
                h.one(Event::Push { hold, message: message() });
                assert_eq!(
                    &*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })),
                    &[Landing::Landed { commit: commit(11) }]
                );
                h.one(Event::Release { hold });
            }
        }
    }

    for (landing, contains) in [(1, true), (12, true), (7, false)] {
        let mut h = Harness::new(LIMITS);
        let (hold, _) = h.prepare(1, spec(b"w", Box::new([retry_repository()])));
        h.next(hold, Done::Succeeded);
        h.next(hold, Done::Succeeded);
        h.next(hold, Done::Fetched { commit: commit(1) });
        h.next(hold, Done::Fetched { commit: commit(landing) });
        h.one(Event::Done { owner: hold, done: Done::Succeeded });
        h.one(Event::Push { hold, message: message() });
        compares(h.next(hold, Done::Unchanged), commit(1), commit(landing));
        let end = h.one(Event::Done { owner: hold, done: Done::IsAncestor { contains } });
        if contains {
            assert_eq!(&*landings(end), &[Landing::Unchanged]);
        } else {
            retry_push(io(end, hold), commit(1), commit(1));
            // Rejection is not promoted to a receipt, including a remote rewind.
            assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Rejected })), &[Landing::Moved]);
        }
    }
}

#[test]
fn a_retry_keeps_a_stronger_expected_and_verifies_an_ambiguous_push() {
    let mut h = Harness::new(LIMITS);
    let repo = Repository { expected: Some(commit(7)), ..retry_repository() };
    let hold = h.ready(1, spec(b"w", Box::new([repo])));
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Unchanged);
    retry_push(h.next(hold, Done::IsAncestor { contains: false }), commit(1), commit(7));
    assert_eq!(h.next(hold, failed(Fault::TimedOut)).kind(), Kind::Fetch);
    assert_eq!(
        &*landings(h.one(Event::Done { owner: hold, done: Done::Fetched { commit: commit(1) } })),
        &[Landing::Landed { commit: commit(1) }]
    );
    // Saving keeps the start baseline even after retry delivery.
    h.one(Event::Save { hold, message: message() });
    assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })), &[Landing::Unchanged]);
    // A later real edit follows the normal commit/push path, with no comparison.
    h.one(Event::Push { hold, message: message() });
    retry_push(h.next(hold, Done::Committed { commit: commit(11) }), commit(11), commit(1));
}

#[test]
fn landing_fetch_failures_fail_preparation_and_never_fall_back() {
    for (fault, detail) in [
        (Fault::Missing { missing: Missing::Branch }, b"fetch landing: Missing(Branch)".as_slice()),
        (Fault::Unreachable, b"fetch landing: Unreachable".as_slice()),
        (Fault::Refused, b"fetch landing: Refused".as_slice()),
    ] {
        for start in pin_starts() {
            let mut h = Harness::new(MERGES);
            let (hold, _) = h.prepare(1, spec(b"w", Box::new([Repository { start, ..retry_repository() }])));
            h.next(hold, Done::Succeeded);
            h.next(hold, Done::Succeeded);
            let mut op = h.next(hold, Done::Fetched { commit: commit(1) });
            for _ in 0..8_u32 {
                if let Op::Fetch { into: Into::Landing, .. } = op {
                    break;
                }
                let done = match op {
                    Op::Fetch { want: Want::Commit { commit }, .. } => Done::Fetched { commit },
                    Op::CheckOut { .. } => Done::Succeeded,
                    Op::DiffPaths { .. } => panic!("this prepare lists no delivery paths"),
                    Op::IsAncestor { .. } => Done::IsAncestor { contains: false },
                    other @ (Op::Point { .. }
                    | Op::Store { .. }
                    | Op::Make { .. }
                    | Op::Look { .. }
                    | Op::Clone { .. }
                    | Op::Fetch { .. }
                    | Op::Create { .. }
                    | Op::Merge { .. }
                    | Op::Commit { .. }
                    | Op::Push { .. }) => panic!("pin must precede the run: {other:?}"),
                };
                op = h.next(hold, done);
            }
            landing_fetch(op);
            let expected = match fault {
                Fault::Missing { missing } => Failure::Missing { repository: 0, missing },
                Fault::Refused => Failure::Refused { repository: 0 },
                Fault::Unreachable | Fault::Broken | Fault::TimedOut | Fault::Cancelled => Failure::Transient,
            };
            assert_eq!(
                prepared(h.one(Event::Done { owner: hold, done: failed(fault) })),
                Prepared::Failed { failure: expected, detail: bytes(detail) }
            );
        }
    }
}

#[test]
fn supplied_landing_cannot_invent_eligibility_or_change_saved_semantics() {
    for start in pin_starts() {
        let (branch, saved, merge) = match start {
            Start::Branch { .. } => (true, false, false),
            Start::Saved { .. } => (false, true, false),
            Start::Merge { .. } | Start::Merged { .. } => (false, false, true),
            Start::Base { .. } | Start::Commit { .. } => (false, false, false),
        };
        let mut h = Harness::new(MERGES);
        let repo = Repository { start, ..retry_repository() };
        let (hold, seen) = pin_ready(&mut h, 1, repo, commit(7));
        assert_eq!(pin_count(&seen), 1);
        h.one(Event::Push { hold, message: message() });
        let next = h.one(Event::Done { owner: hold, done: Done::Unchanged });
        if branch {
            compares(io(next, hold), commit(1), commit(7));
            retry_push(h.next(hold, Done::IsAncestor { contains: false }), commit(1), commit(1));
        } else if saved || merge {
            let Op::Push { commit: pushed, .. } = io(next, hold) else { panic!("deliver saved tree or merge") };
            assert_eq!(pushed, if merge { commit(3) } else { commit(1) });
        } else {
            assert_eq!(&*landings(next), &[Landing::Unchanged], "a pin cannot make a fresh start existing work");
            continue;
        }
        let head = if merge { commit(3) } else { commit(1) };
        assert_eq!(
            &*landings(h.one(Event::Done { owner: hold, done: Done::Succeeded })),
            &[Landing::Landed { commit: head }]
        );
    }

    for start in [
        Start::Base { branch: bytes(b"main") },
        Start::Branch { branch: bytes(b"elsewhere") },
        Start::Commit { commit: commit(1) },
    ] {
        let mut h = Harness::new(MERGES);
        let repo = Repository { start, ..retry_repository() };
        let hold = h.ready(1, spec(b"w", Box::new([repo])));
        h.one(Event::Push { hold, message: message() });
        assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })), &[Landing::Unchanged]);
        h.one(Event::Save { hold, message: message() });
        assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })), &[Landing::Unchanged]);
        assert!(
            !h.facts().as_slice().contains(&Fact::Started { client: Token::new(1), op: Kind::IsAncestor }),
            "ineligible starts never compare"
        );
    }
    let mut h = Harness::new(LIMITS);
    let repo = Repository { push: None, ..retry_repository() };
    let (hold, seen) = pin_ready(&mut h, 1, repo, commit(7));
    assert_eq!(pin_count(&seen), 0, "a supplied landing cannot grant write authority");
    assert_eq!(&*landings(h.one(Event::Push { hold, message: message() })), &[Landing::Unchanged]);
}

#[test]
fn landing_branch_names_are_bounded_at_the_entrance() {
    for branch in [b"".as_slice(), &[b'x'; 17]] {
        let mut h = Harness::new(LIMITS);
        let repo = Repository { landing: Some(bytes(branch)), ..retry_repository() };
        assert_eq!(
            prepared(h.one(Event::Prepare { client: Token::new(1), spec: spec(b"w", Box::new([repo])) })),
            Prepared::Refused { refusal: Refusal::Invalid }
        );
    }
}

#[test]
fn an_unchanged_tree_has_nothing_to_push() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, two(b"w"));
    h.one(Event::Push { hold, message: message() });
    let end = h.one(Event::Done { owner: hold, done: Done::Unchanged });
    assert_eq!(&*landings(end), &[Landing::Unchanged, Landing::Unchanged]);
}

#[test]
fn a_push_that_is_not_a_fast_forward_is_moved_and_others_fail_by_kind() {
    let cases = [
        (Done::Rejected, Landing::Moved),
        (failed(Fault::Refused), explained(Fault::Refused)),
        (
            failed(Fault::Missing { missing: Missing::Repository }),
            explained(Fault::Missing { missing: Missing::Repository }),
        ),
    ];
    for (done, landing) in cases {
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        h.one(Event::Push { hold, message: message() });
        h.next(hold, Done::Committed { commit: commit(11) });
        let end = h.one(Event::Done { owner: hold, done });
        assert_eq!(&*landings(end), &[landing]);
        // The hold is ready again: the next push tries the same commit.
        h.one(Event::Push { hold, message: message() });
        let again = h.next(hold, Done::Unchanged);
        let Op::Push { commit: tried, .. } = again else { panic!("a push: {again:?}") };
        assert_eq!(tried, commit(11));
    }
    // A commit that fails is not pushed.
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.one(Event::Push { hold, message: message() });
    let end = h.one(Event::Done { owner: hold, done: failed(Fault::Broken) });
    assert_eq!(&*landings(end), &[explained(Fault::Broken)]);
}

#[test]
fn pushing_several_repositories_is_not_atomic() {
    let mut h = Harness::new(LIMITS);
    let both = spec(b"w", Box::new([repository(b"a", true), repository(b"b", true)]));
    let hold = h.ready(1, both);
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Committed { commit: commit(11) });
    let commit_b = h.next(hold, Done::Rejected);
    let Op::Commit { parent, .. } = commit_b else { panic!("b is tried: {commit_b:?}") };
    assert_eq!(parent, commit(2));
    h.next(hold, Done::Committed { commit: commit(12) });
    let end = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    assert_eq!(&*landings(end), &[Landing::Moved, Landing::Landed { commit: commit(12) }]);
}

#[test]
fn a_save_pushes_to_the_saved_work_branch_what_changed_since_the_start() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    // Nothing changed: nothing to save.
    h.one(Event::Save { hold, message: message() });
    let end = h.one(Event::Done { owner: hold, done: Done::Unchanged });
    assert_eq!(
        end,
        Request::Saved { client: Token::new(1), outcome: Outcome::Pushed { landings: Box::new([Landing::Unchanged]) } }
    );
    h.one(Event::Save { hold, message: message() });
    let push = h.next(hold, Done::Committed { commit: commit(11) });
    let Op::Push { commit: pushed, branch, .. } = push else { panic!("a push: {push:?}") };
    assert_eq!((pushed, &*branch), (commit(11), &b"saved/1"[..]));
    let end = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    assert_eq!(&*landings(end), &[Landing::Landed { commit: commit(11) }]);
    // Saving leaves the push branch as it was: a push still has it to push.
    h.one(Event::Push { hold, message: message() });
    let push = h.next(hold, Done::Unchanged);
    let Op::Push { commit: pushed, branch, .. } = push else { panic!("a push: {push:?}") };
    assert_eq!((pushed, &*branch), (commit(11), &b"main"[..]));
}

#[test]
fn a_push_or_a_save_beyond_the_limits_is_refused() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    let long = Message { title: bytes(b"t"), body: bytes(&[b'x'; 64]) };
    assert_eq!(refused(h.one(Event::Push { hold, message: long })), Refusal::Invalid);
    let untitled = Message { title: bytes(b""), body: bytes(b"x") };
    assert_eq!(refused(h.one(Event::Push { hold, message: untitled })), Refusal::Invalid);
    let save = Event::Save { hold, message: Message { title: bytes(b"t"), body: bytes(&[b'x'; 64]) } };
    assert_eq!(refused(h.one(save)), Refusal::Invalid);
    // Still ready.
    assert_eq!(io(h.one(Event::Push { hold, message: message() }), hold).kind(), Kind::Commit);
}

#[test]
fn a_push_while_another_operation_is_under_way_is_refused() {
    let mut h = Harness::new(LIMITS);
    let (hold, _) = h.prepare(1, one(b"w"));
    assert_eq!(refused(h.one(Event::Push { hold, message: message() })), Refusal::Busy);
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.one(Event::Push { hold, message: message() });
    let save = Event::Save { hold, message: message() };
    assert_eq!(refused(h.one(save)), Refusal::Busy);
    // The push goes on.
    assert_eq!(h.next(hold, Done::Committed { commit: commit(11) }).kind(), Kind::Push);
}

#[test]
fn an_abort_ends_a_prepare_once_its_operation_has_settled() {
    for done in [failed(Fault::Cancelled), Done::Succeeded] {
        let mut h = Harness::new(LIMITS);
        let (hold, _) = h.prepare(1, two(b"w"));
        h.next(hold, Done::Succeeded);
        assert_eq!(h.one(Event::Abort { hold }), Request::Cancel { owner: hold });
        h.none(Event::Abort { hold });
        // Whichever won the race, nothing more is asked of io.
        assert_eq!(prepared(h.one(Event::Done { owner: hold, done })), Prepared::Aborted);
        assert_eq!(refused(h.one(Event::Push { hold, message: message() })), Refusal::Busy);
        assert_eq!(h.one(Event::Release { hold }), Request::Released { client: Token::new(1) });
        // What the build left is not known.
        let (_, op) = h.prepare(2, two(b"w"));
        assert_eq!(op.kind(), Kind::Make);
    }
}

#[test]
fn an_abort_mid_push_waits_for_the_push_keeps_what_landed_and_reports_the_rest_aborted() {
    let mut h = Harness::new(LIMITS);
    let both = spec(b"w", Box::new([repository(b"a", true), repository(b"b", true)]));
    let hold = h.ready(1, both);
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Committed { commit: commit(11) });
    // The push in flight is not cancelled.
    h.none(Event::Abort { hold });
    h.none(Event::Abort { hold });
    let end = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    assert_eq!(&*landings(end), &[Landing::Landed { commit: commit(11) }, Landing::Aborted]);
    // Ready again: a save goes ahead.
    let save = h.one(Event::Save { hold, message: message() });
    assert_eq!(io(save, hold).kind(), Kind::Commit);
    // An abort while committing pushes nothing, whether the commit was made
    // or cancelled.
    h.none(Event::Abort { hold });
    let end = h.one(Event::Done { owner: hold, done: Done::Committed { commit: commit(13) } });
    assert_eq!(&*landings(end), &[Landing::Aborted, Landing::Aborted]);
    h.one(Event::Push { hold, message: message() });
    h.none(Event::Abort { hold });
    let end = h.one(Event::Done { owner: hold, done: failed(Fault::Cancelled) });
    assert_eq!(&*landings(end), &[Landing::Aborted, Landing::Aborted]);
}

#[test]
fn a_push_that_may_have_landed_is_verified() {
    let cases = [
        (failed(Fault::TimedOut), Done::Fetched { commit: commit(11) }, Landing::Landed { commit: commit(11) }),
        (failed(Fault::Broken), Done::Fetched { commit: commit(1) }, explained(Fault::Broken)),
        (failed(Fault::Unreachable), failed(Fault::Unreachable), explained(Fault::Unreachable)),
        (failed(Fault::TimedOut), failed(Fault::Missing { missing: Missing::Branch }), explained(Fault::TimedOut)),
    ];
    for (ended, found, landing) in cases {
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        h.one(Event::Push { hold, message: message() });
        h.next(hold, Done::Committed { commit: commit(11) });
        // Verified even when the client asked to abort meanwhile.
        h.none(Event::Abort { hold });
        let verify = h.next(hold, ended);
        let Op::Fetch { at, remote: from, want: Want::Branch { branch }, .. } = verify else {
            panic!("the push branch is fetched: {verify:?}");
        };
        assert_eq!((&*at.repository, &*from, &*branch), (&b"a"[..], &*remote(b"a"), &b"main"[..]));
        let end = h.one(Event::Done { owner: hold, done: found });
        assert_eq!(&*landings(end), &[landing]);
    }
    // One that landed is the base of the next push, which with nothing new
    // is that landing again.
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Committed { commit: commit(11) });
    h.next(hold, failed(Fault::TimedOut));
    h.one(Event::Done { owner: hold, done: Done::Fetched { commit: commit(11) } });
    h.one(Event::Push { hold, message: message() });
    let end = h.one(Event::Done { owner: hold, done: Done::Unchanged });
    assert_eq!(&*landings(end), &[Landing::Landed { commit: commit(11) }]);
    // A push refused, or not a fast-forward, is not verified.
    for (ended, landing) in [(failed(Fault::Refused), explained(Fault::Refused)), (Done::Rejected, Landing::Moved)] {
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        h.one(Event::Push { hold, message: message() });
        h.next(hold, Done::Committed { commit: commit(11) });
        let end = h.one(Event::Done { owner: hold, done: ended });
        assert_eq!(&*landings(end), &[landing]);
    }
}

#[test]
fn an_operation_that_broke_ran_out_of_time_or_was_cancelled_leaves_the_workspace_untrusted() {
    for fault in [Fault::Broken, Fault::TimedOut, Fault::Cancelled, Fault::Unreachable, Fault::Refused] {
        // A fetch, in a workspace already cloned.
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        h.one(Event::Release { hold });
        let (hold, _) = h.prepare(2, one(b"w"));
        h.one(Event::Done { owner: hold, done: failed(fault) });
        h.one(Event::Release { hold });
        let (_, op) = h.prepare(3, one(b"w"));
        let damaged = match fault {
            Fault::Broken | Fault::TimedOut | Fault::Cancelled => true,
            Fault::Missing { .. } | Fault::Refused | Fault::Unreachable => false,
        };
        let expected = if damaged { Kind::Make } else { Kind::Fetch };
        assert_eq!(op.kind(), expected, "after a fetch that ended {fault:?}");
        // A commit.
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        h.one(Event::Push { hold, message: message() });
        h.one(Event::Done { owner: hold, done: failed(fault) });
        h.one(Event::Release { hold });
        let (_, op) = h.prepare(2, one(b"w"));
        assert_eq!(op.kind(), expected, "after a commit that ended {fault:?}");
    }
}

#[test]
fn a_missing_base_branch_of_a_read_only_repository_is_missing() {
    let mut h = Harness::new(LIMITS);
    let base = Repository { start: Start::Base { branch: bytes(b"temper/1") }, ..repository(b"a", false) };
    let (hold, _) = h.prepare(1, spec(b"w", Box::new([base])));
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    let end = h.one(Event::Done { owner: hold, done: failed(Fault::Missing { missing: Missing::Branch }) });
    assert_eq!(
        prepared(end),
        Prepared::Failed {
            failure: Failure::Missing { repository: 0, missing: Missing::Branch },
            detail: bytes(b"fetch: Missing(Branch)")
        }
    );
}

#[test]
fn a_workspace_cloned_from_another_remote_is_rebuilt() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.one(Event::Release { hold });
    let moved = Repository { remote: remote(b"elsewhere"), ..repository(b"a", true) };
    let (_, op) = h.prepare(2, spec(b"w", Box::new([moved])));
    assert_eq!(op.kind(), Kind::Make, "the same directory, but another repository");
}

#[test]
fn only_workspaces_whose_disk_is_known_are_listed() {
    let mut h = Harness::new(LIMITS);
    let (hold, _) = h.prepare(1, one(b"w"));
    assert_eq!(h.domain.workstream(0), None, "being built");
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    assert_eq!(h.domain.workstream(0), Some(&b"w"[..]), "cloned");
    h.one(Event::Done { owner: hold, done: failed(Fault::TimedOut) });
    assert_eq!(h.domain.workstream(0), None, "damaged");
    h.one(Event::Release { hold });
    h.ready(2, one(b"v"));
    assert_eq!((h.domain.workstream(0), h.domain.workstream(1)), (Some(&b"v"[..]), None));
}

#[test]
fn a_release_under_way_aborts_first_and_releases_once_settled() {
    let mut h = Harness::new(LIMITS);
    let (hold, _) = h.prepare(1, one(b"w"));
    // An abort, then a release: one cancel.
    assert_eq!(h.one(Event::Abort { hold }), Request::Cancel { owner: hold });
    h.none(Event::Release { hold });
    h.none(Event::Release { hold });
    let requests = h.step(Event::Done { owner: hold, done: failed(Fault::Cancelled) });
    let expected = [
        Request::Prepared { client: Token::new(1), prepared: Prepared::Aborted },
        Request::Released { client: Token::new(1) },
    ];
    assert_eq!(requests.as_slice(), &expected);
    assert_eq!((h.domain.holds(), h.domain.idle()), (0, 1), "reclaimed, and the workspace idle");
    // A release while pushing waits for the push.
    let hold = h.ready(2, one(b"w"));
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Committed { commit: commit(11) });
    h.none(Event::Release { hold });
    let requests = h.step(Event::Done { owner: hold, done: Done::Succeeded });
    let landed = Box::new([Landing::Landed { commit: commit(11) }]);
    let expected = [
        Request::Pushed { client: Token::new(2), outcome: Outcome::Pushed { landings: landed } },
        Request::Released { client: Token::new(2) },
    ];
    assert_eq!(requests.as_slice(), &expected);
}

#[test]
fn a_released_hold_is_stale() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.one(Event::Release { hold });
    h.none(Event::Push { hold, message: message() });
    h.none(Event::Save { hold, message: message() });
    h.none(Event::Abort { hold });
    h.none(Event::Release { hold });
    // Not even once the workspace is held again.
    let again = h.ready(2, one(b"w"));
    assert_ne!(again, hold);
    h.none(Event::Release { hold });
    h.none(Event::Push { hold, message: message() });
}

#[test]
fn an_abort_with_nothing_under_way_does_nothing() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"w"));
    h.none(Event::Abort { hold });
    assert_eq!(io(h.one(Event::Push { hold, message: message() }), hold).kind(), Kind::Commit);
}

#[test]
fn the_facts_tell_what_happened_and_drop_what_does_not_fit() {
    let mut h = Harness::new(LIMITS);
    h.facts();
    let (hold, _) = h.prepare(1, one(b"w"));
    h.next(hold, Done::Succeeded);
    let client = Token::new(1);
    let facts = h.facts();
    let expected = [
        Fact::Held { client, cached: Cached::New },
        Fact::Started { client, op: Kind::Store },
        Fact::Ended { client, done: crate::Ending::Succeeded },
        Fact::Started { client, op: Kind::Make },
        Fact::Ended { client, done: crate::Ending::Succeeded },
        Fact::Started { client, op: Kind::Clone },
    ];
    assert_eq!(facts.as_slice(), &expected);
    let mut h = Harness::new(Limits { facts: 1, ..LIMITS });
    h.begin(1, one(b"w"));
    assert_eq!(h.domain.facts_lost(), 1);
}

#[test]
fn limits_require_workspaces_repositories_and_representable_hold_slots() {
    assert!(valid(&LIMITS));
    assert!(!valid(&Limits { workspaces: 0, ..LIMITS }));
    assert!(!valid(&Limits { preparations: 0, ..LIMITS }));
    assert!(!valid(&Limits { repositories: 0, ..LIMITS }));
    assert!(valid(&Limits { workspaces: u32::MAX / 2, ..LIMITS }));
    assert!(!valid(&Limits { workspaces: u32::MAX / 2 + 1, ..LIMITS }));
}

fn explained(fault: Fault) -> Landing {
    Landing::Explained { fault, diagnostic: crate::git::PushDiagnostic::empty() }
}

#[test]
fn failed_git_output_is_bounded_and_survives_push_verification() {
    let prefix = [b'x'; crate::git::PushDiagnostic::CAPACITY + 32];
    let suffix = b"connection reset after sending commit";
    let mut writer = Writer::new(prefix.len() + suffix.len());
    writer.put(&prefix).expect("the prefix fits");
    writer.put(suffix).expect("the suffix fits");
    let output = writer.finish();
    let diagnostic = crate::git::PushDiagnostic::new(&output, 7);
    assert_eq!(diagnostic.output().len(), crate::git::PushDiagnostic::CAPACITY);
    assert!(diagnostic.output().ends_with(b"connection reset after sending commit"));
    assert_eq!(diagnostic.cut(), 7 + u64::try_from(output.len() - diagnostic.output().len()).expect("small test"));
    for fault in [Fault::Broken, Fault::Unreachable, Fault::TimedOut, Fault::Cancelled] {
        for verification in [Done::Fetched { commit: commit(1) }, failed(Fault::Unreachable)] {
            let mut h = Harness::new(LIMITS);
            let hold = h.ready(1, one(b"w"));
            h.one(Event::Push { hold, message: message() });
            h.next(hold, Done::Committed { commit: commit(11) });
            h.next(hold, Done::FailedWithOutput { fault, diagnostic });
            let end = h.one(Event::Done { owner: hold, done: verification });
            assert_eq!(&*landings(end), &[Landing::Explained { fault, diagnostic }]);
        }
    }
}

#[test]
fn failed_commit_or_refused_push_returns_git_diagnostics() {
    let diagnostic = crate::git::PushDiagnostic::new(b"remote: protected branch hook rejected change", 0);
    for commit_failed in [true, false] {
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        h.one(Event::Push { hold, message: message() });
        if !commit_failed {
            h.next(hold, Done::Committed { commit: commit(11) });
        }
        let fault = if commit_failed { Fault::Broken } else { Fault::Refused };
        let end = h.one(Event::Done { owner: hold, done: Done::FailedWithOutput { fault, diagnostic } });
        assert_eq!(&*landings(end), &[Landing::Explained { fault, diagnostic }]);
    }
}

#[test]
fn merge_preparation_transfers_conflict_paths_and_facts_keep_only_counts() {
    let mut h = Harness::new(MERGES);
    let (hold, _) = h.prepare(1, merge_spec());
    h.next(hold, Done::Succeeded);
    let fetch = h.next(hold, Done::Succeeded);
    let Op::Fetch { want: Want::Branch { .. }, .. } = fetch else { panic!("branch fetch") };
    let base = h.replaced(hold, commit(1));
    let Op::Fetch { want: Want::Commit { commit: c }, .. } = base else { panic!("base fetch") };
    assert_eq!(c, commit(7));
    let Op::CheckOut { commit: c, .. } = h.next(hold, Done::Fetched { commit: commit(7) }) else {
        panic!("checkout branch")
    };
    assert_eq!(c, commit(1));
    let Op::Merge { theirs, .. } = h.next(hold, Done::Succeeded) else { panic!("merge") };
    assert_eq!(theirs, commit(7));
    let files = Box::new([bytes(b"src/a"), bytes(b"src/b")]);
    let ready = h.one(Event::Done { owner: hold, done: Done::Conflicted { files } });
    let Request::Prepared { prepared: Prepared::Ready { workspace, conflicts }, .. } = ready else {
        panic!("ready with conflicts")
    };
    assert_eq!(&*conflicts, &[crate::Conflicts { repository: 0, files: Box::new([bytes(b"src/a"), bytes(b"src/b")]) }]);
    let facts = h.facts();
    assert!(
        facts.as_slice().contains(&Fact::Ended { client: Token::new(1), done: crate::Ending::Conflicted { files: 2 } })
    );
    assert!(facts.as_slice().contains(&Fact::Prepared {
        client: Token::new(1),
        prepared: crate::Preparation::Ready { workspace, conflicts: 2 }
    }));
}

#[test]
fn merge_marker_refusal_preserves_second_parent_until_commit_and_verification_advances_expected_head() {
    let mut h = Harness::new(MERGES);
    let hold = h.ready(1, merge_spec());
    let asked = io(h.one(Event::Push { hold, message: message() }), hold);
    let Op::Commit { parent, merging, .. } = asked else { panic!("merge commit") };
    assert_eq!((parent, merging), (commit(1), Some(commit(7))));
    let end = h.one(Event::Done { owner: hold, done: Done::Conflicted { files: Box::new([bytes(b"src/a")]) } });
    assert_eq!(
        end,
        Request::Pushed {
            client: Token::new(1),
            outcome: Outcome::Pushed {
                landings: Box::new([Landing::Conflicted { files: Box::new([bytes(b"src/a")]) }])
            }
        }
    );
    let retried = io(h.one(Event::Push { hold, message: message() }), hold);
    let Op::Commit { merging, .. } = retried else { panic!("retry merge commit") };
    assert_eq!(merging, Some(commit(7)));
    let push = h.next(hold, Done::Committed { commit: commit(9) });
    let Op::Push { expected, .. } = push else { panic!("conditional push") };
    assert_eq!(expected, Some(commit(1)));
    let Op::Fetch { want: Want::Branch { .. }, .. } = h.next(hold, failed(Fault::TimedOut)) else {
        panic!("verification")
    };
    let landed = h.one(Event::Done { owner: hold, done: Done::Fetched { commit: commit(9) } });
    assert_eq!(
        landed,
        Request::Pushed {
            client: Token::new(1),
            outcome: Outcome::Pushed { landings: Box::new([Landing::Landed { commit: commit(9) }]) }
        }
    );
    let next = io(h.one(Event::Push { hold, message: message() }), hold);
    let Op::Commit { merging, parent, .. } = next else { panic!("ordinary commit") };
    assert_eq!((parent, merging), (commit(9), None));
    let next = h.next(hold, Done::Committed { commit: commit(10) });
    let Op::Push { expected, .. } = next else { panic!("next conditional push") };
    assert_eq!(expected, Some(commit(9)));
}

#[test]
fn merge_starts_require_configured_path_room_and_missing_branches_are_never_created() {
    let mut h = Harness::new(LIMITS);
    assert_eq!(
        h.one(Event::Prepare { client: Token::new(1), spec: merge_spec() }),
        Request::Prepared { client: Token::new(1), prepared: Prepared::Refused { refusal: Refusal::Invalid } }
    );
    let mut h = Harness::new(MERGES);
    let (hold, _) = h.prepare(1, merge_spec());
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    assert_eq!(
        h.one(Event::Done { owner: hold, done: failed(Fault::Missing { missing: Missing::Branch }) }),
        Request::Prepared {
            client: Token::new(1),
            prepared: Prepared::Failed {
                failure: Failure::Missing { repository: 0, missing: Missing::Branch },
                detail: bytes(b"fetch: Missing(Branch)")
            }
        }
    );
}

#[test]
fn oversized_or_duplicate_terminal_paths_are_dropped_without_retention() {
    let cases: [Box<[Box<[u8]>]>; 2] =
        [Box::new([bytes(b"same"), bytes(b"same")]), Box::new([bytes(b"012345678901234567890123456789012")])];
    for files in cases {
        let mut h = Harness::new(MERGES);
        let hold = h.ready(1, merge_spec());
        h.one(Event::Push { hold, message: message() });
        let end = h.one(Event::Done { owner: hold, done: Done::Conflicted { files } });
        let Request::Pushed { outcome: Outcome::Pushed { landings }, .. } = end else { panic!("ended") };
        let Landing::Explained { fault: Fault::Broken, .. } = &landings[0] else { panic!("broken terminal") };
    }
}

#[test]
fn releasing_an_inflight_merge_waits_for_its_terminal_and_never_exposes_a_ready_workspace() {
    for done in [Done::Merged, Done::Conflicted { files: Box::new([bytes(b"src/a")]) }, failed(Fault::Cancelled)] {
        let mut h = Harness::new(MERGES);
        let (hold, _) = h.prepare(1, merge_spec());
        h.next(hold, Done::Succeeded);
        h.next(hold, Done::Succeeded);
        h.replaced(hold, commit(1));
        h.next(hold, Done::Fetched { commit: commit(7) });
        let Op::Merge { .. } = h.next(hold, Done::Succeeded) else { panic!("merge in flight") };
        assert_eq!(h.one(Event::Release { hold }), Request::Cancel { owner: hold });
        h.none(Event::Release { hold });
        assert_eq!(h.domain.holds(), 1, "the process can still touch its workspace");
        let requests = h.step(Event::Done { owner: hold, done });
        assert_eq!(
            requests.as_slice(),
            &[
                Request::Prepared { client: Token::new(1), prepared: Prepared::Aborted },
                Request::Released { client: Token::new(1) }
            ]
        );
        assert_eq!(h.domain.holds(), 0);
        h.none(Event::Release { hold });
    }
}

#[test]
fn a_moved_head_refusal_preserves_the_original_condition_on_retry() {
    let mut h = Harness::new(MERGES);
    let hold = h.ready(1, merge_spec());
    h.one(Event::Push { hold, message: message() });
    let Op::Push { expected, .. } = h.next(hold, Done::Committed { commit: commit(9) }) else {
        panic!("conditional push")
    };
    assert_eq!(expected, Some(commit(1)));
    assert_eq!(landings(h.one(Event::Done { owner: hold, done: Done::Rejected })).as_ref(), &[Landing::Moved]);
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("retry retains the local merge commit")
    };
    assert_eq!((parent, merging), (commit(9), None));
    let Op::Push { expected, commit: head, .. } = h.next(hold, Done::Unchanged) else {
        panic!("retry of unlanded commit")
    };
    assert_eq!((head, expected), (commit(9), Some(commit(1))));
}

#[test]
fn each_git_op_carries_its_repositorys_account() {
    for account in [Some(22), None] {
        let mut h = Harness::new(LIMITS);
        let repo = Repository { account, start: Start::Base { branch: bytes(b"topic") }, ..repository(b"a", true) };
        let (hold, make) = h.prepare(1, spec(b"w", Box::new([repo])));
        let Op::Make { .. } = make else { panic!("workspace made first") };
        assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Clone);
        assert_account(h.next(hold, Done::Succeeded), account, Kind::Fetch);
        assert_account(h.next(hold, failed(Fault::Missing { missing: Missing::Branch })), account, Kind::Fetch);
        assert_account(h.next(hold, Done::Fetched { commit: commit(1) }), account, Kind::Create);
        assert_account(h.next(hold, Done::Exists), account, Kind::Fetch);
        assert_account(h.next(hold, Done::Fetched { commit: commit(1) }), account, Kind::Fetch);
        assert_eq!(h.next(hold, Done::Fetched { commit: commit(1) }).kind(), Kind::Point);
        let Op::CheckOut { .. } = h.next(hold, Done::Succeeded) else { panic!("checkout follows pin") };
        let Request::Prepared { .. } = h.one(Event::Done { owner: hold, done: Done::Succeeded }) else {
            panic!("prepared after checkout")
        };
        let op = io(h.one(Event::Push { hold, message: message() }), hold);
        let Op::Commit { .. } = op else { panic!("commit precedes push") };
        assert_account(h.next(hold, Done::Committed { commit: commit(2) }), account, Kind::Push);
        assert_account(h.next(hold, failed(Fault::TimedOut)), account, Kind::Fetch);
        h.one(Event::Done { owner: hold, done: Done::Fetched { commit: commit(2) } });
    }
}
fn assert_account(op: Op, expected: Option<u64>, kind: Kind) {
    assert_eq!(op.kind(), kind);
    match op {
        Op::Fetch { account, .. } | Op::Create { account, .. } | Op::Push { account, .. } => {
            assert_eq!(account, expected);
        }
        Op::Point { .. }
        | Op::Store { .. }
        | Op::Clone { .. }
        | Op::Look { .. }
        | Op::DiffPaths { .. }
        | Op::IsAncestor { .. }
        | Op::Make { .. }
        | Op::CheckOut { .. }
        | Op::Merge { .. }
        | Op::Commit { .. } => {
            panic!("an operation that authenticates")
        }
    }
}

fn prepare_with_save(length: u32) -> List<Request> {
    let mut spec = one(b"w");
    spec.repositories[0].save = Some(skein_lib::bytes::zeroed(usize::try_from(length).expect("bounded")));
    Harness::new(LIMITS).step(Event::Prepare { client: Token::new(1), spec })
}

#[test]
fn a_saved_work_destination_at_its_name_bound_is_admitted() {
    let requests = prepare_with_save(LIMITS.name_bytes);
    let [Request::Held { .. }, Request::Io { .. }] = requests.as_slice() else { panic!("admitted: {requests:?}") };
}

#[test]
fn a_saved_work_destination_past_its_name_bound_is_refused() {
    assert_eq!(
        prepare_with_save(LIMITS.name_bytes + 1).as_slice(),
        &[Request::Prepared { client: Token::new(1), prepared: Prepared::Refused { refusal: Refusal::Invalid } }]
    );
}

fn fetching_saved_delivery() -> (Harness, Token) {
    let mut h = Harness::new(LIMITS);
    let mut spec = one(b"w");
    spec.repositories[0].start = Start::Saved { branch: bytes(b"saved/1") };
    let (hold, _) = h.prepare(1, spec);
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Clone);
    let Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, .. } = h.next(hold, Done::Succeeded) else {
        panic!("saved branch fetched first")
    };
    assert_eq!(branch.as_ref(), b"saved/1");
    let Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, .. } =
        h.next(hold, Done::Fetched { commit: commit(9) })
    else {
        panic!("one fetch of the distinct delivery branch")
    };
    assert_eq!(branch.as_ref(), b"main");
    (h, hold)
}

#[test]
fn a_saved_producer_delivers_before_its_delivery_branch_exists() {
    let (mut h, hold) = fetching_saved_delivery();
    let op = h.next(hold, failed(Fault::Missing { missing: Missing::Branch }));
    let Op::Point { into: Into::Landing, commit: c, .. } = op else {
        panic!("pin the saved start when the push branch is missing")
    };
    assert_eq!(c, commit(9));
    let op = h.next(hold, Done::Succeeded);
    let Op::CheckOut { clean, .. } = op else { panic!("saved checkout") };
    assert!(!clean);
    h.one(Event::Done { owner: hold, done: Done::Succeeded });
    h.one(Event::Push { hold, message: message() });
    let op = h.next(hold, Done::Unchanged);
    let Op::Push { commit: head, branch, expected, .. } = op else { panic!("restored tree has a push") };
    assert_eq!((head, branch.as_ref(), expected), (commit(9), b"main".as_slice(), None));
}

#[test]
fn a_saved_head_already_on_its_push_branch_delivers_with_a_guard() {
    let mut h = Harness::new(LIMITS);
    let repo = Repository { start: Start::Saved { branch: bytes(b"saved") }, ..retry_repository() };
    let hold = h.ready(1, spec(b"w", Box::new([repo])));
    h.one(Event::Push { hold, message: message() });
    let op = h.next(hold, Done::Unchanged);
    let Op::Push { commit: head, expected, .. } = op else { panic!("saved work is pushed before returning a landing") };
    assert_eq!((head, expected), (commit(1), Some(commit(1))));
    assert_eq!(
        &*landings(h.one(Event::Done { owner: hold, done: Done::Succeeded })),
        &[Landing::Landed { commit: commit(1) }]
    );
    assert!(
        !h.facts().as_slice().contains(&Fact::Started { client: Token::new(1), op: Kind::IsAncestor }),
        "a saved start uses no producer comparison"
    );
    h.one(Event::Push { hold, message: message() });
    assert_eq!(
        &*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })),
        &[Landing::Landed { commit: commit(1) }]
    );
}

#[test]
fn a_saved_start_keeps_its_delivery_branch_as_the_push_baseline() {
    let (mut h, hold) = fetching_saved_delivery();
    let op = h.replaced(hold, commit(3));
    let Op::CheckOut { commit: head, clean, .. } = op else { panic!("saved tree is checked out") };
    assert_eq!((head, clean), (commit(9), false));
    h.one(Event::Done { owner: hold, done: Done::Succeeded });
    h.one(Event::Push { hold, message: message() });
    let op = h.next(hold, Done::Unchanged);
    let Op::Push { commit: head, expected, .. } = op else { panic!("saved tree is delivered") };
    assert_eq!((head, expected), (commit(9), Some(commit(3))));
}

fn merged_spec(heads: &[Commit]) -> Spec {
    let mut repo = repository(b"a", true);
    repo.identity = 3;
    repo.start = Start::Merged { branch: bytes(b"main"), heads: heads.into() };
    Spec { key: bytes(b"merged"), repositories: Box::new([repo]) }
}

/// Complete the fetches and clean checkout; the first ancestry query is in flight.
fn begin_merged(h: &mut Harness, heads: &[Commit]) -> Token {
    let (hold, _) = h.prepare(1, merged_spec(heads));
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    let mut op = h.replaced(hold, commit(1));
    for head in heads {
        let done = if let Op::Point { into: Into::Fetched, commit, .. } = op {
            assert_eq!(commit, *head, "a repeated head is pointed locally");
            Done::Succeeded
        } else {
            let Op::Fetch { want: Want::Commit { commit }, .. } = op else { panic!("unseen head fetched") };
            assert_eq!(commit, *head);
            Done::Fetched { commit: *head }
        };
        op = h.next(hold, done);
    }
    let Op::CheckOut { commit: base, clean, .. } = op else { panic!("base checkout") };
    assert_eq!(base, commit(1));
    assert!(clean);
    let Op::IsAncestor { ancestor, commit: base, .. } = h.next(hold, Done::Succeeded) else { panic!("ancestry") };
    assert_eq!((ancestor, base), (heads[0], commit(1)));
    hold
}

#[test]
fn merged_heads_commit_each_merge_in_order() {
    let mut h = Harness::new(MERGES);
    let hold = begin_merged(&mut h, &[commit(7), commit(8)]);
    let next = commit_merged_head(
        &mut h,
        hold,
        7,
        1,
        11,
        b"Merge 0707070707070707070707070707070707070707070707070707070707070707",
    );
    let Op::IsAncestor { ancestor, commit: ours, .. } = io(next, hold) else { panic!("next head") };
    assert_eq!((ancestor, ours), (commit(8), commit(11)));
    let ready = commit_merged_head(
        &mut h,
        hold,
        8,
        11,
        12,
        b"Merge 0808080808080808080808080808080808080808080808080808080808080808",
    );
    let Request::Prepared { prepared: Prepared::Ready { conflicts, .. }, .. } = ready else { panic!("ready") };
    assert!(conflicts.is_empty());
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Save { hold, message: message() }), hold) else {
        panic!("save")
    };
    assert_eq!((parent, merging), (commit(12), None));
    assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })), &[Landing::Unchanged]);
    h.one(Event::Push { hold, message: message() });
    let Op::Push { commit: pushed, .. } = h.next(hold, Done::Unchanged) else { panic!("first delivery") };
    assert_eq!(pushed, commit(12));
}

fn commit_merged_head(h: &mut Harness, hold: Token, head: u8, parent: u8, merged: u8, title: &[u8]) -> Request {
    let Op::Merge { theirs, .. } = h.next(hold, Done::IsAncestor { contains: false }) else { panic!("merge") };
    assert_eq!(theirs, commit(head));
    let Op::Commit { parent: ours, merging, identity, title: message, body, .. } = h.next(hold, Done::Merged) else {
        panic!("commit merge")
    };
    assert_eq!((ours, merging), (commit(parent), Some(commit(head))));
    assert_eq!(identity, 3);
    assert_eq!(&*message, title);
    assert!(body.is_empty());
    let Op::CheckOut { commit: checked_out, clean, .. } = h.next(hold, Done::Committed { commit: commit(merged) })
    else {
        panic!("check out the merge commit before the next head or ready")
    };
    assert_eq!((checked_out, clean), (commit(merged), true));
    h.one(Event::Done { owner: hold, done: Done::Succeeded })
}

fn comparing_producer_base(saved: bool) -> (Harness, Token) {
    let mut h = Harness::new(MERGES);
    let mut spec = merge_spec();
    spec.repositories[0].landing = Some(bytes(b"base"));
    if saved {
        spec.repositories[0].start = Start::Merge { branch: bytes(b"saved/1"), base: commit(7) };
    }
    let (hold, _) = h.prepare(1, spec);
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Fetched { commit: commit(1) });
    h.next(hold, Done::Fetched { commit: commit(7) });
    let op = h.next(hold, Done::Succeeded);
    let Op::IsAncestor { ancestor, commit: ours, .. } = op else {
        panic!("a producer must compare its base before merging: {op:?}")
    };
    assert_eq!((ancestor, ours), (commit(7), commit(1)));
    (h, hold)
}

#[test]
fn a_producers_contained_saved_start_delivers_its_tree_without_a_second_parent() {
    let (mut h, hold) = comparing_producer_base(true);
    landing_fetch(h.next(hold, Done::IsAncestor { contains: true }));
    let Op::Fetch { want: Want::Branch { branch }, into: Into::Fetched, .. } =
        h.next(hold, Done::Fetched { commit: commit(7) })
    else {
        panic!("fetch push branch")
    };
    assert_eq!(&*branch, b"main");
    let Op::CheckOut { commit: saved, clean, .. } = h.next(hold, Done::Fetched { commit: commit(4) }) else {
        panic!("saved checkout")
    };
    assert_eq!((saved, clean), (commit(1), false));
    h.one(Event::Done { owner: hold, done: Done::Succeeded });
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("ordinary commit")
    };
    assert_eq!((parent, merging), (commit(1), None));
    retry_push(h.next(hold, Done::Unchanged), commit(1), commit(1));
    assert_eq!(
        &*landings(h.one(Event::Done { owner: hold, done: Done::Succeeded })),
        &[Landing::Landed { commit: commit(1) }]
    );
}

#[test]
fn a_producers_clean_merge_is_committed_before_it_is_ready() {
    let (mut h, hold) = comparing_producer_base(false);
    landing_fetch(h.next(hold, Done::IsAncestor { contains: false }));
    let Op::Merge { theirs, .. } = h.next(hold, Done::Fetched { commit: commit(7) }) else { panic!("merge base") };
    assert_eq!(theirs, commit(7));
    let Op::Commit { parent, merging, title, body, identity, .. } = h.next(hold, Done::Merged) else {
        panic!("commit the clean merge before ready")
    };
    assert_eq!((parent, merging), (commit(1), Some(commit(7))));
    assert_eq!(&*title, b"Merge 0707070707070707070707070707070707070707070707070707070707070707");
    assert!(body.is_empty());
    assert_eq!(identity, 0);
    let Op::CheckOut { commit: head, clean, .. } = h.next(hold, Done::Committed { commit: commit(9) }) else {
        panic!("check out the merge to clear its state")
    };
    assert_eq!((head, clean), (commit(9), true));
    let Request::Prepared { prepared: Prepared::Ready { conflicts, .. }, .. } =
        h.one(Event::Done { owner: hold, done: Done::Succeeded })
    else {
        panic!("ready after the merge checkout")
    };
    assert!(conflicts.is_empty());
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("commit new work on the merge")
    };
    assert_eq!((parent, merging), (commit(9), None));
    let Op::Push { commit: head, .. } = h.next(hold, Done::Unchanged) else { panic!("deliver the merge") };
    assert_eq!(head, commit(9));
    assert_eq!(
        &*landings(h.one(Event::Done { owner: hold, done: Done::Succeeded })),
        &[Landing::Landed { commit: commit(9) }]
    );
}

#[test]
fn a_producers_uncontained_base_still_leaves_a_conflicted_two_parent_merge() {
    let (mut h, hold) = comparing_producer_base(false);
    landing_fetch(h.next(hold, Done::IsAncestor { contains: false }));
    let Op::Merge { theirs, .. } = h.next(hold, Done::Fetched { commit: commit(7) }) else { panic!("merge base") };
    assert_eq!(theirs, commit(7));
    let files = Box::new([bytes(b"code")]);
    let ready = h.one(Event::Done { owner: hold, done: Done::Conflicted { files } });
    let Request::Prepared { prepared: Prepared::Ready { conflicts, .. }, .. } = ready else {
        panic!("ready with conflicts")
    };
    assert_eq!(&*conflicts, &[crate::Conflicts { repository: 0, files: Box::new([bytes(b"code")]) }]);
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("two parent commit")
    };
    assert_eq!((parent, merging), (commit(1), Some(commit(7))));
}

#[test]
fn a_failed_producer_base_query_is_transient() {
    let (mut h, hold) = comparing_producer_base(false);
    assert_eq!(
        h.one(Event::Done { owner: hold, done: failed(Fault::Refused) }),
        Request::Prepared {
            client: Token::new(1),
            prepared: Prepared::Failed { failure: Failure::Transient, detail: bytes(b"compare base: Refused") }
        }
    );
}

#[test]
fn a_producers_merge_start_with_its_base_contained_makes_no_merge() {
    let (mut h, hold) = comparing_producer_base(false);
    let Op::Fetch { want: Want::Branch { branch }, into: Into::Landing, .. } =
        h.next(hold, Done::IsAncestor { contains: true })
    else {
        panic!("fetch landing")
    };
    assert_eq!(&*branch, b"base");
    h.next(hold, Done::Fetched { commit: commit(7) });
    let Request::Prepared { prepared: Prepared::Ready { .. }, .. } =
        h.one(Event::Done { owner: hold, done: Done::Succeeded })
    else {
        panic!("ready without a merge")
    };
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("ordinary commit")
    };
    assert_eq!((parent, merging), (commit(1), None));
    compares(h.next(hold, Done::Unchanged), commit(1), commit(7));
    let Op::Push { commit: head, expected, .. } = h.next(hold, Done::IsAncestor { contains: false }) else {
        panic!("guarded push")
    };
    assert_eq!((head, expected), (commit(1), Some(commit(1))));
    assert_eq!(
        &*landings(h.one(Event::Done { owner: hold, done: Done::Succeeded })),
        &[Landing::Landed { commit: commit(1) }]
    );
}

#[test]
fn merged_head_already_contained_makes_no_merge_or_checkout() {
    let mut h = Harness::new(MERGES);
    let hold = begin_merged(&mut h, &[commit(7)]);
    let Request::Prepared { prepared: Prepared::Ready { .. }, .. } =
        h.one(Event::Done { owner: hold, done: Done::IsAncestor { contains: true } })
    else {
        panic!("ready without a merge")
    };
    let Op::Commit { parent, merging, .. } = io(h.one(Event::Push { hold, message: message() }), hold) else {
        panic!("ordinary commit")
    };
    assert_eq!((parent, merging), (commit(1), None));
    assert_eq!(&*landings(h.one(Event::Done { owner: hold, done: Done::Unchanged })), &[Landing::Unchanged]);
}

#[test]
fn merged_head_conflict_fails_naming_it() {
    let mut h = Harness::new(MERGES);
    let hold = begin_merged(&mut h, &[commit(7), commit(8)]);
    h.next(hold, Done::IsAncestor { contains: false });
    h.next(hold, Done::Merged);
    h.next(hold, Done::Committed { commit: commit(11) });
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::IsAncestor { contains: false });
    assert_eq!(
        h.one(Event::Done { owner: hold, done: Done::Conflicted { files: Box::new([bytes(b"code")]) } }),
        Request::Prepared {
            client: Token::new(1),
            prepared: Prepared::Failed {
                failure: Failure::Conflicted { repository: 0, head: commit(8) },
                detail: bytes(b"merge: Conflicted: head 0808080808080808080808080808080808080808080808080808080808080808 does not merge into main")
            }
        }
    );
    assert_eq!(h.domain.holds(), 1);
    assert_eq!(
        h.one(Event::Save { hold, message: message() }),
        Request::Saved { client: Token::new(1), outcome: Outcome::Refused { refusal: Refusal::Busy } }
    );
    assert_eq!(
        h.one(Event::Push { hold, message: message() }),
        Request::Pushed { client: Token::new(1), outcome: Outcome::Refused { refusal: Refusal::Busy } }
    );
    h.one(Event::Release { hold });
    assert_eq!(h.domain.holds(), 0);
}

#[test]
fn merged_head_checkout_failure_ends_preparation_transient() {
    for (done, detail) in [
        (Done::Failed { fault: Fault::Broken }, b"check out head: Broken".as_slice()),
        (
            Done::FailedWithOutput {
                fault: Fault::Broken,
                diagnostic: crate::git::PushDiagnostic::new(b"checkout failed", 0),
            },
            b"check out head: Broken: checkout failed".as_slice(),
        ),
    ] {
        let mut h = Harness::new(MERGES);
        let hold = begin_merged(&mut h, &[commit(7)]);
        h.next(hold, Done::IsAncestor { contains: false });
        h.next(hold, Done::Merged);
        h.next(hold, Done::Committed { commit: commit(11) });
        assert_eq!(
            h.one(Event::Done { owner: hold, done }),
            Request::Prepared {
                client: Token::new(1),
                prepared: Prepared::Failed { failure: Failure::Transient, detail: bytes(detail) }
            }
        );
    }
}

#[test]
fn merged_heads_empty_or_too_many_are_invalid() {
    // channel.md, 4: one to 64 heads; the two invalid sides share the refusal.
    const HEADS: usize = 64;
    for heads in [&[][..], &[commit(7); HEADS + 1][..]] {
        let mut h = Harness::new(MERGES);
        assert_eq!(
            h.one(Event::Prepare { client: Token::new(1), spec: merged_spec(heads) }),
            Request::Prepared { client: Token::new(1), prepared: Prepared::Refused { refusal: Refusal::Invalid } }
        );
        assert_eq!(h.domain.holds(), 0);
    }
    let mut h = Harness::new(MERGES);
    let hold = begin_merged(&mut h, &[commit(7); HEADS]);
    for _ in 1..HEADS {
        h.next(hold, Done::IsAncestor { contains: true });
    }
    let Request::Prepared { prepared: Prepared::Ready { .. }, .. } =
        h.one(Event::Done { owner: hold, done: Done::IsAncestor { contains: true } })
    else {
        panic!("ready without a merge")
    };
}

#[test]
fn a_first_hold_looks_in_spec_order_then_fetches_and_caches_the_clones() {
    let mut h = Harness::reusing(LIMITS);
    let (hold, op) = h.prepare(1, two(b"work"));
    assert_eq!(op, Op::Look { at: place(Token::new(0), b"a"), remote: remote(b"a") });
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(1), cached: Cached::Looking }));
    assert_eq!(
        h.next(hold, Done::Looked { holds: true }),
        Op::Look { at: place(Token::new(0), b"b"), remote: remote(b"b") }
    );
    assert_eq!(h.next(hold, Done::Looked { holds: true }).kind(), Kind::Fetch);
    assert_eq!(h.domain.workstream(0), Some(&b"work"[..]));
    assert_eq!(h.replaced(hold, commit(1)).kind(), Kind::CheckOut);
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Fetch);
    assert_eq!(h.next(hold, Done::Fetched { commit: commit(2) }).kind(), Kind::CheckOut);
    h.one(Event::Done { owner: hold, done: Done::Succeeded });
    h.one(Event::Release { hold });
    let (_, op) = h.prepare(2, two(b"work"));
    assert_eq!(op.kind(), Kind::Fetch);
}

#[test]
fn a_look_that_does_not_hold_makes_then_clones() {
    for holds_first in [false, true] {
        let mut h = Harness::reusing(LIMITS);
        let (hold, _) = h.prepare(1, two(b"work"));
        if holds_first {
            h.next(hold, Done::Looked { holds: true });
        }
        assert_eq!(
            h.next(hold, Done::Looked { holds: false }),
            Op::Make { workspace: Token::new(0), stores: Box::new([store_token(b"a"), store_token(b"b")]) }
        );
        assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Clone);
    }
}

#[test]
fn a_first_saved_start_makes_without_looking() {
    for reuse in [true, false] {
        let mut h = if reuse { Harness::reusing(LIMITS) } else { Harness::new(LIMITS) };
        let repo = Repository { start: Start::Saved { branch: bytes(b"saved") }, ..repository(b"b", false) };
        let (_, op) = h.prepare(1, spec(b"work", Box::new([repository(b"a", true), repo])));
        assert_eq!(op, Op::Make { workspace: Token::new(0), stores: Box::new([store_token(b"a"), store_token(b"b")]) });
        assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(1), cached: Cached::New }));
    }
}

#[test]
fn a_failed_look_spoils_the_workspace_and_fails_transient() {
    for (fault, detail) in [
        (Fault::Broken, b"look: Broken".as_slice()),
        (Fault::TimedOut, b"look: TimedOut".as_slice()),
        (Fault::Cancelled, b"look: Cancelled".as_slice()),
    ] {
        let mut h = Harness::reusing(LIMITS);
        let (hold, _) = h.prepare(1, one(b"work"));
        assert_eq!(
            h.one(Event::Done { owner: hold, done: failed(fault) }),
            Request::Prepared {
                client: Token::new(1),
                prepared: Prepared::Failed { failure: Failure::Transient, detail: bytes(detail) }
            }
        );
        h.one(Event::Release { hold });
        h.facts();
        let (_, op) = h.prepare(2, one(b"work"));
        assert_eq!(op.kind(), Kind::Make);
        assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(2), cached: Cached::Rebuilt }));
    }
}

#[test]
fn aborting_or_releasing_a_look_waits_for_its_terminal() {
    for release in [false, true] {
        let mut h = Harness::reusing(LIMITS);
        let (hold, _) = h.prepare(1, one(b"work"));
        let event = if release { Event::Release { hold } } else { Event::Abort { hold } };
        assert_eq!(h.one(event), Request::Cancel { owner: hold });
        let requests = h.step(Event::Done { owner: hold, done: Done::Looked { holds: true } });
        assert_eq!(requests.get(0), Some(&Request::Prepared { client: Token::new(1), prepared: Prepared::Aborted }));
        assert_eq!(requests.len(), if release { 2 } else { 1 });
        if release {
            assert_eq!(requests.get(1), Some(&Request::Released { client: Token::new(1) }));
        }
    }
}

#[test]
fn a_look_uses_the_local_deadline() {
    let mut h = Harness::reusing(LIMITS);
    let (hold, _) = h.begin(1, one(b"work"));
    let Request::Io { op, deadline, .. } = h.one(Event::Done { owner: hold, done: Done::Succeeded }) else {
        panic!("look after Store")
    };
    assert_eq!(op.kind(), Kind::Look);
    assert_eq!(deadline, h.env.now.saturating_add(LIMITS.local_timeout));
}

#[test]
fn unchecked_work_pushes_to_its_own_branch_and_the_next_commit_is_fresh() {
    for (saved_work, push_lands) in [(false, true), (true, true), (false, false), (true, false)] {
        let mut h = Harness::new(LIMITS);
        let hold = h.ready(1, one(b"w"));
        let op = io(h.one(Event::PushUnchecked { hold }), hold);
        let Op::Commit { title, body, always, snapshot, .. } = op else { panic!("save commits: {op:?}") };
        assert_eq!(snapshot, Some(crate::git::Snapshot { remote: remote(b"a"), account: None }));
        assert_eq!(title.as_ref(), b"Unchecked: work in progress, saved before a question");
        assert!(body.is_empty());
        assert!(!always);
        let op = h.next(hold, Done::Committed { commit: commit(11) });
        let Op::Push { branch, .. } = op else { panic!("save pushes: {op:?}") };
        assert_eq!(branch.as_ref(), b"main");
        let done = if push_lands { Done::Succeeded } else { failed(Fault::Refused) };
        h.one(Event::Done { owner: hold, done });
        let event = if saved_work {
            Event::Save { hold, message: message() }
        } else {
            Event::Push { hold, message: message() }
        };
        let op = io(h.one(event), hold);
        let Op::Commit { parent, always, title, body, .. } = op else { panic!("next commit: {op:?}") };
        assert_eq!(parent, commit(11));
        assert!(always, "an unchecked head requires a fresh commit even with an equal tree");
        assert_eq!((title.as_ref(), body.as_ref()), (b"Fix it".as_slice(), b"Because.".as_slice()));
        h.next(hold, Done::Committed { commit: commit(12) });
        h.one(Event::Done { owner: hold, done: Done::Succeeded });
        let op = io(h.one(Event::Push { hold, message: message() }), hold);
        let Op::Commit { always, .. } = op else { panic!("later commit: {op:?}") };
        assert!(!always, "the replacement head is no longer the unchecked commit");
    }
}

#[test]
fn a_made_remote_needs_no_store_for_another_workstream() {
    let mut h = Harness::new(LIMITS);
    let first = h.ready(1, one(b"a"));
    let (second, op) = h.begin(2, one(b"b"));
    let Op::Make { workspace, stores } = op else { panic!("no Store after the first made it: {op:?}") };
    assert_eq!(stores.as_ref(), &[store_token(b"a")]);
    assert_eq!(
        h.next(second, Done::Succeeded),
        Op::Clone { at: place(workspace, b"a"), remote: remote(b"a"), store: store_token(b"a") }
    );
    h.one(Event::Release { hold: first });
}
#[test]
fn queued_prepare_starts_when_ready_with_a_fresh_deadline_and_its_own_facts() {
    for fresh_store in [false, true] {
        let mut h = Harness::new(Limits { preparations: 1, ..LIMITS });
        let (first, _) = h.begin(1, one(b"a"));
        h.facts();
        let spec = if fresh_store { other_store(b"b") } else { one(b"b") };
        let second = waiting(&mut h, 2, spec);
        let client = Token::new(2);
        assert_eq!(h.facts().as_slice(), &[Fact::Held { client, cached: Cached::New }, Fact::Queued { client },]);
        h.env.now = NOW.saturating_add(Duration::from_secs(300));
        assert_eq!(h.next(first, Done::Succeeded).kind(), Kind::Make);
        let workspace = checkout_one(&mut h, first);
        h.facts();
        let requests = h.step(Event::Done { owner: first, done: Done::Succeeded });
        let timeout = if fresh_store { h.env.limits.remote_timeout } else { h.env.limits.local_timeout };
        let kind = if fresh_store { Kind::Store } else { Kind::Make };
        assert_eq!(requests.len(), 2, "the returned permit is used in this step");
        assert_eq!(
            requests.get(0),
            Some(&Request::Prepared {
                client: Token::new(1),
                prepared: Prepared::Ready { workspace, conflicts: Box::new([]) },
            })
        );
        let Some(Request::Io { owner, op, deadline }) = requests.get(1) else { panic!("the queued prepare starts") };
        assert_eq!((*owner, op.kind(), *deadline), (second, kind, h.env.now.saturating_add(timeout)));
        assert_eq!(
            h.facts().as_slice(),
            &[
                Fact::Ended { client: Token::new(1), done: crate::Ending::Succeeded },
                Fact::Prepared {
                    client: Token::new(1),
                    prepared: crate::Preparation::Ready { workspace, conflicts: 0 }
                },
                Fact::Started { client, op: kind },
            ]
        );
    }
}

#[test]
fn queued_prepares_follow_arrival_order_when_permits_return() {
    let mut h = Harness::new(Limits { workspaces: 4, preparations: 1, ..LIMITS });
    let (mut active, _) = h.begin(1, one(b"a"));
    let second = waiting(&mut h, 8, one(b"b"));
    let third = waiting(&mut h, 3, one(b"c"));
    let fourth = waiting(&mut h, 6, one(b"d"));
    for next in [second, third, fourth] {
        let mut requests = h.step(Event::Done { owner: active, done: failed(Fault::Broken) });
        assert_eq!(requests.len(), 2, "one prepare ends and one starts");
        let Request::Prepared { prepared: Prepared::Failed { .. }, .. } = requests.swap_remove(0).expect("ends") else {
            panic!("the active prepare failed")
        };
        assert_eq!(io(requests.swap_remove(0).expect("admitted"), next).kind(), Kind::Store);
        active = next;
    }
}

#[test]
fn queued_abort_and_release_remove_the_hold_without_io_or_cancel() {
    for release in [false, true] {
        let mut h = Harness::new(Limits { workspaces: 4, preparations: 1, ..LIMITS });
        let (first, _) = h.begin(1, one(b"a"));
        let second = waiting(&mut h, 2, one(b"b"));
        let middle = waiting(&mut h, 3, one(b"c"));
        let fourth = waiting(&mut h, 4, one(b"d"));
        let event = if release { Event::Release { hold: middle } } else { Event::Abort { hold: middle } };
        let requests = h.step(event);
        let mut expected = List::uncapped();
        expected.push(Request::Prepared { client: Token::new(3), prepared: Prepared::Aborted }).expect("room");
        if release {
            expected.push(Request::Released { client: Token::new(3) }).expect("room");
        }
        assert_eq!(requests, expected, "a queued hold has nothing to cancel");
        let mut active = first;
        for next in [second, fourth] {
            let mut requests = h.step(Event::Done { owner: active, done: failed(Fault::Broken) });
            assert_eq!(requests.len(), 2, "the cancelled middle hold is skipped");
            io(requests.swap_remove(1).expect("oldest queued"), next);
            active = next;
        }
    }
}

#[test]
fn an_active_abort_keeps_the_permit_until_any_operation_end() {
    for done in [Done::Succeeded, failed(Fault::Broken), failed(Fault::Cancelled), failed(Fault::TimedOut)] {
        let mut h = Harness::new(Limits { preparations: 1, ..LIMITS });
        let (first, _) = h.begin(1, one(b"a"));
        let second = waiting(&mut h, 2, other_store(b"b"));
        assert_eq!(h.one(Event::Abort { hold: first }), Request::Cancel { owner: first });
        h.none(Event::Abort { hold: first });
        let requests = h.step(Event::Done { owner: first, done: done.clone() });
        assert_eq!(requests.len(), 2);
        assert_eq!(requests.get(0), Some(&Request::Prepared { client: Token::new(1), prepared: Prepared::Aborted }));
        let Some(Request::Io { owner, .. }) = requests.get(1) else { panic!("the oldest queued starts in this step") };
        assert_eq!(*owner, second);
    }
}

#[test]
fn a_store_waiter_keeps_its_permit_until_it_ends() {
    let mut h = Harness::new(Limits { workspaces: 3, preparations: 2, ..LIMITS });
    let (first, _) = h.begin(1, one(b"a"));
    let second = waiting(&mut h, 2, one(b"b"));
    let third = waiting(&mut h, 3, other_store(b"c"));
    assert_eq!(refused(h.one(Event::Push { hold: second, message: message() })), Refusal::Busy);
    assert_eq!(refused(h.one(Event::Save { hold: second, message: message() })), Refusal::Busy);
    let requests = h.step(Event::Abort { hold: second });
    assert_eq!(requests.len(), 2);
    assert_eq!(requests.get(0), Some(&Request::Prepared { client: Token::new(2), prepared: Prepared::Aborted }));
    let Some(Request::Io { owner, op: Op::Store { remote: source, .. }, .. }) = requests.get(1) else {
        panic!("ending a store wait returns its permit")
    };
    assert_eq!((*owner, source.as_ref()), (third, remote(b"b").as_ref()));
    assert_eq!(h.one(Event::Abort { hold: first }), Request::Cancel { owner: first });
}

#[test]
fn a_failed_store_resumes_its_oldest_waiter_before_admitting_queued_prepares() {
    let mut h = Harness::new(Limits { workspaces: 3, preparations: 2, ..LIMITS });
    let (first, store) = h.begin(1, one(b"a"));
    let second = waiting(&mut h, 2, one(b"b"));
    let third = waiting(&mut h, 3, one(b"c"));
    let mut requests = h.step(Event::Done { owner: first, done: failed(Fault::Broken) });
    assert_eq!(requests.len(), 2, "the admitted third now waits on the second's store");
    assert_eq!(io(requests.swap_remove(1).expect("oldest store waiter"), second), store);
    let requests = h.step(Event::Done { owner: second, done: Done::Succeeded });
    assert_eq!(requests.len(), 2);
    make(requests.get(0).expect("maker"), second, &[store_token(b"a")]);
    make(requests.get(1).expect("admitted waiter"), third, &[store_token(b"a")]);
}

#[test]
fn ready_pushes_and_saves_do_not_wait_for_preparation_permits() {
    for save in [false, true] {
        let mut h = Harness::new(Limits { workspaces: 3, preparations: 1, ..LIMITS });
        let ready = h.ready(1, one(b"a"));
        h.begin(2, one(b"b"));
        let queued = waiting(&mut h, 3, one(b"c"));
        assert_eq!(refused(h.one(Event::Push { hold: queued, message: message() })), Refusal::Busy);
        assert_eq!(refused(h.one(Event::Save { hold: queued, message: message() })), Refusal::Busy);
        let event = if save {
            Event::Save { hold: ready, message: message() }
        } else {
            Event::Push { hold: ready, message: message() }
        };
        assert_eq!(io(h.one(event), ready).kind(), Kind::Commit);
    }
}

#[test]
fn entrance_refusals_do_not_wait_for_preparation_permits() {
    let mut h = Harness::new(Limits { preparations: 1, ..LIMITS });
    let (first, _) = h.begin(1, one(b"a"));
    let second = waiting(&mut h, 2, one(b"b"));
    for (spec, expected) in [
        (one(b"a"), Refusal::Busy),
        (one(b"c"), Refusal::Full),
        (one(b"a-key-beyond-the-name-limit"), Refusal::Invalid),
    ] {
        assert_eq!(refused(h.one(Event::Prepare { client: Token::new(3), spec })), expected);
    }
    let mut requests = h.step(Event::Done { owner: first, done: failed(Fault::Broken) });
    assert_eq!(requests.len(), 2);
    io(requests.swap_remove(1).expect("only the admitted prepare queued"), second);
    assert_eq!(h.step(Event::Done { owner: second, done: failed(Fault::Broken) }).len(), 1);
}

/// Admit a prepare that waits without asking io for anything.
fn waiting(h: &mut Harness, client: u64, spec: Spec) -> Token {
    let Request::Held { client: to, hold } = h.one(Event::Prepare { client: Token::new(client), spec }) else {
        panic!("a waiter is admitted");
    };
    assert_eq!(to, Token::new(client));
    hold
}

fn make(request: &Request, hold: Token, expected: &[Token]) {
    let Request::Io { owner, op: Op::Make { stores, .. }, .. } = request else {
        panic!("the prepare goes on without its own Store: {request:?}");
    };
    assert_eq!(*owner, hold);
    assert_eq!(stores.as_ref(), expected);
}

#[test]
fn concurrent_prepares_share_one_store_maker() {
    let mut h = Harness::new(LIMITS);
    let (first, store) = h.begin(1, one(b"a"));
    let Op::Store { store, .. } = store else { panic!("the first makes the store") };
    let second = waiting(&mut h, 2, one(b"b"));
    h.facts();
    let requests = h.step(Event::Done { owner: first, done: Done::Succeeded });
    assert_eq!(requests.len(), 2);
    make(requests.get(0).expect("maker"), first, &[store]);
    make(requests.get(1).expect("waiter"), second, &[store]);
    let facts = h.facts();
    for client in [1, 2] {
        assert!(
            facts.as_slice().contains(&Fact::Started { client: Token::new(client), op: Kind::Make }),
            "each resumed operation is told as its own client's fact: {facts:?}"
        );
    }
}

#[test]
fn a_failed_or_cancelled_store_passes_to_only_the_oldest_waiter() {
    for fault in [Fault::Broken, Fault::Cancelled, Fault::Missing { missing: Missing::Repository }] {
        let mut h = Harness::new(Limits { workspaces: 3, preparations: 3, repositories: 1, ..LIMITS });
        // Fill the table so every making, including a successor's, carries aside.
        for name in [b"x", b"y", b"z"] {
            let mut repo = repository(b"a", false);
            repo.remote = remote(name);
            let hold = h.ready(10, spec(name, Box::new([repo])));
            h.one(Event::Release { hold });
        }
        let (first, store) = h.begin(1, one(b"a"));
        let Op::Store { aside: Some(_), .. } = store else { panic!("a full table evicts a remote") };
        let second = waiting(&mut h, 2, one(b"b"));
        let third = waiting(&mut h, 3, one(b"c"));
        if fault == Fault::Cancelled {
            assert_eq!(h.one(Event::Abort { hold: first }), Request::Cancel { owner: first });
        }
        let mut requests = h.step(Event::Done { owner: first, done: failed(fault) });
        assert_eq!(requests.len(), 2, "only one waiter is asked");
        let Request::Prepared { client, prepared: ending } = requests.swap_remove(0).expect("maker ends") else {
            panic!("maker ends");
        };
        assert_eq!(client, Token::new(1));
        match fault {
            Fault::Cancelled => assert_eq!(ending, Prepared::Aborted),
            Fault::Broken | Fault::Missing { .. } => {
                let Prepared::Failed { failure, .. } = ending else { panic!("maker failed") };
                let expected = if fault == Fault::Broken {
                    Failure::Transient
                } else {
                    Failure::Missing { repository: 0, missing: Missing::Repository }
                };
                assert_eq!(failure, expected);
            }
            Fault::Refused | Fault::Unreachable | Fault::TimedOut => unreachable!(),
        }
        assert_eq!(io(requests.swap_remove(0).expect("successor"), second), store, "same token and aside");
        if fault == (Fault::Missing { missing: Missing::Repository }) {
            let mut requests = h.step(Event::Done { owner: second, done: failed(fault) });
            assert_eq!(requests.len(), 2);
            let Some(Request::Prepared { client, prepared: Prepared::Failed { .. } }) = requests.get(0) else {
                panic!("second maker fails too");
            };
            assert_eq!(*client, Token::new(2));
            assert_eq!(io(requests.swap_remove(1).expect("last waiter"), third), store);
            let Prepared::Failed { failure, .. } = prepared(h.one(Event::Done { owner: third, done: failed(fault) }))
            else {
                panic!("each waiter tries once");
            };
            assert_eq!(failure, Failure::Missing { repository: 0, missing: Missing::Repository });
        } else {
            let requests = h.step(Event::Done { owner: second, done: Done::Succeeded });
            assert_eq!(requests.len(), 2);
            let Op::Store { store, .. } = store else { unreachable!() };
            make(requests.get(0).expect("successor"), second, &[store]);
            make(requests.get(1).expect("remaining waiter"), third, &[store]);
        }
    }
}

#[test]
fn a_waiter_ends_at_once_without_cancelling_the_maker() {
    for release in [false, true] {
        let mut h = Harness::new(LIMITS);
        let (first, op) = h.begin(1, one(b"a"));
        let Op::Store { store, .. } = op else { panic!("maker") };
        let second = waiting(&mut h, 2, one(b"b"));
        assert_eq!(refused(h.one(Event::Push { hold: second, message: message() })), Refusal::Busy);
        assert_eq!(refused(h.one(Event::Save { hold: second, message: message() })), Refusal::Busy);
        let event = if release { Event::Release { hold: second } } else { Event::Abort { hold: second } };
        let requests = h.step(event);
        assert_eq!(requests.len(), if release { 2 } else { 1 });
        assert_eq!(requests.get(0), Some(&Request::Prepared { client: Token::new(2), prepared: Prepared::Aborted }));
        if release {
            assert_eq!(requests.get(1), Some(&Request::Released { client: Token::new(2) }));
        }
        let request = h.one(Event::Done { owner: first, done: Done::Succeeded });
        make(&request, first, &[store]);
        if !release {
            assert_eq!(h.one(Event::Release { hold: second }), Request::Released { client: Token::new(2) });
        }
    }
}

#[test]
fn opposite_repository_orders_finish_with_one_store_per_remote() {
    let mut h = Harness::new(LIMITS);
    let (first, a) = h.begin(1, two(b"first"));
    let (second, b) = h.begin(2, spec(b"second", Box::new([repository(b"b", false), repository(b"a", true)])));
    let Op::Store { store: a, remote: source, .. } = a else { panic!("a maker") };
    assert_eq!(source, remote(b"a"));
    let Op::Store { store: b, remote: source, .. } = b else { panic!("b maker") };
    assert_eq!(source, remote(b"b"));
    h.none(Event::Done { owner: first, done: Done::Succeeded });
    let requests = h.step(Event::Done { owner: second, done: Done::Succeeded });
    assert_eq!(requests.len(), 2);
    make(requests.get(0).expect("second"), second, &[b, a]);
    make(requests.get(1).expect("first"), first, &[a, b]);
    for (hold, client) in [(first, 1), (second, 2)] {
        let mut op = h.next(hold, Done::Succeeded);
        let mut ready = false;
        for _ in 0..16_u32 {
            let done = match &op {
                Op::Store { .. } => panic!("exactly one Store per remote"),
                Op::Fetch { at, .. } => Done::Fetched { commit: start_of(at) },
                Op::Make { .. } | Op::Point { .. } | Op::Clone { .. } | Op::CheckOut { .. } => Done::Succeeded,
                Op::Look { .. }
                | Op::Create { .. }
                | Op::IsAncestor { .. }
                | Op::DiffPaths { .. }
                | Op::Merge { .. }
                | Op::Commit { .. }
                | Op::Push { .. } => panic!("branch preparation only"),
            };
            match h.one(Event::Done { owner: hold, done }) {
                Request::Io { op: next, .. } => op = next,
                Request::Prepared { client: to, prepared: Prepared::Ready { .. } } => {
                    assert_eq!(to, Token::new(client));
                    ready = true;
                    break;
                }
                other @ (Request::Held { .. }
                | Request::Prepared { .. }
                | Request::Pushed { .. }
                | Request::Saved { .. }
                | Request::Released { .. }
                | Request::Cancel { .. }) => {
                    panic!("both become ready: {other:?}");
                }
            }
        }
        assert!(ready, "both become ready within the preparation steps");
        h.one(Event::Release { hold });
    }
}

#[test]
fn store_tokens_survive_a_restart_and_repository_reordering() {
    assert_eq!(store_token(b"a"), Token::new(8_130_188_230_355_731_069));
    for [a, b] in [[b"a", b"b"], [b"b", b"a"]] {
        let mut h = Harness::new(LIMITS);
        let spec = spec(b"work", Box::new([repository(a, true), repository(b, false)]));
        let (hold, first) = h.begin(1, spec);
        for (op, name) in [(first, a), (h.next(hold, Done::Succeeded), b)] {
            let Op::Store { store, .. } = op else { panic!("store") };
            assert_eq!(store, store_token(name));
        }
    }
}

#[test]
fn a_full_store_table_replaces_only_an_unborrowed_slot_under_a_new_token() {
    let mut h = Harness::new(Limits { repositories: 1, ..LIMITS });
    let held = h.ready(1, one(b"held"));
    let mut other = one(b"old");
    other.repositories[0].remote = remote(b"b");
    let idle = h.ready(2, other);
    h.one(Event::Release { hold: idle });
    let mut replacement = one(b"new");
    replacement.repositories[0].remote = remote(b"c");
    let (_, op) = h.begin(3, replacement);
    let Op::Store { store, aside, .. } = op else { panic!("new remote needs a store") };
    assert_eq!(aside, Some(store_token(b"b")), "the held store was not evicted");
    assert_ne!(Some(store), aside);
    assert_eq!(store, store_token(b"c"), "the new remote names the store");
    h.one(Event::Release { hold: held });
}
#[test]
fn a_damaged_fetch_rebuilds_its_workspace_but_never_its_store() {
    for fault in [Fault::Broken, Fault::TimedOut, Fault::Cancelled] {
        let mut h = Harness::new(LIMITS);
        let first = h.ready(1, one(b"w"));
        h.one(Event::Release { hold: first });
        let (hold, op) = h.begin(2, one(b"w"));
        let Op::Fetch { store, .. } = op else { panic!("reused workspace fetches") };
        let Request::Prepared { prepared: Prepared::Failed { .. }, .. } =
            h.one(Event::Done { owner: hold, done: failed(fault) })
        else {
            panic!("failed prepare")
        };
        h.one(Event::Release { hold });
        let (next, op) = h.begin(3, one(b"w"));
        let Op::Make { stores, .. } = op else { panic!("workspace rebuilt without Store: {op:?}") };
        assert_eq!(stores.as_ref(), &[store]);
        let Op::Clone { store: borrowed, .. } = h.next(next, Done::Succeeded) else { panic!("clone after Make") };
        assert_eq!(borrowed, store);
    }
}
#[test]
fn store_capacity_must_fit_even_when_hold_capacity_fits() {
    assert!(valid(&Limits { workspaces: 1, preparations: 1, repositories: u32::MAX, ..LIMITS }));
    assert!(!valid(&Limits { workspaces: 2, preparations: 2, repositories: u32::MAX, ..LIMITS }));
}

#[test]
fn a_failed_store_is_not_marked_made_and_is_tried_again() {
    for (fault, expected) in [
        (Fault::Broken, Failure::Transient),
        (
            Fault::Missing { missing: Missing::Repository },
            Failure::Missing { repository: 0, missing: Missing::Repository },
        ),
    ] {
        let mut h = Harness::new(LIMITS);
        let (hold, first) = h.begin(1, one(b"w"));
        let Op::Store { .. } = first else { panic!("new remote") };
        let Prepared::Failed { failure, .. } = prepared(h.one(Event::Done { owner: hold, done: failed(fault) })) else {
            panic!("Store failure ends prepare")
        };
        assert_eq!(failure, expected);
        h.one(Event::Release { hold });
        let (_, again) = h.begin(2, one(b"w"));
        assert_eq!(again, first, "a failed Store is tried again, not borrowed");
    }
}
#[test]
fn design_delivery_and_push_verification_fetch_through_the_repositorys_store() {
    for design in [false, true] {
        let mut h = Harness::new(LIMITS);
        let mut spec = one(b"w");
        if design {
            spec.repositories[0].landing = spec.repositories[0].push.clone();
        }
        let hold = h.ready(1, spec);
        h.one(Event::Push { hold, message: message() });
        let mut op = h.next(hold, Done::Committed { commit: commit(11) });
        if !design {
            let Op::Push { .. } = op else { panic!("ordinary push") };
            op = h.next(hold, failed(Fault::TimedOut));
        }
        let Op::Fetch { store, remote: source, .. } = op else { panic!("design tip or push verification fetched") };
        assert_eq!(store, store_token(b"a"));
        assert_eq!(source, remote(b"a"));
    }
}

#[test]
fn prepares_and_design_deliveries_authenticate_lazy_fetches_as_the_repository() {
    for account in [Some(22), None] {
        for start in pin_starts() {
            let mut h = Harness::new(MERGES);
            let repo = Repository { start, identity: 3, account, ..retry_repository() };
            let (_, ops) = pin_ready(&mut h, 1, repo, commit(8));
            let mut checked_out = false;
            for op in &ops {
                if let Op::CheckOut { identity, account: owner, .. } | Op::Merge { identity, account: owner, .. } = op {
                    assert_eq!((*identity, *owner), (3, account));
                    checked_out = true;
                }
            }
            assert!(checked_out, "every prepare checks out its start");
        }
        let mut h = Harness::new(MERGES);
        let repo = Repository { identity: 7, account, landing: Some(bytes(b"main")), ..repository(b"a", true) };
        let hold = h.ready(1, spec(b"design", Box::new([repo])));
        h.one(Event::Push { hold, message: message() });
        let Op::Fetch { .. } = h.next(hold, Done::Committed { commit: commit(11) }) else { panic!("design tip") };
        let Op::IsAncestor { .. } = h.next(hold, Done::Fetched { commit: commit(12) }) else { panic!("compare") };
        let merge = h.next(hold, Done::IsAncestor { contains: false });
        let Op::Merge { identity, account: owner, .. } = merge else { panic!("design merge") };
        assert_eq!((identity, owner), (7, account));
        let Op::Commit { .. } = h.next(hold, Done::Merged) else { panic!("merged commit") };
        let checkout = h.next(hold, Done::Committed { commit: commit(13) });
        let Op::CheckOut { identity, account: owner, .. } = checkout else { panic!("design checkout") };
        assert_eq!((identity, owner), (7, account));
    }
}
