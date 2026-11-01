//! Resumable forge-directory removal beneath data
//! (temper/docs/plan/playground-reset-r4/notes/contracts.md, 3;
//! temper/docs/design/draft/playground-reset.md, 4).
use crate::{lines::joined, records::error_name};
use alloc::boxed::Box;
use skein_io::kernel::{Complete, Done, Entry, Error, Fd, LONGEST_NAME, Op, OpenHow, Submit};
use skein_lib::{Decimal, List, Queue, Token, bytes};

const OP: Token = Token::new(u64::MAX - 11);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Idle,
    Verify,
    VerifyClose,
    MakeBox,
    Box,
    MakeTrash,
    BoxClose,
    Trash,
    Parent,
    Move,
    List,
    Remove,
    Directory,
    Child,
    ListChild,
    Lift,
    Restart,
    Close,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Work {
    Move,
    Sweep,
    Retry,
}
/// One kernel operation and at most three open directories; no recursion.
#[derive(Debug)]
pub(crate) struct Sweep {
    phase: Phase,
    waiting: bool,
    box_dir: Option<Fd>,
    trash: Option<Fd>,
    child: Option<Fd>,
    forge: u16,
    original: Box<[u8]>,
    spelling: Box<[u8]>,
    parent: Box<[u8]>,
    name: Box<[u8]>,
    entry: Box<[u8]>,
    lifted: Box<[u8]>,
    work: Work,
    failed: bool,
    notice: Option<Box<[u8]>>,
    answer: Option<bool>,
}
impl Sweep {
    pub fn new() -> Self {
        Self {
            phase: Phase::Trash,
            waiting: false,
            box_dir: None,
            trash: None,
            child: None,
            forge: 0,
            original: Box::new([]),
            spelling: Box::new([]),
            parent: Box::new([]),
            name: Box::new([]),
            entry: Box::new([]),
            lifted: Box::new([]),
            work: Work::Sweep,
            failed: false,
            notice: None,
            answer: None,
        }
    }
    pub fn owns(token: Token) -> bool {
        token == OP
    }
    pub fn ready(&self) -> bool {
        self.phase == Phase::Idle && self.notice.is_none() && self.answer.is_none()
    }
    pub fn pending(&self) -> bool {
        !self.waiting && !self.ready()
    }
    pub fn has_notice(&self) -> bool {
        self.notice.is_some()
    }
    pub fn take_notice(&mut self) -> Option<Box<[u8]>> {
        self.notice.take()
    }
    pub fn take_answer(&mut self) -> Option<bool> {
        self.answer.take()
    }
    pub fn sweep(&mut self) {
        assert!(self.ready(), "one sweep");
        self.failed = false;
        self.work = Work::Sweep;
        self.entry = Box::new([]);
        self.phase = Phase::Trash;
    }
    pub fn move_directory(&mut self, forge: u16, directory: Box<[u8]>, data: &[u8]) {
        assert!(self.ready(), "one sweep");
        self.failed = false;
        self.forge = forge;
        self.original = directory;
        self.entry = bytes::copy_of(Decimal::of(u64::from(forge)).as_bytes());
        self.work = Work::Move;
        let Some(path) = beneath(&self.original, data) else {
            self.outside();
            return;
        };
        self.spelling = path.spelling;
        let mut split = 0;
        for (index, byte) in path.normal.iter().enumerate() {
            if *byte == b'/' {
                split = index.checked_add(1).expect("path index");
            }
        }
        self.parent = if split == 0 {
            bytes::copy_of(b".")
        } else {
            bytes::copy_of(path.normal.get(..split.checked_sub(1).expect("path separator")).expect("path parent"))
        };
        self.name = bytes::copy_of(path.normal.get(split..).expect("path name"));
        self.phase = Phase::Verify;
    }
    fn outside(&mut self) {
        self.notice = Some(joined(&[
            b"box: forge ",
            Decimal::of(u64::from(self.forge)).as_bytes(),
            b": directory outside data: ",
            &self.original,
            b"\n",
        ]));
        self.phase = Phase::Close;
    }
    fn fail(&mut self, error: Error) {
        self.notice = Some(joined(&[b"box: sweep box/trash/", &self.entry, b": ", &error_name(error), b"\n"]));
        self.failed = true;
        self.phase = Phase::Close;
    }
    fn close(&mut self) -> Option<Op> {
        if let Some(fd) = self.child.take() {
            return Some(Op::Close { fd });
        }
        if let Some(fd) = self.trash.take() {
            return Some(Op::Close { fd });
        }
        if let Some(fd) = self.box_dir.take() {
            return Some(Op::Close { fd });
        }
        None
    }
    pub fn tick(&mut self, root: Fd, serial: &mut u64, out: &mut Queue<Submit>) {
        if self.waiting || self.notice.is_some() || self.answer.is_some() || out.room() == 0 {
            return;
        }
        let kind = match self.phase {
            Phase::Idle => return,
            Phase::Verify => Op::Open { root, path: self.spelling.clone(), how: OpenHow::DirectoryNoFollow },
            Phase::VerifyClose => Op::Close { fd: self.child.take().expect("verified directory") },
            Phase::MakeBox => Op::MakeDirectory { dir: root, name: bytes::copy_of(b"box"), mode: 0o700 },
            Phase::Box => Op::Open { root, path: bytes::copy_of(b"box"), how: OpenHow::DirectoryNoFollow },
            Phase::MakeTrash => {
                Op::MakeDirectory { dir: self.box_dir.expect("box"), name: bytes::copy_of(b"trash"), mode: 0o700 }
            }
            Phase::BoxClose => Op::Close { fd: self.box_dir.take().expect("box") },
            Phase::Trash => Op::Open { root, path: bytes::copy_of(b"box/trash"), how: OpenHow::DirectoryNoFollow },
            Phase::Parent => Op::Open { root, path: self.parent.clone(), how: OpenHow::DirectoryNoFollow },
            Phase::Move => Op::Rename {
                from_dir: self.child.expect("source parent"),
                from: self.name.clone(),
                to_dir: self.trash.expect("trash"),
                to: bytes::copy_of(Decimal::of(u64::from(self.forge)).as_bytes()),
                no_replace: true,
            },
            Phase::List | Phase::ListChild => Op::List {
                fd: if self.phase == Phase::List { self.trash.expect("trash") } else { self.child.expect("child") },
                entries: Box::new([Entry::BLANK]),
                names: bytes::zeroed(LONGEST_NAME),
            },
            Phase::Remove | Phase::Directory => Op::Remove {
                dir: self.trash.expect("trash"),
                name: self.entry.clone(),
                directory: self.phase == Phase::Directory,
            },
            Phase::Child => {
                Op::Open { root: self.trash.expect("trash"), path: self.entry.clone(), how: OpenHow::DirectoryNoFollow }
            }
            Phase::Lift => {
                let to = joined(&[
                    Decimal::of(u64::from(number(&self.entry))).as_bytes(),
                    b"-",
                    Decimal::of(*serial).as_bytes(),
                ]);
                *serial = serial.checked_add(1).expect("box serial");
                Op::Rename {
                    from_dir: self.child.expect("child"),
                    from: self.lifted.clone(),
                    to_dir: self.trash.expect("trash"),
                    to,
                    no_replace: true,
                }
            }
            Phase::Restart | Phase::Close => {
                if let Some(op) = self.close() {
                    op
                } else {
                    if self.phase == Phase::Restart {
                        self.phase = Phase::Trash;
                    } else {
                        self.phase = Phase::Idle;
                        self.answer = Some(!self.failed);
                    }
                    return;
                }
            }
        };
        out.push(Submit { op: OP, kind });
        self.waiting = true;
    }
    pub fn complete(&mut self, complete: Complete) {
        assert!(self.waiting, "one sweep operation");
        self.waiting = false;
        match complete.result {
            Err(Error::Escape | Error::TooManyLinks) if [Phase::Verify, Phase::Parent].contains(&self.phase) => {
                self.outside();
            }
            Err(Error::NotFound) if [Phase::Verify, Phase::Parent, Phase::Move].contains(&self.phase) => {
                self.phase = Phase::Close;
            }
            Err(Error::NotFound) if self.phase == Phase::Trash => self.phase = Phase::Close,
            Err(Error::Exists) if self.phase == Phase::MakeBox => self.phase = Phase::Box,
            Err(Error::Exists) if self.phase == Phase::MakeTrash => self.phase = Phase::BoxClose,
            Err(Error::Exists) if self.phase == Phase::Move => {
                self.work = Work::Retry;
                self.phase = Phase::Restart;
            }
            Err(Error::Exists) if self.phase == Phase::Lift => {}
            Err(Error::IsADirectory) if self.phase == Phase::Remove => self.phase = Phase::Directory,
            Err(Error::NotEmpty) if self.phase == Phase::Directory => self.phase = Phase::Child,
            Err(Error::NotFound)
                if [Phase::Remove, Phase::Directory, Phase::Child, Phase::Lift].contains(&self.phase) =>
            {
                self.phase = Phase::Restart;
            }
            Err(error) => self.fail(error),
            Ok(done) => match self.phase {
                Phase::Verify => {
                    self.child = Some(fd(done));
                    self.phase = Phase::VerifyClose;
                }
                Phase::VerifyClose => self.phase = Phase::MakeBox,
                Phase::MakeBox => self.phase = Phase::Box,
                Phase::Box => {
                    self.box_dir = Some(fd(done));
                    self.phase = Phase::MakeTrash;
                }
                Phase::MakeTrash => self.phase = Phase::BoxClose,
                Phase::BoxClose => self.phase = Phase::Trash,
                Phase::Trash => {
                    self.trash = Some(fd(done));
                    self.phase = if self.work == Work::Move { Phase::Parent } else { Phase::List };
                }
                Phase::Parent => {
                    self.child = Some(fd(done));
                    self.phase = Phase::Move;
                }
                Phase::Move => self.phase = Phase::Close,
                Phase::List | Phase::ListChild => {
                    let Done::Count(count) = done else { unreachable!("sweep list count") };
                    if count == 0 {
                        if self.phase == Phase::List && self.work == Work::Retry {
                            self.work = Work::Move;
                            self.phase = Phase::Restart;
                        } else if self.phase == Phase::List {
                            self.phase = Phase::Close;
                        } else {
                            self.phase = Phase::Restart;
                        }
                    } else {
                        let Op::List { entries, names, .. } = complete.kind else { unreachable!("sweep list") };
                        let name =
                            bytes::copy_of(entries.first().expect("one entry").name(&names).expect("kernel entry"));
                        if self.phase == Phase::List {
                            self.entry = name;
                            self.phase = Phase::Remove;
                        } else {
                            self.lifted = name;
                            self.phase = Phase::Lift;
                        }
                    }
                }
                Phase::Remove | Phase::Directory | Phase::Lift => self.phase = Phase::Restart,
                Phase::Child => {
                    self.child = Some(fd(done));
                    self.phase = Phase::ListChild;
                }
                Phase::Restart | Phase::Close => {}
                Phase::Idle => unreachable!("idle sweep completion"),
            },
        }
    }
}
fn fd(done: Done) -> Fd {
    let Done::Fd(fd) = done else { unreachable!("sweep open") };
    fd
}
fn number(entry: &[u8]) -> u16 {
    let mut number = 0_u16;
    for byte in entry {
        if *byte == b'-' {
            break;
        }
        if !byte.is_ascii_digit() {
            return 0;
        }
        let Some(next) = number.checked_mul(10) else {
            return 0;
        };
        let Some(next) = next.checked_add(u16::from(byte.checked_sub(b'0').expect("digit"))) else {
            return 0;
        };
        number = next;
    }
    number
}
#[derive(Debug)]
struct Path {
    normal: Box<[u8]>,
    spelling: Box<[u8]>,
}
fn beneath(directory: &[u8], data: &[u8]) -> Option<Path> {
    let relative = if directory.starts_with(b"/") {
        let mut end = data.len();
        for byte in data.iter().rev() {
            if end <= 1 || *byte != b'/' {
                break;
            }
            end = end.checked_sub(1)?;
        }
        let data = data.get(..end)?;
        if data == b"/" { directory.get(1..)? } else { directory.strip_prefix(data)?.strip_prefix(b"/")? }
    } else {
        directory
    };
    let mut path = List::with_capacity(u32::try_from(relative.len()).ok()?);
    let mut start = 0;
    for at in 0..=relative.len() {
        if at != relative.len() && relative.get(at) != Some(&b'/') {
            continue;
        }
        let part = relative.get(start..at)?;
        start = at.checked_add(1)?;
        if part == b".." {
            if path.is_empty() {
                return None;
            }
            let mut end = 0;
            for (index, byte) in path.as_slice().iter().enumerate() {
                if *byte == b'/' {
                    end = index;
                }
            }
            path.truncate(u32::try_from(end).ok()?);
        } else if !part.is_empty() && part != b"." {
            if !path.is_empty() {
                path.push(b'/').ok()?;
            }
            for byte in part {
                if *byte == 0 {
                    return None;
                }
                path.push(*byte).ok()?;
            }
        }
    }
    if path.is_empty() || path.as_slice() == b"box" || path.as_slice().starts_with(b"box/") {
        return None;
    }
    Some(Path { normal: path.into_boxed(), spelling: bytes::copy_of(relative) })
}
