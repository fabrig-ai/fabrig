//! Orders at the client's application, transport and store boundaries
//! (jig/docs/design/client/domain/client.md, 3, 6, 7 and 11;
//! jig/docs/design/client/domain/root.md, 3 and 5).
mod world;

use jig_client::{Asker, Event, Handle, Key, Link, Readout, Told, contract as c};
use jig_client_world::test;
use skein_lib::{Time, Wall};
use world::{Command, Edge, Mode, World};

const OCCUPIED: Handle = Handle { asker: Asker::JIG, number: 100 };
const A: Handle = Handle { asker: Asker::JIG, number: 101 };
const B: Handle = Handle { asker: Asker::JIG, number: 102 };
const C: Handle = Handle { asker: Asker::JIG, number: 103 };
const CANCELLED: Handle = Handle { asker: Asker::JIG, number: 104 };
const FIRST: Key = Key { asker: Asker::JIG, number: 1 };
const SECOND: Key = Key { asker: Asker::JIG, number: 2 };
const THIRD: Key = Key { asker: Asker::JIG, number: 3 };
const FIRST_NAME: [u8; 16] = [1; 16];
const SECOND_NAME: [u8; 16] = [2; 16];
const THIRD_NAME: [u8; 16] = [3; 16];
const PROJECT: u32 = 1;
const TASK: u64 = 9;
const OTHER_TASK: u64 = 10;
const BLOCKING_PROJECT: u32 = 10;
const OBSOLETE_PROJECT: u32 = 20;
const PROJECT_A: u32 = 21;
const PROJECT_B: u32 = 22;
const PROJECT_C: u32 = 23;
const CANCELLED_PROJECT: u32 = 40;
// One exchange leaves no transport credit behind the occupying read.
const ONE_EXCHANGE: u32 = 1;
// One cursor item per entrance forces record/send work across visits.
const ONE_OUTPUT: u32 = 1;
// The focused request table retries within its first 10 ms backoff.
const RETRY_AT: Time = Time::from_nanos(10_000_000);

fn read(handle: Handle, project: u32) -> Command {
    Command::Client(Event::Read { handle, read: jig_client::Readable::Jig(c::Read::Project { project }) })
}

fn send(key: Key, name: [u8; 16]) -> Command {
    Command::Client(Event::Send { key, name: Some(name), request: world::stop() })
}

fn page(handle: Handle, why: c::ReadNone) -> Edge {
    Edge::Told(Told::Page { handle, page: Readout::Jig(c::ReadResult::None(why)) })
}

/// `link_requests_dispatched` and `client_settled`: an entrance's old-link
/// consequences settle before its new work and the already queued next entrance.
#[test]
fn link_consequences_settle_before_the_next_entrance_under_load() {
    // The me opening and two task openings fill the three exchange places.
    const EXCHANGES: u32 = 3;
    let mut limits = test::limits().client;
    limits.exchanges = EXCHANGES;
    let world = World::run(
        Mode::Linked(limits),
        vec![
            ("watch_a", Command::watch(A, TASK)),
            ("watch_b", Command::watch(B, OTHER_TASK)),
            ("read_a", read(A, PROJECT)),
            ("read_b", read(B, PROJECT)),
            ("credentials", Command::credentials()),
            ("next", read(C, PROJECT)),
            ("next_done", Command::ReplyRead(PROJECT)),
            ("stale", Command::ReleaseReads),
        ],
    );
    assert_eq!(
        world.at("credentials"),
        &[
            Edge::CloseStream,
            page(A, c::ReadNone::SignedOut),
            page(B, c::ReadNone::SignedOut),
            Edge::Told(Told::Ended { handle: A, why: jig_client::Why::SignedOut }),
            Edge::Told(Told::Ended { handle: B, why: jig_client::Why::SignedOut }),
            Edge::Told(Told::Link { link: Link::SignedOut }),
            Edge::Told(Told::Link { link: Link::Starting }),
            Edge::OpenStream,
        ]
    );
    assert_eq!(world.at("next"), &[Edge::Read(PROJECT)]);
    assert_eq!(world.at("next_done"), &[page(C, c::ReadNone::Unknown)]);
    assert!(world.at("stale").is_empty());
}

/// `deferred_reads_dispatched`: current deferred demands retain FIFO precedence
/// over a fresh entrance while obsolete demands cannot return.
#[test]
fn deferred_reads_leave_in_order_under_load() {
    let mut limits = test::limits().client;
    limits.exchanges = ONE_EXCHANGE;
    let world = World::run(
        Mode::Linked(limits),
        vec![
            ("opening_done", Command::AdmitMe),
            ("occupied", read(OCCUPIED, BLOCKING_PROJECT)),
            ("obsolete", read(A, OBSOLETE_PROJECT)),
            ("a", read(A, PROJECT_A)),
            ("cancelled", read(CANCELLED, CANCELLED_PROJECT)),
            ("unwatch", Command::Client(Event::Unwatch { handle: CANCELLED })),
            ("b", read(B, PROJECT_B)),
            ("free", Command::ReplyRead(BLOCKING_PROJECT)),
            ("c", read(C, PROJECT_C)),
            ("a_done", Command::ReplyRead(PROJECT_A)),
            ("b_done", Command::ReplyRead(PROJECT_B)),
            ("c_done", Command::ReplyRead(PROJECT_C)),
            ("stale", Command::ReleaseReads),
        ],
    );
    assert_eq!(world.reads(), vec![BLOCKING_PROJECT, PROJECT_A, PROJECT_B, PROJECT_C]);
    assert_eq!(world.at("free"), &[page(OCCUPIED, c::ReadNone::Unknown), Edge::Read(PROJECT_A)]);
    assert_eq!(world.at("a_done"), &[page(A, c::ReadNone::Unknown), Edge::Read(PROJECT_B)]);
    assert_eq!(world.at("b_done"), &[page(B, c::ReadNone::Unknown), Edge::Read(PROJECT_C)]);
    assert_eq!(world.at("c_done"), &[page(C, c::ReadNone::Unknown)]);
    for label in ["obsolete", "a", "cancelled", "unwatch", "b", "c", "stale"] {
        assert!(world.at(label).is_empty(), "{label}");
    }
}

/// `request_records_durable`: an earlier completion cannot open the later
/// write's barrier; failing that later write ends only its own obligation.
#[test]
fn a_later_write_holds_its_requests_under_load() {
    let world = World::run(
        Mode::Linked(test::limits().client),
        vec![
            ("first", send(FIRST, FIRST_NAME)),
            ("second", send(SECOND, SECOND_NAME)),
            ("earlier_stored", Command::Stored(true)),
            ("later_failed", Command::Stored(false)),
            ("erase_stored", Command::AckAll),
            ("retry", Command::Fire(RETRY_AT)),
            ("finish", Command::FinishRequests),
            ("erased", Command::AckAll),
        ],
    );
    assert_eq!(world.at("earlier_stored"), &[]);
    assert_eq!(
        world.answers(),
        vec![
            (SECOND, jig_client::Outcome::NotKept),
            (FIRST, jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Changed)))),
        ]
    );
    assert_eq!(world.sends(), vec![FIRST_NAME]);
    assert_eq!(world.sent_at(FIRST_NAME), "retry");
}

/// `request_records_dispatched`: a one-item cursor visit still drains pending
/// records ahead of eligible sends, including restoration, retirement and retry.
#[test]
fn request_records_precede_their_sends_across_visits() {
    let mut limits = test::limits().client;
    limits.domain.requests.out = ONE_OUTPUT;
    let kept =
        test::list([world::record(FIRST, FIRST_NAME, None), world::record(SECOND, SECOND_NAME, Some(Wall::EPOCH))]);
    let world = World::run(
        Mode::Unlinked(limits, kept),
        vec![
            ("new", send(THIRD, THIRD_NAME)),
            ("visit_a", Command::Visit),
            ("visit_b", Command::Visit),
            ("visit_c", Command::Visit),
            ("retry_lost", Command::LoseRequest(SECOND_NAME)),
            ("first_done", Command::FinishRequest(FIRST_NAME)),
            ("retry", Command::Fire(RETRY_AT)),
            ("retry_visit", Command::Visit),
            ("finish", Command::FinishRequests),
            ("drain", Command::Visit),
        ],
    );
    assert_eq!(
        world.records_and_sends(),
        vec![
            Edge::Save(FIRST_NAME),
            Edge::Save(THIRD_NAME),
            Edge::Send(FIRST_NAME),
            Edge::Send(SECOND_NAME),
            Edge::Send(THIRD_NAME),
            Edge::Erase(FIRST_NAME),
            Edge::Send(SECOND_NAME),
            Edge::Erase(THIRD_NAME),
            Edge::Erase(SECOND_NAME),
        ]
    );
}

/// `child_records_dispatched`: the family page's child records precede its
/// transport partition even when the shell holds storage and exchanges. Three
/// restored drafts share one retained draft place, producing two ordered erases.
#[test]
fn child_records_precede_transport_within_a_step() {
    let world = World::run(
        Mode::with_drafts(),
        vec![
            ("page_b", Command::Go(world::PAGE_B)),
            ("page_c", Command::Go(world::PAGE_C)),
            ("store", Command::AckAll),
        ],
    );
    assert_eq!(
        world.at("me"),
        vec![
            Edge::ChildSave(jig_client_test_domain::store::Record::First(
                jig_client_test_domain::first::Record::Place {
                    version: 1,
                    party: world::PARTY,
                    address: world::PAGE_A
                }
            )),
            Edge::ChildErase(jig_client_test_domain::store::Key::Jig(jig_client_view::store::StoreKey::Draft {
                task: world::OLD_DRAFT
            })),
            Edge::ChildErase(jig_client_test_domain::store::Key::Jig(jig_client_view::store::StoreKey::Draft {
                task: world::MIDDLE_DRAFT
            })),
            Edge::ReadFamily(world::PAGE_A.item),
            Edge::WatchFamily(world::PAGE_A.item),
        ]
    );
    assert_eq!(world.at("page_b"), world::page_outputs(world::PAGE_B));
    assert_eq!(world.at("page_c"), world::page_outputs(world::PAGE_C));
}

/// `child_requests_dispatched`: interleaved jig and family entrances finish
/// their nested routes before the next caller's output reaches the shell.
#[test]
fn nested_routes_finish_before_their_caller_continues() {
    let world = World::run(
        Mode::root(),
        vec![
            ("family", Command::Go(world::PAGE_B)),
            ("jig", Command::JigInbox),
            ("family_again", Command::Go(world::PAGE_C)),
            ("old_reads", Command::ReleaseReads),
            ("store", Command::AckAll),
        ],
    );
    assert_eq!(world.at("family"), world::page_outputs(world::PAGE_B));
    assert_eq!(world.at("jig"), world::inbox_outputs());
    assert_eq!(world.at("family_again"), world::page_outputs(world::PAGE_C));
}
