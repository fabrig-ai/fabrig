//! Main's counts (domain/worker/checkouts.md): a prepare fetches them after its pin.

use super::*;

#[test]
fn every_pin_fetches_counts_before_resuming_its_own_prepare() {
    let named_tip = commit(8);
    let counts_tip = commit(2);
    let identity = 7;
    let account = Some(22);
    for (landing, pin, merge_head, pin_kind, next_kind) in [
        (Some(bytes(b"base")), None, None, Kind::Fetch, Kind::CheckOut),
        (Some(bytes(b"main")), None, None, Kind::Point, Kind::CheckOut),
        (None, None, None, Kind::Point, Kind::CheckOut),
        (None, Some(named_tip), None, Kind::Fetch, Kind::CheckOut),
        (None, Some(commit(1)), None, Kind::Point, Kind::CheckOut),
        (None, None, Some(counts_tip), Kind::Point, Kind::Fetch),
        (None, None, Some(commit(1)), Kind::Point, Kind::Point),
    ] {
        let start = match merge_head {
            Some(head) => Start::Merged { branch: bytes(b"main"), heads: Box::new([head]) },
            None => Start::Branch { branch: bytes(b"main") },
        };
        let repo = Repository { start, identity, account, landing, pin, ..repository(b"a", true) };
        let mut h = Harness::new(MERGES);
        // The unseen merged head is still fetched even if Counts returned that commit.
        let (_, ops) = pin_trace(&mut h, 1, repo, named_tip, Done::Fetched { commit: counts_tip });
        let [
            Op::Make { workspace, .. },
            Op::Clone { .. },
            Op::Fetch { into: Into::Fetched, .. },
            pinned @ (Op::Fetch { into: Into::Landing, .. } | Op::Point { into: Into::Landing, .. }),
            counts,
            next,
            ..,
        ] = ops.as_slice()
        else {
            panic!("start, pin, counts and continuation: {ops:?}")
        };
        assert_eq!(pinned.kind(), pin_kind);
        assert_eq!(
            counts,
            &Op::Counts {
                at: place(*workspace, b"a"),
                store: store_token(b"a"),
                remote: remote(b"a"),
                identity,
                account,
            }
        );
        assert_eq!(next.kind(), next_kind);
        let mut count = 0_u32;
        for op in &ops {
            if op.kind() == Kind::Counts {
                count = count.checked_add(1).expect("bounded prepare");
            }
        }
        assert_eq!(count, 1);
    }
}

fn fetching_counts() -> (Harness, Token) {
    let mut h = Harness::new(LIMITS);
    let (hold, _) = h.prepare(1, one(b"w"));
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Succeeded);
    h.next(hold, Done::Fetched { commit: commit(1) });
    assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Counts);
    (h, hold)
}

#[test]
fn counts_faults_fail_preparation_like_fetch_faults() {
    const PREFIX: &[u8] = b"fetch counts: ";
    const SUFFIX: &[u8] =
        b" (workspace 0, repository 0 a, command none, operation 0 ms of 60000 ms, prepare 0 ms, queued 0 ms)";
    const FIXED_BYTES: usize = PREFIX.len() + SUFFIX.len();
    for (fault, failure, name) in [
        (Fault::Unreachable, Failure::Transient, b"Unreachable".as_slice()),
        (Fault::Refused, Failure::Refused { repository: 0 }, b"Refused".as_slice()),
        (Fault::Broken, Failure::Transient, b"Broken".as_slice()),
        (
            Fault::Missing { missing: Missing::Branch },
            Failure::Missing { repository: 0, missing: Missing::Branch },
            b"Missing(Branch)".as_slice(),
        ),
    ] {
        let (mut h, hold) = fetching_counts();
        let length = FIXED_BYTES.checked_add(name.len()).expect("expected detail is short");
        let mut detail = Writer::new(length);
        for field in [PREFIX, name, SUFFIX] {
            detail.put(field).expect("the expected detail fits");
        }
        assert_eq!(
            prepared(h.one(Event::Done { owner: hold, done: failed(fault) })),
            Prepared::Failed { failure, detail: detail.finish() }
        );
    }
}

#[test]
fn interrupting_counts_ends_the_prepare_aborted_even_on_success() {
    const CLIENT: Token = Token::new(1);
    const ABORTED: Request = Request::Prepared { client: CLIENT, prepared: Prepared::Aborted };
    const RELEASED: Request = Request::Released { client: CLIENT };
    for (release, expected) in [(false, &[ABORTED][..]), (true, &[ABORTED, RELEASED][..])] {
        let (mut h, hold) = fetching_counts();
        let interrupt = if release { Event::Release { hold } } else { Event::Abort { hold } };
        assert_eq!(h.one(interrupt), Request::Cancel { owner: hold });
        let settled = h.step(Event::Done { owner: hold, done: Done::Succeeded });
        assert_eq!(settled.as_slice(), expected);
    }
}
