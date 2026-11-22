//! Fresh fake boxes leave a person's preferred ports free (protocol/setup/box-backends.md, 1).
use super::{init_ports, tests::Effective};
use std::{net::SocketAddr, num::NonZeroU16};

#[test]
fn a_fresh_fake_box_writes_none_of_the_persons_preferred_ports() {
    let [clients, webhooks] = init_ports(None);
    let reserve = |requested, available| NonZeroU16::new(requested).unwrap_or(available);
    let clients = reserve(clients, NonZeroU16::new(17100).expect("scripted clients port"));
    let webhooks = reserve(webhooks, clients.checked_add(1).expect("scripted following webhooks port"));
    let prepared = Effective::with_listeners(clients, webhooks);
    let document: serde_json::Value = serde_json::from_slice(&prepared.document()).expect("deployment JSON");
    for pointer in ["/application/clients/listener/socket", "/application/webhooks/listener/socket"] {
        let address: SocketAddr = document
            .pointer(pointer)
            .expect("listener")
            .as_str()
            .expect("socket string")
            .parse()
            .expect("socket address");
        assert!(!(8367..=8371).contains(&address.port()), "{pointer}: a fake box wrote preferred port {address}");
    }
}

#[test]
fn fake_ports_use_the_given_environment() {
    let environment = [(
        Box::from(b"TEMPER_INTERNAL_FAKE_BOX_PORTS".as_slice()),
        Box::from(b"7101,7102,7103,7104,7105,7106,7107,7108,7109".as_slice()),
    )];
    let chosen = super::ports_from(&environment).expect("ports").expect("chosen ports");
    assert_eq!(chosen.listeners, [7101, 7102, 7103, 7104, 7105]);
    assert_eq!(chosen.peers, [7106, 7107, 7108, 7109]);
    assert!(super::ports_from(&[]).expect("no ports").is_none());
    let empty = [(Box::from(b"TEMPER_INTERNAL_FAKE_BOX_PORTS".as_slice()), Box::from(b"".as_slice()))];
    assert!(super::ports_from(&empty).expect("empty is unset").is_none());
}
