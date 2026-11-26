use crate::support;
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
    sha256::Sha256,
};
use skein_tls_kit::pki;
use skein_world::{Deadline, Starved};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    thread,
};
use temper_forge_setup::{
    fetch::{Fetch, Fetched, Pin},
    setup::Origin,
};
use temper_forge_setup_shell::fetch;
fn sha(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finish()
}
fn pem(root: &[u8]) -> Vec<u8> {
    let mut writer = Writer::new(
        base64::encoded_len(Alphabet::Standard, u32::try_from(root.len()).expect("fetch fixture"))
            .expect("fetch fixture")
            .try_into()
            .expect("fetch fixture"),
    );
    base64::encode(Alphabet::Standard, root, &mut writer).expect("fetch fixture");
    format!("-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n", String::from_utf8_lossy(&writer.finish()))
        .into_bytes()
}
#[expect(clippy::disallowed_methods, reason = "the local TLS file server is a real socket peer on its own test thread")]
fn server(listener: TcpListener, response: Vec<u8>) -> thread::JoinHandle<Result<Vec<u8>, Starved>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept().expect("fetch fixture");
        let mut deadline = Deadline::arm("the setup fetch peer's request", skein_lib::Duration::from_secs(2));
        deadline.watch_process(std::process::id());
        let mut tls = rustls_std::StreamOwned::new(
            rustls_std::ServerConnection::new(pki::Server::plain().config()).expect("fetch fixture"),
            socket,
        );
        let mut head = Vec::new();
        let mut byte = [0];
        while !head.ends_with(b"\r\n\r\n") {
            if deadline.passed() {
                return Err(support::expired(&deadline));
            }
            tls.sock.set_read_timeout(Some(support::timeout(&deadline))).expect("fetch read timeout");
            tls.sock.set_write_timeout(Some(support::timeout(&deadline))).expect("fetch handshake timeout");
            match tls.read(&mut byte) {
                Ok(1) => head.push(byte[0]),
                Err(why) if why.kind() == std::io::ErrorKind::Interrupted => {}
                Err(why) if matches!(why.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => {
                    if deadline.passed() {
                        return Err(support::expired(&deadline));
                    }
                }
                Ok(_) | Err(_) => return Ok(head),
            }
        }
        let mut deadline = Deadline::arm("the setup fetch peer's response", skein_lib::Duration::from_secs(2));
        deadline.watch_process(std::process::id());
        tls.sock.set_write_timeout(Some(support::timeout(&deadline))).expect("fetch write timeout");
        if let Err(why) = tls.write_all(&response).and_then(|()| tls.flush()) {
            if matches!(why.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) && deadline.passed()
            {
                return Err(support::expired(&deadline));
            }
            panic!("fetch response: {why}");
        }
        Ok(head)
    })
}
#[test]
fn local_tls_fetch() {
    const BOUND: u64 = 1_048_576;
    const UNPACKED: usize = 32_768;
    let binary = vec![b'x'; UNPACKED];
    let good = body_response(&temper_fetch_shell::testing::zstd(&binary));
    for (name, root, trusted) in [("right", pki::ROOT, true), ("roots", pki::OTHER, false)] {
        let scratch = support::Scratch::new(&format!("fetch-{name}"));
        let file = scratch.path().join("cache/binary");
        let part_file = scratch.path().join("cache/binary.part");
        let packed_file = scratch.path().join("cache/binary.zst.part");
        fs::create_dir_all(file.parent().expect("fetch fixture")).expect("fetch fixture");
        let roots = scratch.path().join("root.pem");
        fs::write(&roots, pem(root)).expect("fetch fixture");
        let listener = TcpListener::bind("127.0.0.1:0").expect("fetch fixture");
        let port = listener.local_addr().expect("fetch fixture").port();
        let server = server(listener, good.clone());
        let url = format!("https://127.0.0.1:{port}/binary");
        let result = fetch::run(
            &support::io(),
            &skein_shell::Trust::Machine { ssl_cert_file: Some(roots) },
            Fetch::new(
                Box::new([Pin {
                    binary: Fetched::read(b"fetched forgejo 0 bytes").expect("progress fixture").binary,
                    origin: Origin { host: b"127.0.0.1".as_slice().into(), port },
                    path: b"/binary".as_slice().into(),
                    digest: sha(&binary),
                    file: file.as_os_str().as_bytes().into(),
                    packed: true,
                }]),
                BOUND,
            ),
            &mut Vec::new(),
        );
        // The fetch has settled before a peer starvation verdict can end the test.
        let head = server.join().expect("fetch fixture").unwrap_or_else(|starved| starved.pass());
        let expected = if trusted {
            assert!(result.is_ok(), "{name}: {result:?}");
            assert_eq!(fs::read(&file).expect("fetch fixture"), binary);
            assert_eq!(fs::metadata(&file).expect("fetch fixture").permissions().mode() & 0o777, 0o755);
            format!("GET /binary HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n")
        } else {
            let words = result.expect_err("fetch refused").words();
            assert!(words.starts_with(&format!("{url}: TLS:")), "{words}");
            assert!(!file.exists(), "{name}");
            String::new()
        };
        assert!(!part_file.exists(), "{name}");
        assert!(!packed_file.exists(), "{name}");
        assert_eq!(String::from_utf8(head).expect("fetch fixture"), expected, "{name}");
        scratch.remove();
    }
}

fn body_response(body: &[u8]) -> Vec<u8> {
    let mut response =
        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
    response.extend_from_slice(body);
    response
}

#[test]
#[expect(clippy::disallowed_methods, reason = "two concurrent launcher commands must serialize one local TLS fetch")]
fn concurrent_claude_commands_make_one_request() {
    let scratch = support::Scratch::new("claude-concurrent");
    let file = scratch.path().join("cache/claude");
    let roots = scratch.path().join("root.pem");
    fs::write(&roots, pem(pki::ROOT)).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let observer = listener.try_clone().unwrap();
    let peer = server(listener, body_response(b"abc"));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let commands: Vec<_> = (0..2)
        .map(|_| {
            let (file, roots, barrier) = (file.clone(), roots.clone(), barrier.clone());
            thread::spawn(move || {
                let pin = Pin {
                    binary: temper_forge_setup::fetch::Named::ClaudeCode,
                    origin: Origin { host: b"127.0.0.1".as_slice().into(), port },
                    path: b"/binary".as_slice().into(),
                    digest: sha(b"abc"),
                    file: file.as_os_str().as_bytes().into(),
                    packed: false,
                };
                barrier.wait();
                let mut out = Vec::new();
                let result = temper_fetch_shell::fetch::claude_code(
                    &support::io(),
                    &skein_shell::Trust::Machine { ssl_cert_file: Some(roots) },
                    &file,
                    pin,
                    3,
                    &mut out,
                );
                (result, out)
            })
        })
        .collect();
    let results: Vec<_> = commands.into_iter().map(|command| command.join().unwrap()).collect();
    let head = peer.join().unwrap().unwrap_or_else(|starved| starved.pass());
    assert!(head.starts_with(b"GET /binary "));
    for (result, _) in &results {
        assert!(result.is_ok(), "{result:?}");
    }
    assert_eq!(results.iter().filter(|(_, out)| out.starts_with(b"fetching Claude Code\n")).count(), 1);
    assert_eq!(results.iter().filter(|(_, out)| out.is_empty()).count(), 1);
    observer.set_nonblocking(true).unwrap();
    assert_eq!(observer.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    assert_eq!(fs::read(&file).unwrap(), b"abc");
    assert!(!file.with_extension("part").exists());
    scratch.remove();
}
