//! A goal's reds (temper/docs/design/domain/forge/reds.md, 2, 3 and 6).
pub use crate::Filer as Recorder;
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Red {
    pub goal: u64,
    pub key: Box<[u8]>,
    pub head: Box<[u8]>,
    pub why: Box<[u8]>,
    pub by: Recorder,
    pub at: u64,
    pub generation: u64,
}
/// A change's latest passed check of a goal's red (reds.md, 5.7).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RedPass {
    pub change: u64,
    pub key: Box<[u8]>,
    pub generation: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RecordRed {
    pub goal: u64,
    pub key: Box<[u8]>,
    pub head: Box<[u8]>,
    pub why: Box<[u8]>,
    pub by: Recorder,
    pub at: u64,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RedRecorded {
    pub generation: u64,
    pub replaced: Option<Red>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RedRefusal {
    NoGoal,
    Full { bound: u32 },
    NoSuch,
}

/// The deciding snapshot includes the address needed to replay refusals (reds.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RedReply {
    pub goal: u64,
    pub key: Box<[u8]>,
    pub result: Result<RedOutcome, RedRefusal>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RedOutcome {
    Recorded { head: Box<[u8]>, recorded: RedRecorded },
    Ended(Red),
}
impl RedRefusal {
    #[must_use]
    pub fn words(self, goal: u64, key: &[u8]) -> Box<[u8]> {
        use skein_lib::{Decimal, List};
        let number = Decimal::of(goal);
        let mut words = List::uncapped();
        let parts: &[&[u8]] = match self {
            Self::NoGoal => &[b"red: task ", number.as_bytes(), b" holds no goal branch; only a goal's run keeps reds"],
            Self::NoSuch => {
                &[b"red: goal ", number.as_bytes(), b" holds no red ", key, b"; red list shows what it holds"]
            }
            Self::Full { bound } => {
                let bound = Decimal::of(u64::from(bound));
                for part in [
                    b"red: goal ".as_slice(),
                    number.as_bytes(),
                    b" holds its most reds (at most ",
                    bound.as_bytes(),
                    b"); end one, or look at what its branch carries",
                ] {
                    for &byte in part {
                        words.push(byte).expect("bounded red refusal");
                    }
                }
                return words.into_boxed();
            }
        };
        for part in parts {
            for &byte in *part {
                words.push(byte).expect("bounded red refusal");
            }
        }
        words.into_boxed()
    }
}
/// The owned bytes beside the reply's inline fields.
#[must_use]
pub fn reply_bytes(reply: &RedReply) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(reply.key.len());
    match &reply.result {
        Ok(RedOutcome::Recorded { head, recorded }) => {
            bytes = bytes.checked_add(skein_lib::len_u64(head.len()))?;
            if let Some(red) = &recorded.replaced {
                bytes = bytes.checked_add(row_bytes(red)?)?;
            }
        }
        Ok(RedOutcome::Ended(red)) => bytes = bytes.checked_add(row_bytes(red)?)?,
        Err(RedRefusal::NoGoal | RedRefusal::NoSuch | RedRefusal::Full { .. }) => {}
    }
    Some(bytes)
}
#[must_use]
pub fn row_bytes(red: &Red) -> Option<u64> {
    skein_lib::len_u64(red.key.len())
        .checked_add(skein_lib::len_u64(red.head.len()))?
        .checked_add(skein_lib::len_u64(red.why.len()))
}

/// The check's opaque, single-line key, without surrounding whitespace (reds.md, 2).
#[must_use]
#[expect(clippy::disallowed_methods, reason = "the red decoding boundary judges UTF-8 and whitespace")]
pub fn key_fits(key: &[u8], most: u32) -> bool {
    if !crate::settings::words_fit(key, most) {
        return false;
    }
    let Ok(text) = core::str::from_utf8(key) else { return false };
    text.trim() == text
}
