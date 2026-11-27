use super::*;

const TOPIC: &[u8] = b"topic";
const SAVED: &[u8] = b"saved/1";
const BASE: &[u8] = b"goal";
const CLIENT: Token = Token::new(1);

fn fetched_tip(branch: &[u8], op: Op) {
    let Op::Fetch { want: Want::Branch { branch: destination }, .. } = op else { panic!("read destination: {op:?}") };
    assert_eq!(&*destination, branch);
}

fn missing_branch() -> Done {
    failed(Fault::Missing { missing: Missing::Branch })
}

fn ask(hold: Token, branch: &[u8], expected: Option<Commit>, head: Commit) -> Request {
    Request::AdmitPush { client: CLIENT, hold, repository: 0, branch: bytes(branch), expected, commit: head }
}

#[test]
fn delivery_and_both_saves_wait_and_use_the_fresh_tip_as_their_lease() {
    for which in 0..3_u32 {
        for expected in [None, Some(commit(8))] {
            let mut h = Harness::new(LIMITS);
            let repo = Repository { admit: true, push: Some(bytes(TOPIC)), ..repository(b"a", true) };
            let hold = h.ready(CLIENT.raw(), spec(b"w", Box::new([repo])));
            let (event, branch) = match which {
                0 => (Event::Push { hold, message: message() }, TOPIC),
                1 => (Event::PushUnchecked { hold }, TOPIC),
                2 => (Event::Save { hold, message: message() }, SAVED),
                _ => unreachable!("three ways to write"),
            };
            let Op::Commit { at, .. } = io(h.one(event), hold) else { panic!("write first commits locally") };
            let head = commit(9);
            fetched_tip(branch, h.next(hold, Done::Committed { commit: head }));
            let done = match expected {
                Some(commit) => Done::Fetched { commit },
                None => missing_branch(),
            };
            assert_eq!(h.one(Event::Done { owner: hold, done }), ask(hold, branch, expected, head));
            let op = io(h.one(Event::PushAdmission { hold, answer: Ok(()) }), hold);
            let leased = match expected {
                Some(_) => Op::Push {
                    at,
                    remote: remote(b"a"),
                    commit: head,
                    branch: bytes(branch),
                    expected,
                    identity: 0,
                    account: None,
                },
                None => Op::Create {
                    at,
                    remote: remote(b"a"),
                    commit: head,
                    branch: bytes(branch),
                    identity: 0,
                    account: None,
                },
            };
            assert_eq!(op, leased);
            let end = h.one(Event::Done { owner: hold, done: Done::Succeeded });
            let outcome = Outcome::Pushed { landings: Box::new([Landing::Landed { commit: head }]) };
            let expected = if which == 2 {
                Request::Saved { client: CLIENT, outcome }
            } else {
                Request::Pushed { client: CLIENT, outcome }
            };
            assert_eq!(end, expected);
            h.one(Event::Release { hold });
        }
    }
}

#[test]
fn a_base_creation_rechecks_absence_and_waits_for_admission() {
    for admitted in [false, true] {
        let mut h = Harness::new(LIMITS);
        let repo = Repository { admit: true, start: Start::Base { branch: bytes(BASE) }, ..repository(b"a", true) };
        let (hold, _) = h.prepare(CLIENT.raw(), spec(b"w", Box::new([repo])));
        assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Clone);
        fetched_tip(BASE, h.next(hold, Done::Succeeded));
        let default = h.next(hold, missing_branch());
        let Op::Fetch { want: Want::Default, .. } = default else { panic!("fetch the default first") };
        let head = commit(9);
        fetched_tip(BASE, h.next(hold, Done::Fetched { commit: head }));
        assert_eq!(h.one(Event::Done { owner: hold, done: missing_branch() }), ask(hold, BASE, None, head));
        if admitted {
            let Op::Create { branch, commit: created, .. } =
                io(h.one(Event::PushAdmission { hold, answer: Ok(()) }), hold)
            else {
                panic!("admission permits creation, not an unguarded push");
            };
            assert_eq!((&*branch, created), (BASE, head));
            assert_eq!(h.next(hold, Done::Succeeded).kind(), Kind::Fetch);
            assert_eq!(h.replaced(hold, commit(1)).kind(), Kind::CheckOut);
            let Prepared::Ready { .. } = prepared(h.one(Event::Done { owner: hold, done: Done::Succeeded })) else {
                panic!("prepared")
            };
        } else {
            let rows = h.one(Event::PushAdmission { hold, answer: Err(bytes(b"no room")) });
            assert_eq!(
                prepared(rows),
                Prepared::Failed {
                    failure: Failure::Refused { repository: 0 },
                    detail: bytes(b"temper did not admit the push to goal: no room"),
                }
            );
        }
        h.one(Event::Release { hold });
    }
}

#[test]
fn aborting_a_waiting_push_withdraws_the_question_without_io() {
    let mut h = Harness::new(LIMITS);
    let repo = Repository { admit: true, push: Some(bytes(TOPIC)), ..repository(b"a", true) };
    let hold = h.ready(CLIENT.raw(), spec(b"w", Box::new([repo])));
    h.one(Event::Push { hold, message: message() });
    h.next(hold, Done::Committed { commit: commit(9) });
    assert_eq!(h.one(Event::Done { owner: hold, done: missing_branch() }), ask(hold, TOPIC, None, commit(9)));
    let rows = h.step(Event::Abort { hold });
    assert_eq!(
        rows.as_slice(),
        &[
            Request::WithdrawAdmission { client: CLIENT },
            Request::Pushed { client: CLIENT, outcome: Outcome::Pushed { landings: Box::new([Landing::Aborted]) } },
        ]
    );
    h.none(Event::PushAdmission { hold, answer: Ok(()) });
    h.one(Event::Release { hold });
}
