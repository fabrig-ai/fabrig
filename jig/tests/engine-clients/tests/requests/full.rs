//! A full real people table, through the engine codec and client link/domain.
use super::{Peer, bearer, real_core_settings, route_real_core};
use jig_client::{self as client, contract as c};
use jig_client_protocol_link as link;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, List, Queue, Time, Wall};

// The fixture's owner is party 1. Its bootstrap chat and one ordinary refusal fill the table.
const SEED: u64 = 391;
const FILL_KEY: u8 = 51; // distinct from the bootstrap chat and carried NAME
const EXPORT_ACCOUNT: u32 = 9; // its account is never looked up while admission is full
const PARTY: u64 = 1;
const HELD: u32 = 2;
const WALL: Wall = Wall::from_nanos(1_000_000_000);
const RETENTION: Duration = Duration::from_secs(120);
const KEY: client::Key = client::Key { asker: client::Asker(7), number: 1 };
const NAME: [u8; 16] = [52; 16];

#[test]
fn full_people_answers_reach_the_carried_clients_outcome_with_facts() {
    let mut world = jig_core_kit::stories::offers::world_with(SEED, None, |limits| {
        limits.core.people.requests = HELD;
        limits.core.people.request_retention = RETENTION;
    });
    let bootstrap_at = world
        .store
        .rows
        .values()
        .find_map(|row| {
            if let root::Record::Core(core::Record::People(people::Stored::Answer { ask, at, .. })) = row
                && matches!(ask.as_ref(), people::Ask::Create { .. })
            {
                Some(*at)
            } else {
                None
            }
        })
        .expect("retained bootstrap chat");
    let room_at =
        Wall::from_nanos(bootstrap_at.as_nanos().checked_add(RETENTION.as_nanos()).expect("bounded retention"));
    let (now, _) = world.clocks();
    world.set_clock(now, WALL);
    let sign_in = world.sign_in.expect("member");
    jig_core_kit::fixtures::ask_as(&mut world, sign_in, FILL_KEY, people::Ask::Stop { project: 1, task: u64::MAX });
    let mut peer = Peer::started(&real_core_settings());
    let bearer = bearer(&mut peer, world.sign_in.expect("member"), PARTY);

    // Reuse the joint world's receiving profile; only authentication is scripted.
    let limits = jig_client_kit::test::limits().client;
    let receiving = link::Limits::new(limits.domain, limits.exchanges).expect("receiving profile");
    let mut client = client::Client::unlinked(&limits, SEED).expect("client");
    let mut output = Queue::uncapped();
    client.step_unlinked(Time::ZERO, WALL, client::Event::StartUnlinked { kept: List::with_capacity(0) }, &mut output);
    client.step_unlinked(Time::ZERO, WALL, client::Event::SignIn { with: client::With::Transport }, &mut output);
    while output.pop().is_some() {}
    let mut me = link::translate::documents::frame(jig_client_test_records::frame_record(), &receiving).expect("me");
    me.party.number = PARTY;
    client.step_unlinked(
        Time::ZERO,
        WALL,
        client::Event::Up {
            up: client::Up::Watched {
                watch: skein_lib::Token::new(0),
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
            },
        },
        &mut output,
    );
    while output.pop().is_some() {}
    client.step_unlinked(
        Time::ZERO,
        WALL,
        client::accounts::move_out(KEY, NAME, c::AskAccountHolder::Own, EXPORT_ACCOUNT),
        &mut output,
    );
    let mut codec = link::Link::new(&receiving).expect("client codec");
    let mut wire = Queue::uncapped();
    while let Some(request) = output.pop() {
        if let client::UnlinkedRequest::Down { down: client::Down::Send { attempt, key, request, value } } = request {
            codec.step(Time::ZERO, link::Event::Send { attempt, key, ask: request, value }, &mut wire);
        }
    }
    codec.settle(&mut wire);
    let Some(link::Request::Exchange { exchange, message: link::Message::Request(body), .. }) = wire.pop() else {
        panic!("one keyed export")
    };
    let socket = peer.post(b"/jig/v1/request", body, Some(&bearer));
    route_real_core(&mut peer, &mut world);
    codec.step(
        Time::ZERO,
        link::Event::Reply { exchange, message: Box::from(peer.answered(socket).expect("full response")) },
        &mut wire,
    );
    let Some(link::Request::Answer { attempt, envelope, answer, why }) = wire.pop() else { panic!("client envelope") };
    client.step_unlinked(
        Time::ZERO,
        WALL,
        client::Event::Up { up: client::Up::Answer { attempt, envelope, answer, why } },
        &mut output,
    );
    let terminal: Vec<_> = output
        .iter()
        .filter_map(|request| {
            if let client::UnlinkedRequest::Told { told: client::Told::Answered { key, outcome } } = request {
                Some((*key, outcome.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(terminal, [(KEY, client::Outcome::Full { held: HELD, room_at })]);
}
