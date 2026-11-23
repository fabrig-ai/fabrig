//! P109: a store's fetch count asks for packing before readiness
//! (temper/docs/design/domain/worker/checkouts.md, Kept packed).

use super::*;

// Kept packed: the start and counts fetches plus a distinct pin make
// three fetches. Seven such prepares and a local pin leave 23; another
// local pin brings it to 25. After reset, eight groups of three stop at 24.
const DUE: u32 = 25;
const BELOW_DUE: u32 = 23;
const DISTINCT_PIN_PREPARES: u32 = 7;
const PREPARES_AFTER_RESET: u32 = 8;
const AFTER_RESET: u32 = 49;
// Store, make, clone, start, pin, counts, checkout.
const PREPARE_OPERATIONS: u32 = 7;

#[test]
fn growing_packs_trigger_packing() {
    for pack_end in [Done::Succeeded, failed(Fault::Broken)] {
        let mut h = Harness::new(LIMITS);
        h.observe_reds = true;
        let mut fetches = 0;

        for _ in 0..DISTINCT_PIN_PREPARES {
            prepare_and_release(&mut h, &mut fetches, true, None);
        }
        prepare_and_release(&mut h, &mut fetches, false, None);
        assert_eq!(fetches, BELOW_DUE);
        prepare_and_release(&mut h, &mut fetches, false, Some(pack_end));
        assert_eq!(fetches, DUE);

        // Either end resets both counts, and a fault still leaves the
        // workspace ready and reusable, without making its store again.
        for _ in 0..PREPARES_AFTER_RESET {
            prepare_and_release(&mut h, &mut fetches, true, None);
        }
        assert_eq!(fetches, AFTER_RESET, "24 more fetches ask for no Pack");
        assert_eq!(h.domain.workspaces(), 1);
    }
}

/// Prepare the same workstream, optionally fetching a distinct pin as well
/// as its start. Observe every operation, including the last prepare file.
fn prepare_and_release(h: &mut Harness, fetches: &mut u32, pin: bool, pack_end: Option<Done>) {
    let pin = if pin { Some(commit(8)) } else { None };
    let repo = Repository { pin, ..repository(b"a", true) };
    let (hold, mut op) = h.begin(1, spec(b"packing", Box::new([repo])));
    let mut checkout = None;
    for _ in 0..PREPARE_OPERATIONS {
        let done = match op {
            Op::Store { store, .. } => {
                assert_eq!(*fetches, 0, "the store is made only once");
                assert_eq!(store, store_token(b"a"));
                Done::Succeeded
            }
            Op::Make { .. } | Op::Clone { .. } => {
                assert_eq!(*fetches, 0, "the same workspace is reused");
                Done::Succeeded
            }
            Op::Fetch { at, store, want, .. } => {
                assert_eq!(store, store_token(b"a"), "every fetch goes into one store");
                assert_eq!(at.repository.as_ref(), b"a");
                *fetches = fetches.checked_add(1).expect("bounded fetch count");
                let commit = match want {
                    Want::Commit { commit } => commit,
                    Want::Branch { .. } => commit(1),
                    Want::Default => panic!("the start and pin are explicit"),
                };
                Done::Fetched { commit }
            }
            Op::Counts { store, .. } => {
                assert_eq!(store, store_token(b"a"), "counts fetches into the same store");
                *fetches = fetches.checked_add(1).expect("bounded fetch count");
                Done::Succeeded
            }
            Op::Point { .. } => Done::Succeeded,
            Op::CheckOut { at, .. } => {
                checkout = Some(at);
                break;
            }
            other @ (Op::Reset { .. }
            | Op::Reds { .. }
            | Op::Look { .. }
            | Op::Pack { .. }
            | Op::Create { .. }
            | Op::IsAncestor { .. }
            | Op::Brings { .. }
            | Op::DiffPaths { .. }
            | Op::Merge { .. }
            | Op::Commit { .. }
            | Op::Keep { .. }
            | Op::Push { .. }) => panic!("prepare reaches checkout before packing: {other:?}"),
        };
        op = h.next(hold, done);
    }
    let at = checkout.expect("prepare reaches its checkout");
    no_ready(h);
    let reds = h.next(hold, Done::Succeeded);
    let Op::Reds { action: crate::git::Reds::Prepare { .. }, .. } = reds else {
        panic!("the prepare files precede packing: {reds:?}");
    };
    no_ready(h);
    let request = h.one(Event::Done { owner: hold, done: Done::Succeeded });
    let ready = if let Some(done) = pack_end {
        let Request::Io { owner, op, .. } = request else {
            panic!("missing Pack after 25 store fetches, before Ready: {request:?}");
        };
        assert_eq!(owner, hold);
        assert_eq!(op, Op::Pack { at: at.clone(), store: Some(store_token(b"a")) });
        no_ready(h);
        h.one(Event::Done { owner: hold, done })
    } else {
        request
    };
    assert_eq!(
        ready,
        Request::Prepared {
            client: Token::new(1),
            prepared: Prepared::Ready { workspace: at.workspace, conflicts: Box::new([]) },
        },
        "no Pack before the threshold, nor a second Pack for the same repository"
    );
    assert!(h.facts().as_slice().contains(&Fact::Prepared {
        client: Token::new(1),
        prepared: crate::facts::Preparation::Ready { workspace: at.workspace, conflicts: 0 },
    }));
    assert_eq!(h.one(Event::Release { hold }), Request::Released { client: Token::new(1) });
    h.facts();
}

fn no_ready(h: &mut Harness) {
    for fact in &h.facts() {
        if let Fact::Prepared { .. } = fact {
            panic!("no Ready fact before the prepare's operations end: {fact:?}");
        }
    }
}
