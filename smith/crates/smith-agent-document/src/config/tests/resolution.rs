use super::{ConfigurationRefusal, Decoded, decode, shaped_document};
use skein_llm_connection::Transport;
use std::net::SocketAddr;

fn mixed_endpoints() -> Decoded {
    let mut document = shaped_document();
    let mut tls = document["endpoints"][0].clone();
    tls["name"] = "remote".into();
    tls["number"] = 1.into();
    tls["transport"] = "tls".into();
    tls["server_name"] = "example.test".into();
    tls["trust"] = serde_json::json!({"system": false, "pem": ["absent-fixture-roots.pem"]});
    document["endpoints"].as_array_mut().expect("endpoint list").insert(0, tls);
    decode(&serde_json::to_vec(&document).expect("configuration JSON")).expect("decode mixed transports")
}

fn fixture_address(address: &str) -> Result<SocketAddr, ConfigurationRefusal> {
    address.parse().map_err(|why| format!("fixture endpoint {address}: {why}").into())
}

#[test]
fn supplied_trust_is_used_only_for_tls_endpoints() {
    let root = include_bytes!("../../../../../tests/protocol-llm/fixtures/root.der");
    let (fixture, _) = skein_tls::Config::from_der(&[root.as_slice().into()], &[b"fixture"]).expect("fixture trust");
    let configured =
        mixed_endpoints().resolve_with(fixture_address, Some(&fixture), |_| None).expect("resolve both endpoints");
    let [tls, plaintext] = configured.service.llm_endpoints.destinations().as_slice() else {
        panic!("both configured destinations are retained");
    };
    let Transport::Tls { trust, .. } = &tls.transport else {
        panic!("the first destination uses TLS");
    };
    // The fixture's ALPN distinguishes it from trust loaded by native resolution.
    assert_eq!(format!("{trust:?}"), format!("{fixture:?}"), "TLS carries the supplied configuration");
    assert!(matches!(&plaintext.transport, Transport::Plaintext), "the loopback endpoint stays plaintext");
}

#[test]
fn tls_refuses_resolution_without_supplied_trust() {
    let refusal = mixed_endpoints().resolve_with(fixture_address, None, |_| None).err().expect("TLS requires trust");
    assert_eq!(refusal.detail(), "TLS endpoint requires supplied trust");
}
