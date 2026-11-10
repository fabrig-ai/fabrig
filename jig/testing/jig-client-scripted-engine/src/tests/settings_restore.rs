//! A restore is proposed by any party and carries its complete answer (client/domain/contract/requests.md, 2).
use super::*;
#[test]
fn a_non_owner_can_ask_for_a_scripted_restore() {
    for answer in [
        Answer::Done(Outcome::Restored {
            proposal: 41,
            to: 3,
            left_out: list(vec![
                Setting::ProjectPolicy { project: 7 },
                Setting::Application { kind: 2, scope: 7, number: 9 },
            ]),
        }),
        Answer::Refused { refusal: Refusal::Forgotten { oldest: 4 }, at: None },
    ] {
        let ask = Ask::Restore { revision: 3, reason: text("restore λ".as_bytes()) };
        let mut setup = script();
        assert_ne!(
            setup.truth.frames.get(&REQUESTER).expect("fixture party frame").deployment_role,
            Some(HeldRole::Owner)
        );
        setup.rules.push(Rule { party: Some(REQUESTER), ask: ask.clone(), answer: answer.clone(), changes: vec![] });
        let mut engine = Engine::new(setup, 300).expect("valid script");
        send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask.clone());
        let answers = engine.poll(Time::from_nanos(10_000_001));
        assert_eq!(final_answer(&answers[0]), &answer);
        assert_eq!(engine.commits().len(), 1);
        assert_eq!(engine.commits()[0].ask, Some(ask));
    }
}
