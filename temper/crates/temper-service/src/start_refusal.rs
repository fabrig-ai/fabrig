//! Refused-start stderr, including writes named before store decoding (protocol/README.md, 4).
use alloc::boxed::Box;
use jig_protocol_store::OpenRefusal;
use skein_lib::{Decimal, Writer};

// jig's protocol/store.md, 10.3 caps the refusal's retained keys at 64.
const KEYS: usize = 64;
const REMEDY: &[u8] = b" is still in flight; start the build before this one and stop it to settle it\n";

/// Render jig's words followed by temper's write identities, and the refused exit status.
#[must_use]
pub fn refused_start(why: &OpenRefusal, deployment: &[u8]) -> (Box<[u8]>, u8) {
    let mut measuring = Lines { writer: Writer::new(0), length: 0, measuring: true };
    render(&mut measuring, why, deployment);
    let mut lines = Lines { writer: Writer::new(measuring.length), length: 0, measuring: false };
    render(&mut lines, why, deployment);
    (lines.writer.finish(), 1)
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
    if let OpenRefusal::InFlight { count, keys, .. } = why {
        let shown = keys.len().min(KEYS);
        for key in keys.iter().take(KEYS) {
            if let Some((task, entry)) = temper_protocol_store::keys::flight(key) {
                lines.put(b"temper serve: task ");
                lines.number(task);
                lines.put(b"'s forge write ");
                lines.number(entry);
            } else {
                lines.put(b"temper serve: forge write key 0x");
                for byte in key {
                    lines.put(&[hex(byte >> 4), hex(byte & 15)]);
                }
            }
            lines.put(REMEDY);
        }
        let more = count.saturating_sub(u32::try_from(shown).expect("at most 64 keys"));
        if more > 0 {
            lines.put(b"temper serve: and ");
            lines.number(u64::from(more));
            lines.put(b" more forge writes in flight\n");
        }
    }
}
fn hex(nibble: u8) -> u8 {
    *b"0123456789abcdef".get(usize::from(nibble)).expect("masked nibble")
}
