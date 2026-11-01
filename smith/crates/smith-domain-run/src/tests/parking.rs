//! Finite waits at a turn boundary preserve work for a fresh activation (smith/docs/design/domain/run.md, sections 6 and 9.1).
use super::*;

#[test]
fn finite_wait_on_the_last_reserved_turn_parks_and_leaves_queued_words_unread() {
    for queued in [false, true] {
        let mut h = Harness::new(LIMITS);
        let mut selected = charter();
        selected.budget.turns = 3;
        selected.reserve = crate::charter::Reserve { turns: 1, time: Duration::from_secs(10) };
        let (run, main) = h.running_on(1, 100, selected);
        if queued {
            drop(h.step(Event::Message {
                run,
                name: Token::new(99),
                label: bytes(b"person"),
                text: bytes(b"Go ahead"),
                finish_requested: false,
            }));
        }
        drop(h.step(Event::Used { conversation: main, spend: Spend { turns: 2, ..Spend::ZERO } }));
        let crate::CompletionPermit::Final { .. } =
            crate::completion_permit(&mut h.domain, main, h.env.now, crate::Most::Unreserved)
        else {
            panic!("ordinary turns enter wind-down")
        };
        assert_eq!(crate::reserve(&mut h.domain, main, Token::new(500), crate::Most::Unreserved), Ok(()));
        assert!(crate::settle_reservation(&mut h.domain, main, Token::new(500), 0));
        drop(h.step(Event::Used { conversation: main, spend: Spend { turns: 1, ..Spend::ZERO } }));
        let emitted = h.step(Event::Delegated {
            conversation: main,
            call: Token::new(20),
            name: crate::CallName { activation: 1, completion: 3, position: 0 },
            ask: Ask::Wait,
            deadline: EXPIRY,
        });
        assert_eq!(
            emitted.as_ref(),
            &[
                Request::Waiting { host_run: Token::new(1), read: None },
                returned(20, Returned::Withheld),
                Request::Close { peer: PEER },
            ]
        );
        let answer = answered(h.step(ended(main, End::Closed, Spend { turns: 3, ..Spend::ZERO }))).1;
        let Answer::Parked { spent, .. } = answer else { panic!("the wait parks") };
        assert_eq!(spent.turns, 3);
        let mut unread = 0;
        for fact in &facts(&mut h) {
            if let Fact::MessageUnread { name, .. } = fact {
                assert_eq!(*name, Token::new(99));
                unread += 1;
            }
        }
        assert_eq!(unread, usize::from(queued));
    }
}

#[test]
fn finite_wait_keeps_hard_money_and_wall_limits_even_on_the_last_turn() {
    for money in [false, true] {
        let mut h = Harness::new(LIMITS);
        let mut selected = charter();
        selected.budget.turns = 3;
        selected.reserve = crate::charter::Reserve { turns: 1, time: Duration::from_secs(10) };
        let (_, main) = h.running_on(1, 100, selected);
        let spend = Spend { turns: 3, units: if money { BUDGET.spend } else { 0 }, ..Spend::ZERO };
        drop(h.step(Event::Used { conversation: main, spend }));
        if !money {
            h.after(BUDGET.time);
        }
        let emitted = h.step(Event::Delegated {
            conversation: main,
            call: Token::new(20),
            name: crate::CallName { activation: 1, completion: 3, position: 0 },
            ask: Ask::Wait,
            deadline: EXPIRY,
        });
        assert_eq!(emitted.as_ref(), &[returned(20, Returned::Refused { refusal: AskRefusal::Over })]);
    }
}
