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
}

trait Wait<S> {
    fn timeout(&self) -> Option<std::time::Duration>;
    fn check(&mut self, stream: &mut Option<S>) -> io::Result<()>;
    fn retry(&mut self, error: io::Error, stream: &mut Option<S>) -> io::Result<()>;
}

impl Wait<TcpStream> for Bound {
    fn timeout(&self) -> Option<std::time::Duration> {
        match self {
            Self::World(patience) => patience.socket_timeout(),
            Self::Owned(deadline) => Some(socket_timeout(deadline)),
        }
    }

    fn check(&mut self, stream: &mut Option<TcpStream>) -> io::Result<()> {
        match self {
            Self::World(patience) => match patience.failed() {
                Some(figures) => Err(io::Error::new(io::ErrorKind::TimedOut, figures.to_string())),
                None => Ok(()),
            },
            Self::Owned(deadline) if deadline.passed() => match deadline.expired() {
                Expired::Starved(starved) => {
                    drop(stream.take());
                    Err(io::Error::other(StarvedSocket(starved)))
                }
                Expired::Failed(figures) => Err(io::Error::new(io::ErrorKind::TimedOut, figures.to_string())),
            },
            Self::Owned(_) => Ok(()),
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
                        Err(io::Error::other(StarvedSocket(starved)))
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
    framed: Framed<TcpStream>,
    scope: Scope,
}
impl Socket {
    #[must_use]
    pub fn new(stream: TcpStream, scope: Scope) -> Self {
        Self { framed: Framed { stream: Some(stream) }, scope }
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
        self.framed.stream()
    }

    pub fn write_all(&mut self, what: &'static str, within: Duration, bytes: &[u8]) -> io::Result<()> {
        self.framed.write_all(&mut Bound::arm(what, within, self.scope), bytes)
    }

    pub fn read_to_end(&mut self, what: &'static str, within: Duration, limit: usize) -> io::Result<Vec<u8>> {
        self.framed.read_to_end(&mut Bound::arm(what, within, self.scope), limit)
    }

    pub fn read_exact(&mut self, what: &'static str, within: Duration, bytes: &mut [u8]) -> io::Result<()> {
        self.framed.read_exact(&mut Bound::arm(what, within, self.scope), bytes)
    }

    pub fn head(&mut self, what: &'static str, within: Duration) -> io::Result<Vec<u8>> {
        self.framed.head(&mut Bound::arm(what, within, self.scope))
    }
}

trait Stream: Read + Write {
    fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> io::Result<()>;
    fn set_write_timeout(&mut self, timeout: Option<std::time::Duration>) -> io::Result<()>;
}

impl Stream for TcpStream {
    fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> io::Result<()> {
        TcpStream::set_read_timeout(self, timeout)
    }

    fn set_write_timeout(&mut self, timeout: Option<std::time::Duration>) -> io::Result<()> {
        TcpStream::set_write_timeout(self, timeout)
    }
}

struct Framed<S> {
    stream: Option<S>,
}

impl<S: Stream> Framed<S> {
    fn stream(&mut self) -> &mut S {
        self.stream.as_mut().expect("open story socket")
    }

    fn write_all(&mut self, bound: &mut impl Wait<S>, mut bytes: &[u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            self.stream().set_write_timeout(bound.timeout())?;
            match self.stream().write(bytes) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(count) => bytes = &bytes[count..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
            if !bytes.is_empty() {
                bound.check(&mut self.stream)?;
            }
        }
        Ok(())
    }

    fn read_to_end(&mut self, bound: &mut impl Wait<S>, limit: usize) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut buffer = [0; 8192];
        while bytes.len() < limit {
            self.stream().set_read_timeout(bound.timeout())?;
            let length = buffer.len().min(limit - bytes.len());
            match self.stream().read(&mut buffer[..length]) {
                Ok(0) => break,
                Ok(count) => bytes.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
            if bytes.len() < limit {
                bound.check(&mut self.stream)?;
            }
        }
        Ok(bytes)
    }

    fn read_exact(&mut self, bound: &mut impl Wait<S>, mut bytes: &mut [u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            self.stream().set_read_timeout(bound.timeout())?;
            match self.stream().read(bytes) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(count) => bytes = &mut bytes[count..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
            if !bytes.is_empty() {
                bound.check(&mut self.stream)?;
            }
        }
        Ok(())
    }

    fn head(&mut self, bound: &mut impl Wait<S>) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        while !bytes.ends_with(b"\r\n\r\n") {
            self.stream().set_read_timeout(bound.timeout())?;
            let mut byte = [0];
            match self.stream().read(&mut byte) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(_) => bytes.push(byte[0]),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => bound.retry(error, &mut self.stream)?,
            }
            assert!(bytes.len() <= 65_536, "bounded HTTP head");
            if !bytes.ends_with(b"\r\n\r\n") {
                bound.check(&mut self.stream)?;
            }
        }
        Ok(bytes)
    }
}

pub mod tests {
    use super::{Framed, Stream, Wait};
    use std::{
        io::{self, Read, Write},
        time::Duration,
    };

    struct Script {
        incoming: &'static [u8],
        outgoing: Vec<u8>,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    }

    const TIMEOUT: Duration = Duration::from_secs(1);

    impl Read for Script {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            assert_eq!(self.read_timeout, Some(TIMEOUT));
            let length = bytes.len().min(3);
            self.incoming.read(&mut bytes[..length])
        }
    }

    impl Write for Script {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            assert_eq!(self.write_timeout, Some(TIMEOUT));
            let length = bytes.len().min(3);
            self.outgoing.extend_from_slice(&bytes[..length]);
            Ok(length)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Stream for Script {
        fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
            self.read_timeout = timeout;
            Ok(())
        }

        fn set_write_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
            self.write_timeout = timeout;
            Ok(())
        }
    }

    struct Pending;
    impl Wait<Script> for Pending {
        fn timeout(&self) -> Option<Duration> {
            Some(TIMEOUT)
        }

        fn check(&mut self, _: &mut Option<Script>) -> io::Result<()> {
            Ok(())
        }

        fn retry(&mut self, error: io::Error, _: &mut Option<Script>) -> io::Result<()> {
            Err(error)
        }
    }

    pub fn bounded_reads_preserve_the_next_bytes_and_eof_is_an_error() {
        let mut socket = Framed {
            stream: Some(Script {
                incoming: b"head\r\n\r\nbodytail",
                outgoing: Vec::new(),
                read_timeout: None,
                write_timeout: None,
            }),
        };
        socket.write_all(&mut Pending, b"request").expect("request");
        assert_eq!(socket.stream().outgoing, b"request");
        assert_eq!(socket.head(&mut Pending).expect("head"), b"head\r\n\r\n");
        assert_eq!(socket.read_to_end(&mut Pending, 4).expect("body"), b"body");
        let mut tail = [0; 4];
        socket.read_exact(&mut Pending, &mut tail).expect("tail");
        assert_eq!(&tail, b"tail");
        let error = socket.read_exact(&mut Pending, &mut tail).expect_err("EOF fails");
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }
}
