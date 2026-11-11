use super::fixtures;
use crate::v1;

fn tokens(forge: v1::Making) -> v1::Request {
    v1::Request::Connect(
        v1::Connect::new(
            &fixtures::limits(),
            v1::ConnectParts {
                forge,
                how: v1::How::ForgejoTokens(
                    v1::ForgejoTokens::new(&fixtures::limits(), v1::ForgejoTokensParts {}).unwrap(),
                ),
            },
        )
        .unwrap(),
    )
}
fn connected(host: &[u8], port: u16) -> v1::Making {
    let v1::ForgeAsk::Managed(value) = super::managed(host, port) else { unreachable!() };
    super::making_new(v1::ForgeAsk::Connected(value))
}
#[expect(clippy::disallowed_macros, reason = "golden fixtures are relative to the test crate")]
fn golden(request: v1::Request, name: &str) {
    let record = v1::Requesting::new(&fixtures::limits(), v1::RequestingParts { request }).unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden").join(name);
    if temper_test_env::rewrite() {
        std::fs::write(&path, crate::requesting(&record)).unwrap();
    }
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(crate::requesting(&record).as_ref(), bytes);
    assert_eq!(crate::decode_requesting(&bytes, &fixtures::limits()), Some(record));
    assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &fixtures::limits()), None);
}
#[test]
fn tokens_connect_of_a_new_connected_forge_round_trips_its_golden() {
    golden(tokens(connected(b"forge.example", 443)), "connect_forgejo_tokens_new.bin");
}
#[test]
fn tokens_connect_of_a_kept_forge_round_trips_its_golden() {
    golden(tokens(super::kept(7)), "connect_forgejo_tokens_kept.bin");
}
#[test]
fn replace_round_trips_its_golden() {
    golden(
        v1::Request::Replace(v1::Replace::new(&fixtures::limits(), v1::ReplaceParts { forge: 7 }).unwrap()),
        "replace.bin",
    );
}
#[test]
fn connected_forges_refuse_invalid_hosts_and_zero_ports() {
    for (host, port) in [(b"".as_slice(), 443), (b"a\0b", 443), (b"a/b", 443), (b"host", 0)] {
        for request in super::making_requests(connected(host, port)) {
            let record = v1::Requesting::new(&fixtures::limits(), v1::RequestingParts { request }).unwrap();
            assert_eq!(crate::decode_requesting(&crate::requesting(&record), &fixtures::limits()), None);
        }
    }
}
