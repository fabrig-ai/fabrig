use super::fixtures;
use crate::v1;

fn tokens(forge: v1::Making) -> v1::Request {
    v1::Request::Connect(
        v1::Connect {
                forge,
                how: v1::How::ForgejoTokens(
                    v1::ForgejoTokens {},
                ),
            },
    )
}
fn connected(host: &[u8], port: u16) -> v1::Making {
    let v1::ForgeAsk::Managed(value) = super::managed(host, port) else { unreachable!() };
    super::making_new(v1::ForgeAsk::Connected(value))
}
fn round_trip(request: v1::Request) {
    let record = v1::Requesting { request };
    let bytes = crate::requesting(&record);
    assert_eq!(crate::decode_requesting(&bytes, &fixtures::limits()), Some(record));
    assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &fixtures::limits()), None);
}
#[test]
fn tokens_connect_of_a_new_connected_forge_round_trips() {
    round_trip(tokens(connected(b"forge.example", 443)));
}
#[test]
fn tokens_connect_of_a_kept_forge_round_trips() {
    round_trip(tokens(super::kept(7)));
}
#[test]
fn replace_round_trips() {
    round_trip(v1::Request::Replace(v1::Replace { forge: 7 }));
}
#[test]
fn connected_forges_refuse_invalid_hosts_and_zero_ports() {
    for (host, port) in [(b"".as_slice(), 443), (b"a\0b", 443), (b"a/b", 443), (b"host", 0)] {
        for request in super::making_requests(connected(host, port)) {
            let record = v1::Requesting { request };
            assert_eq!(crate::decode_requesting(&crate::requesting(&record), &fixtures::limits()), None);
        }
    }
}
