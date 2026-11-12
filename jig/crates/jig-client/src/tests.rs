mod accounts;
mod facts;
mod fixtures;
mod reads;
mod shapes;
mod sign_in;
mod store;
mod thread;
mod unlinked;
mod unreadable;
mod versions;
mod words;
use crate::*;
use fixtures::{Client, ME, PARTY, PROJECT, STOP, TASK, task};
#[test]
fn a_watch_asked_of_the_client_leaves_as_one_opening_on_the_stream() {
    let mut c = Client::signed_in();
    c.step(Event::Watch { handle: TASK, subject: task() });
    let [Request::Exchange { credential: Some(_), message: Message::Watch(_), .. }] = c.requests() else {
        panic!("one watch exchange: {:?}", c.requests());
    };
}
#[test]
fn a_reply_reaches_the_domain_as_its_decoded_answer() {
    let mut c = Client::signed_in();
    c.step(Event::Read { handle: TASK, read: Readable::Jig(contract::Read::Project { project: PROJECT }) });
    let exchange = c.exchange();
    // ReadAnswer::None(Unknown) is encoded by the fixture.
    c.step(Event::Reply { exchange, message: fixtures::unknown_read() });
    let [Request::Told { told: Told::Page { handle, page } }] = c.requests() else {
        panic!("one page: {:?}", c.requests());
    };
    assert_eq!(*handle, TASK);
    assert_eq!(*page, Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown)));
}
#[test]
fn an_application_watch_on_me_shares_the_client_s_own() {
    let mut c = Client::signed_in();
    c.step(Event::Watch { handle: ME, subject: Watchable::Jig(contract::Subject::Me) });
    let [Request::Told { told: Told::Changed { handle } }] = c.requests() else {
        panic!("one change: {:?}", c.requests());
    };
    assert_eq!(*handle, ME);
    assert_eq!(c.machine.document(ME).unwrap().state, State::Live);
    assert_eq!(c.machine.link().party, Some(PARTY));
}
#[test]
fn an_engine_not_serving_jig_s_version_leaves_the_link_out_of_date() {
    let mut c = Client::new();
    c.step(Event::SignIn { with: With::Held(skein_lib::Secret::new(Box::new([7; 40]))) });
    c.stream(fixtures::hello(0));
    assert_eq!(c.machine.link().state, Link::OutOfDate);
    assert!(!c.machine.link().unreadable);
    c.step(Event::Watch { handle: TASK, subject: task() });
    assert!(c.requests().is_empty());
    assert_eq!(c.machine.next_deadline(), None);
}
#[test]
fn a_request_parked_offline_is_sent_once_me_returns() {
    let mut c = Client::signed_in();
    c.step(Event::StreamGone);
    assert_eq!(c.machine.link().state, Link::Offline);
    assert!(!c.machine.link().unreadable);
    c.step(Event::Send { name: None, key: STOP, request: fixtures::stop() });
    assert_eq!(c.machine.pending(STOP).unwrap().progress, Progress::Parked);
    assert_eq!(fixtures::keyed_count(c.requests()), 0);
    c.step(Event::Stored);
    c.machine.fire(skein_lib::Time::from_nanos(100_000_000), skein_lib::Wall::EPOCH, &mut skein_lib::Queue::uncapped());
    c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    let watch = c.opened_watch();
    c.step(Event::Stored);
    c.stream(fixtures::frame(watch));
    assert_eq!(c.machine.link().state, Link::Live);
    c.step(Event::Stored);
    assert_eq!(fixtures::keyed_count(c.requests()), 1);
}
#[test]
fn a_read_whose_exchange_is_lost_is_answered_unreachable() {
    let mut c = Client::signed_in();
    c.step(Event::Read { handle: TASK, read: Readable::Jig(contract::Read::Project { project: PROJECT }) });
    c.step(Event::Lost { exchange: c.exchange() });
    let [Request::Told { told: Told::Page { handle, page } }] = c.requests() else {
        panic!("one terminal: {:?}", c.requests());
    };
    assert_eq!(*handle, TASK);
    assert_eq!(*page, Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unreachable)));
}

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    let base = fixtures::limits();
    assert!(valid(&base));
    let cases =
        [Limits { domain: DomainLimits { watches: 0, ..base.domain }, ..base }, Limits { exchanges: 0, ..base }];
    for (case, limits) in cases.into_iter().enumerate() {
        assert!(!valid(&limits), "case {case}");
    }
}

#[test]
fn presentation_reads_the_clients_configured_receiving_bounds() {
    let limits = fixtures::limits();
    let client = crate::Client::unlinked(&limits, 1).unwrap();
    assert_eq!(client.domain_limits(), &limits.domain);
}

#[test]
fn a_full_stream_reaches_the_domain_and_reopens_after_backoff() {
    for bound in [StreamBound::Component, StreamBound::Transport] {
        let mut c = Client::new();
        c.step(Event::SignIn { with: With::Held(skein_lib::Secret::new(Box::new([7; 40]))) });
        let full = StreamsFull { bound, size: 2 };
        c.step(Event::Streamed { message: Streamed::Full(full) });
        assert_eq!(c.machine.link().streams_full, Some(full));
        assert_eq!(c.machine.link().state, Link::Offline);
        let mut told = false;
        let mut closed = false;
        for request in c.requests() {
            if let Request::Told { told: Told::Link { .. } } = request {
                told = true;
            }
            if let Request::CloseStream = request {
                closed = true;
            }
        }
        assert!(told);
        assert!(closed);
        c.fire(c.machine.next_deadline().unwrap());
        let mut opened = false;
        for request in c.requests() {
            if let Request::OpenStream { .. } = request {
                opened = true;
            }
        }
        assert!(opened);
        assert_eq!(c.machine.link().streams_full, Some(full));
        c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
        let watch = c.opened_watch();
        c.stream(fixtures::frame(watch));
        assert_eq!(c.machine.link().state, Link::Live);
        assert_eq!(c.machine.link().streams_full, None);
    }
}

#[test]
fn the_link_view_names_a_watch_refused_busy() {
    use jig_client_protocol_wire::v20 as w;
    let mut c = Client::signed_in();
    assert!(!c.machine.link().watches_full);
    c.step(Event::Watch { handle: TASK, subject: task() });
    let watch = c.opened_watch();
    c.stream(fixtures::event(&w::StreamEvent::Ended(
        w::WatchEnded::new(&w::CEILINGS, w::WatchEndedParts { watch, why: w::WatchEnd::Busy }).unwrap(),
    )));
    assert!(c.machine.link().watches_full);
    assert_eq!(c.machine.link().state, Link::Live);
    let [Request::Told { told: Told::Changed { handle } }] = c.requests() else {
        panic!("one change: {:?}", c.requests());
    };
    assert_eq!(*handle, TASK);
    c.fire(c.machine.next_deadline().unwrap());
    assert!(c.machine.link().watches_full);
    let exchange = c.exchange();
    let answer = w::OpenAnswer::Admitted;
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut bytes).unwrap();
    c.step(Event::Reply { exchange, message: bytes.finish() });
    assert!(!c.machine.link().watches_full);
    assert_eq!(c.machine.link().state, Link::Live);
    assert!(c.requests().is_empty());
}

mod approval;
