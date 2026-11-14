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
    Job { owner: crate::Owner, name: Box<str> },
    Layout { family: Box<str>, number: u16, id: u16 },
    Text([u8; 32]),
}
const fn owner_byte(owner: crate::Owner) -> u8 {
    match owner {
        crate::Owner::Skein => 0,
        crate::Owner::Smith => 1,
        crate::Owner::Jig => 2,
        crate::Owner::Application => 3,
    }
}
pub(super) fn write(writer: &mut KeyWriter, key: &Own) {
    match key {
        Own::Layout { family, number, id } => {
            writer.tag(6);
            writer.bytes(family.as_bytes());
            for byte in number.to_be_bytes().into_iter().chain(id.to_be_bytes()) {
                writer.tag(byte);
            }
        }
        Own::Text(digest) => {
            writer.tag(7);
            for byte in digest {
                writer.tag(*byte);
            }
        }
        Own::Job { owner, name } => {
            writer.tag(8);
            writer.tag(owner_byte(*owner));
            writer.bytes(name.as_bytes());
        }
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
        6 => Own::Layout {
            family: Box::from(core::str::from_utf8(&reader.bytes()?).ok()?),
            number: u16::from_be_bytes([reader.tag()?, reader.tag()?]),
            id: u16::from_be_bytes([reader.tag()?, reader.tag()?]),
        },
        7 => {
            let mut digest = [0; 32];
            for byte in &mut digest {
                *byte = reader.tag()?;
            }
            Own::Text(digest)
        }
        8 => Own::Job {
            owner: match reader.tag()? {
                0 => crate::Owner::Skein,
                1 => crate::Owner::Smith,
                2 => crate::Owner::Jig,
                3 => crate::Owner::Application,
                _ => return None,
            },
            name: Box::from(core::str::from_utf8(&reader.bytes()?).ok()?),
        },
        1 => Own::Transcript { task: reader.u64()? },
        2 => Own::Attempt { task: reader.u64()?, attempt: reader.u64()? },
        _ => return None,
    })
}
