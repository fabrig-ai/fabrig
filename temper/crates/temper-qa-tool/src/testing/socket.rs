//! Nonblocking byte streams for the world's sockets and children (temper/docs/design/testing/qa/README.md, 6).
use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt,
    io::{self, Read, Write},
    rc::Rc,
};

type Receive = Box<dyn FnMut(&[u8])>;

#[derive(Default)]
struct End {
    bytes: VecDeque<u8>,
    writer_closed: bool,
    reader_closed: bool,
    receive: Option<Receive>,
}

impl fmt::Debug for End {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("End")
            .field("bytes", &self.bytes)
            .field("writer_closed", &self.writer_closed)
            .field("reader_closed", &self.reader_closed)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Connection {
    incoming: Rc<RefCell<End>>,
    outgoing: Rc<RefCell<End>>,
}

impl Connection {
    /// Both ends of a stream. Dropping either end leaves its sent bytes to be drained before EOF.
    pub fn pair() -> (Self, Self) {
        let incoming = Rc::new(RefCell::new(End::default()));
        let outgoing = Rc::new(RefCell::new(End::default()));
        (
            Self { incoming: incoming.clone(), outgoing: outgoing.clone() },
            Self { incoming: outgoing, outgoing: incoming },
        )
    }

    /// Let this peer consume incoming writes as they happen, without another thread or a clock poll.
    /// The script takes the place of reading these bytes from the stream.
    pub fn script_receive(&self, receive: impl FnMut(&[u8]) + 'static) {
        let mut incoming = self.incoming.borrow_mut();
        assert!(incoming.bytes.is_empty() && incoming.receive.is_none(), "script a fresh receive end");
        incoming.receive = Some(Box::new(receive));
    }

    /// Send EOF after the queued bytes while keeping this end open for reading.
    pub fn close_write(&self) {
        self.outgoing.borrow_mut().writer_closed = true;
    }
}

impl Read for Connection {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let mut incoming = self.incoming.borrow_mut();
        if bytes.is_empty() || incoming.writer_closed && incoming.bytes.is_empty() {
            return Ok(0);
        }
        if incoming.bytes.is_empty() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let count = bytes.len().min(incoming.bytes.len());
        for (byte, sent) in bytes.iter_mut().zip(incoming.bytes.drain(..count)) {
            *byte = sent;
        }
        Ok(count)
    }
}

impl Write for Connection {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut outgoing = self.outgoing.borrow_mut();
        if outgoing.writer_closed || outgoing.reader_closed {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        if let Some(receive) = &mut outgoing.receive {
            receive(bytes);
        } else {
            outgoing.bytes.extend(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.close_write();
        let mut incoming = self.incoming.borrow_mut();
        incoming.reader_closed = true;
        incoming.bytes.clear();
    }
}
