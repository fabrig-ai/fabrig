//! Fresh fake boxes leave a person's preferred ports free (protocol/setup/box.md, 5).
use super::{Directories, bytes, make_private, prepare};
use std::{fs, net::SocketAddr, os::unix::fs::PermissionsExt};

#[test]
fn a_fresh_fake_box_writes_none_of_the_persons_preferred_ports() {
    let scratch = skein_scratch::Scratch::new("fake-box-ports");
    let root = scratch.path();
    let source = Directories {
        config: bytes(&root.join("config")),
        data: bytes(&root.join("data")),
        cache: bytes(&root.join("cache")),
        origins: [skein_shell::Origin::Flag; 3],
    };
    let bundle = root.join("bundle");
    make_private(&bundle.join("0000000000000001"), skein_shell::effective_user()).expect("fixture bundle");
    fs::write(bundle.join("index.html"), b"<!doctype html><html><body>fixture</body></html>").expect("fixture page");
    fs::write(bundle.join("0000000000000001/main.js"), b"// fixture\n").expect("fixture script");
    let adapter = root.join("fake-claude");
    fs::write(&adapter, b"#!/bin/sh\nexit 1\n").expect("fixture adapter");
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).expect("executable fixture adapter");
    let prepared = prepare(&source, Some(&bundle), &adapter, None, &[]).expect("fresh fake box");
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared.directories.document()).expect("written deployment"))
            .expect("deployment JSON");
    for pointer in [
        "/application/clients/listener/socket",
        "/application/machine/owned/listen/socket",
        "/application/machine/fake/listen/socket",
        "/application/demo/llm/listen/socket",
        "/application/webhooks/listener/socket",
    ] {
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
