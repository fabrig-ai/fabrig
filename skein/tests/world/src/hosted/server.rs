//! The HTTP script answers over loopback and keeps the received request heads.
use super::Script;
use skein_io::kernel::{Complete, Done, Error, Family, Fd, Op, Signalled, Submit};
use skein_lib::Token;
use std::net::{Ipv4Addr, SocketAddr};

const WORK: Token = Token::new(0);
const SIGNAL: Token = Token::new(u64::MAX);
const CANCEL: Token = Token::new(u64::MAX - 1);
const HEAD_LIMIT: usize = 8192;

#[derive(Debug, PartialEq, Eq)]
enum Cancel {
    Unsent,
    Pending,
    Answered,
}

#[derive(Debug)]
enum Phase {
    Announce { output: Fd, prefix: &'static str, suffix: &'static str },
    Listening,
    Serve(u16),
    Stop(Cancel),
    Closed,
}

#[derive(Debug)]
pub(super) struct Server {
    phase: Phase,
    addr: Option<SocketAddr>,
    listener: Option<Fd>,
    connection: Option<Fd>,
    head: Vec<u8>,
    working: bool,
}

impl Server {
    pub fn announce(script: &mut Script, output: Fd, prefix: &'static str, suffix: &'static str) -> Self {
        let mut server = Self {
            phase: Phase::Announce { output, prefix, suffix },
            addr: None,
            listener: None,
            connection: None,
            head: Vec::new(),
            working: false,
        };
        server.submit(script, Op::Socket { family: Family::Ipv4 });
        server
    }

    pub fn serve(&mut self, script: &mut Script, status: u16) {
        assert!(matches!(self.phase, Phase::Listening), "the script announced its listening address");
        self.phase = Phase::Serve(status);
        script
            .submissions
            .push(Submit { op: SIGNAL, kind: Op::ReadSignal { fd: script.signal.expect("server signal source") } });
        self.submit(script, self.accept());
    }

    pub fn active(&self) -> bool {
        !matches!(self.phase, Phase::Listening | Phase::Closed)
    }

    pub fn iterate(&mut self, script: &mut Script) {
        let mut answer = None;
        while let Some(complete) = script.completions.pop() {
            match complete.op {
                SIGNAL => {
                    assert!(matches!(complete.result, Ok(Done::Signalled(Signalled::Termination { .. }))));
                    self.phase = Phase::Stop(Cancel::Unsent);
                }
                CANCEL => {
                    assert!(matches!(complete.result, Ok(Done::Nothing) | Err(Error::TooLate)));
                    self.phase = Phase::Stop(Cancel::Answered);
                }
                WORK => {
                    self.working = false;
                    assert!(answer.replace(complete).is_none(), "one sequential socket or pipe operation");
                }
                other => panic!("unexpected server operation {other:?}"),
            }
        }
        if matches!(self.phase, Phase::Stop(_)) {
            if let Some(answer) = answer {
                match (answer.kind, answer.result) {
                    (_, Ok(Done::Accepted { fd, .. })) => self.connection = Some(fd),
                    (Op::Close { fd }, Ok(Done::Nothing)) if self.connection == Some(fd) => self.connection = None,
                    _ => {}
                }
            }
            self.close(script);
        } else if let Some(answer) = answer
            && let Some(next) = self.advance(script, answer)
        {
            self.submit(script, next);
        }
    }

    fn submit(&mut self, script: &mut Script, kind: Op) {
        assert!(!self.working, "the server's socket and pipe operations are sequential");
        self.working = true;
        script.submissions.push(Submit { op: WORK, kind });
    }

    fn accept(&self) -> Op {
        Op::Accept { fd: self.listener.expect("listening socket") }
    }

    fn read(fd: Fd) -> Op {
        Op::Recv { fd, buf: Box::new([0; 1024]) }
    }

    fn advance(&mut self, script: &mut Script, answer: Complete) -> Option<Op> {
        match (answer.kind, answer.result) {
            (Op::Socket { .. }, Ok(Done::Fd(fd))) => {
                self.listener = Some(fd);
                Some(Op::Bind { fd, addr: (Ipv4Addr::LOCALHOST, 0).into() })
            }
            (Op::Bind { fd, .. }, Ok(Done::Bound(addr))) => {
                self.addr = Some(addr);
                Some(Op::Listen { fd, backlog: 4 })
            }
            (Op::Listen { .. }, Ok(Done::Nothing)) => {
                let Phase::Announce { output, prefix, suffix } = self.phase else { panic!("listen precedes the line") };
                let addr = self.addr.expect("bound address");
                let bytes = format!("{prefix}{addr}{suffix}\n").into_bytes().into_boxed_slice();
                Some(Op::PipeWrite { fd: output, bytes, from: 0 })
            }
            (Op::PipeWrite { fd, bytes, from }, Ok(Done::Count(wrote))) => {
                let from = from + wrote;
                if usize::try_from(from).expect("line fits") < bytes.len() {
                    Some(Op::PipeWrite { fd, bytes, from })
                } else {
                    self.phase = Phase::Listening;
                    None
                }
            }
            (Op::Accept { .. }, Ok(Done::Accepted { fd, .. })) => {
                self.connection = Some(fd);
                self.head.clear();
                Some(Self::read(fd))
            }
            (Op::Recv { fd, buf, .. }, Ok(Done::Count(count))) if count > 0 => {
                self.head.extend_from_slice(&buf[..usize::try_from(count).expect("received bytes fit")]);
                if let Some(end) = self.head.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    self.head.truncate(end + 4);
                    script.request_heads.push(std::mem::take(&mut self.head).into_boxed_slice());
                    let Phase::Serve(status) = self.phase else { panic!("answer while serving") };
                    let bytes = format!("HTTP/1.1 {status} Scripted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .into_bytes()
                        .into_boxed_slice();
                    Some(Op::Send { fd, bytes, from: 0 })
                } else {
                    assert!(self.head.len() <= HEAD_LIMIT, "the fixture receives a bounded HTTP request head");
                    Some(Self::read(fd))
                }
            }
            (Op::Send { fd, bytes, from }, Ok(Done::Count(wrote))) => {
                let from = from + wrote;
                if usize::try_from(from).expect("reply fits") < bytes.len() {
                    Some(Op::Send { fd, bytes, from })
                } else {
                    Some(Op::Close { fd })
                }
            }
            (Op::Recv { fd, .. }, Ok(Done::Count(0)) | Err(Error::Reset))
            | (Op::Send { fd, .. }, Err(Error::Reset | Error::BrokenPipe)) => Some(Op::Close { fd }),
            (Op::Close { .. }, Ok(Done::Nothing)) => {
                self.connection = None;
                Some(self.accept())
            }
            (op, result) => panic!("scripted server: {op:?} answered {result:?}"),
        }
    }

    fn close(&mut self, script: &mut Script) {
        let Phase::Stop(cancel) = &mut self.phase else { unreachable!("stop before closing") };
        if self.working {
            if *cancel == Cancel::Unsent {
                script.submissions.push(Submit { op: CANCEL, kind: Op::Cancel { target: WORK } });
                *cancel = Cancel::Pending;
            }
        } else if *cancel != Cancel::Pending {
            if let Some(fd) = self.connection.take().or_else(|| self.listener.take()) {
                self.submit(script, Op::Close { fd });
            } else {
                self.phase = Phase::Closed;
            }
        }
    }
}
