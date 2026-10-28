//! A local TLS server for the fetch command's pinned path.
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
};
use skein_tls_world::pki;
use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::Path,
    thread,
    time::Duration,
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
fn serve(listener: TcpListener, response: &'static [u8]) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept().expect("fetch fixture");
        socket.set_read_timeout(Some(Duration::from_secs(4))).expect("fetch fixture");
        socket.set_write_timeout(Some(Duration::from_secs(4))).expect("fetch fixture");
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

pub fn roots(path: &Path) {
    fs::write(path, pem(pki::ROOT)).expect("root bundle");
}
pub struct Server {
    pub address: SocketAddr,
    thread: thread::JoinHandle<Vec<u8>>,
}
#[must_use]
pub fn server() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").expect("file server");
    let address = listener.local_addr().expect("bound address");
    let thread = serve(listener, b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc");
    Server { address, thread }
}
impl Server {
    pub fn check(self) {
        let arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => panic!("test host architecture"),
        };
        let expected = format!(
            "GET /forgejo/forgejo/releases/download/v17.0.0/forgejo-17.0.0-linux-{arch} HTTP/1.1\r\nHost: {}\r\nUser-Agent: temper\r\nConnection: close\r\n\r\n",
            self.address
        );
        assert_eq!(self.thread.join().expect("file server"), expected.as_bytes());
    }
}
