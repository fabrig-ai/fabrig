//! Best-effort maintenance at the end of a prepare (checkouts.md, *Kept packed*).

use super::*;
use crate::{Candidate, CandidateMember, CandidateSpec, Combination};

fn warm(h: &mut Harness, prepares: u64) {
    for client in 1..=prepares {
        let hold = h.ready(client, one(b"w"));
        h.one(Event::Release { hold });
    }
}

fn checkout_reused(h: &mut Harness, client: u64) -> (Token, Token) {
    h.facts();
    let (hold, fetch) = h.begin(client, one(b"w"));
    assert_eq!(h.facts().get(0), Some(&Fact::Held { client: Token::new(client), cached: Cached::Reused }));
    assert_eq!(fetch.kind(), Kind::Fetch);
    let checkout = h.replaced(hold, commit(1));
    assert_eq!(checkout.kind(), Kind::CheckOut);
    (hold, workspace_of(&checkout))
}

fn pack_request(h: &Harness, hold: Token, workspace: Token, store: Option<Token>) -> Request {
    Request::Io {
        owner: hold,
        op: Op::Pack { at: place(workspace, b"a"), store },
        deadline: h.env.now.saturating_add(h.env.limits.local_timeout),
    }
}

fn due_pack(h: &mut Harness) -> Token {
    warm(h, 24);
    let (hold, workspace) = checkout_reused(h, 25);
    let pack = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    assert_eq!(pack, pack_request(h, hold, workspace, Some(store_token(b"a"))));
    hold
}

#[test]
fn pack_faults_leave_the_prepare_ready_and_its_workspace_reusable() {
    for fault in [Fault::Broken, Fault::TimedOut, Fault::Cancelled] {
        let mut h = Harness::new(LIMITS);
        let hold = due_pack(&mut h);
        let ready = prepared(h.one(Event::Done { owner: hold, done: failed(fault) }));
        let Prepared::Ready { .. } = ready else {
            panic!("Pack ending in {fault:?} leaves the prepare ready: {ready:?}")
        };
        h.one(Event::Release { hold });
        let (next, _) = checkout_reused(&mut h, 26);
        let next = h.one(Event::Done { owner: next, done: Done::Succeeded });
        let Request::Prepared { prepared: Prepared::Ready { .. }, .. } = next else {
            panic!("a fault does not make the next prepare pack again: {next:?}");
        };
    }
}

#[test]
fn interrupting_pack_cancels_it_and_ends_aborted_without_spoiling_the_workspace() {
    for release in [false, true] {
        let mut h = Harness::new(LIMITS);
        let hold = due_pack(&mut h);
        let interrupt = if release { Event::Release { hold } } else { Event::Abort { hold } };
        assert_eq!(h.one(interrupt), Request::Cancel { owner: hold });
        let settled = h.step(Event::Done { owner: hold, done: failed(Fault::Cancelled) });
        let client = Token::new(25);
        let aborted = Request::Prepared { client, prepared: Prepared::Aborted };
        if release {
            assert_eq!(settled.as_slice(), &[aborted, Request::Released { client }]);
        } else {
            assert_eq!(settled.as_slice(), &[aborted]);
            h.one(Event::Release { hold });
        }
        checkout_reused(&mut h, 26);
    }
}

#[test]
fn a_pack_resets_its_shared_store_before_ending() {
    let mut h = Harness::new(LIMITS);
    let _in_flight = due_pack(&mut h);
    let (hold, make) = h.begin(26, one(b"other"));
    assert_eq!(make.kind(), Kind::Make, "the other workspace borrows the made store");
    checkout_one(&mut h, hold);
    let ready = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    let Request::Prepared { prepared: Prepared::Ready { .. }, .. } = ready else {
        panic!("the other prepare does not pack the store again: {ready:?}");
    };
}

#[test]
fn a_verifying_fetch_brings_the_next_prepares_pack_sooner() {
    let mut h = Harness::new(LIMITS);
    warm(&mut h, 22);
    let hold = h.ready(23, one(b"w"));
    assert_eq!(io(h.one(Event::Push { hold, message: message() }), hold).kind(), Kind::Commit);
    let pushed = commit(11);
    assert_eq!(h.next(hold, Done::Committed { commit: pushed }).kind(), Kind::Push);
    assert_eq!(h.next(hold, failed(Fault::Unreachable)).kind(), Kind::Fetch);
    let end = h.one(Event::Done { owner: hold, done: Done::Fetched { commit: pushed } });
    assert_eq!(landings(end).as_ref(), &[Landing::Landed { commit: pushed }]);
    h.one(Event::Release { hold });

    let (next, make) = h.begin(24, one(b"other"));
    assert_eq!(make.kind(), Kind::Make);
    let workspace = checkout_one(&mut h, next);
    let pack = h.one(Event::Done { owner: next, done: Done::Succeeded });
    assert_eq!(pack, pack_request(&h, next, workspace, Some(store_token(b"a"))));
}

fn candidate_spec() -> Spec {
    let mut repo = repository(b"a", false);
    repo.save = None;
    repo.prefixes = Box::new([bytes(b"prefix")]);
    repo.start = Start::Prepare {
        request: Box::new(CandidateSpec {
            base: commit(1),
            from: commit(1),
            upstream: None,
            squash: false,
            members: Box::new([CandidateMember {
                number: 1,
                head: commit(2),
                message: message(),
                squash_message: message(),
            }]),
        }),
    };
    spec(b"w", Box::new([repo]))
}

/// Prepare on the warmed clone: two checkouts and a merge count, but the
/// candidate's commit has no snapshot and leaves no fetch pack of its own.
fn candidate_checkout(h: &mut Harness, combined: Commit) -> (Token, Token) {
    let (hold, fetch) = h.begin(2, candidate_spec());
    assert_eq!(fetch.kind(), Kind::Fetch);
    assert_eq!(h.next(hold, Done::Fetched { commit: commit(1) }).kind(), Kind::Fetch);
    assert_eq!(h.next(hold, Done::Fetched { commit: commit(2) }).kind(), Kind::CheckOut);
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Merge);
    let committing = h.next(hold, Done::Merged);
    let Op::Commit { snapshot: None, .. } = committing else { panic!("candidate commit: {committing:?}") };
    assert_eq!(h.next(hold, Done::Committed { commit: combined }).kind(), Kind::Fetch);
    let checkout = h.next(hold, failed(Fault::Missing { missing: Missing::Branch }));
    assert_eq!(checkout.kind(), Kind::CheckOut);
    (hold, workspace_of(&checkout))
}

#[test]
fn a_candidate_packs_after_the_merge_and_checkouts_reach_the_threshold() {
    for (snapshots, due) in [(20_u32, false), (21, true)] {
        let mut h = Harness::new(MERGES);
        let hold = h.ready(1, one(b"w"));
        for _ in 0..snapshots {
            let snapshot = io(h.one(Event::Push { hold, message: message() }), hold);
            let Op::Commit { snapshot: Some(_), .. } = snapshot else { panic!("delivery snapshot: {snapshot:?}") };
            let end = h.one(Event::Done { owner: hold, done: Done::Committed { commit: commit(1) } });
            assert_eq!(landings(end).as_ref(), &[Landing::Unchanged]);
        }
        h.one(Event::Release { hold });
        let combined = commit(3);
        let (job, workspace) = candidate_checkout(&mut h, combined);
        let mut end = h.one(Event::Done { owner: job, done: Done::Succeeded });
        if due {
            assert_eq!(end, pack_request(&h, job, workspace, None));
            end = h.one(Event::Done { owner: job, done: Done::Succeeded });
        }
        let members = Box::new([Combination::Combined { member: 1, commit: combined, squash: false }]);
        assert_eq!(prepared(end), Prepared::Candidate { workspace, outcome: Candidate { members } });
    }
}
