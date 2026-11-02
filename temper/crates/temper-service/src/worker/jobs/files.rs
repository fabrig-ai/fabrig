//! Bounded files below the worker root (temper/docs/design/protocol/tooling.md, 3 and 6).
use alloc::boxed::Box;
use io::kernel::{self, Done, Fd, Op, OpenHow};
use skein_io as io;
use skein_lib::{Token, Writer};
use temper_worker_domain::tooling::Event;

#[derive(Debug)]
pub(in crate::worker) struct File {
    owner: Token,
    fd: Option<Fd>,
    phase: Phase,
    write: bool,
    bytes: Box<[u8]>,
    at: u32,
    most: u32,
    result: Option<Writer>,
    failed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Open,
    Existing,
    Truncate,
    Transfer,
    Size,
    Close,
}
#[derive(Debug)]
pub(in crate::worker) enum Next {
    Run(Op),
    End(Event),
}
impl File {
    pub(in crate::worker) fn owner(&self) -> Token {
        self.owner
    }
    pub(in crate::worker) fn write(owner: Token, root: Fd, path: Box<[u8]>, bytes: Box<[u8]>) -> (Self, Op) {
        (
            Self {
                owner,
                fd: None,
                phase: Phase::Open,
                write: true,
                bytes,
                at: 0,
                most: 0,
                result: None,
                failed: false,
            },
            Op::Open { root, path, how: OpenHow::CreateNoFollow { mode: Some(0o600) } },
        )
    }
    pub(in crate::worker) fn read(owner: Token, root: Fd, path: Box<[u8]>, most: u32) -> (Self, Op) {
        (
            Self {
                owner,
                fd: None,
                phase: Phase::Open,
                write: false,
                bytes: Box::new([]),
                at: 0,
                most,
                result: Some(Writer::new(usize::try_from(most).expect("file bound"))),
                failed: false,
            },
            Op::Open { root, path, how: OpenHow::ReadNoFollow },
        )
    }
    fn end(&mut self) -> Next {
        Next::End(if self.write {
            Event::Written { owner: self.owner, written: !self.failed }
        } else {
            Event::Read {
                owner: self.owner,
                bytes: if self.failed { None } else { Some(self.result.take().expect("read buffer").finish()) },
            }
        })
    }
    fn close(&mut self) -> Next {
        self.phase = Phase::Close;
        Next::Run(Op::Close { fd: self.fd.expect("file opened") })
    }
    fn transfer(&mut self) -> Op {
        let fd = self.fd.expect("file opened");
        if self.write {
            Op::Write {
                fd,
                bytes: core::mem::replace(&mut self.bytes, Box::new([])),
                from: self.at,
                at: u64::from(self.at),
            }
        } else {
            let left = self.most.checked_sub(self.at).expect("bounded read");
            Op::Read {
                fd,
                buf: skein_lib::bytes::zeroed(usize::try_from(left.min(4096)).expect("bounded read")),
                at: u64::from(self.at),
            }
        }
    }
    pub(in crate::worker) fn complete(&mut self, complete: kernel::Complete, abort: bool) -> Next {
        self.failed |= abort;
        if self.phase == Phase::Close {
            self.failed |= complete.result.is_err();
            return self.end();
        }
        match self.phase {
            Phase::Open | Phase::Existing => {
                if self.write && self.phase == Phase::Open && complete.result == Err(kernel::Error::Exists) && !abort {
                    let Op::Open { root, path, .. } = complete.kind else { unreachable!("file open completion") };
                    self.phase = Phase::Existing;
                    return Next::Run(Op::Open { root, path, how: OpenHow::WriteNoFollow });
                }
                match complete.result {
                    Ok(Done::Fd(fd)) => self.fd = Some(fd),
                    Err(_) => {
                        self.failed = true;
                        return self.end();
                    }
                    Ok(_) => unreachable!("file open completion"),
                }
                if self.write && self.phase == Phase::Existing && !self.failed {
                    self.phase = Phase::Truncate;
                    return Next::Run(Op::Truncate { fd: self.fd.expect("file opened"), length: 0 });
                }
            }
            Phase::Transfer => match complete.result {
                Ok(Done::Count(count)) => {
                    if self.write {
                        let Op::Write { bytes, .. } = complete.kind else { unreachable!("file write completion") };
                        self.bytes = bytes;
                    } else {
                        let Op::Read { buf, .. } = complete.kind else { unreachable!("file read completion") };
                        if count == 0 {
                            return self.close();
                        }
                        let bytes = buf.get(..usize::try_from(count).expect("read count")).expect("kernel read count");
                        if self.result.as_mut().expect("read buffer").put(bytes).is_err() {
                            self.failed = true;
                        }
                    }
                    self.at = self.at.checked_add(count).expect("bounded file offset");
                }
                Err(_) => self.failed = true,
                Ok(_) => unreachable!("file transfer completion"),
            },
            Phase::Size => {
                self.failed |= match complete.result {
                    Ok(Done::Stat(stat)) => stat.kind != kernel::Kind::File || stat.size > u64::from(self.most),
                    Err(_) => true,
                    Ok(_) => unreachable!("file size completion"),
                };
                return self.close();
            }
            Phase::Truncate => self.failed |= complete.result.is_err(),
            Phase::Close => unreachable!("close handled above"),
        }
        if self.failed || (self.write && usize::try_from(self.at).expect("file offset") == self.bytes.len()) {
            return self.close();
        }
        if !self.write && self.at == self.most {
            self.phase = Phase::Size;
            return Next::Run(Op::Stat { fd: self.fd.expect("file opened") });
        }
        self.phase = Phase::Transfer;
        Next::Run(self.transfer())
    }
}
