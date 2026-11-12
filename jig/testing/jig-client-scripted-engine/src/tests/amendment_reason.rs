//! Whole request receiving and durable key equality include the supplied reason.
use super::*;
fn amendment(reason: &[u8]) -> Ask {
    Ask::Amend {
        project: PROJECT,
        task: 8,
        revision: Some(Revision { value: 1 }),
        reason: text(reason),
        change: Box::new(Amendment {
            llm: None,
            addressed: None,
            spec: None,
            instructions: Some(text(b"")),
            wake: None,
            dependencies: None,
            authority: None,
            tracked: None,
        }),
    }
}

#[test]
fn durable_amendment_key_replay_keeps_reason_and_changed_reason_is_a_conflict() {
    let mut setup = script();
    let mut row = task();
    row.acts = list(vec![Offered { effect: None, act: ActKind::Amend, admission: Admission::Allowed }]);
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    let ask = amendment(UTF8_TEXTS[2]);
    let mut engine = Engine::new(setup, 1).expect("valid script");
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask.clone());
    send(&mut engine, 1, REQUESTER_OTHER_SIGN_IN, 1, 2, ask.clone());
    assert_eq!(engine.poll(Time::from_nanos(9_999_999)).len(), 0);
    assert_eq!((engine.commits()).len(), 0);
    let up = engine.poll(Time::from_nanos(10_000_001));
    assert_eq!(up.len(), 2);
    assert_eq!(final_answer(&up[0]), final_answer(&up[1]));
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.commits()[0].ask.as_ref(), Some(&ask));
    assert_eq!(engine.commits()[0].key, Some([1; 16]));
    assert_eq!(engine.commits()[0].durable, Some(Time::from_nanos(10_000_000)));
    send(&mut engine, 20_000_000, REQUESTER_SIGN_IN, 1, 3, amendment(b"different"));
    assert_eq!(
        final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]),
        &Answer::Refused { refusal: Refusal::KeyConflict, at: None }
    );
    send(&mut engine, 20_000_000, REQUESTER_OTHER_SIGN_IN, 1, 4, ask.clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&up[0]));
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.commits()[0].ask.as_ref(), Some(&ask));
}

const UTF8_TEXTS: [&[u8]; 3] = [&[b'~'; 32], &[b'~'; 33], b"~\0!"];
