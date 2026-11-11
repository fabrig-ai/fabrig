//! Classified blocking socket operations (skein/docs/foundation/testing-strategy.md, 2.10).
use super::patience::socket_timeout;
use skein_lib::Duration;
use skein_world::{Deadline, Expired, end_to_end::Binary};
use std::{
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpStream},
};

/// A socket owner closes before passing; a world's callback defers to its owner.
pub struct Socket<'a> {
    stream: Option<TcpStream>,
    within: Duration,
    names: [Option<&'static str>; 2],
    absolute: Option<Deadline>,
    fixed: bool,
    deferred: bool,
    defer: bool,
    binary: Option<&'a Binary>,
}

impl<'a> Socket<'a> {
    pub fn owned(socket: TcpStream, within: Duration, names: [&'static str; 2]) -> Self {
        Self {
            stream: Some(socket),
            within,
            names: names.map(Some),
            absolute: None,
            fixed: false,
            deferred: false,
            defer: false,
            binary: None,
        }
    }

    /// Shared peers and probes retain their enclosing owner's verdict and cleanup.
    #[must_use]
    pub fn deferred(socket: TcpStream, within: Duration, names: [&'static str; 2]) -> Self {
        let mut socket = Self::owned(socket, within, names);
        socket.defer = true;
        socket
    }

    pub fn connect(
        address: SocketAddr,
        what: &'static str,
        within: Duration,
        names: [&'static str; 2],
        defer: bool,
        binary: Option<&'a Binary>,
    ) -> io::Result<Self> {
        let mut deadline = Deadline::arm(what, within);
        if let Some(binary) = binary {
            deadline.watch(binary);
        }
        let mut deferred = false;
        let socket = loop {
            let result = if deferred {
                TcpStream::connect(address)
            } else {
                TcpStream::connect_timeout(&address, socket_timeout(&deadline))
            };
            match result {
                Ok(socket) => break socket,
                Err(error) if timeout(&error) => {
                    if !deadline.passed() {
                        continue;
                    }
                    match deadline.expired() {
                        Expired::Starved(starved) if !defer => starved.pass(),
                        Expired::Starved(_) => deferred = true,
                        Expired::Failed(figures) => {
                            eprintln!("socket wait failed: {figures}");
                            return Err(io::Error::new(error.kind(), format!("{error}; {figures}")));
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        };
        Ok(Self {
            stream: Some(socket),
            within,
            names: names.map(Some),
            absolute: None,
            fixed: false,
            deferred,
            defer,
            binary,
        })
    }

    /// Keep the receiver's absolute bound across its accepted socket's operations.
    #[must_use]
    pub fn until(socket: TcpStream, deadline: Option<Deadline>) -> Self {
        Self {
            stream: Some(socket),
            within: Duration::ZERO,
            names: [None; 2],
            deferred: deadline.is_none(),
            absolute: deadline,
            fixed: true,
            defer: true,
            binary: None,
        }
    }

    /// Read-only fixture bounds leave writes blocking, as their callers arrange.
    #[must_use]
    pub fn reading(socket: TcpStream, within: Duration, what: &'static str) -> Self {
        let mut socket = Self::owned(socket, within, [what, what]);
        socket.names[1] = None;
        socket
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.stream.as_ref().expect("open socket").shutdown(how)
    }

    fn deadline(&mut self, what: Option<&'static str>) -> Option<Deadline> {
        if self.fixed {
            return self.absolute.take();
        }
        if self.deferred {
            return None;
        }
        let mut deadline = Deadline::arm(what?, self.within);
        if let Some(binary) = self.binary {
            deadline.watch(binary);
        }
        Some(deadline)
    }

    fn retry(&mut self, error: io::Error, deadline: &mut Option<Deadline>) -> io::Result<()> {
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok(());
        }
        if timeout(&error)
            && let Some(bound) = deadline
        {
            if !bound.passed() {
                return Ok(());
            }
            match bound.expired() {
                Expired::Starved(_) if self.defer => {
                    self.deferred = true;
                    *deadline = None;
                    return Ok(());
                }
                Expired::Starved(starved) => {
                    let socket = self.stream.take().expect("open socket");
                    let _ = socket.shutdown(Shutdown::Both);
                    drop(socket);
                    starved.pass();
                }
                Expired::Failed(figures) => {
                    eprintln!("socket wait failed: {figures}");
                    return Err(io::Error::new(error.kind(), format!("{error}; {figures}")));
                }
            }
        }
        Err(error)
    }
}

fn timeout(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock)
}

impl Read for Socket<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let mut deadline = self.deadline(self.names[0]);
        loop {
            let socket = self.stream.as_mut().expect("open socket");
            socket.set_read_timeout(deadline.as_ref().map(socket_timeout))?;
            match socket.read(bytes) {
                Ok(count) => {
                    if self.fixed {
                        self.absolute = deadline;
                    }
                    return Ok(count);
                }
                Err(error) => self.retry(error, &mut deadline)?,
            }
        }
    }
}

impl Write for Socket<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut deadline = self.deadline(self.names[1]);
        loop {
            let socket = self.stream.as_mut().expect("open socket");
            socket.set_write_timeout(deadline.as_ref().map(socket_timeout))?;
            match socket.write(bytes) {
                Ok(count) => {
                    if self.fixed {
                        self.absolute = deadline;
                    }
                    return Ok(count);
                }
                Err(error) => self.retry(error, &mut deadline)?,
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.as_mut().expect("open socket").flush()
    }
}
