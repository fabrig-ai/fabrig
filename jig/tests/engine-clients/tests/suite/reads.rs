use jig_client_protocol_wire::v28 as w;
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{Reader, Writer};
fn read() -> Box<[u8]> {
    let limits = fixtures::limits().wire;
    let read = w::Read::Me;
    let mut record = Writer::new(skein_lib::length::usize_of(read.measure()));
    read.encode(&mut record).expect("me read");
    let value = w::Owned::new(&limits, w::OwnedParts { owner: w::Owner::Jig, version: 28, record: record.finish() })
        .expect("frame read");
    let mut bytes = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut bytes).expect("read envelope");
    bytes.finish()
}
#[test]
fn a_native_read_waits_for_its_released_answer() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let socket = peer.post(b"/jig/v1/read", read(), Some(&bearer));
    let Some(core::Event::Read { reply, sign_in, read }) = peer.take_core() else { panic!("authenticated read") };
    assert_eq!(sign_in, fixtures::FIRST_SIGN_IN);
    assert_eq!(read, core::reads::Read::Frame);
    assert_eq!(peer.answered(socket), None);
    peer.answer_read(reply, core::reads::Answer::None(core::reads::Why::Gone));
    let limits = fixtures::limits().wire;
    let expected = w::ReadAnswer::None(
        w::ReadAnswerNone::new(&limits, w::ReadAnswerNoneParts { value: w::ReadNone::Gone }).expect("gone"),
    );
    assert_eq!(
        w::ReadAnswer::decode(&limits, &mut Reader::new(peer.answered(socket).expect("read answer"))),
        Ok(expected)
    );
}

fn batch_parts(bytes: &[u8]) -> Vec<w::BatchReply> {
    let mut reader = Reader::new(bytes);
    let mut parts = Vec::new();
    while !reader.is_empty() {
        let length = reader.u32().expect("whole frame length");
        let frame = reader.bytes(length).expect("whole frame");
        parts.push(w::BatchReply::decode(&fixtures::limits().wire, &mut Reader::new(frame)).expect("part"));
    }
    parts
}
#[test]
fn a_batch_streams_a_released_read_while_another_waits_for_durability() {
    let limits = fixtures::limits().wire;
    let value = w::Owned::decode(&limits, &mut Reader::new(&read())).expect("read envelope");
    let mut members = skein_lib::List::with_capacity(2);
    members.push(w::Member::Read(value.clone())).expect("A");
    members.push(w::Member::Read(value)).expect("B");
    let batch = w::Batch::new(&limits, w::BatchParts { members }).expect("batch");
    let mut bytes = Writer::new(skein_lib::length::usize_of(batch.measure()));
    batch.encode(&mut bytes).expect("batch bytes");
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let socket = peer.post(b"/jig/v1/batch", bytes.finish(), Some(&bearer));
    let Some(core::Event::Read { reply: a, .. }) = peer.take_core() else { panic!("A read") };
    let Some(core::Event::Read { reply: b, .. }) = peer.take_core() else { panic!("B read") };
    assert_eq!(peer.answered(socket), None);
    // Script the held-answer boundary: A reflects an undurable commit; B does not.
    peer.answer_read(b, core::reads::Answer::None(core::reads::Why::Gone));
    let (status, body) = peer.response(socket);
    assert_eq!(status, Some(200));
    let parts = batch_parts(body);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts.first().expect("B part").member(), 1);
    assert_eq!(peer.answered(socket), None, "the body is open while A waits");
    peer.answer_read(a, core::reads::Answer::None(core::reads::Why::Unknown));
    let parts = batch_parts(peer.answered(socket).expect("Done ends the HTTP body"));
    assert_eq!(parts.iter().map(w::BatchReply::member).collect::<Vec<_>>(), [1, 0]);
    assert!(peer.take_core().is_none());
}

#[test]
fn a_family_read_hands_its_scope_and_window_to_the_application() {
    use jig_protocol_engine_clients::{Served, ToApplication};
    let mut settings = fixtures::settings();
    settings.served = skein_lib::List::with_capacity(2);
    settings.served.push(Served { family: None, first: 28, last: 28 }).expect("jig");
    settings.served.push(Served { family: Some(1), first: 1, last: 1 }).expect("family");
    let mut peer = Peer::started(&settings);
    let bearer = fixtures::signed_in(&mut peer);
    let record = fixtures::family_read(w::Scope::Own, fixtures::window(3));
    peer.post(b"/jig/v1/read", record, Some(&bearer));
    let Some(ToApplication::Read { scope, window, family, .. }) = peer.take_application() else {
        panic!("family read")
    };
    assert_eq!(family, 1);
    assert_eq!(scope, jig_domain_people::ApplicationScope::Own);
    assert_eq!(window, core::documents::Window { rows: 3, after: None });
}
