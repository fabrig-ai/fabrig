use super::*;
use crate::backend::fixtures::pop_io;

const PIPE: Token = Token::new(31);
const MAXIMUM: u32 = 1024;

#[test]
fn a_matching_acknowledgement_releases_the_sent_request_once() {
    let mut interrupt = Interrupt::new();
    let mut below = Queue::uncapped();
    interrupt.prepare(MAXIMUM, &mut below).expect("bounded control line");
    assert!(below.is_empty());
    interrupt.offer(Some(PIPE), &mut below);
    let Some(io::Request::Output { down: stream::OutputDown::Room { right, .. }, .. }) = pop_io(&mut below) else {
        panic!("one room request")
    };
    assert_eq!(right, INTERRUPT_RIGHT);
    interrupt.offer(Some(PIPE), &mut below);
    assert!(below.is_empty(), "one outstanding output right");
    interrupt.send(PIPE, &mut below, &mut None);
    let Some(io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. }) = pop_io(&mut below) else {
        panic!("one control line")
    };
    assert_eq!(Some(bytes), records::interrupt_request(1, MAXIMUM));
    assert!(!interrupt.acknowledge(Some(b"wrong"), &mut below));
    let id = records::interrupt_id(1);
    assert!(interrupt.acknowledge(Some(&id), &mut below));
    assert!(!interrupt.acknowledge(Some(&id), &mut below));
    interrupt.send(PIPE, &mut below, &mut None);
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
    interrupt.send(PIPE, &mut below, &mut None);
    assert_eq!(below.len(), 1);
    below.clear();
    interrupt.send(PIPE, &mut below, &mut None);
    assert!(below.is_empty());
    interrupt.prepare(MAXIMUM, &mut below).expect("next control line");
    assert_eq!(interrupt.sequence(), 2);
    interrupt.forget(&mut below);
    interrupt.offer(None, &mut below);
    assert!(below.is_empty());
    interrupt.restart(&mut below);
    assert_eq!(interrupt.sequence(), 0);
}

#[test]
fn an_interrupt_keeps_its_whole_input_only_when_asked() {
    for (keep, early) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut interrupt = Interrupt::new();
        let mut below = Queue::uncapped();
        let mut given = if keep { Some(Queue::uncapped()) } else { None };
        interrupt.prepare(MAXIMUM, &mut below).expect("bounded control line");
        if early {
            let id = records::interrupt_id(1);
            assert!(interrupt.acknowledge(Some(&id), &mut below));
        }
        interrupt.offer(Some(PIPE), &mut below);
        below.clear();
        interrupt.send(PIPE, &mut below, &mut given);
        let Some(io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. }) = pop_io(&mut below) else {
            panic!("control line")
        };
        if keep {
            let given = given.as_mut().expect("asked");
            let Some(Given::Input(input)) = given.pop() else { panic!("kept input") };
            assert_eq!(input, bytes);
            assert!(given.is_empty());
        } else {
            assert!(given.is_none());
        }
    }
}
