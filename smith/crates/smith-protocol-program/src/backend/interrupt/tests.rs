use super::*;

const PIPE: Token = Token::new(31);
const MAXIMUM: u32 = 1024;

#[test]
fn a_matching_acknowledgement_releases_the_sent_request_once() {
    let mut interrupt = Interrupt::new();
    let mut below = Queue::uncapped();
    interrupt.prepare(MAXIMUM, &mut below).expect("bounded control line");
    assert!(below.is_empty());
    interrupt.offer(Some(PIPE), &mut below);
    let Some(io::Request::Output { down: stream::OutputDown::Room { right, .. }, .. }) = below.pop() else {
        panic!("one room request")
    };
    assert_eq!(right, INTERRUPT_RIGHT);
    interrupt.offer(Some(PIPE), &mut below);
    assert!(below.is_empty(), "one outstanding output right");
    interrupt.send(PIPE, &mut below);
    let Some(io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. }) = below.pop() else {
        panic!("one control line")
    };
    assert_eq!(Some(bytes), records::interrupt_request(1, MAXIMUM));
    assert!(!interrupt.acknowledge(Some(b"wrong"), &mut below));
    let id = records::interrupt_id(1);
    assert!(interrupt.acknowledge(Some(&id), &mut below));
    assert!(!interrupt.acknowledge(Some(&id), &mut below));
    interrupt.send(PIPE, &mut below);
    assert!(below.is_empty());
    assert_eq!(interrupt.sequence(), 1);
}

#[test]
fn an_early_acknowledgement_keeps_the_unsent_line() {
    let mut interrupt = Interrupt::new();
    let mut below = Queue::uncapped();
    interrupt.prepare(MAXIMUM, &mut below).expect("bounded control line");
    let id = records::interrupt_id(1);
    assert!(interrupt.acknowledge(Some(&id), &mut below));
    interrupt.offer(Some(PIPE), &mut below);
    below.clear();
    interrupt.send(PIPE, &mut below);
    assert_eq!(below.len(), 1);
    below.clear();
    interrupt.send(PIPE, &mut below);
    assert!(below.is_empty());
    interrupt.prepare(MAXIMUM, &mut below).expect("next control line");
    assert_eq!(interrupt.sequence(), 2);
    interrupt.forget(&mut below);
    interrupt.offer(None, &mut below);
    assert!(below.is_empty());
    interrupt.restart(&mut below);
    assert_eq!(interrupt.sequence(), 0);
}
