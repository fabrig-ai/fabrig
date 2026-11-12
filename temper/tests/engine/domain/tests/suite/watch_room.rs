//! Watch admission and terminal ownership (temper/docs/design/protocol/configuration/limits.md).
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

// One shipped client's watches, including me (protocol/configuration/limits.md).
const ROOM: u32 = 26;
const FIRST: Token = Token::new(1_000);
const EXTRA: Token = Token::new(2_000);
const OTHER_REPLY: Token = Token::new(3_000);
const OTHER_IDENTITY: u64 = 8; // Walking's first owner has identity 7.
const SEED: u64 = 91; // The direct driver's configuration.

fn driver(streams: u32) -> Driver {
    let limits = root::Limits { streams, ..walking::limits() };
    driver_with_limits(&limits)
}

pub(super) fn driver_with_limits(limits: &root::Limits) -> Driver {
    let mut driver = Driver::configured(Store::new(), [walking::config(SEED), walking::config(SEED)], limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}

fn watch(driver: &mut Driver, sign_in: u64, watcher: Token) {
    driver.send(root::Event::Party(root::PartyInput::Watch {
        watcher,
        sign_in,
        subject: Watched::Frame,
        window: Window { rows: 1, after: None },
    }));
    driver.settle();
}

fn fill(driver: &mut Driver, room: u32) {
    let session = driver.session();
    for offset in 0..room {
        watch(driver, session, Token::new(FIRST.raw() + u64::from(offset)));
    }
}

fn documents(driver: &Driver) -> usize {
    driver.delivered.iter().filter(|delivery| matches!(delivery, root::Delivery::Document { .. })).count()
}

fn busy(driver: &Driver, watcher: Token) -> bool {
    driver.immediate.iter().any(|request| matches!(request,
        root::Request::Party(root::PartyRequest::WatchRefused { watcher: token, why: jig_domain_core::WatchRefusal::Busy })
        if *token == watcher))
}

fn unwatch(driver: &mut Driver) {
    driver.send(root::Event::Party(root::PartyInput::Unwatch { watcher: FIRST }));
    driver.settle();
}

fn take_delivery(driver: &mut Driver) {
    driver.send(root::Event::Party(root::PartyInput::Delivered { watcher: FIRST }));
    driver.settle();
}

#[test]
fn a_full_room_refuses_busy_and_a_finished_watch_gives_it_back() {
    let mut driver = driver(1);
    fill(&mut driver, ROOM);
    assert_eq!(documents(&driver), 26, "every watch in the room is admitted");
    let session = driver.session();
    watch(&mut driver, session, EXTRA);
    assert!(busy(&driver, EXTRA));
    take_delivery(&mut driver);
    unwatch(&mut driver);
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        root::Delivery::WatchEnded { watcher, why: jig_domain_views::End::Unwatched } if *watcher == FIRST)));
    driver.immediate.clear();
    watch(&mut driver, session, EXTRA);
    assert!(!busy(&driver, EXTRA));
    assert_eq!(documents(&driver), 27, "the finished watch gave its place back");
}

#[test]
fn a_closing_watch_holds_its_room_until_its_delivery_is_taken() {
    let mut driver = driver(1);
    fill(&mut driver, ROOM);
    unwatch(&mut driver);
    let session = driver.session();
    watch(&mut driver, session, EXTRA);
    assert!(busy(&driver, EXTRA), "Unwatch waits for its delivery terminal");
    assert!(!driver.delivered.iter().any(|delivery| matches!(delivery,
        root::Delivery::WatchEnded { watcher, .. } if *watcher == FIRST)));
    take_delivery(&mut driver);
    driver.immediate.clear();
    watch(&mut driver, session, EXTRA);
    assert!(!busy(&driver, EXTRA));
    assert_eq!(documents(&driver), 27, "the delivery terminal released the place");
}

#[test]
fn two_parties_share_the_deployments_room() {
    for (streams, room) in [(1, ROOM), (2, 52)] {
        let mut driver = driver(streams);
        driver.send(root::Event::Party(root::PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(OTHER_REPLY),
            identity: people::Identity {
                key: temper_engine_domain_forge::user_key(1, OTHER_IDENTITY),
                login: b"other".as_slice().into(),
                name: b"Other".as_slice().into(),
            },
        }));
        driver.settle();
        let other = driver
            .delivered
            .iter()
            .find_map(|delivery| match delivery {
                root::Delivery::WebReply { to, reply: people::Reply::SignedIn { sign_in, .. } }
                    if *to == ReplyTo::new(OTHER_REPLY) =>
                {
                    Some(*sign_in)
                }
                _ => None,
            })
            .expect("second party signs in");
        fill(&mut driver, room);
        assert_eq!(documents(&driver), usize::try_from(room).unwrap(), "one party may hold the whole room");
        watch(&mut driver, other, EXTRA);
        assert!(busy(&driver, EXTRA), "the deployment's room is shared at {streams} streams");
    }
}
