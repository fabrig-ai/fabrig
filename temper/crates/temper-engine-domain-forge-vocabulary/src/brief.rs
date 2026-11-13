//! Forge-owned brief sections, gathered from current provider facts
//! (jig's domain/connectors.md, section 9). Pull sections stop at their cutoff
//! (temper/docs/design/domain/forge/change/repairs.md, A resolution).
use crate::{BriefRead, brief_words::bounds};
use alloc::boxed::Box;
use skein_lib::List;

/// The home-access reason's 209 fixed bytes plus three owner/repository pairs.
/// Adoption bounds each name by `Limits::name_bytes`, whose ceiling is
/// `PANEL_NAME_BYTES` (1,024 bytes for each owner and repository name).
pub const HOME_SECTION_BYTES: u32 = 209 + 6 * crate::PANEL_NAME_BYTES;

/// The adoption hold's words (domain/forge/branches.md), bounded by the
/// caller's read allotment and the longest admitted names. Omitted bytes follow
/// the other forge sections' cuts.
#[must_use]
pub fn home_access_reason(owner: &[u8], repository: &[u8], bytes: u32) -> BriefRead {
    let mut words = BriefWords::with_capacity(bytes.min(HOME_SECTION_BYTES), false);
    let parts: [&[u8]; 12] = [
        b"This chat began before ",
        owner,
        b"/",
        repository,
        b" was adopted, so you can read ",
        owner,
        b"/",
        repository,
        b" but not push to it. Tell the person, in your next reply: \"This chat began before ",
        owner,
        b"/",
        repository,
    ];
    for part in parts {
        append(&mut words, part);
    }
    append(&mut words, b" was adopted, so I can read it but not push. Start a new chat to push.\"");
    brief_part(&words, 1, bytes)
}
#[derive(Hash, Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the top inspects the buffer and omitted bytes; tail policy stays private"
)]
pub struct BriefWords {
    pub bytes: List<u8>,
    pub left: u64,
    tail: bool,
}

impl BriefWords {
    pub fn with_capacity(capacity: u32, tail: bool) -> BriefWords {
        BriefWords { bytes: List::with_capacity(capacity), left: 0, tail }
    }
}
pub fn append(out: &mut BriefWords, bytes: &[u8]) {
    for byte in bytes {
        if out.bytes.room() > 0 {
            out.bytes.push(*byte).expect("checked message room");
        } else {
            out.left = out.left.saturating_add(1);
            if out.tail && !out.bytes.is_empty() {
                for index in 1..out.bytes.len() {
                    *out.bytes.get_mut(index.saturating_sub(1)).expect("Within words") =
                        *out.bytes.get(index).expect("Within words");
                }
                *out.bytes.get_mut(out.bytes.len().saturating_sub(1)).expect("Nonempty words") = *byte;
            }
        }
    }
}

/// One byte's two lowercase hexadecimal digits, the high nibble first.
#[must_use]
pub fn hex(byte: u8) -> [u8; 2] {
    let high = *b"0123456789abcdef".get(usize::from(byte >> 4_u8)).expect("high nibble is hexadecimal");
    let low = *b"0123456789abcdef".get(usize::from(byte & 15_u8)).expect("low nibble is hexadecimal");
    [high, low]
}

pub fn brief_part(words: &BriefWords, parts: u32, bytes: u32) -> BriefRead {
    if parts == 0 {
        return BriefRead::Failed(bounds());
    }
    let kept = words.bytes.as_slice().len().min(skein_lib::usize_of(bytes));
    let start = if words.tail { words.bytes.as_slice().len().saturating_sub(kept) } else { 0 };
    BriefRead::Got {
        bytes: Box::from(words.bytes.as_slice().get(start..start.saturating_add(kept)).expect("Kept is within source")),
        left: words.left.saturating_add(
            u64::try_from(words.bytes.as_slice().len().checked_sub(kept).expect("Kept is within source"))
                .expect("bounded section"),
        ),
    }
}
