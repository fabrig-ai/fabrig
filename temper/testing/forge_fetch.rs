//! A cache and an HTTP reply supplied to the pinned fetch driver.
use super::{
    Failed,
    model::{Event, Request, url},
    zstd,
};
use skein_lib::sha256::Sha256;
use std::{cell::RefCell, path::Path};

pub const ROOTS: &str = "/fixture/root.pem";
pub const CACHE: &[u8] = b"/fixture/forgejo/17.0.1/forgejo";
const PACKED: &[u8] = b"/fixture/forgejo/17.0.1/forgejo.zst.part";
const PART: &[u8] = b"/fixture/forgejo/17.0.1/forgejo.part";
pub const RELEASE: &[u8] = b"/temper/forgejo/17.0.1/forgejo-17.0.1-linux-amd64.zst";
const URL: &str = "https://mirror.example.invalid/temper/forgejo/17.0.1/forgejo-17.0.1-linux-amd64.zst";
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[derive(Clone, Copy, Debug)]
pub enum Row {
    Digest,
    Present,
    MissingRoots,
}

impl Row {
    pub fn stdout(self) -> Vec<u8> {
        match self {
            Self::Digest => {
                let bytes = zstd(b"abc").len();
                format!("fetched forgejo 0 of {bytes} bytes\nfetched forgejo {bytes} of {bytes} bytes\n").into_bytes()
            }
            Self::Present | Self::MissingRoots => Vec::new(),
        }
    }

    pub fn diagnostic(self, pinned: &str) -> String {
        match self {
            Self::Digest => format!("{URL}: digest differs: pinned {pinned}, got {ABC_SHA256}"),
            Self::Present => format!("{}: does not match the pinned digest", std::str::from_utf8(CACHE).expect("path")),
            Self::MissingRoots => format!("{ROOTS}: roots: Absent"),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Effect {
    Roots,
    Hash,
    Get,
    Open,
    Read,
    Write,
    Unpack,
    RemovePart,
    RemovePacked,
}

pub struct Script {
    row: Option<Row>,
    packed: Vec<u8>,
    state: RefCell<State>,
}

#[derive(Default)]
struct State {
    effects: Vec<Effect>,
    final_file: Option<Vec<u8>>,
    packed: Option<Vec<u8>>,
    part: Option<Vec<u8>>,
}

impl Script {
    pub fn new(row: Option<Row>) -> Self {
        let final_file = matches!(row, Some(Row::Present)).then(|| b"mine".to_vec());
        Self { row, packed: zstd(b"abc"), state: RefCell::new(State { final_file, ..State::default() }) }
    }

    pub fn roots(&self, path: &Path) -> Result<(), Failed> {
        assert_eq!(path, Path::new(ROOTS));
        self.state.borrow_mut().effects.push(Effect::Roots);
        if matches!(self.row, Some(Row::MissingRoots)) {
            return Err(Failed::Roots(skein_shell::RootsRefusal::Unread {
                path: path.into(),
                why: skein_shell::Unread::Absent,
            }));
        }
        Ok(())
    }

    pub fn answer(&self, request: Request) -> Event {
        let state = &mut *self.state.borrow_mut();
        match request {
            Request::Hash { file } => {
                assert_eq!(&*file, CACHE);
                state.effects.push(Effect::Hash);
                Event::Hashed(state.final_file.as_ref().map(|bytes| {
                    let mut hash = Sha256::new();
                    hash.update(bytes);
                    hash.finish()
                }))
            }
            Request::Get { origin, path } => {
                assert_eq!(&*path, RELEASE);
                assert_eq!(&*url(&origin, &path), URL.as_bytes());
                state.effects.push(Effect::Get);
                Event::Answered { status: 200, length: Some(u64::try_from(self.packed.len()).unwrap()), location: None }
            }
            Request::Open { part } => {
                assert_eq!(&*part, PACKED);
                state.effects.push(Effect::Open);
                assert_eq!(state.packed.replace(Vec::new()), None);
                Event::Done
            }
            Request::Read => {
                state.effects.push(Effect::Read);
                if state.packed.as_ref().expect("opened packed file").is_empty() {
                    Event::Chunk(self.packed.as_slice().into())
                } else {
                    Event::Ended
                }
            }
            Request::Write { offset, bytes } => {
                assert_eq!((offset, &*bytes), (0, self.packed.as_slice()));
                state.effects.push(Effect::Write);
                state.packed.as_mut().expect("opened packed file").extend_from_slice(&bytes);
                Event::Done
            }
            Request::Unpack { packed, part, bound } => {
                assert_eq!((&*packed, &*part), (PACKED, PART));
                assert!(bound >= u64::try_from(self.packed.len()).unwrap());
                assert_eq!(state.packed.as_deref(), Some(self.packed.as_slice()));
                state.effects.push(Effect::Unpack);
                assert_eq!(state.part.replace(b"abc".to_vec()), None);
                let mut hash = Sha256::new();
                hash.update(b"abc");
                Event::Unpacked(super::model::Unpacked::File { digest: hash.finish(), bytes: 3 })
            }
            Request::Remove { part } => {
                if &*part == PART {
                    state.effects.push(Effect::RemovePart);
                    assert_eq!(state.part.take().as_deref(), Some(b"abc".as_slice()));
                } else {
                    assert_eq!(&*part, PACKED);
                    state.effects.push(Effect::RemovePacked);
                    assert_eq!(state.packed.take().as_deref(), Some(self.packed.as_slice()));
                }
                Event::Done
            }
            Request::Finish { .. } | Request::Say(_) | Request::End(_) => {
                panic!("unexpected fetch request: {request:?}")
            }
        }
    }

    pub fn check(self) {
        use Effect::{Get, Hash, Open, Read, RemovePacked, RemovePart, Roots, Unpack, Write};
        let state = self.state.into_inner();
        let expected: &[Effect] = match self.row {
            None => &[],
            Some(Row::Digest) => &[Roots, Hash, Get, Open, Read, Write, Read, Unpack, RemovePart, RemovePacked],
            Some(Row::Present) => &[Roots, Hash],
            Some(Row::MissingRoots) => &[Roots],
        };
        assert_eq!(state.effects, expected, "{:?}", self.row);
        assert_eq!(state.final_file.as_deref(), matches!(self.row, Some(Row::Present)).then_some(b"mine".as_slice()));
        assert_eq!(state.part, None);
        assert_eq!(state.packed, None);
    }
}
