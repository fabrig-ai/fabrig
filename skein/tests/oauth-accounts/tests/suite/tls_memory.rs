use skein_heap::Meter;
use skein_lib::{Env, List, Queue, Time, Wall, bytes};
use skein_oauth::SavedToken;
use skein_oauth_accounts::{Account, Component, Request};

// Bounds four refresh slots holding maximal records and one TLS connection each.
const REFRESH_SLOTS_CEILING: u64 = 700_000;

#[test]
fn every_refresh_slot_holds_its_maximal_records_and_tls_connection() {
    use skein_oauth_accounts::{Endpoint, Keeper, Transport};
    let mut limits = skein_oauth_accounts_world::world::limits();
    limits.accounts = 4;
    limits.exchanges = 4;
    limits.io.sockets = 6;
    let trust = skein_tls_kit::pki::client(&[]);
    let meter = Meter::new();
    meter.start();
    let mut configured = List::with_capacity(limits.accounts);
    for account in 0..limits.accounts {
        let registration = skein_oauth::Registration {
            authorization_url: bytes::copy_of(b"https://localhost/authorize"),
            token_endpoint: bytes::copy_of(b"https://localhost/token"),
            client_id: bytes::copy_of(&[b'c'; 64]),
            redirect_uri: bytes::copy_of(b"http://localhost:31234/callback"),
            scope: bytes::copy_of(&[b's'; 64]),
            wire: skein_oauth::WireFormat::Form,
            client_secret: None,
            pkce_for_confidential: false,
            metadata_claim: None,
            terms: None,
        };
        assert!(
            configured
                .push(Account::SignIn {
                    origins: List::with_capacity(0),
                    shared: false,
                    registration,
                    endpoint: Endpoint {
                        address: "127.0.0.1:31000".parse().expect("loopback"),
                        transport: Transport::Tls { server_name: skein_tls_kit::pki::name(), trust: trust.clone() },
                    },
                    keeper: Keeper::Owner {
                        kept: Some(SavedToken {
                            key: account,
                            generation: 1,
                            access_token: skein_lib::Secret::new(bytes::copy_of(&[b'a'; 256])),
                            refresh_token: Some(skein_lib::Secret::new(bytes::copy_of(&[b'r'; 256]))),
                            metadata: Some(bytes::copy_of(&[b'm'; 256])),
                            expires_at: Wall::from_nanos(5_000_000_000),
                            issued_client_id: None,
                            granted_scopes: None,
                            subject: None,
                            id_token: None,
                            earliest_refresh_at: None,
                        })
                    },
                })
                .is_ok()
        );
    }
    let crypto = skein_lib::CryptoRng::new(&skein_world::crypto_seed(17), 0).expect("32-byte world seed");
    let mut component = Component::new(configured, &limits, crypto, b"oauth-accounts-world").expect("configured slots");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut above = Queue::uncapped();
    let mut io = Queue::uncapped();
    for account in 0..limits.accounts {
        component.down(&env, Request::Grant { account }, &mut above, &mut io, &mut Queue::with_capacity(1));
        assert!(above.is_empty(), "grant waits for a kept refresh");
    }
    let mut connects = 0;
    for _ in 0..64 {
        if !component.has_work() {
            break;
        }
        component.fire(&env, &mut above, &mut io, &mut Queue::with_capacity(1));
        assert!(above.is_empty());
        while let Some(request) = io.pop() {
            assert!(matches!(request, skein_io::Request::Connect { .. }));
            connects += 1;
        }
    }
    assert_eq!(connects, limits.exchanges, "one TLS connection in every slot");
    let measured = meter.end();
    assert!(measured.peak() <= REFRESH_SLOTS_CEILING, "all OAuth TLS exchange slots: {measured:?}");
}
