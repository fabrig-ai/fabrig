//! Terminal task keys in successful engine journal writes and syncs
//! (jig/docs/design/protocol/store.md, 5.1; skein/docs/design/kv.md, 3 and 4).
use skein_io::kernel::{Complete, Done, Op};
use skein_lib::{Queue, Reader};
use std::collections::BTreeMap;

#[derive(Default)]
pub(in crate::world) struct Terminals {
    pub(super) revision: u64,
    pub(super) ended: Vec<u64>,
    #[cfg(test)]
    pub(super) last_sync: Option<skein_lib::Token>,
    logs: BTreeMap<i32, Vec<u64>>,
}
impl Terminals {
    pub(in crate::world) fn observe(&mut self, completions: &Queue<Complete>) {
        self.revision = self.revision.saturating_add(1);
        self.ended.clear();
        for completion in completions {
            match (&completion.kind, &completion.result) {
                (Op::Open { path, .. }, Ok(Done::Fd(fd))) => {
                    let leaf = path.rsplit(|byte| *byte == b'/').next().expect("open path");
                    if leaf
                        .strip_prefix(b"log-")
                        .is_some_and(|number| number.len() == 20 && number.iter().all(u8::is_ascii_digit))
                    {
                        self.logs.insert(fd.raw(), Vec::new());
                    }
                }
                (Op::Write { fd, bytes, from, .. }, Ok(Done::Count(count))) => {
                    // File io retains the whole frame across short writes. Observe it once,
                    // when its last suffix completes; parse keys and skip all value bytes.
                    if usize::try_from(*from + *count).ok() == Some(bytes.len())
                        && let Some(pending) = self.logs.get_mut(&fd.raw())
                        && let Some(ended) = terminal_keys(bytes)
                    {
                        pending.extend(ended);
                    }
                }
                (Op::Sync { fd }, Ok(Done::Nothing)) => {
                    if let Some(pending) = self.logs.get_mut(&fd.raw()) {
                        #[cfg(test)]
                        if !pending.is_empty() {
                            self.last_sync = Some(completion.op);
                        }
                        self.ended.append(pending);
                    }
                }
                (Op::Close { fd }, _) => {
                    self.logs.remove(&fd.raw());
                }
                _ => {}
            }
        }
    }
}

/// The trusted writer supplies one SKVC frame per write. Its successful sync
/// witnesses the terminal record, independent of any retained dependency stub.
fn terminal_keys(bytes: &[u8]) -> Option<Vec<u64>> {
    // Header (22 bytes) and checksum (4); a group write may contain many frames.
    const FRAME_MIN: usize = 26;
    let mut frame = Reader::new(bytes);
    let mut ended = Vec::new();
    for _ in 0..bytes.len() / FRAME_MIN {
        if frame.is_empty() {
            break;
        }
        if frame.bytes(4)? != b"SKVC" || frame.u16()? != 1 {
            return None;
        }
        let _commit = frame.u64()?;
        let count = frame.u32()?;
        let body = frame.u32()?;
        let mut records = Reader::new(frame.bytes(body)?);
        // The store owns checksum validation; this observer reads its own completed writes.
        frame.skip(4)?;
        for _ in 0..count {
            let kind = records.u8()?;
            let length = records.u32()?;
            let key = records.bytes(length)?;
            match kind {
                1 => {
                    let length = records.u32()?;
                    records.skip(length)?;
                    if let Some(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Ended(task))) =
                        jig_protocol_store::keys::decode(key)
                    {
                        ended.push(task);
                    }
                }
                2 => {}
                _ => return None,
            }
        }
        if !records.is_empty() {
            return None;
        }
    }
    if frame.is_empty() { Some(ended) } else { None }
}
