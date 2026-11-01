//! Group files beneath the data root
//! (temper/docs/plan/playground-reset-r4/notes/contracts.md, 3).
use crate::{lines::joined, logs::error_name, machine::Input};
use alloc::boxed::Box;
use jig_protocol_config::{DocumentLimits, document, read};
use skein_io::kernel::{Complete, Done, Error, Fd, Op, OpenHow, Submit};
use skein_lib::{Decimal, List, Queue, Token, bytes};
const OP: Token = Token::new(u64::MAX - 10);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Idle,
    Open,
    Read,
    ReadClose,
    Box,
    Directory,
    Remove,
    DirectoryClose,
    BoxClose,
}
#[derive(Debug)]
pub(crate) struct Groups {
    phase: Phase,
    forge: u16,
    bound: u32,
    bytes: List<u8>,
    box_dir: Option<Fd>,
    dir: Option<Fd>,
    part: u8,
    waiting: bool,
    notice: Option<Box<[u8]>>,
    answer: Option<Input>,
    result: Option<Input>,
    directory: Option<Box<[u8]>>,
}
impl Groups {
    pub fn new(bound: u32) -> Self {
        Self {
            phase: Phase::Idle,
            forge: 0,
            bound,
            bytes: List::with_capacity(bound),
            box_dir: None,
            dir: None,
            part: 0,
            waiting: false,
            notice: None,
            answer: None,
            result: None,
            directory: None,
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
    pub fn take_answer(&mut self) -> Option<Input> {
        self.answer.take()
    }
    pub fn read(&mut self, forge: u16) {
        assert!(self.ready(), "one group file operation");
        self.forge = forge;
        self.bytes.clear();
        self.directory = None;
        self.phase = Phase::Open;
    }
    pub fn take_directory(&mut self) -> Option<Box<[u8]>> {
        self.directory.take()
    }
    pub fn remove(&mut self, forge: u16) {
        assert!(self.ready(), "one group file operation");
        self.forge = forge;
        self.part = 0;
        self.result = Some(Input::Removed { forge });
        self.phase = Phase::Box;
    }
    fn directory(&self) -> &'static [u8] {
        if self.part == 0 { b"started" } else { b"groups" }
    }
    fn path(&self) -> Box<[u8]> {
        joined(&[b"box/", self.directory(), b"/", Decimal::of(u64::from(self.forge)).as_bytes()])
    }
    pub fn tick(&mut self, root: Fd, out: &mut Queue<Submit>) {
        if self.waiting || self.notice.is_some() || self.answer.is_some() || out.room() == 0 {
            return;
        }
        let kind = match self.phase {
            Phase::Idle => return,
            Phase::Open => Op::Open {
                root,
                path: joined(&[b"box/groups/", Decimal::of(u64::from(self.forge)).as_bytes()]),
                how: OpenHow::ReadNoFollow,
            },
            Phase::Read => Op::Read {
                fd: self.dir.expect("group file"),
                buf: bytes::zeroed(4096),
                at: u64::from(self.bytes.len()),
            },
            Phase::ReadClose => Op::Close { fd: self.dir.take().expect("group file") },
            Phase::Box => Op::Open { root, path: bytes::copy_of(b"box"), how: OpenHow::DirectoryNoFollow },
            Phase::Directory => Op::Open {
                root: self.box_dir.expect("box directory"),
                path: bytes::copy_of(self.directory()),
                how: OpenHow::DirectoryNoFollow,
            },
            Phase::Remove => Op::Remove {
                dir: self.dir.expect("group directory"),
                name: bytes::copy_of(Decimal::of(u64::from(self.forge)).as_bytes()),
                directory: false,
            },
            Phase::DirectoryClose => Op::Close { fd: self.dir.take().expect("group directory") },
            Phase::BoxClose => Op::Close { fd: self.box_dir.take().expect("box directory") },
        };
        out.push(Submit { op: OP, kind });
        self.waiting = true;
    }
    fn finish(&mut self) {
        self.phase = Phase::Idle;
        self.answer = self.result.take();
    }
    fn read_failed(&mut self, reason: &[u8], too_long: bool) {
        self.notice =
            Some(joined(&[b"box: box/groups/", Decimal::of(u64::from(self.forge)).as_bytes(), b": ", reason, b"\n"]));
        self.result = Some(Input::NotRead { forge: self.forge, too_long });
        if self.dir.is_some() {
            self.phase = Phase::ReadClose;
        } else {
            self.finish();
        }
    }
    fn remove_failed(&mut self, error: Error) {
        self.notice = Some(joined(&[b"box: ", &self.path(), b": ", &error_name(error), b"\n"]));
        self.result = Some(Input::NotRemoved { forge: self.forge });
        if self.dir.is_some() {
            self.phase = Phase::DirectoryClose;
        } else if self.box_dir.is_some() {
            self.phase = Phase::BoxClose;
        } else {
            self.finish();
        }
    }
    fn next_part(&mut self) {
        if self.part == 0 && self.result == Some(Input::Removed { forge: self.forge }) {
            self.part = 1;
            self.phase = Phase::Directory;
        } else {
            self.phase = Phase::BoxClose;
        }
    }
    pub fn complete(&mut self, complete: Complete) {
        assert!(self.waiting, "one group file operation");
        self.waiting = false;
        match self.phase {
            Phase::Open => match complete.result {
                Ok(Done::Fd(fd)) => {
                    self.dir = Some(fd);
                    self.phase = Phase::Read;
                }
                Err(Error::NotFound) => {
                    self.result = Some(Input::Read { forge: self.forge, playground: None });
                    self.finish();
                }
                Err(error) => self.read_failed(&error_name(error), false),
                Ok(_) => unreachable!("group open"),
            },
            Phase::Read => match complete.result {
                Ok(Done::Count(count)) => {
                    let Op::Read { buf, .. } = complete.kind else { unreachable!("group read buffer") };
                    let read = buf.get(..usize::try_from(count).expect("read count")).expect("kernel count");
                    for byte in read {
                        if self.bytes.push(*byte).is_err() {
                            self.read_failed(b"group file too long", true);
                            return;
                        }
                    }
                    if count == 0 {
                        match members(self.bytes.as_slice(), self.bound) {
                            Some(members) => {
                                self.directory = members.directory;
                                self.result =
                                    Some(Input::Read { forge: self.forge, playground: Some(members.playground) });
                                self.phase = Phase::ReadClose;
                            }
                            None => self.read_failed(b"malformed group file", false),
                        }
                    }
                }
                Err(error) => self.read_failed(&error_name(error), false),
                Ok(_) => unreachable!("group read"),
            },
            Phase::ReadClose => {
                if let Err(error) = complete.result {
                    self.read_failed(&error_name(error), false);
                } else {
                    self.finish();
                }
            }
            Phase::Box => match complete.result {
                Ok(Done::Fd(fd)) => {
                    self.box_dir = Some(fd);
                    self.phase = Phase::Directory;
                }
                Err(Error::NotFound) => self.finish(),
                Err(error) => self.remove_failed(error),
                Ok(_) => unreachable!("box open"),
            },
            Phase::Directory => match complete.result {
                Ok(Done::Fd(fd)) => {
                    self.dir = Some(fd);
                    self.phase = Phase::Remove;
                }
                Err(Error::NotFound) => self.next_part(),
                Err(error) => self.remove_failed(error),
                Ok(_) => unreachable!("directory open"),
            },
            Phase::Remove => match complete.result {
                Ok(Done::Nothing) | Err(Error::NotFound) => self.phase = Phase::DirectoryClose,
                Err(error) => self.remove_failed(error),
                Ok(_) => unreachable!("group remove"),
            },
            Phase::DirectoryClose => {
                if let Err(error) = complete.result {
                    self.remove_failed(error);
                } else {
                    self.next_part();
                }
            }
            Phase::BoxClose => {
                if let Err(error) = complete.result {
                    self.remove_failed(error);
                } else {
                    self.finish();
                }
            }
            Phase::Idle => unreachable!("idle group completion"),
        }
    }
}
#[derive(Debug)]
struct Members {
    playground: bool,
    directory: Option<Box<[u8]>>,
}
fn members(input: &[u8], bound: u32) -> Option<Members> {
    let document = document(input, &DocumentLimits { bytes: bound, tokens: bound, depth: 128 }).ok()?;
    let mut object = read::Object::new(&document, 0).ok()?;
    let at = object.member(&document, b"playground")?;
    // The rest is the steps' own. Ambiguous members cannot choose a directory.
    let mut occurrences = 0_u32;
    let mut setups = 0_u32;
    let mut directories = 0_u32;
    let setup = object.member(&document, b"setup");
    let mut setup_start = document.len();
    let mut setup_end = 0;
    let mut directory = None;
    if let Some(at) = setup {
        let mut object = read::Object::new(&document, at).ok()?;
        setup_start = at;
        setup_end = read::after(&document, at).ok()?;
        if let Some(at) = object.member(&document, b"directory") {
            directory = Some(read::text(&document, at, bound).ok()?);
        }
    }
    let mut depth = 0_u32;
    for index in 0..document.len() {
        let token = document.token(index)?;
        match token.kind {
            skein_json::Kind::ObjectStart | skein_json::Kind::ArrayStart => depth = depth.checked_add(1)?,
            skein_json::Kind::ObjectEnd | skein_json::Kind::ArrayEnd => depth = depth.checked_sub(1)?,
            skein_json::Kind::Key if depth == 1 && document.text(token)? == b"playground" => {
                occurrences = occurrences.checked_add(1)?;
            }
            skein_json::Kind::Key if depth == 1 && document.text(token)? == b"setup" => {
                setups = setups.checked_add(1)?;
            }
            skein_json::Kind::Key
                if depth == 2 && document.text(token)? == b"directory" && index > setup_start && index < setup_end =>
            {
                directories = directories.checked_add(1)?;
            }
            skein_json::Kind::Key
            | skein_json::Kind::String
            | skein_json::Kind::Number
            | skein_json::Kind::True
            | skein_json::Kind::False
            | skein_json::Kind::Null
            | skein_json::Kind::Long => {}
        }
    }
    if occurrences != 1 || setups > 1 || directories > 1 {
        return None;
    }
    Some(Members { playground: read::boolean(&document, at).ok()?, directory })
}
