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
    fetch::{Bottle, Failure, Fetch, Fetched, Pin},
    setup::Origin,
};
use temper_forge_setup_shell::fetch::{self, Failed};
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
    let good = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc".as_slice();
    for (name, initial, initial_part, response, bound, root, expected) in [
        ("right", None, None, good, 3, pki::ROOT, "ok"),
        ("wrong", None, None, b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nxyz", 3, pki::ROOT, "digest"),
        ("matching", Some(b"abc".as_slice()), None, good, 3, pki::ROOT, "ok"),
        ("present", Some(b"xyz".as_slice()), None, good, 3, pki::ROOT, "present"),
        ("leftover", None, Some(b"long interrupted part".as_slice()), good, 3, pki::ROOT, "ok"),
        ("404", None, None, b"HTTP/1.1 404 Missing\r\nContent-Length: 0\r\n\r\n", 3, pki::ROOT, "status"),
        (
            "chunked",
            None,
            None,
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nab\r\n2\r\ncd\r\n0\r\n\r\n",
            3,
            pki::ROOT,
            "bytes",
        ),
        ("roots", None, None, good, 3, pki::OTHER, "tls"),
    ] {
        let scratch = support::Scratch::new(&format!("fetch-{name}"));
        let file = scratch.path().join("cache/binary");
        let part = scratch.path().join("cache/binary.part");
        fs::create_dir_all(file.parent().expect("fetch fixture")).expect("fetch fixture");
        if let Some(bytes) = initial {
            fs::write(&file, bytes).expect("fetch fixture");
        }
        if let Some(bytes) = initial_part {
            fs::write(&part, bytes).expect("fetch fixture");
        }
        let roots = scratch.path().join("root.pem");
        fs::write(&roots, pem(root)).expect("fetch fixture");
        let listener = TcpListener::bind("127.0.0.1:0").expect("fetch fixture");
        let address = listener.local_addr().expect("fetch fixture");
        let mut unused_listener = Some(listener);
        let server = if initial.is_none() {
            Some(server(unused_listener.take().expect("fetch listener"), response.to_vec()))
        } else {
            None
        };
        let url = format!("https://127.0.0.1:{}/binary", address.port());
        let mut progress = Vec::new();
        let result = fetch::run(
            &support::io(),
            &skein_shell::Trust::Machine { ssl_cert_file: Some(roots) },
            Fetch::new(
                Box::new([Pin {
                    binary: Fetched::read(b"fetched forgejo 0 bytes").expect("progress fixture").binary,
                    origin: Origin { host: b"127.0.0.1".as_slice().into(), port: address.port() },
                    path: b"/binary".as_slice().into(),
                    digest: sha(b"abc"),
                    file: file.as_os_str().as_bytes().into(),
                    bottle: None,
                }]),
                bound,
            ),
            &mut progress,
        );
        // The fetch has settled before a peer starvation verdict can end the test.
        let head = server.map(|server| server.join().expect("fetch fixture").unwrap_or_else(|starved| starved.pass()));
        check_outcome(expected, result, name, &url, &file);
        if initial.is_some() {
            assert!(progress.is_empty(), "present files say nothing");
        }
        if let Some(initial) = initial {
            assert_eq!(fs::read(&file).expect("fetch fixture"), initial);
        } else if expected == "ok" {
            assert_eq!(fs::read(&file).expect("fetch fixture"), b"abc");
            assert_eq!(fs::metadata(&file).expect("fetch fixture").permissions().mode() & 0o777, 0o755);
        } else {
            assert!(!file.exists(), "{name}");
        }
        assert!(!part.exists(), "{name}");
        if let Some(head) = head {
            let expected = if expected == "tls" {
                String::new()
            } else {
                format!(
                    "GET /binary HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n",
                    address.port()
                )
            };
            assert_eq!(String::from_utf8(head).expect("fetch fixture"), expected, "{name}");
        } else {
            let listener = unused_listener.expect("unused fetch listener");
            listener.set_nonblocking(true).expect("fetch fixture");
            assert_eq!(listener.accept().expect_err("fetch refused").kind(), std::io::ErrorKind::WouldBlock);
        }
        scratch.remove();
    }
}

fn check_outcome(expected: &str, result: Result<(), Failed>, name: &str, url: &str, file: &std::path::Path) {
    match expected {
        "ok" => assert!(result.is_ok(), "{name}: {result:?}"),
        "digest" => {
            assert!(matches!(result, Err(Failed::Refused(Failure::Digest { .. }))));
            assert_eq!(
                result.expect_err("fetch refused").words(),
                format!(
                    "{url}: digest differs: pinned ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad, got 3608bca1e44ea6c4d268eb6db02260269892c0b42b86bbf1e77a6fa16c3c9282"
                )
            );
        }
        "present" => assert_eq!(
            result.expect_err("fetch refused").words(),
            format!("{}: does not match the pinned digest", file.display())
        ),
        "status" => assert_eq!(result.expect_err("fetch refused").words(), format!("{url}: HTTP status 404")),
        "bytes" => {
            assert_eq!(result.expect_err("fetch refused").words(), format!("{url}: body past setup.fetch_bytes"));
        }
        "claude-bytes" => {
            assert_eq!(result.expect_err("fetch refused").words(), format!("{url}: body past the pinned size"));
        }
        "tls" => {
            let words = result.expect_err("fetch refused").words();
            assert!(words.starts_with(&format!("{url}: TLS:")), "{words}");
        }
        _ => unreachable!(),
    }
}

fn tarball(scratch: &support::Scratch, binary: &[u8]) -> Vec<u8> {
    use std::process::{Command, Stdio};
    let keg = scratch.path().join("keg");
    fs::create_dir_all(keg.join("bin")).expect("bottle fixture");
    fs::write(keg.join("bin/member"), binary).expect("bottle fixture");
    fs::write(keg.join("bin/empty"), []).expect("bottle fixture");
    let archive = scratch.path().join("bottle.tar.gz");
    let status = Command::new("/usr/bin/tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(keg)
        .args(["bin/member", "bin/empty"])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("make bottle fixture");
    assert!(status.success(), "make bottle fixture: {status}");
    fs::read(archive).expect("bottle fixture")
}
#[derive(Clone, Copy, Debug)]
enum BottleOutcome {
    Ok,
    Digest,
    Bytes,
    Extraction,
    Redirect,
    Gone,
}
fn bottle_outcome(result: Result<(), Failed>, expected: BottleOutcome, name: &str) {
    match (result, expected) {
        (Ok(()), BottleOutcome::Ok)
        | (Err(Failed::Refused(Failure::Digest { .. })), BottleOutcome::Digest)
        | (Err(Failed::Refused(Failure::Bytes { .. })), BottleOutcome::Bytes)
        | (Err(Failed::Refused(Failure::Extraction { .. })), BottleOutcome::Extraction)
        | (Err(Failed::Refused(Failure::Redirect { .. })), BottleOutcome::Redirect)
        | (Err(Failed::Refused(Failure::Gone { .. })), BottleOutcome::Gone) => {}
        (got, want) => panic!("{name}: wanted {want:?}, got {got:?}"),
    }
}
fn body_response(body: &[u8], chunked: bool) -> Vec<u8> {
    if chunked {
        let mut response =
            format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n", body.len())
                .into_bytes();
        response.extend_from_slice(body);
        response.extend_from_slice(b"\r\n0\r\n\r\n");
        response
    } else {
        let mut response =
            format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
        response.extend_from_slice(body);
        response
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table of bottle fetches over two TLS peers")]
fn local_tls_bottle_fetch() {
    // Two host chunks, compressing below the extraction bound, exercise both reads.
    const BINARY_BYTES: usize = 32_768;
    let scratch = support::Scratch::new("bottle-fetch");
    let binary = vec![b'x'; BINARY_BYTES];
    let archive = tarball(&scratch, &binary);
    let good = body_response(&archive, false);
    let roots = scratch.path().join("root.pem");
    fs::write(&roots, pem(pki::ROOT)).expect("bottle fixture");
    for (name, response, archive_digest, binary_digest, member, bound, token, leftover, expected) in [
        (
            "right",
            good.clone(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Ok,
        ),
        (
            "mirror",
            good.clone(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            false,
            false,
            BottleOutcome::Ok,
        ),
        (
            "leftover",
            good.clone(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            true,
            BottleOutcome::Ok,
        ),
        (
            "archive digest",
            good.clone(),
            sha(b"wrong"),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Digest,
        ),
        (
            "binary digest",
            good.clone(),
            sha(&archive),
            sha(b"wrong"),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Digest,
        ),
        (
            "archive length",
            good.clone(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            archive.len() as u64 - 1,
            true,
            false,
            BottleOutcome::Bytes,
        ),
        (
            "archive stream",
            body_response(&archive, true),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            archive.len() as u64 - 1,
            true,
            false,
            BottleOutcome::Bytes,
        ),
        (
            "extraction bound",
            good.clone(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64 - 1,
            true,
            false,
            BottleOutcome::Bytes,
        ),
        (
            "missing member",
            good.clone(),
            sha(&archive),
            sha(&binary),
            b"bin/missing".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Extraction,
        ),
        (
            "empty member",
            good.clone(),
            sha(&archive),
            sha(b""),
            b"bin/empty".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Extraction,
        ),
        (
            "corrupt archive",
            body_response(b"broken", false),
            sha(b"broken"),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Extraction,
        ),
        (
            "second redirect",
            b"HTTP/1.1 307 Again\r\nLocation: https://storage.test/object\r\nContent-Length: 0\r\n\r\n".to_vec(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Redirect,
        ),
        (
            "gone",
            b"HTTP/1.1 404 Missing\r\nContent-Length: 0\r\n\r\n".to_vec(),
            sha(&archive),
            sha(&binary),
            b"bin/member".as_slice(),
            BINARY_BYTES as u64,
            true,
            false,
            BottleOutcome::Gone,
        ),
    ] {
        let cache = scratch.path().join(name);
        fs::create_dir_all(&cache).expect("bottle fixture");
        let file = cache.join("binary");
        let part = cache.join("binary.part");
        let archive_file = cache.join("binary.archive");
        if leftover {
            let longer = vec![b'z'; BINARY_BYTES + 1];
            fs::write(&part, &longer).expect("bottle fixture");
            fs::write(&archive_file, longer).expect("bottle fixture");
        }
        let first = TcpListener::bind("127.0.0.1:0").expect("bottle fixture");
        let second = TcpListener::bind("127.0.0.1:0").expect("bottle fixture");
        let first_port = first.local_addr().expect("bottle fixture").port();
        let second_port = second.local_addr().expect("bottle fixture").port();
        let second = server(second, response);
        let first = server(first, format!("HTTP/1.1 307 Redirect\r\nLocation: https://127.0.0.1:{second_port}/object?signature=a%2Fb\r\nContent-Length: 0\r\n\r\n").into_bytes());
        let result = fetch::run(
            &support::io(),
            &skein_shell::Trust::Machine { ssl_cert_file: Some(roots.clone()) },
            Fetch::new(
                Box::new([Pin {
                    binary: Fetched::read(b"fetched forgejo 0 bytes").expect("progress fixture").binary,
                    origin: Origin { host: b"127.0.0.1".as_slice().into(), port: first_port },
                    path: b"/bottle".as_slice().into(),
                    digest: binary_digest,
                    file: file.as_os_str().as_bytes().into(),
                    bottle: Some(Bottle { digest: archive_digest, member: member.into(), token }),
                }]),
                bound,
            ),
            &mut Vec::new(),
        );
        let first = first.join().expect("bottle peer").unwrap_or_else(|starved| starved.pass());
        let second = second.join().expect("bottle peer").unwrap_or_else(|starved| starved.pass());
        bottle_outcome(result, expected, name);
        let authorization = if token { "Authorization: Bearer QQ==\r\n" } else { "" };
        assert_eq!(first, format!("GET /bottle HTTP/1.1\r\nHost: 127.0.0.1:{first_port}\r\nUser-Agent: temper\r\n{authorization}Connection: close\r\n\r\n").as_bytes(), "{name}");
        assert_eq!(second, format!("GET /object?signature=a%2Fb HTTP/1.1\r\nHost: 127.0.0.1:{second_port}\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n").as_bytes(), "{name}");
        match expected {
            BottleOutcome::Ok => {
                assert_eq!(fs::read(&file).expect("cached binary"), binary, "{name}");
                assert_eq!(fs::metadata(&file).expect("cached binary").permissions().mode() & 0o777, 0o755, "{name}");
            }
            BottleOutcome::Digest
            | BottleOutcome::Bytes
            | BottleOutcome::Extraction
            | BottleOutcome::Redirect
            | BottleOutcome::Gone => assert!(!file.exists(), "{name}"),
        }
        assert!(!part.exists(), "{name}: part removed");
        assert!(!archive_file.exists(), "{name}: archive removed");
    }
    scratch.remove();
}

fn fetch_to_writer(name: &str, chunked: bool, out: &mut dyn Write) {
    let scratch = support::Scratch::new(name);
    let roots = scratch.path().join("root.pem");
    fs::write(&roots, pem(pki::ROOT)).expect("fetch root");
    let file = scratch.path().join("cache/binary");
    let listener = TcpListener::bind("127.0.0.1:0").expect("fetch peer");
    let port = listener.local_addr().expect("fetch address").port();
    let peer = server(listener, body_response(b"abc", chunked));
    let result = fetch::run(
        &support::io(),
        &skein_shell::Trust::Machine { ssl_cert_file: Some(roots) },
        Fetch::new(
            Box::new([Pin {
                binary: Fetched::read(b"fetched runner 0 bytes").expect("progress fixture").binary,
                origin: Origin { host: b"127.0.0.1".as_slice().into(), port },
                path: b"/binary".as_slice().into(),
                digest: sha(b"abc"),
                file: file.as_os_str().as_bytes().into(),
                bottle: None,
            }]),
            3,
        ),
        out,
    );
    peer.join().expect("fetch peer").unwrap_or_else(|starved| starved.pass());
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs::read(&file).expect("fetched binary"), b"abc");
    assert!(!file.with_file_name("binary.part").exists());
    scratch.remove();
}

#[test]
fn the_lines_reach_the_writer_in_order() {
    for (chunked, expected) in [
        (false, "fetched runner 0 of 3 bytes\nfetched runner 3 of 3 bytes\n"),
        (true, "fetched runner 0 bytes\nfetched runner 3 bytes\n"),
    ] {
        let mut out = Vec::new();
        fetch_to_writer("fetch-progress-lines", chunked, &mut out);
        assert_eq!(out, expected.as_bytes());
    }
}

#[test]
fn a_writer_that_refuses_fails_no_fetch() {
    // A full slice accepts no byte and write_all returns WriteZero.
    let mut out = &mut [][..];
    fetch_to_writer("fetch-progress-refusing", false, &mut out);
    assert!(out.is_empty());
}

#[test]
fn claude_fetches_once_and_refuses_a_wrong_digest_or_existing_size() {
    for (name, initial, body, expected) in [
        ("missing", None, b"abc".as_slice(), "ok"),
        ("digest", None, b"xyz".as_slice(), "digest"),
        ("bound", None, b"abcd".as_slice(), "claude-bytes"),
        ("size", Some(b"other size".as_slice()), b"abc".as_slice(), "present"),
    ] {
        let scratch = support::Scratch::new(&format!("claude-fetch-{name}"));
        let file = scratch.path().join("cache/claude");
        let roots = scratch.path().join("root.pem");
        fs::write(&roots, pem(pki::ROOT)).unwrap();
        if let Some(initial) = initial {
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, initial).unwrap();
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let observer = listener.try_clone().unwrap();
        let peer = initial.is_none().then(|| server(listener, body_response(body, false)));
        let pin = Pin {
            binary: temper_forge_setup::fetch::Named::ClaudeCode,
            origin: Origin { host: b"127.0.0.1".as_slice().into(), port },
            path: b"/binary".as_slice().into(),
            digest: sha(b"abc"),
            file: file.as_os_str().as_bytes().into(),
            bottle: None,
        };
        let trust = skein_shell::Trust::Machine { ssl_cert_file: Some(roots) };
        let mut out = Vec::new();
        let result = temper_fetch_shell::fetch::claude_code(&support::io(), &trust, &file, pin.clone(), 3, &mut out);
        if let Some(peer) = peer {
            assert!(peer.join().unwrap().unwrap_or_else(|starved| starved.pass()).starts_with(b"GET /binary "));
        }
        check_outcome(expected, result, name, &format!("https://127.0.0.1:{port}/binary"), &file);
        if expected == "ok" {
            assert_eq!(fs::read(&file).unwrap(), b"abc");
            assert_eq!(
                out,
                b"fetching Claude Code\nfetched claude-code 0 of 3 bytes\nfetched claude-code 3 of 3 bytes\n"
            );
            out.clear();
            // No roots are needed on a cache hit, and no lock is opened.
            fs::remove_file(file.with_extension("lock")).unwrap();
            fs::create_dir(file.with_extension("lock")).unwrap();
            temper_fetch_shell::fetch::claude_code(&support::io(), &trust, &file, pin, 3, &mut out).unwrap();
            assert!(out.is_empty());
        } else if let Some(initial) = initial {
            assert_eq!(fs::read(&file).unwrap(), initial);
            assert!(out.is_empty());
        } else {
            assert!(!file.exists());
        }
        assert!(!file.with_extension("part").exists());
        observer.set_nonblocking(true).unwrap();
        assert_eq!(observer.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        scratch.remove();
    }
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
    let peer = server(listener, body_response(b"abc", false));
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
                    bottle: None,
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
