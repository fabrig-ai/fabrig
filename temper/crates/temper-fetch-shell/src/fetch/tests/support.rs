//! The pinned fetch driver over skein's disk and a scripted plaintext peer.
use super::super::cache::Cache;
use super::super::{Event, Failed, HTTP, WireRequest, body_demand, http, http_answer, request_head};
use skein_fake_machine::{How, Item, Machine, Opened, Refusal};
use skein_lib::{
    Env, Intake, Queue, Time, Wall,
    stream::{Down, Read, Up},
};
use std::{collections::VecDeque, io, path::Path};

pub(super) const FILE: &[u8] = b"/cache/binary";
pub(super) const PART: &[u8] = b"/cache/binary.part";
pub(super) const PACKED: &[u8] = b"/cache/binary.zst.part";
const PARENT: &str = "/cache";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Name {
    Binary,
    Part,
    Packed,
}

impl Name {
    fn at(path: &Path) -> Self {
        match path.as_os_str().as_encoded_bytes() {
            FILE => Self::Binary,
            PART => Self::Part,
            PACKED => Self::Packed,
            other => panic!("unexpected cache path: {other:?}"),
        }
    }
    fn basename(self) -> &'static [u8] {
        match self {
            Self::Binary => b"binary",
            Self::Part => b"binary.part",
            Self::Packed => b"binary.zst.part",
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum Effect {
    Hash,
    Parent(u32),
    Open(Name, bool, u32),
    Mode(Name, u32),
    Write(Name, u64, usize),
    ReadPacked,
    Sync(Name),
    Rename(Name, Name),
    SyncParent,
    Remove(Name),
}

pub(super) struct CacheScript {
    disk: Machine,
    root: Opened,
    pub effects: Vec<Effect>,
}

impl CacheScript {
    pub fn new(initial: Option<&[u8]>, stale: Option<&[u8]>, stale_packed: Option<&[u8]>) -> Self {
        let mut disk = Machine::new();
        let mut items = Vec::new();
        if initial.is_some() || stale.is_some() || stale_packed.is_some() {
            items.push(Item::directory(b"cache"));
        }
        for (name, bytes) in [(FILE, initial), (PART, stale), (PACKED, stale_packed)] {
            if let Some(bytes) = bytes {
                items.push(Item::file(&name[1..], bytes));
            }
        }
        let root = disk.lay(&items);
        Self { disk, root, effects: Vec::new() }
    }

    pub fn bytes(&mut self, name: Name) -> Option<Vec<u8>> {
        self.read(name).expect("read the fixture's bytes")
    }
    pub fn settled(&self) {
        assert_eq!(self.disk.open_handles(), 1, "only the root is still open");
    }
    fn read(&mut self, name: Name) -> io::Result<Option<Vec<u8>>> {
        let Some(directory) = self.directory()? else { return Ok(None) };
        let opened = self.disk.open(directory, name.basename(), How::Read);
        self.disk.close(directory);
        let file = match opened {
            Ok(file) => file,
            Err(Refusal::NotFound) => return Ok(None),
            Err(why) => return Err(unexpected(why)),
        };
        let size = u32::try_from(self.disk.stat(file).size).expect("fixture size");
        let result = self.disk.read(file, 0, size);
        self.disk.close(file);
        result.map(Some).map_err(unexpected)
    }
    fn directory(&mut self) -> io::Result<Option<Opened>> {
        match self.disk.open(self.root, b"cache", How::Directory) {
            Ok(directory) => Ok(Some(directory)),
            Err(Refusal::NotFound) => Ok(None),
            Err(why) => Err(unexpected(why)),
        }
    }
    fn in_cache<T>(&mut self, effect: impl FnOnce(&mut Machine, Opened) -> Result<T, Refusal>) -> io::Result<T> {
        let directory = self.directory()?.ok_or(io::ErrorKind::NotFound)?;
        let result = effect(&mut self.disk, directory);
        self.disk.close(directory);
        result.map_err(unexpected)
    }
}

fn unexpected(why: Refusal) -> io::Error {
    io::Error::other(format!("fixture cache: {why:?}"))
}

impl Cache for CacheScript {
    type Input = io::Cursor<Vec<u8>>;
    type Part = (Name, Opened);

    fn open_input(&mut self, file: &Path) -> io::Result<Self::Input> {
        let name = Name::at(file);
        self.effects.push(match name {
            Name::Binary => Effect::Hash,
            Name::Packed => Effect::ReadPacked,
            Name::Part => panic!("an unpacked staging file is not read"),
        });
        Ok(io::Cursor::new(self.read(name)?.ok_or(io::ErrorKind::NotFound)?))
    }
    fn make_parent(&mut self, parent: &Path, mode: u32) -> io::Result<()> {
        assert_eq!(parent, Path::new(PARENT));
        self.effects.push(Effect::Parent(mode));
        match self.disk.make_directory(self.root, b"cache", mode) {
            Ok(()) | Err(Refusal::Exists) => Ok(()),
            Err(why) => Err(unexpected(why)),
        }
    }
    fn open_part(&mut self, part: &Path, truncate: bool, mode: u32) -> io::Result<Self::Part> {
        let name = Name::at(part);
        self.effects.push(Effect::Open(name, truncate, mode));
        self.in_cache(|disk, dir| {
            let file = match disk.open(dir, name.basename(), How::Create { mode }) {
                Ok(file) => file,
                Err(Refusal::Exists) => disk.open(dir, name.basename(), How::Write)?,
                Err(why) => return Err(why),
            };
            if truncate && let Err(why) = disk.truncate(file, 0) {
                disk.close(file);
                return Err(why);
            }
            Ok((name, file))
        })
    }
    fn set_mode(&mut self, &(name, _): &Self::Part, mode: u32) -> io::Result<()> {
        // The fake machine has no chmod; keep the requested mode in the trace.
        self.effects.push(Effect::Mode(name, mode));
        Ok(())
    }
    fn write_at(&mut self, (name, part): &mut Self::Part, offset: u64, bytes: &[u8]) -> io::Result<()> {
        self.effects.push(Effect::Write(*name, offset, bytes.len()));
        self.disk.write(*part, offset, bytes).map_err(unexpected)
    }
    fn sync(&mut self, &(name, part): &Self::Part) -> io::Result<()> {
        self.effects.push(Effect::Sync(name));
        self.disk.sync(part);
        Ok(())
    }
    fn rename(&mut self, part: &Path, file: &Path) -> io::Result<()> {
        let (from, to) = (Name::at(part), Name::at(file));
        self.effects.push(Effect::Rename(from, to));
        self.in_cache(|disk, dir| disk.rename(dir, from.basename(), dir, to.basename(), false))
    }
    fn sync_parent(&mut self, parent: &Path) -> io::Result<()> {
        assert_eq!(parent, Path::new(PARENT));
        self.effects.push(Effect::SyncParent);
        self.in_cache(|disk, dir| {
            disk.sync(dir);
            Ok(())
        })
    }
    fn remove(&mut self, part: &Path) -> io::Result<()> {
        let name = Name::at(part);
        self.effects.push(Effect::Remove(name));
        self.in_cache(|disk, dir| disk.remove(dir, name.basename(), false))
    }
    fn close(&mut self, (_, part): Self::Part) {
        self.disk.close(part);
    }
}

pub(super) struct Wire<'a> {
    responses: VecDeque<&'a [u8]>,
    active: Option<Plain>,
    pub heads: Vec<Vec<u8>>,
}

impl<'a> Wire<'a> {
    pub fn new(responses: &[&'a [u8]]) -> Self {
        Self { responses: responses.iter().copied().collect(), active: None, heads: Vec::new() }
    }

    pub fn answer(&mut self, request: WireRequest) -> Result<Event, Failed> {
        let plaintext = match request {
            WireRequest::Get { origin, path } => {
                let response = self.responses.pop_front().expect("no unscripted Get");
                self.active.insert(Plain::new(&origin, &path, response))
            }
            WireRequest::Read => {
                let plaintext = self.active.as_mut().expect("Read follows Get");
                plaintext.down(body_demand());
                plaintext
            }
        };
        let event = plaintext.answer();
        if !plaintext.sent.is_empty() {
            self.heads.push(std::mem::take(&mut plaintext.sent));
        }
        event.map_err(|why| Failed::Transport { url: plaintext.url.clone(), why })
    }

    pub fn check_heads(&self, expected: &[&[u8]]) {
        assert!(self.responses.is_empty(), "each scripted hop gets a request");
        assert_eq!(self.heads, expected, "request heads in wire order");
    }
}

const ENV: Env<http::Limits> = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: HTTP };

struct Plain {
    client: http::Client,
    up: Queue<http::Event>,
    down: Queue<Down>,
    input: Intake,
    sent: Vec<u8>,
    url: Box<[u8]>,
}

impl Plain {
    fn new(origin: &temper_fetch::Origin, path: &[u8], response: &[u8]) -> Self {
        let cap = u32::try_from(response.len()).expect("small fixture").max(http::largest_read(&HTTP));
        let mut input = Intake::with_capacity(cap);
        input.append(response).expect("room for the fixture");
        let mut plain = Self {
            client: http::Client::new(&HTTP),
            up: Queue::uncapped(),
            down: Queue::uncapped(),
            input,
            sent: Vec::new(),
            url: temper_fetch::fetch::url(origin, path),
        };
        plain.down(http::Request::Call(request_head(origin, path)));
        plain
    }
    fn down(&mut self, request: http::Request) {
        http::down(&mut self.client, &ENV, request, &mut self.up, &mut self.down);
    }
    fn answer(&mut self) -> Result<Event, String> {
        loop {
            if let Some(answer) = self.up.pop() {
                if let Some(answer) = http_answer(answer) {
                    return answer;
                }
                continue;
            }
            match self.down.pop().expect("the plaintext exchange is waiting for its peer") {
                Down::Demand { read: Read::Nothing, room: 0 } => {}
                Down::Demand { room, .. } if room > 0 => {
                    http::up(&mut self.client, &ENV, Up::Room, &mut self.up, &mut self.down);
                }
                Down::Demand { read, .. } => {
                    let next = self.input.meet(read).map_or(Up::End, Up::Bytes);
                    http::up(&mut self.client, &ENV, next, &mut self.up, &mut self.down);
                }
                Down::Send(bytes) => self.sent.extend_from_slice(&bytes),
                Down::Finish => unreachable!("the HTTP client does not finish its peer"),
            }
        }
    }
}
