//! Completion-located cut points in the service's submission stream.
use crate::referee::durable;
use skein_io::kernel::{Complete, Done, Fd, Op};
use std::collections::BTreeMap;
#[derive(Default, Debug)]
pub(super) struct Points {
    pub copy: Option<u32>,
    pub patch: Option<u32>,
    pub switch: Option<u32>,
    pub wires: Option<u32>,
}
impl Points {
    pub fn fixed(&self) -> [u32; 4] {
        [
            self.copy.expect("copy first write"),
            self.patch.expect("patch commit"),
            self.switch.expect("switch rename"),
            self.wires.expect("wires commit"),
        ]
    }
}
pub(super) struct Capture {
    paths: BTreeMap<i32, Vec<u8>>,
    seals: BTreeMap<i32, bool>,
}
impl Capture {
    pub fn new(root: Fd) -> Self {
        Self { paths: BTreeMap::from([(root.raw(), Vec::new())]), seals: BTreeMap::new() }
    }
    pub fn complete(&mut self, complete: &Complete, next: u32, points: &mut Points) {
        match (&complete.kind, &complete.result) {
            (Op::Open { root, path, .. }, Ok(Done::Fd(fd))) => {
                if let Some(parent) = self.paths.get(&root.raw()) {
                    let mut name = parent.clone();
                    if !name.is_empty() {
                        name.push(b'/');
                    }
                    if path.as_ref() != b"." {
                        name.extend_from_slice(path);
                    }
                    self.paths.insert(fd.raw(), name);
                }
            }
            (Op::Write { fd, bytes, from, .. }, Ok(Done::Count(count))) => {
                if let Some(path) = self.paths.get(&fd.raw())
                    && path.starts_with(b"generations/2/")
                {
                    points.copy.get_or_insert(next);
                    if path.rsplit(|b| *b == b'/').next().is_some_and(|p| p.starts_with(b"log-")) {
                        let bytes = &bytes
                            [usize::try_from(*from).expect("offset")..usize::try_from(*from + *count).expect("count")];
                        if !bytes.starts_with(b"SKVC") {
                            return;
                        }
                        let format = jig_protocol_store::keys::encode_own(&jig_protocol_store::keys::Own::Format, 64)
                            .expect("format key");
                        for (_, ops) in durable::frames(bytes, 0).0 {
                            for op in ops {
                                if let skein_kv::Op::Put { key, value } = op
                                    && key == format
                                    && jig_protocol_store::codec::component::Format::decode(&value)
                                        .expect("format")
                                        .levels
                                        .application
                                        == Some(1)
                                {
                                    self.seals.insert(fd.raw(), true);
                                }
                            }
                        }
                    }
                }
            }
            (Op::Sync { fd }, Ok(Done::Nothing)) if self.seals.remove(&fd.raw()).is_some() => {
                points.patch.get_or_insert(next);
            }
            (Op::Rename { to, .. }, Ok(Done::Nothing)) if to.as_ref() == b"active" => {
                points.switch.get_or_insert(next);
            }
            (Op::Close { fd }, _) => {
                self.paths.remove(&fd.raw());
                self.seals.remove(&fd.raw());
            }
            _ => {}
        }
    }
}
