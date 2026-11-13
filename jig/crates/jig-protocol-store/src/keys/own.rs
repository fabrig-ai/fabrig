//! Component rows (protocol/store/records.md, section 3, and protocol/store/transcripts.md, section 1).
use alloc::boxed::Box;
use skein_kv::{KeyReader, KeyWriter};
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Own {
    Format,
    Transcript { task: u64 },
    Attempt { task: u64, attempt: u64 },
    Layouts,
    Wires,
    Opening(Box<str>),
}
pub(super) fn write(writer: &mut KeyWriter, key: &Own) {
    match key {
        Own::Opening(worker) => {
            writer.tag(5);
            writer.bytes(worker.as_bytes());
        }
        Own::Wires => {
            writer.tag(4);
        }
        Own::Layouts => {
            writer.tag(3);
        }
        Own::Format => {
            writer.tag(0);
        }
        Own::Transcript { task } => {
            writer.tag(1);
            writer.u64(*task);
        }
        Own::Attempt { task, attempt } => {
            writer.tag(2);
            writer.u64(*task);
            writer.u64(*attempt);
        }
    }
}
#[expect(clippy::disallowed_methods, reason = "the protocol key codec validates worker names as UTF-8")]
pub(super) fn read(reader: &mut KeyReader<'_>) -> Option<Own> {
    Some(match reader.tag()? {
        0 => Own::Format,
        3 => Own::Layouts,
        4 => Own::Wires,
        5 => Own::Opening(Box::from(core::str::from_utf8(&reader.bytes()?).ok()?)),
        1 => Own::Transcript { task: reader.u64()? },
        2 => Own::Attempt { task: reader.u64()?, attempt: reader.u64()? },
        _ => return None,
    })
}
