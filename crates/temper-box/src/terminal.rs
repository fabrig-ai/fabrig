//! Pure terminal words and targets (protocol/setup.md, 7.2).
use crate::lines::joined;
use alloc::boxed::Box;
use skein_lib::{List, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fact {
    Starting { step: usize },
    Stopping,
    Stopped,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    Stdout,
    Stderr,
}
#[derive(PartialEq, Eq, Debug)]
pub struct Item {
    pub target: Target,
    pub bytes: Box<[u8]>,
}
pub const PRELUDE: &[u8] = b"\nOpen this link in a browser on this machine. It signs you in as this\ntemper's owner, once and for a short while, so keep it private.\nOpened again once spent, it prints a fresh one here.\n\n";
pub const FOOTER: &[u8] = b"\nCtrl-C stops temper.\n\n";
pub const REFUSAL: &[u8] = b"box: serve: sign-in link too long\n";
/// Largest fixed item, also allowing the ten-byte UTC prefix.
pub const FIXED_BYTES: u32 = 256;

#[derive(Debug)]
pub struct Formatter {
    first_printed: bool,
}
#[expect(clippy::new_without_default, reason = "the programming model uses explicit constructors")]
impl Formatter {
    #[must_use]
    pub fn new() -> Self {
        Self { first_printed: false }
    }
    #[must_use]
    pub fn block_entries(&self) -> u32 {
        if self.first_printed { 2 } else { 3 }
    }
    /// Caller reserves `block_entries` before extracting the line. State changes
    /// only after the complete adjacent block has entered the terminal FIFO.
    #[must_use]
    pub fn block(&self, url: Box<[u8]>, wall: Wall) -> List<Item> {
        let mut items = List::with_capacity(self.block_entries());
        let words = if self.first_printed {
            self.notice(b"A fresh sign-in link:\n", wall)
        } else {
            Item { target: Target::Stderr, bytes: skein_lib::bytes::copy_of(PRELUDE) }
        };
        items.push(words).expect("block count");
        items.push(Item { target: Target::Stdout, bytes: url }).expect("block count");
        if !self.first_printed {
            items.push(Item { target: Target::Stderr, bytes: skein_lib::bytes::copy_of(FOOTER) }).expect("block count");
        }
        items
    }
    pub fn admitted_block(&mut self) {
        self.first_printed = true;
    }
    #[must_use]
    pub fn fact(&self, fact: Fact, name: &[u8], wall: Wall) -> Item {
        let words = match fact {
            Fact::Starting { .. } => joined(&[b"Starting ", name, b"\n"]),
            Fact::Stopping => skein_lib::bytes::copy_of(b"Stopping\n"),
            Fact::Stopped => skein_lib::bytes::copy_of(b"Stopped\n"),
        };
        self.notice(&words, wall)
    }
    #[must_use]
    pub fn notice(&self, words: &[u8], wall: Wall) -> Item {
        let bytes = if self.first_printed {
            let day = wall.as_secs().rem_euclid(86_400);
            let hours = pair(day.div_euclid(3_600));
            let minutes = pair(day.div_euclid(60).rem_euclid(60));
            let seconds = pair(day.rem_euclid(60));
            joined(&[&hours, b":", &minutes, b":", &seconds, b"Z ", words])
        } else {
            skein_lib::bytes::copy_of(words)
        };
        Item { target: Target::Stderr, bytes }
    }
}
fn pair(value: u64) -> [u8; 2] {
    [
        b'0'.checked_add(u8::try_from(value.div_euclid(10)).expect("two digits")).expect("ASCII digit"),
        b'0'.checked_add(u8::try_from(value.rem_euclid(10)).expect("one digit")).expect("ASCII digit"),
    ]
}
