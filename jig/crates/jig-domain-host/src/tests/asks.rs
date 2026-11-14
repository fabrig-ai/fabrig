use super::*;
use crate::AskAnswer;

fn ask(h: &mut Harness, run: u64, attempt: u64, bytes: &[u8]) -> Box<[Request]> {
    h.step(Event::Ask { run: Token::new(run), attempt: Token::new(attempt), bytes: bytes.into() })
}
fn asked(h: &mut Harness, number: u64, answer: AskAnswer) -> Box<[Request]> {
    h.step(Event::Down(LinkDown::Asked { run: Token::new(1), attempt: Token::new(1001), number, answer }))
}
fn refused(run: u64, attempt: u64, number: u64) -> Request {
    Request::Asked { run: Token::new(run), attempt: Token::new(attempt), number, answer: AskAnswer::Refused }
}
fn sent(run: u64, number: u64, bytes: &[u8]) -> Request {
    Request::Up(LinkUp::Ask {
        run: Token::new(run),
        attempt: Token::new(run.checked_add(1000).expect("small run")),
        number,
        bytes: bytes.into(),
    })
}

#[test]
fn asks_are_numbered_per_attempt_and_bounded_at_two() {
    let mut h = Harness::new();
    h.assign(1);
    h.assign(2);
    assert_eq!(&*ask(&mut h, 1, 1001, b"xxxx"), &[sent(1, 1, b"xxxx")]);
    assert_eq!(&*ask(&mut h, 1, 1001, b"x"), &[sent(1, 2, b"x")]);
    assert_eq!(&*ask(&mut h, 1, 1001, b"x"), &[refused(1, 1001, 3)]);
    assert_eq!(&*ask(&mut h, 2, 1002, b"x"), &[sent(2, 1, b"x")]);
    asked(&mut h, 1, AskAnswer::Refused);
    assert_eq!(&*ask(&mut h, 1, 1001, b"x"), &[sent(1, 4, b"x")]);
}

#[test]
fn unhosted_attempts_and_oversized_asks_are_refused_at_once() {
    let mut h = Harness::new();
    assert_eq!(&*ask(&mut h, 1, 1001, b"x"), &[refused(1, 1001, 0)]);
    h.assign(1);
    assert_eq!(&*ask(&mut h, 1, 999, b"x"), &[refused(1, 999, 0)]);
    assert_eq!(&*ask(&mut h, 1, 1001, b"xxxxx"), &[refused(1, 1001, 1)]);
    assert_eq!(&*ask(&mut h, 1, 1001, b"xxxx"), &[sent(1, 2, b"xxxx")]);
}

#[test]
fn only_the_first_answer_for_an_outstanding_ask_reaches_the_root() {
    let mut h = Harness::new();
    h.assign(1);
    ask(&mut h, 1, 1001, b"x");
    assert!(asked(&mut h, 2, AskAnswer::Refused).is_empty());
    assert!(
        h.step(Event::Down(LinkDown::Asked {
            run: Token::new(1),
            attempt: Token::new(999),
            number: 1,
            answer: AskAnswer::Refused
        }))
        .is_empty()
    );
    assert_eq!(
        &*asked(&mut h, 1, AskAnswer::Bytes(b"yyyy".as_slice().into())),
        &[Request::Asked {
            run: Token::new(1),
            attempt: Token::new(1001),
            number: 1,
            answer: AskAnswer::Bytes(b"yyyy".as_slice().into())
        }]
    );
    assert!(asked(&mut h, 1, AskAnswer::Refused).is_empty());
    ask(&mut h, 1, 1001, b"x");
    assert_eq!(&*asked(&mut h, 2, AskAnswer::Bytes(b"yyyyy".as_slice().into())), &[refused(1, 1001, 2)]);
}

#[test]
fn losing_the_link_refuses_each_outstanding_ask_without_a_timer() {
    let mut h = Harness::new();
    h.assign(1);
    ask(&mut h, 1, 1001, b"x");
    ask(&mut h, 1, 1001, b"x");
    assert_eq!(h.domain.next_deadline(), None);
    assert_eq!(&*h.step(Event::LinkLost), &[refused(1, 1001, 1), refused(1, 1001, 2)]);
    assert!(h.step(Event::LinkLost).is_empty());
    assert!(asked(&mut h, 1, AskAnswer::Refused).is_empty());
    assert_eq!(&*ask(&mut h, 1, 1001, b"x"), &[sent(1, 3, b"x")]);
}

#[test]
fn an_attempt_no_longer_hosted_forgets_its_asks() {
    let mut h = Harness::new();
    let (owner, _) = h.assign(1);
    ask(&mut h, 1, 1001, b"x");
    h.step(Event::Unprepared { owner, failure: Preparation::Transient, detail: Box::new([]) });
    assert!(!h.domain.names.contains_key(&Token::new(1)), "the terminal keeps only its answer, not its asks");
    assert!(asked(&mut h, 1, AskAnswer::Refused).is_empty());
    assert!(h.step(Event::LinkLost).is_empty());
    assert_eq!(&*ask(&mut h, 1, 1001, b"x"), &[refused(1, 1001, 0)]);
}
