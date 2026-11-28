use alloc::collections::BTreeMap;
use skein_forge::api::{Answer, Commit, Entry, EntryKind, Error, Read};
use temper_protocol_temper_dir::Reading;

pub const HEAD: Commit = [0x53; 32];
pub const ROOT: &str = ".temper";
pub const OFFERS: &str = ".temper/ways.toml";
pub const WAYS: &str = ".temper/ways";
pub const CHARTERS: &str = ".temper/charters";
pub const HOOKS: &str = ".temper/hooks";
pub const CONFIG: &str = ".temper/hooks/config.toml";
pub const PAIR: &str = ".temper/ways/pair.toml";
// temper/docs/design/protocol/temper-dir.md, sections 3 and 8.
pub const FILE_BYTES: u32 = 65_536;
pub const HOOK_BYTES: u32 = 4_096;
pub const MAX_FILES: usize = 64;
pub const MAX_TOTAL: usize = 524_288;
pub const MAX_FAULTS: usize = 16;

#[derive(Default)]
pub struct Tree {
    nodes: BTreeMap<String, Node>,
}

enum Node {
    Directory { entries: Vec<Entry>, cut: bool },
    File { bytes: Box<[u8]>, cut: bool },
}

impl Tree {
    pub fn entry(&mut self, path: &str, kind: EntryKind) {
        let (parent, name) = path.rsplit_once('/').expect("path below .temper");
        match self.nodes.entry(parent.to_owned()).or_insert_with(empty_listing) {
            Node::Directory { entries, .. } => entries.push(Entry { name: name.as_bytes().into(), kind }),
            Node::File { .. } => panic!("{parent} is not a listing"),
        }
    }

    pub fn dir(&mut self, path: &str) {
        if path != ROOT {
            self.entry(path, EntryKind::Directory);
        }
        self.nodes.insert(path.to_owned(), empty_listing());
    }

    pub fn file(&mut self, path: &str, bytes: &[u8]) {
        self.entry(path, EntryKind::File);
        self.nodes.insert(path.to_owned(), Node::File { bytes: bytes.into(), cut: false });
    }

    pub fn cut(&mut self, path: &str) {
        match self.nodes.get_mut(path).expect("a tree can cut one of its answers") {
            Node::Directory { cut, .. } | Node::File { cut, .. } => *cut = true,
        }
    }

    pub fn answer(&self, head: Commit, path: &str, max_bytes: u32) -> Result<Answer, Error> {
        match self.nodes.get(path) {
            Some(Node::Directory { entries, cut }) => Ok(directory(head, path, entries, *cut)),
            Some(Node::File { bytes, cut }) => {
                let len = bytes.len().min(usize::try_from(max_bytes).expect("the fixture file limit fits usize"));
                Ok(file(head, path, &bytes[..len], *cut || len < bytes.len()))
            }
            None => Err(Error::Missing),
        }
    }

    pub fn read(&self, answer_bytes: u32) -> (Reading, Vec<String>) {
        let mut reading = Reading::new(HEAD, answer_bytes);
        let mut paths = Vec::new();
        while let Some(request) = reading.next() {
            let (head, path, bytes) = request_file(request);
            assert!(!paths.contains(&path), "a path was read twice: {path}");
            reading.take(&self.answer(head, &path, bytes)).expect("the tree answers the request");
            paths.push(path);
        }
        (reading, paths)
    }
}

fn empty_listing() -> Node {
    Node::Directory { entries: Vec::new(), cut: false }
}

pub fn file(head: Commit, path: &str, bytes: &[u8], truncated: bool) -> Answer {
    Answer::File { head, path: path.as_bytes().into(), bytes: bytes.into(), truncated }
}

pub fn directory(head: Commit, path: &str, entries: &[Entry], truncated: bool) -> Answer {
    Answer::Directory { head, path: path.as_bytes().into(), entries: entries.into(), truncated }
}

// Every other forge operation is unexpected at this boundary.
pub fn request_file(request: Read) -> (Commit, String, u32) {
    let Read::File { head, path, max_bytes } = request else { panic!("expected File, got {request:?}") };
    (head, String::from_utf8(path.into_vec()).expect("the fixture paths are UTF-8"), max_bytes)
}
