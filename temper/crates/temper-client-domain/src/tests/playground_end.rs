//! The banner's reset and the welcome it opens (temper/docs/design/draft/playground-reset.md, 6.2).
use super::{demo, fixtures as f};
use crate::{Action, Address, Domain, Event, Request, Step, forge::demo::EndFailure, pages::Act};
use jig_client::{Down, Readout, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn leave(root: &mut Domain) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(Act::LeavePlayground) })
}
fn read(root: &mut Domain, state: w::PlaygroundState) -> Queue<Request> {
    demo::deliver_demo_read(root, demo::page_with_state(state))
}
fn opened() -> Domain {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(f::chats::me()))),
                },
            },
        },
    );
    read(&mut root, w::PlaygroundState::Running);
    root
}
fn sent(out: &Queue<Request>) -> Option<(skein_lib::RequestKey, jig_client::Key)> {
    let mut key = None;
    let mut stable = None;
    for request in out {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Save { record } } } = request {
            stable = Some(record.request.key);
        }
        if let Request::Client {
            request:
                jig_client::Request::Down {
                    down: Down::Send { key: sent, request: jig_client::Ask::Family { scope, record }, .. },
                },
        } = request
        {
            assert!(key.is_none(), "one reset sent");
            assert_eq!(*scope, c::Scope::Deployment);
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            assert_eq!(
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request(),
                &w::Request::EndPlayground
            );
            key = Some(*sent);
        }
    }
    let key = key?;
    Some((key, stable.expect("reset is kept")))
}
fn answered(root: &mut Domain, key: jig_client::Key, outcome: jig_client::Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), jig_client::Told::Answered { key, outcome }, &mut out);
    out
}
fn outcome(outcome: w::Outcome) -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
    }))
}
fn read_handle(out: &Queue<Request>) -> Token {
    let mut found = None;
    for request in out {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Read { read, what } } } = request {
            let jig_client::Readable::Family { scope, record, .. } = what else { panic!("family read") };
            assert!(found.is_none(), "one read");
            assert_eq!(*scope, c::Scope::Deployment);
            assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::Playground);
            found = Some(*read);
        }
    }
    found.expect("playground read")
}
fn delivered(root: &mut Domain, read: Token, result: Readout) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result } } })
}
fn shows(out: &Queue<Request>) -> u32 {
    let mut count = 0_u32;
    for request in out {
        if let Request::Show { address, replace } = request {
            assert_eq!(*address, Address::Welcome { step: Step::Forge { forge: None } });
            assert!(!replace, "push the Forge step");
            assert_eq!(crate::address::print(address).as_ref(), b"/welcome?at=forge");
            count = count.checked_add(1).unwrap();
        }
    }
    count
}

#[test]
fn set_up_temper_sends_one_end_playground() {
    let mut root = opened();
    assert!(sent(&leave(&mut root)).is_some());
    assert!(root.forge().demo_leaving());
    assert!(sent(&leave(&mut root)).is_none());
}

#[test]
fn reset_then_not_started_opens_the_forge_step_once() {
    let mut root = opened();
    let (_, key) = sent(&leave(&mut root)).unwrap();
    let out = answered(&mut root, key, outcome(w::Outcome::Reset));
    assert_eq!(shows(&out), 0);
    assert!(root.forge().demo_leaving(), "keep leaving until the read answers");
    assert!(sent(&leave(&mut root)).is_none());
    let out = delivered(&mut root, read_handle(&out), demo::page_with_state(w::PlaygroundState::NotStarted));
    assert_eq!(shows(&out), 1);
    assert!(!root.forge().demo_leaving());
    assert!(root.demo().is_none());
    assert_eq!(shows(&read(&mut root, w::PlaygroundState::NotStarted)), 0);
}

#[test]
fn a_resetting_read_awaits_its_own_end() {
    let mut root = opened();
    let out = read(&mut root, w::PlaygroundState::Resetting);
    let (_, key) = sent(&out).unwrap();
    assert!(root.forge().demo_leaving());
    assert!(sent(&read(&mut root, w::PlaygroundState::Resetting)).is_none());
    assert!(sent(&leave(&mut root)).is_none());
    let out = answered(&mut root, key, outcome(w::Outcome::Reset));
    let out = delivered(&mut root, read_handle(&out), demo::page_with_state(w::PlaygroundState::NotStarted));
    assert_eq!(shows(&out), 1);
}

#[test]
fn a_refused_end_offers_try_again() {
    let mut root = opened();
    let first = sent(&leave(&mut root)).unwrap();
    answered(
        &mut root,
        first.1,
        outcome(w::Outcome::EndRefused(
            w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
        )),
    );
    assert_eq!(root.forge().demo_end_failure(), Some(EndFailure::NotOffered));
    assert!(!root.forge().demo_leaving());
    let retry = sent(&leave(&mut root)).unwrap();
    assert_ne!(first, retry);
    assert!(root.forge().demo_end_failure().is_none());
    assert!(root.forge().demo_leaving());
}

#[test]
fn an_unknown_end_reads_again() {
    for answer in [outcome(w::Outcome::Unknown), jig_client::Outcome::Unknown, jig_client::Outcome::Lost] {
        let mut root = opened();
        let (_, key) = sent(&leave(&mut root)).unwrap();
        let out = answered(&mut root, key, answer);
        let handle = read_handle(&out);
        assert_eq!(shows(&out), 0);
        assert!(root.forge().demo_end_failure().is_none());
        assert!(sent(&delivered(&mut root, handle, demo::page_with_state(w::PlaygroundState::Resetting))).is_some());
        assert!(root.forge().demo_leaving());
    }
}

#[test]
fn a_resetting_read_before_me_waits_for_the_identity_then_sends_its_end() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    assert!(sent(&read(&mut root, w::PlaygroundState::Resetting)).is_none());
    assert!(root.forge().demo_leaving());
    let out = event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(f::chats::me()))),
                },
            },
        },
    );
    assert!(sent(&out).is_some());
    assert!(sent(&read(&mut root, w::PlaygroundState::Resetting)).is_none());
}

#[test]
fn a_not_started_read_waits_for_the_end_out_before_opening_the_forge_step() {
    let mut root = opened();
    let (_, key) = sent(&leave(&mut root)).unwrap();
    assert_eq!(shows(&read(&mut root, w::PlaygroundState::NotStarted)), 0);
    assert!(root.forge().demo_leaving());
    let out = answered(&mut root, key, outcome(w::Outcome::Reset));
    let out = delivered(&mut root, read_handle(&out), demo::page_with_state(w::PlaygroundState::NotStarted));
    assert_eq!(shows(&out), 1);
}

#[test]
fn another_operations_answer_or_a_trailing_byte_is_not_an_end_answer() {
    use crate::forge::demo::{EndAnswer, end_answer};
    for (outcome, expected) in [(w::Outcome::Running, None), (w::Outcome::Reset, Some(EndAnswer::Reset))] {
        let jig_client::Outcome::Answered(value) = self::outcome(outcome) else { unreachable!() };
        assert_eq!(end_answer(&value, &f::limits().forge), expected);
    }
    let jig_client::Outcome::Answered(jig_client::Answered::Family(mut family)) = outcome(w::Outcome::Reset) else {
        unreachable!()
    };
    let mut bytes = skein_lib::Writer::new(family.bytes.len().checked_add(1).unwrap());
    bytes.put(&family.bytes).unwrap();
    bytes.put(&[0]).unwrap();
    family.bytes = bytes.finish();
    assert!(end_answer(&jig_client::Answered::Family(family), &f::limits().forge).is_none());
}

#[test]
fn a_refused_recovered_end_shows_the_failure_and_allows_retry() {
    let mut root = opened();
    let (_, key) = sent(&read(&mut root, w::PlaygroundState::Resetting)).unwrap();
    answered(
        &mut root,
        key,
        outcome(w::Outcome::EndRefused(
            w::EndRefused::new(&w::CEILINGS, w::EndRefusedParts { why: w::EndRefusal::NotOffered }).unwrap(),
        )),
    );
    assert!(!root.forge().demo_leaving());
    assert_eq!(root.forge().demo_end_failure(), Some(EndFailure::NotOffered));
    assert!(sent(&leave(&mut root)).is_some());
}
