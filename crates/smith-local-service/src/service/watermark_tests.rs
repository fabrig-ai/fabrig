//! Tests local watermark admission and backlog accounting (protocol/limits.md, section 3.8).

use alloc::boxed::Box;
use skein_lib::{Time, Wall};
use smith_local_domain as local;

use crate::tests::config;
use crate::{Service, iterate};

// Two admitted entrances hold admission, then the line leaves Credential and
// SaveState waiting on the full shell edge. The watermark is their entry count.
const HELD_BACKLOG: u32 = 2;

fn admitted_at_watermark() -> Service {
    let mut config = config();
    config.limits.watermark = HELD_BACKLOG;
    let mut service = Service::new(config, 7).unwrap();
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    service.store_requests().pop().expect("initial Load handed to the store");
    for _ in 0..service.shell.capacity() {
        service.shell.push(local::Request::Credential { account: 99 });
    }
    service.local_event(local::Event::Loaded { state: None, transcript: None, deliveries: Box::new([]) });
    service.local_event(local::Event::Line { text: b"admitted".as_slice().into() });
    service
}

fn feed_line(service: &mut Service, line: &[u8]) {
    for byte in line {
        service.feed(*byte);
    }
}

#[test]
fn a_line_waits_at_the_watermark_while_an_admitted_answer_is_taken() {
    // The first iteration consumes Loaded and the already admitted line;
    // the next iteration still has no shell credit for their requests.
    const HELD_ITERATIONS: u32 = 2;
    let mut service = admitted_at_watermark();
    feed_line(&mut service, b"later\n");
    for _ in 0..HELD_ITERATIONS {
        iterate(&mut service, Time::ZERO, Wall::EPOCH);
        assert_eq!(service.terminal_events.len(), 1);
        assert!(service.local_events.is_empty(), "the admitted store answer was stepped");
        assert_eq!(service.local_requests.len(), HELD_BACKLOG);
    }
    while service.shell.pop().is_some() {}
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.terminal_events.is_empty(), "returning shell credit releases admission");
    let Some(local::Event::Line { text }) = service.local_events.iter().next() else {
        panic!("the released line waits at the local entrance")
    };
    assert_eq!(text.as_ref(), b"later");
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.local_events.is_empty(), "the released line reaches the local domain");
}

#[test]
fn an_interrupt_is_taken_at_the_watermark() {
    let mut service = admitted_at_watermark();
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    service.interrupt();
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.terminal_events.is_empty());
    assert!(service.local_events.is_empty());
    assert_eq!(service.interrupted_credentials, 1, "Interrupt cancels the undispatched credential prefix");
}

#[test]
fn work_pending_is_false_while_only_a_waiting_line_remains() {
    let mut service = admitted_at_watermark();
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    feed_line(&mut service, b"later\n");
    assert!(!service.work_pending(Time::ZERO), "the shell holds the backlog at the watermark");
    service.shell.pop().expect("shell returns edge credit");
    assert!(service.work_pending(Time::ZERO));
}

#[test]
fn the_backlog_counts_internal_queues_and_never_edges() {
    use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
    use skein_lib::Token;
    use smith_host_domain::{Input, process};
    let mut service = Service::new(config(), 7).unwrap();
    let owner = Token::new(91);
    service.completions.push(Complete { op: owner, kind: Op::Close { fd: Fd::new(9) }, result: Ok(Done::Nothing) });
    service.submissions.push(Submit { op: owner, kind: Op::Close { fd: Fd::new(9) } });
    service.output.push(b"edge".as_slice().into());
    service.shell.push(local::Request::Credential { account: 99 });
    service.interrupt();
    service.writer_events().push(skein_io::Event::Closed { owner });
    assert_eq!(service.backlog(), 0, "all service and process edges are excluded");
    service.local_events.push(local::Event::StateSaved);
    assert_eq!(service.backlog(), 1);
    service.local_requests.push(local::Request::Credential { account: 7 });
    assert_eq!(service.backlog(), 2);
    service.host_events.push(Input::Process(process::Event::Hangup { owner }));
    assert_eq!(service.backlog(), 3);
    service.lower.push(process::Request::Read { owner, process: owner });
    assert_eq!(service.backlog(), 4);
    service.below.process_mut().store_request(smith_local_protocol::StoreRequest::Load);
    assert_eq!(service.backlog(), 5, "process internal handoffs contribute too");
    service.writer_request(skein_io::Request::Close { entity: owner });
    assert_eq!(service.backlog(), 6, "writer requests join the internal IO handoff");
}
