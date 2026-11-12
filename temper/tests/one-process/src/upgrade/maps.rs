//! Independent map recovery from the fake disk's durable snapshot and log frames.
use crate::referee::durable;
use skein_fake_machine::{How, Machine, Opened};
use std::collections::BTreeMap;

type Rows = BTreeMap<Vec<u8>, Vec<u8>>;
pub(super) struct Maps {
    pub active: u64,
    pub store: Rows,
    pub secrets: Rows,
}
impl Maps {
    pub fn read(disk: &mut Machine) -> Self {
        let root = disk.reopen_root(0);
        let active = file(disk, root, b"active");
        let active = std::str::from_utf8(&active)
            .expect("active generation text")
            .strip_suffix('\n')
            .expect("active newline")
            .parse()
            .expect("active generation number");
        let store = map(disk, root, format!("generations/{active}/store").as_bytes());
        let secrets = map(disk, root, format!("generations/{active}/secrets").as_bytes());
        disk.close(root);
        Self { active, store, secrets }
    }
    pub fn format(&self) -> jig_protocol_store::codec::component::Format {
        let key = jig_protocol_store::keys::encode_own(&jig_protocol_store::keys::Own::Format, 64).expect("format key");
        jig_protocol_store::codec::component::Format::decode(self.store.get(key.as_ref()).expect("format row"))
            .expect("format decodes")
    }
}
fn file(disk: &mut Machine, root: Opened, path: &[u8]) -> Vec<u8> {
    let fd = disk.open(root, path, How::ReadNoFollow).expect("map file");
    let size = u32::try_from(disk.stat(fd).size).expect("world file fits u32");
    let bytes = disk.read(fd, 0, size).expect("whole map file");
    disk.close(fd);
    assert_eq!(bytes.len(), usize::try_from(size).expect("file fits usize"));
    bytes
}
fn apply(rows: &mut Rows, ops: Box<[skein_kv::Op]>) {
    for op in ops {
        match op {
            skein_kv::Op::Put { key, value } => {
                rows.insert(key.into_vec(), value.into_vec());
            }
            skein_kv::Op::Erase { key } => {
                rows.remove(key.as_ref());
            }
        }
    }
}
fn map(disk: &mut Machine, root: Opened, path: &[u8]) -> Rows {
    let fd = disk.open(root, path, How::DirectoryNoFollow).expect("generation map");
    let mut files = Vec::new();
    loop {
        let entries = disk.list(fd, 32, 4096).expect("map files");
        if entries.is_empty() {
            break;
        }
        files.extend(entries.into_iter().map(|(_, name)| name));
    }
    files.sort();
    let mut rows = Rows::new();
    let mut after = 0;
    if files.iter().any(|name| name.as_ref() == b"snapshot") {
        let bytes = file(disk, fd, b"snapshot");
        let (number, ops) = durable::snapshots::read(&bytes).expect("whole snapshot");
        after = number;
        apply(&mut rows, ops);
    }
    for name in files.iter().filter(|name| name.starts_with(b"log-")) {
        let bytes = file(disk, fd, name);
        let (frames, used) = durable::frames(&bytes, after);
        assert_eq!(used, bytes.len(), "whole log {name:?}");
        for (number, ops) in frames {
            assert_eq!(number, after + 1, "consecutive log commits");
            apply(&mut rows, ops);
            after = number;
        }
    }
    disk.close(fd);
    rows
}
