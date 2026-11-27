//! Admission through the worker's codec handoffs and the real hub.
use super::*;

const HEAD: Commit = Commit::new([9; 32]);
const TIP: Commit = Commit::new([8; 32]);

fn waiting() -> (Harness, Token, u64, u64) {
    let mut items = fixtures::items(&[b"app"]);
    let repo = &mut items.repositories[0];
    repo.admit = true;
    repo.tag = 173;
    repo.access = wire::Access::Writable {
        push: b"topic".as_slice().into(),
        expected: None,
        landing: None,
        delivered: None,
        pin: None,
    };
    let mut h = prepared_items(items);
    let hold = landed::delivering(&mut h);
    let rows = h.step(Event::Done { owner: hold, done: Done::Committed { commit: HEAD } });
    let [
        Request::Git { .. },
        Request::Io { op: Op::Fetch { want: temper_worker_domain_checkout::git::Want::Branch { branch }, .. }, .. },
    ] = rows.as_ref()
    else {
        panic!("just a destination read: {rows:?}")
    };
    assert_eq!(branch.as_ref(), b"topic");
    let rows = h.step(Event::Done { owner: hold, done: Done::Fetched { commit: TIP } });
    let [Request::AdmitAsk { client, cookie, ask }] = rows.as_ref() else {
        panic!("just an admission question: {rows:?}")
    };
    assert_eq!(
        *ask,
        wire::Ask::Admit {
            tag: 173,
            branch: b"topic".as_slice().into(),
            expected: Some(TIP.raw()),
            commit: HEAD.raw()
        }
    );
    let (client, cookie) = (*client, *cookie);
    let rows = h.step(Event::AdmitAsk { client, cookie, bytes: b"ask".as_slice().into() });
    let [Request::Link(host::LinkUp::Ask { run, attempt, number, bytes })] = rows.as_ref() else {
        panic!("only a numbered hub ask: {rows:?}")
    };
    assert_eq!((*run, *attempt, bytes.as_ref()), (fixtures::RUN, fixtures::ATTEMPT, b"ask".as_slice()));
    (h, client, cookie, *number)
}

#[test]
fn a_push_is_refused_on_each_failure_to_obtain_admission() {
    let cases = [
        (Some(Some(wire::Answer::Unadmitted { words: b"no room".as_slice().into() })), b"no room".as_slice()),
        (Some(None), b"the engine's answer did not decode".as_slice()),
        (Some(Some(wire::Answer::Landed)), b"the engine answered a different ask".as_slice()),
        (None, b"the engine refused the ask".as_slice()),
        (None, b"the engine did not answer in time".as_slice()),
    ];
    for (decoded, why) in cases {
        let (mut h, client, cookie, number) = waiting();
        let rows = match decoded {
            Some(answer) => {
                let rows = landed::reply(&mut h, number, host::AskAnswer::Bytes(b"answer".as_slice().into()));
                assert_eq!(
                    rows.as_ref(),
                    &[Request::AdmitAnswer { client, cookie, bytes: b"answer".as_slice().into() }]
                );
                h.step(Event::AdmitAnswer { client, cookie, answer })
            }
            None if why == b"the engine did not answer in time" => {
                h.env.now =
                    h.env.now.saturating_add(Duration::from_millis(u64::from(h.env.limits.checkout.reds_read_ms)));
                landed::expired(&mut h)
            }
            None => landed::reply(&mut h, number, host::AskAnswer::Refused),
        };
        let [Request::Process(process)] = rows.as_ref() else { panic!("only the delivery refusal, no IO: {rows:?}") };
        let agent::process::Request::Send {
            message: agent::Down::Answer { reply: agent::Reply::Delivery(agent::Delivery::Refused(refusal)), .. },
            ..
        } = process.as_ref()
        else {
            panic!("refused: {process:?}")
        };
        assert_eq!(
            refusal.explanation(),
            &*skein_lib::bytes::concat(&[b"temper did not admit the push to topic: ", why])
        );
        assert!(landed::reply(&mut h, number, host::AskAnswer::Bytes(b"late".as_slice().into())).is_empty());
        assert!(h.step(Event::AdmitAnswer { client, cookie, answer: Some(wire::Answer::Admitted) }).is_empty());
    }
}
