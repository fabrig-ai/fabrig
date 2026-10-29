//! Story sockets and their waits (skein/docs/foundation/testing-strategy.md, 2.10).
use super::patience::{Patience, socket_timeout};
use skein_lib::Duration;
use skein_world::{Deadline, Expired, Starved};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
};

#[derive(Clone, Copy)]
pub enum Scope {
    /// A world owns the caller's processes and final verdict.
    World,
    /// The helper owns this socket outside a world.
    Owned,
}

enum Bound {
    World(Patience),
    Owned(Deadline),
}
impl Bound {
    fn arm(what: &'static str, within: Duration, scope: Scope) -> Self {
        match scope {
            Scope::World => Self::World(Patience::arm(what, within)),
            Scope::Owned => Self::Owned(Deadline::arm(what, within)),
        }
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        match self {
            Self::World(patience) => patience.socket_timeout(),
            Self::Owned(deadline) => Some(socket_timeout(deadline)),
        }
    }

    fn retry(&mut self, error: io::Error, stream: &mut Option<TcpStream>) -> io::Result<()> {
        match self {
            Self::World(patience) => patience.socket_retry(error),
            Self::Owned(deadline)
                if matches!(error.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock) && deadline.passed() =>
            {
                match deadline.expired() {
                    Expired::Starved(starved) => {
                        drop(stream.take());
                        return Err(io::Error::other(StarvedSocket(starved)));
                    }
                    Expired::Failed(figures) => Err(io::Error::new(error.kind(), format!("{error}; {figures}"))),
                }
            }
            Self::Owned(_) => Err(error),
        }
    }
}

/// An owning caller receives starvation only after its socket is closed.
#[derive(Debug)]
pub struct StarvedSocket(pub Starved);
impl std::fmt::Display for StarvedSocket {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.figures().fmt(out)
    }
}
impl std::error::Error for StarvedSocket {}

/// Call after releasing the story's child owners; passing skips destructors.
pub fn finish(error: io::Error) -> ! {
    let message = error.to_string();
    match error.into_inner().and_then(|inner| inner.downcast::<StarvedSocket>().ok()) {
        Some(starved) => starved.0.pass(),
        None => panic!("the story socket failed: {message}"),
    }
}

/// Partial reads and writes survive a world-owned wait's starvation deferral.
pub struct Socket {
    stream: Option<TcpStream>,
    scope: Scope,
}
impl Socket {
    pub fn new(stream: TcpStream, scope: Scope) -> Self {
        Self { stream: Some(stream), scope }
    }

    pub fn connect(what: &'static str, within: Duration, address: SocketAddr, scope: Scope) -> io::Result<Self> {
        let mut bound = Bound::arm(what, within, scope);
        loop {
            let result = match bound.timeout() {
                Some(timeout) => TcpStream::connect_timeout(&address, timeout),
                None => TcpStream::connect(address),
            };
            match result {
                Ok(stream) => return Ok(Self::new(stream, scope)),
                Err(error) => bound.retry(error, &mut None)?,
            }
        }
    }

    pub fn stream(&mut self) -> &mut TcpStream {
        self.stream.as_mut().expect("open story socket")
    }

    pub fn write_all(&mut self, what: &'static str, within: Duration, mut bytes: &[u8]) -> io::Result<()> {
        let mut bound = Bound::arm(what, within, self.scope);
        while !bytes.is_empty() {
            self.stream().set_write_timeout(bound.timeout())?;
            match self.stream().write(bytes) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(count) => bytes = &bytes[count..],
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
        }
        Ok(())
    }

    pub fn read_to_end(&mut self, what: &'static str, within: Duration, limit: usize) -> io::Result<Vec<u8>> {
        let mut bound = Bound::arm(what, within, self.scope);
        let mut bytes = Vec::new();
        let mut buffer = [0; 8192];
        while bytes.len() < limit {
            self.stream().set_read_timeout(bound.timeout())?;
            let length = buffer.len().min(limit - bytes.len());
            match self.stream().read(&mut buffer[..length]) {
                Ok(0) => break,
                Ok(count) => bytes.extend_from_slice(&buffer[..count]),
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
        }
        Ok(bytes)
    }

    pub fn read_exact(&mut self, what: &'static str, within: Duration, mut bytes: &mut [u8]) -> io::Result<()> {
        let mut bound = Bound::arm(what, within, self.scope);
        while !bytes.is_empty() {
            self.stream().set_read_timeout(bound.timeout())?;
            match self.stream().read(bytes) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(count) => bytes = &mut bytes[count..],
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
        }
        Ok(())
    }

    pub fn head(&mut self, what: &'static str, within: Duration) -> io::Result<Vec<u8>> {
        let mut bound = Bound::arm(what, within, self.scope);
        let mut bytes = Vec::new();
        while !bytes.ends_with(b"\r\n\r\n") {
            self.stream().set_read_timeout(bound.timeout())?;
            let mut byte = [0];
            match self.stream().read(&mut byte) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(_) => bytes.push(byte[0]),
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
            assert!(bytes.len() <= 65_536, "bounded HTTP head");
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{Scope, Socket};
    use skein_lib::Duration;
    use std::{
        io::{self, Read, Write},
        net::{TcpListener, TcpStream},
    };

    #[test]
    fn bounded_reads_preserve_the_next_bytes_and_eof_is_an_error() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("local fixture listener");
        let client = TcpStream::connect(listener.local_addr().expect("address")).expect("fixture connection");
        let (mut peer, _) = listener.accept().expect("fixture peer");
        let mut socket = Socket::new(client, Scope::World);
        let within = Duration::from_secs(1);
        socket.write_all("the socket fixture's request", within, b"request").expect("request");
        let mut request = [0; 7];
        peer.read_exact(&mut request).expect("request bytes");
        assert_eq!(&request, b"request");
        peer.write_all(b"head\r\n\r\nbodytail").expect("fixture reply");
        assert_eq!(socket.head("the socket fixture's head", within).expect("head"), b"head\r\n\r\n");
        assert_eq!(socket.read_to_end("the socket fixture's body", within, 4).expect("body"), b"body");
        let mut tail = [0; 4];
        socket.read_exact("the socket fixture's tail", within, &mut tail).expect("tail");
        assert_eq!(&tail, b"tail");
        drop(peer);
        let error = socket.read_exact("the socket fixture's EOF", within, &mut tail).expect_err("EOF fails");
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }
}
