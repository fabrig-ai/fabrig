//! A window's in-memory rate crosses the engine's client stream whole.
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_accounts as a;
use jig_domain_core::{self as core, documents as d};
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{Duration, List, Reader, Wall};

fn document(rate: Option<u32>) -> d::Document {
    d::Document::Accounts(Box::new(d::Accounts {
        program_hosts: List::with_capacity(0),
        accounts: List::from_boxed(Box::new([d::AccountLine {
            list: None,
            number: 1,
            place: 0,
            uses: None,
            plan: true,
            provider: Some(1),
            provider_name: None,
            manage_usage: None,
            display: d::Text { bytes: b"Personal".as_slice().into(), cut: false },
            holder: a::Holder::Party(fixtures::FIRST_PERSON),
            sources: List::with_capacity(0),
            facts: List::from_boxed(Box::new([d::AccountFactLine {
                rate,
                fact: a::AccountFact {
                    kind: a::KIND_USAGE,
                    label: b"five_hour".as_slice().into(),
                    value: a::FactValue::Window(a::FactWindow {
                        length: Some(Duration::from_secs(18_000)),
                        used: Some(2350),
                        resets: Some(Wall::from_nanos(99)),
                        state: a::WINDOW_OPEN,
                    }),
                    seen: Wall::from_nanos(11),
                    source: Some(2),
                    origin: a::ORIGIN_PROGRAM,
                },
            }])),
            cut: false,
        }])),
        signing: List::with_capacity(0),
        last_end: None,
        lendable: false,
        lending: None,
        lent_end: None,
        usable: true,
        settled: true,
        cut: false,
        readiness: List::with_capacity(0),
        readiness_cut: false,
    }))
}

#[test]
fn usage_windows_reach_a_client_with_and_without_a_rate() {
    for rate in [None, Some(600), Some(u32::MAX)] {
        let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
        let stream = peer.open_stream();
        let watcher = peer.open_watch(stream, &w::Subject::Accounts);
        peer.release_delivery(watcher, document(rate));
        let events = peer.events(stream);
        let w::StreamEvent::Delivery(delivery) = events.last().expect("accounts delivered") else { panic!("delivery") };
        assert_eq!(delivery.document.owner, w::Owner::Jig);
        assert_eq!(delivery.document.version, w::VERSION);
        let w::DeliveryValue::Accounts(accounts) =
            w::DeliveryValue::decode(&w::CEILINGS, &mut Reader::new(&delivery.document.record)).unwrap()
        else {
            panic!("accounts")
        };
        let mut reader = Reader::new(&accounts.accounts);
        let length = reader.u32().unwrap();
        let bytes = reader.bytes(length).unwrap();
        let row = w::AccountLine::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
        let fact = row.facts.get(0).unwrap();
        let w::AccountFactValue::Window(window) = &fact.value else { panic!("window") };
        assert_eq!(window.rate, rate);
        assert_eq!(window.used, Some(2350));
        assert_eq!(window.length, Some(Duration::from_secs(18_000)));
        assert_eq!(window.resets, Some(99));
        assert_eq!(window.state, a::WINDOW_OPEN);
        assert_eq!(fact.label.as_ref(), b"five_hour");
        assert_eq!(fact.seen, 11);
        assert_eq!(fact.source, Some(2));
        assert_eq!(fact.origin, a::ORIGIN_PROGRAM);
        assert!(reader.is_empty());
        assert_eq!(peer.requested_subject(), Some(&core::documents::Watched::Accounts));
    }
}
