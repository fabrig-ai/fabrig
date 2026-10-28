pub mod support;
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
    sha256::Sha256,
};
use skein_tls_world::pki;
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    thread,
    time::Duration,
};
use temper_forge_setup::{
    fetch::{Failure, Fetch, Pin},
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
fn server(listener: TcpListener, response: &'static [u8]) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept().expect("fetch fixture");
        socket.set_read_timeout(Some(Duration::from_secs(2))).expect("fetch fixture");
        socket.set_write_timeout(Some(Duration::from_secs(2))).expect("fetch fixture");
        let mut tls = rustls::StreamOwned::new(
            rustls::ServerConnection::new(pki::Server::plain().config()).expect("fetch fixture"),
            socket,
        );
        let mut head = Vec::new();
        let mut byte = [0];
        while !head.ends_with(b"\r\n\r\n") {
            match tls.read(&mut byte) {
                Ok(1) => head.push(byte[0]),
                Ok(_) | Err(_) => return head,
            }
        }
        tls.write_all(response).expect("fetch fixture");
        tls.flush().expect("fetch fixture");
        head
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
        let server =
            if initial.is_none() { Some(server(listener.try_clone().expect("fetch fixture"), response)) } else { None };
        let url = format!("https://127.0.0.1:{}/binary", address.port());
        let result = fetch::run(
            &support::io(),
            &skein_shell::Trust::Machine { ssl_cert_file: Some(roots) },
            Fetch::new(
                Box::new([Pin {
                    origin: Origin { host: b"127.0.0.1".as_slice().into(), port: address.port() },
                    path: b"/binary".as_slice().into(),
                    digest: sha(b"abc"),
                    file: file.as_os_str().as_bytes().into(),
                }]),
                bound,
            ),
        );
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
            "tls" => {
                let words = result.expect_err("fetch refused").words();
                assert!(words.starts_with(&format!("{url}: TLS:")), "{words}");
            }
            _ => unreachable!(),
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
        if let Some(server) = server {
            let head = server.join().expect("fetch fixture");
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
            listener.set_nonblocking(true).expect("fetch fixture");
            assert_eq!(listener.accept().expect_err("fetch refused").kind(), std::io::ErrorKind::WouldBlock);
        }
        scratch.remove();
    }
}
