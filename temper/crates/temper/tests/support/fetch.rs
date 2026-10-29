//! A local TLS server for the fetch command's pinned path.
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
};
use skein_tls_world::pki;
use skein_world::{Deadline, Expired, Starved};
use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::Path,
    thread,
};
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
fn serve(listener: TcpListener, response: &'static [u8]) -> thread::JoinHandle<Result<Vec<u8>, Starved>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept().expect("fetch fixture");
        let mut deadline = Deadline::arm("the fetch command peer's request", skein_lib::Duration::from_secs(4));
        deadline.watch_process(std::process::id());
        let mut tls = rustls::StreamOwned::new(
            rustls::ServerConnection::new(pki::Server::plain().config()).expect("fetch fixture"),
            socket,
        );
        let mut head = Vec::new();
        let mut byte = [0];
        while !head.ends_with(b"\r\n\r\n") {
            if deadline.passed() {
                return Err(expired(&deadline));
            }
            tls.sock.set_read_timeout(Some(timeout(&deadline))).expect("fetch read timeout");
            tls.sock.set_write_timeout(Some(timeout(&deadline))).expect("fetch handshake timeout");
            match tls.read(&mut byte) {
                Ok(1) => head.push(byte[0]),
                Err(why) if why.kind() == std::io::ErrorKind::Interrupted => {}
                Err(why) if matches!(why.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => {
                    if deadline.passed() {
                        return Err(expired(&deadline));
                    }
                }
                Ok(_) | Err(_) => return Ok(head),
            }
        }
        let mut deadline = Deadline::arm("the fetch command peer's response", skein_lib::Duration::from_secs(4));
        deadline.watch_process(std::process::id());
        tls.sock.set_write_timeout(Some(timeout(&deadline))).expect("fetch write timeout");
        if let Err(why) = tls.write_all(response).and_then(|()| tls.flush()) {
            if matches!(why.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) && deadline.passed()
            {
                return Err(expired(&deadline));
            }
            panic!("fetch response: {why}");
        }
        Ok(head)
    })
}

pub fn roots(path: &Path) {
    fs::write(path, pem(pki::ROOT)).expect("root bundle");
}
pub struct Server {
    pub address: SocketAddr,
    thread: Option<thread::JoinHandle<Result<Vec<u8>, Starved>>>,
}
#[must_use]
pub fn server() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").expect("file server");
    let address = listener.local_addr().expect("bound address");
    let thread = serve(listener, b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc");
    Server { address, thread: Some(thread) }
}
impl Server {
    pub fn check(&mut self) {
        let arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => panic!("test host architecture"),
        };
        let expected = format!(
            "GET /forgejo/forgejo/releases/download/v17.0.0/forgejo-17.0.0-linux-{arch} HTTP/1.1\r\nHost: {}\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n",
            self.address
        );
        let head = self
            .thread
            .take()
            .expect("one file server check")
            .join()
            .expect("file server")
            .unwrap_or_else(|starved| starved.pass());
        assert_eq!(head, expected.as_bytes());
    }
}

fn timeout(deadline: &Deadline) -> std::time::Duration {
    std::time::Duration::from_nanos(deadline.remaining().as_nanos()).max(std::time::Duration::from_millis(1))
}
fn expired(deadline: &Deadline) -> Starved {
    match deadline.expired() {
        Expired::Starved(starved) => starved,
        Expired::Failed(figures) => panic!("the fetch peer did not answer: {figures}"),
    }
}
