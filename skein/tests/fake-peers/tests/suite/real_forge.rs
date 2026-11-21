use fixtures::{SEED, address, config, limits, protocol_limits, request};
use skein_fake_peers::forge;
use skein_fake_peers_kit::forge as fixtures;
use skein_shell::Host;

#[path = "../forge/git.rs"]
mod git;

#[test]
fn a_tls_peer_serves_forgejo_over_the_fixed_test_chain() {
    use rustls_std::{
        ClientConfig, ClientConnection, RootCertStore,
        pki_types::{CertificateDer, ServerName},
    };
    use skein_lib::Duration;
    use skein_shell::{Clock, Kernel};
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::Arc,
    };

    let mut peer =
        forge::Peer::new(address(), limits(), config(), protocol_limits(), fixtures::shared(), SEED).unwrap();
    peer.tls().unwrap();
    let kernel = Kernel::open(skein_shell::Config { operations: peer.operations() }).expect("kernel ring");
    let mut world = git::Loop { peer, kernel, clock: Clock::new() };
    let deadline = skein_world::Deadline::arm("the TLS forge listener, exchange and drain", Duration::from_secs(6));
    while world.peer.address().is_none() {
        world.pass();
        if world.peer.address().is_none() && deadline.passed() {
            world.expired(&deadline);
        }
    }
    let bound = world.peer.address().unwrap();
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(skein_fake_peers::transport::ROOT)).unwrap();
    let config = ClientConfig::builder_with_provider(Arc::new(rustls_std::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let mut tls = ClientConnection::new(Arc::new(config), ServerName::try_from("127.0.0.1").unwrap()).unwrap();
    let mut stream = TcpStream::connect(bound).unwrap();
    stream.set_nonblocking(true).unwrap();
    tls.writer().write_all(&request(b"/api/v1/repos/org/repo")).unwrap();
    let mut response = Vec::new();
    loop {
        world.pass();
        match tls.complete_io(&mut stream) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => panic!("TLS exchange: {error}"),
        }
        let mut buffer = [0; 4096];
        match tls.reader().read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => response.extend_from_slice(&buffer[..count]),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            // HTTP's length delimits this response even when the peer closes the socket.
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(error) => panic!("TLS response: {error}"),
        }
        if deadline.passed() {
            drop(stream);
            world.expired(&deadline);
        }
    }
    assert!(response.starts_with(b"HTTP/1.1 200 "), "{}", String::from_utf8_lossy(&response));
    let split = skein_lib::bytes::find(&response, b"\r\n\r\n").expect("HTTP head");
    let head = std::str::from_utf8(&response[..split]).unwrap();
    let length: usize = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().unwrap())
        })
        .expect("HTTP response length");
    assert_eq!(response.len() - split - 4, length, "complete HTTP body over TLS");
    let html = format!("\"html_url\":\"https://{bound}/org/repo\"");
    assert!(skein_lib::bytes::find(&response, html.as_bytes()).is_some(), "{}", String::from_utf8_lossy(&response));
    drop(stream);
    world.peer.shutdown();
    while !world.peer.is_empty() {
        world.pass();
        if !world.peer.is_empty() && deadline.passed() {
            world.expired(&deadline);
        }
    }
}
