//! Mirrors the process tier's `support/sign_in.rs` issuer: two sign-ins of one synthetic `OpenID` grant,
//! the same registration and varying callback port, hosted by the browser harness.
use skein_fake_peers::{Transport, oauth};
use skein_lib::Duration;

pub(super) fn peer(address: std::net::SocketAddr, registration: &serde_json::Value) -> oauth::Peer {
    let bytes =
        |member: &str| -> Box<[u8]> { registration[member].as_str().expect("registration member").as_bytes().into() };
    let profile = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small);
    let limits = skein_fake_oauth::Limits {
        document: profile.driver.client.document,
        uri_bytes: 1024,
        request_bytes: 4096,
        // A redeemed code keeps its place alongside the second sign-in's.
        codes: 2,
        rotations: 2,
        carried: 5,
        plans: 2,
    };
    let mut peer = oauth::Peer::new(
        address,
        Transport::Plaintext,
        skein_fake_peers::Limits {
            io: profile.io,
            // Each sign-in tab may hold a connection beside the engine's.
            connections: 4,
            queue: 64,
            plaintext: 131_072,
            ciphertext: 131_072,
            observations: 64,
            observation_bytes: 4096,
        },
        skein_fake_oauth::Config {
            authorization_url: bytes("authorization"),
            token_endpoint: bytes("token"),
            client_id: bytes("client"),
            client_secret: None,
            redirect_uri: b"http://127.0.0.1:1455/callback".as_slice().into(),
            refresh_token: skein_lib::Secret::new(b"fake-refresh".as_slice().into()),
            terms: Some(skein_fake_oauth::Terms {
                issued: Box::new([b"issued-client".as_slice().into()]),
                omit_issued: false,
                varying_port: true,
                revocation_endpoint: Some(bytes("revocation")),
                openid: Some(skein_fake_oauth::OpenId {
                    issuer: bytes("issuer"),
                    key_set_url: bytes("keys"),
                    subject: b"browser-person".as_slice().into(),
                    lifetime: Duration::from_secs(300),
                    header_kid: true,
                    fault: None,
                }),
            }),
        },
        limits,
        skein_http::server::Limits {
            response: oauth::least_response(&limits).expect("issuer response"),
            body: u64::from(limits.request_bytes),
            ..profile.driver.server
        },
    )
    .expect("local issuer");
    sign_in_grants(&mut peer, registration);
    peer
}

fn sign_in_grants(peer: &mut oauth::Peer, registration: &serde_json::Value) {
    let scopes = registration["scopes"]
        .as_array()
        .expect("shipped scopes")
        .iter()
        .map(|scope| scope.as_str().expect("scope"))
        .collect::<Vec<_>>()
        .join(" ");
    for _ in 0..2 {
        peer.queue(skein_fake_oauth::Plan {
            status: 200,
            body: skein_fake_oauth::Body::Token(skein_oauth::TokenResponse {
                access_token: skein_lib::Secret::new(b"browser-access".as_slice().into()),
                refresh_token: Some(skein_lib::Secret::new(b"fake-refresh".as_slice().into())),
                expires_in: 300,
                scope: Some(scopes.as_bytes().into()),
                id_token: None,
                earliest_refresh_at: None,
            }),
            delay: Duration::ZERO,
            retry_after: Duration::ZERO,
        })
        .expect("each sign-in has a fake grant");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_issuer_scripts_both_sign_ins() {
        let sign_in = serde_json::json!({
            "issuer":"http://127.0.0.1:9", "authorization":"http://127.0.0.1:9/authorize",
            "token":"http://127.0.0.1:9/token", "revocation":"http://127.0.0.1:9/revoke",
            "keys":"http://127.0.0.1:9/keys", "client":"temper-test", "scopes":["openid","chat"]
        });
        let peer = super::peer("127.0.0.1:9".parse().expect("issuer address"), &sign_in);
        assert_eq!(peer.granted().len(), 2, "each sign-in redeems its own scripted code grant");
    }
}
