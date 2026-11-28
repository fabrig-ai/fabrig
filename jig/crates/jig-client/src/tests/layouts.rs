use super::fixtures::{self, Client, ME, PROJECT, STOP, TASK};
use crate::*;
use jig_client_protocol_wire::{self as w, Record as _};
use skein_lib::{Duration, List, Time};

const FAMILY: u16 = 1;
const VERSION: u32 = 1;
const LAYOUT: [u8; 32] = [1; 32];
const OTHER: [u8; 32] = [2; 32];
const OWNERS: &[Spoken] = &[Spoken { family: FAMILY, version: VERSION, layout: LAYOUT }];

fn starting(owners: &'static [Spoken]) -> Client {
    let limits = Limits { domain: DomainLimits { owners, ..fixtures::domain_limits() }, ..fixtures::limits() };
    let mut c = Client::with_limits(limits, List::with_capacity(0));
    c.step(Event::SignIn { with: With::Transport });
    c
}

fn opened(app: Option<[u8; 32]>) -> w::Opened {
    let family = w::Owner::Family(w::Family { number: FAMILY });
    let served = List::from_boxed(Box::new([
        w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION },
        w::Served { owner: family, first: VERSION, last: VERSION },
    ]));
    let hello = w::Hello { stream: [7; 16], served, heartbeat: Duration::from_millis(100) };
    let layout = match app {
        Some(layout) => Some(jig_client_test_records::family_layout(FAMILY, layout)),
        None => None,
    };
    let w::StreamEvent::Opened(opened) =
        jig_client_test_records::opened_with_layouts(hello, jig_client_test_records::frame_record(), layout.as_slice())
    else {
        unreachable!("fixture Opened")
    };
    opened
}

fn family() -> Family {
    Family { family: FAMILY, version: VERSION, bytes: Box::new([0]) }
}
fn window() -> contract::Window {
    contract::Window { rows: 1, after: None }
}
fn family_read() -> Event {
    Event::Read {
        handle: TASK,
        read: Readable::Family { scope: contract::Scope::Own, window: window(), record: family() },
    }
}
fn jig_read() -> Event {
    Event::Read { handle: ME, read: Readable::Jig(contract::Read::Project { project: PROJECT }) }
}
fn no_transport(c: &Client) {
    for request in c.requests() {
        if let Request::Exchange { .. } | Request::OpenStream { .. } = request {
            panic!("transport left: {request:?}");
        }
    }
}

#[test]
fn another_or_missing_jig_layout_refuses_before_reading_me() {
    for layout in [Some(OTHER), None] {
        let mut c = starting(&[]);
        c.step(jig_read());
        let mut opened = opened(None);
        opened.layouts = List::with_capacity(1);
        if let Some(layout) = layout {
            opened.layouts.push(w::Layout { owner: w::Owner::Jig, layout }).unwrap();
        }
        let me_bytes = usize::try_from(opened.me.measure(&w::CEILINGS).unwrap()).unwrap();
        let bytes = fixtures::event(&w::StreamEvent::Opened(opened));
        let head_bytes = bytes.len().checked_sub(me_bytes).expect("fixed head");
        // The unreadable me must not hide the named refusal.
        c.stream(Box::from(&bytes[..head_bytes]));
        let link = c.machine.link();
        assert_eq!(link.state, Link::OutOfDate);
        assert!(!link.unreadable, "the fixed head refuses the layout by name");
        assert!(c.requests().contains(&Request::CloseStream));
        assert_eq!(fixtures::pages(c.requests()).as_slice(), &[(ME, Readout::NotServed { family: None })]);
        no_transport(&c);
        c.fire(Time::from_nanos(u64::MAX));
        no_transport(&c);
    }
}

#[test]
fn another_or_missing_owner_layout_refuses_only_its_work() {
    for layout in [Some(OTHER), None] {
        let mut c = starting(OWNERS);
        c.step(Event::Watch {
            handle: TASK,
            subject: Watchable::Family { scope: contract::Scope::Own, window: window(), record: family() },
        });
        c.step(family_read());
        no_transport(&c);
        assert!(fixtures::pages(c.requests()).is_empty());

        c.stream(fixtures::event(&w::StreamEvent::Opened(opened(layout))));
        let link = c.machine.link();
        assert_eq!(link.state, Link::Live);
        assert_eq!(link.not_served, &[FAMILY]);
        let refusal = [(TASK, Readout::NotServed { family: Some(FAMILY) })];
        assert_eq!(fixtures::pages(c.requests()).as_slice(), &refusal);
        assert!(c.requests().contains(&Request::Told { told: Told::Ended { handle: TASK, why: Why::NotServed } }));
        no_transport(&c);

        c.step(Event::Send {
            key: STOP,
            name: None,
            request: Ask::Family { scope: contract::Scope::Own, record: family() },
        });
        assert_eq!(fixtures::saved(c.requests()).len(), 1, "the refused owner's request is kept");
        assert_eq!(c.machine.pending(STOP).expect("request kept").progress, Progress::NotServed);
        no_transport(&c);
        c.step(family_read());
        assert_eq!(fixtures::pages(c.requests()).as_slice(), &refusal);
        no_transport(&c);
    }
}

#[test]
fn a_read_after_a_drop_waits_for_the_next_opened() {
    let mut c = Client::signed_in();
    c.step(Event::StreamGone);
    c.step(jig_read());
    no_transport(&c);
    assert!(fixtures::pages(c.requests()).is_empty());
    let until = c.machine.next_deadline().expect("reopening deadline");
    c.fire(until);
    let [Request::OpenStream { .. }] = c.requests() else { panic!("opens only the stream") };
    c.step_at(until, Event::Streamed { message: Streamed::Record(fixtures::hello(w::VERSION)) });
    assert_eq!(c.machine.link().state, Link::Live);
    let mut sent = 0_u32;
    for request in c.requests() {
        if let Request::Exchange { message: Message::Read(_), .. } = request {
            sent = sent.checked_add(1).unwrap();
        }
    }
    assert_eq!(sent, 1);
}
