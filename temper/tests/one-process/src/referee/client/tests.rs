use super::{Connection, LandingRead, Received, client, wire};
use jig_client_protocol_wire::Record as _;
use skein_lib::{Writer, base64};

#[test]
fn stream_receipts_join_data_lines_and_keep_the_unfinished_event() {
    // The input holds one complete event followed by the unfinished second one.
    const FIRST_EVENT: usize = 1;
    const BOTH_EVENTS: usize = 2;
    let value = wire::DeliveryValue::Terminal(wire::Terminal { login: None });
    let mut record = Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("record size"),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut record).expect("delivery record");
    let document = wire::Owned { owner: wire::Owner::Jig, version: wire::VERSION, record: record.finish() };
    let event = wire::StreamEvent::Delivery(wire::Delivery { watch: 1, document });
    let mut record = Writer::new(
        usize::try_from(event.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("event size"),
    );
    event.write(&jig_client_protocol_wire::CEILINGS, &mut record).expect("stream event");
    let record = record.finish();
    let encoded_length = base64::encoded_len(base64::Alphabet::Standard, u32::try_from(record.len()).expect("size"))
        .expect("base64 length");
    let mut encoded = Writer::new(usize::try_from(encoded_length).expect("encoded size"));
    base64::encode(base64::Alphabet::Standard, &record, &mut encoded).expect("base64 event");
    let encoded = encoded.finish();
    // Cut within a base64 quartet, not between complete encoded records.
    let (first, rest) = encoded.split_at(1);
    let mut connection = Connection::default();
    connection.body.extend_from_slice(b": keepalive\n\ndata:");
    connection.body.extend_from_slice(first);
    connection.body.extend_from_slice(b"\ndata: ");
    connection.body.extend_from_slice(rest);
    connection.body.extend_from_slice(b"\n\ndata: ");
    connection.body.extend_from_slice(&encoded);
    let mut received = Vec::new();
    connection.stream(&mut received, false, &mut LandingRead::Off);
    assert_eq!(received.len(), FIRST_EVENT, "only the complete event releases a receipt");
    assert_eq!(connection.body, [b"data: ".as_slice(), &encoded].concat(), "partial event is kept");
    connection.body.extend_from_slice(b"\n\n");
    connection.stream(&mut received, false, &mut LandingRead::Off);
    assert_eq!(received.len(), BOTH_EVENTS, "each complete event releases one receipt");
    assert_eq!(connection.body, b"".as_slice());
    for receipt in received {
        assert!(matches!(receipt, Received::Delivery(client::Delivery::Terminal(value)) if value.login.is_none()));
    }
}
