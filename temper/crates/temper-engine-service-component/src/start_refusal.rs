//! Refused-start stderr, including writes named before store decoding (protocol/binaries.md).
use alloc::boxed::Box;
use jig_protocol_store::OpenRefusal;
use skein_lib::{Decimal, Writer};

/// Render jig's words followed by layout and job details, and the refused exit status.
#[must_use]
pub fn refused_start(why: &OpenRefusal, deployment: &[u8]) -> (Box<[u8]>, u8) {
    let mut measuring = Lines { writer: Writer::new(0), length: 0, measuring: true };
    render(&mut measuring, why, deployment);
    let mut lines = Lines { writer: Writer::new(measuring.length), length: 0, measuring: false };
    render(&mut lines, why, deployment);
    (lines.writer.finish(), 1)
}

/// Render the named unreadable-row refusal (temper/docs/design/protocol/store-restart.md, 1).
#[must_use]
pub fn refused_row_start(row: &crate::UnreadableRow, deployment: &[u8]) -> (Box<[u8]>, u8) {
    let mut measuring = Lines { writer: Writer::new(0), length: 0, measuring: true };
    render_row(&mut measuring, row, deployment);
    let mut lines = Lines { writer: Writer::new(measuring.length), length: 0, measuring: false };
    render_row(&mut lines, row, deployment);
    (lines.writer.finish(), 1)
}

fn render_row(lines: &mut Lines, row: &crate::UnreadableRow, deployment: &[u8]) {
    let Some((number, name)) = row.family() else {
        lines.put(b"temper serve: restore forge: store load failed\n");
        return;
    };
    lines.put(b"temper serve: The store in ");
    lines.put(deployment);
    lines.put(b" holds a ");
    lines.put(name.as_bytes());
    lines.put(b" record, family ");
    lines.number(u64::from(number));
    lines.put(b", that this build cannot read, and the start stopped there. Start the build that wrote it again, or, to start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of ");
    lines.put(deployment);
    lines.put(b" together, and start again: nothing is deleted, and the next start makes a new, empty store.\n");
}

struct Lines {
    writer: Writer,
    length: usize,
    measuring: bool,
}
impl Lines {
    fn put(&mut self, bytes: &[u8]) {
        self.length = self.length.checked_add(bytes.len()).expect("bounded refusal fits usize");
        if !self.measuring {
            self.writer.put(bytes).expect("refusal was measured");
        }
    }
    fn number(&mut self, number: u64) {
        self.put(Decimal::of(number).as_bytes());
    }
}
fn render(lines: &mut Lines, why: &OpenRefusal, deployment: &[u8]) {
    lines.put(b"temper serve: ");
    for (index, words) in why.words().iter().enumerate() {
        if index > 0 {
            lines.put(deployment);
        }
        lines.put(words.as_bytes());
    }
    lines.put(b"\n");
    if let OpenRefusal::Layout { family, value_type, id, why: jig_protocol_store::Unplanned::Retired, .. } = why {
        lines.put(b"temper serve: family ");
        lines.put(family.as_bytes());
        lines.put(b" value type ");
        lines.number(u64::from(*value_type));
        lines.put(b" layout ");
        lines.number(u64::from(*id));
        lines.put(b": the value type is retired, and its keys hold a value\n");
    }
    if let OpenRefusal::Job { owner, job, key, why } = why {
        let owner = match owner {
            jig_protocol_store::Owner::Skein => "skein",
            jig_protocol_store::Owner::Smith => "smith",
            jig_protocol_store::Owner::Jig => "jig",
            jig_protocol_store::Owner::Application => "temper",
        };
        lines.put(b"temper serve: job ");
        lines.put(owner.as_bytes());
        lines.put(b" ");
        lines.put(job.as_bytes());
        if let Some(key) = key {
            lines.put(b" key 0x");
            for byte in key {
                lines.put(&[hex(byte >> 4), hex(byte & 15)]);
            }
        }
        lines.put(b": ");
        lines.put(why.as_bytes());
        lines.put(b"\n");
    }
}
fn hex(nibble: u8) -> u8 {
    *b"0123456789abcdef".get(usize::from(nibble)).expect("masked nibble")
}
