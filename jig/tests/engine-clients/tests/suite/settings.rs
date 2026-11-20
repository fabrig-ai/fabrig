//! Native settings ingress uses the same held-answer boundary as other reads.
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core::{self as core, reads, settings};
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{List, Reader, Writer};

fn encoded(read: &w::Read) -> Box<[u8]> {
    let mut bytes = Writer::new(skein_lib::length::usize_of(
        read.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    read.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("read bytes");
    let value = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() };
    let mut bytes = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("owned bytes");
    bytes.finish()
}
#[test]
fn every_settings_read_waits_for_the_core_answer() {
    let limits = fixtures::limits().wire;
    for (request, expected) in [
        (w::Read::Settings, reads::Read::Settings),
        (
            w::Read::Setting(w::ReadSetting { setting: w::Setting::Rules, revision: 6, proposal: None }),
            reads::Read::Setting { setting: settings::Setting::Rules, revision: 6, proposal: None },
        ),
        (w::Read::Revision(w::ReadRevision { number: 3 }), reads::Read::Revision { number: 3 }),
        (w::Read::Revisions(w::ReadRevisions { before: Some(5) }), reads::Read::Revisions { before: Some(5) }),
    ] {
        let mut peer = Peer::started(&fixtures::settings());
        let bearer = fixtures::signed_in(&mut peer);
        let socket = peer.post(b"/jig/v1/read", encoded(&request), Some(&bearer));
        let Some(core::Event::Read { reply, read, .. }) = peer.take_core() else {
            panic!("authenticated settings read")
        };
        assert_eq!(read, expected);
        assert!(peer.answered(socket).is_none());
        peer.answer_read(reply, reads::Answer::None(reads::Why::Unknown));
        let answer = w::ReadAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("held answer")))
            .expect("answer");
        assert_eq!(answer, w::ReadAnswer::None(w::ReadAnswerNone { value: w::ReadNone::Unknown }));
    }
}
#[test]
fn settings_watch_delivers_its_inventory_on_the_stream() {
    let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    let stream = peer.open_stream();
    let watcher = peer.open_watch(stream, &w::Subject::Settings);
    assert_eq!(peer.requested_subject(), Some(&core::documents::Watched::Settings));
    peer.release_delivery(
        watcher,
        core::documents::Document::Settings(Box::new(core::documents::Settings {
            newest: 6,
            settings: List::with_capacity(0),
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        })),
    );
    let events = peer.events(stream);
    let w::StreamEvent::Delivery(delivery) = events.last().expect("inventory delivery") else {
        panic!("settings delivery")
    };
    let record = delivery.document.record.as_ref();
    let value = w::DeliveryValue::decode(&fixtures::limits().wire, &mut Reader::new(record)).expect("inventory record");
    let w::DeliveryValue::Settings(value) = value else { panic!("settings inventory") };
    assert_eq!(value.newest, 6);
}
