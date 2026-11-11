//! Agent card words (temper/docs/design/domain/forge/change/branch-path.md).
use alloc::boxed::Box;
use skein_lib::bytes;

/// The named tool families, followed by the charter's merge boundary.
#[must_use]
#[expect(clippy::fn_params_excessive_bools, reason = "the contract names five independent tool-family flags")]
pub fn can(inspect: bool, modify: bool, shell: bool, agents: bool, settings: bool) -> &'static [u8] {
    let families =
        u8::from(inspect) | (u8::from(modify) << 1_u8) | (u8::from(shell) << 2_u8) | (u8::from(agents) << 3_u8);
    match families {
        0 => {
            if settings {
                b"read temper's records and the forge; propose settings; never change anything"
            } else {
                b"read the facts in its task; never merge"
            }
        }
        1 => b"read the code and the forge; never merge",
        2 => b"change and push its branch; never merge",
        3 => b"read the code and the forge; change and push its branch; never merge",
        4 => b"run commands; never merge",
        5 => b"read the code and the forge; run commands; never merge",
        6 => b"change and push its branch; run commands; never merge",
        7 => b"read the code and the forge; change and push its branch; run commands; never merge",
        8 => b"start sub-agents; never merge",
        9 => b"read the code and the forge; start sub-agents; never merge",
        10 => b"change and push its branch; start sub-agents; never merge",
        11 => b"read the code and the forge; change and push its branch; start sub-agents; never merge",
        12 => b"run commands; start sub-agents; never merge",
        13 => b"read the code and the forge; run commands; start sub-agents; never merge",
        14 => b"change and push its branch; run commands; start sub-agents; never merge",
        15 => b"read the code and the forge; change and push its branch; run commands; start sub-agents; never merge",
        _ => unreachable!("four tool-family bits"),
    }
}

/// Keep `value` whole within `bound`, otherwise cut at the last word's end with "…".
/// With no space in reach, cut at a character's end; invalid UTF-8 or a bound below three gives nothing.
#[must_use]
pub fn cut_on_word(value: &[u8], bound: usize) -> Box<[u8]> {
    let marker = "…".as_bytes();
    if bound < marker.len() || !bytes::is_utf8(value) {
        return Box::default();
    }
    if value.len() <= bound {
        return value.into();
    }
    let room = bound.checked_sub(marker.len()).expect("marker fits bound");
    let mut word_end = None;
    let mut trimmed_end = 0;
    for (at, byte) in value.iter().enumerate() {
        if at > room {
            break;
        }
        if *byte == b' ' {
            word_end = Some(trimmed_end);
        } else {
            trimmed_end = at.checked_add(1).expect("character within input");
        }
    }
    let prefix = match word_end {
        Some(end) => value.get(..end).expect("word within input"),
        None => bytes::utf8_prefix(value, room),
    };
    bytes::concat(&[prefix, marker])
}

/// When a role runs and what it does, cut on a word within the card's `WHO` bound.
#[must_use]
pub fn role_who(when: &[u8], does: &[u8]) -> Box<[u8]> {
    use crate::WHO;
    if does.is_empty() || when.len() >= WHO.saturating_sub(b": ".len()) {
        return cut_on_word(when, WHO);
    }
    let room = WHO.saturating_sub(when.len()).saturating_sub(b": ".len());
    bytes::concat(&[when, b": ", &cut_on_word(does, room)])
}

/// Through the first ". " or ".\n", trimmed, then cut on a word within `bound`.
#[must_use]
pub fn first_sentence(instructions: &[u8], bound: usize) -> Box<[u8]> {
    let mut sentence = instructions;
    for (at, pair) in instructions.windows(2).enumerate() {
        if pair == b". " || pair == b".\n" {
            let end = at.checked_add(1).expect("full stop within input");
            sentence = instructions.get(..end).expect("sentence prefix");
            break;
        }
    }
    cut_on_word(sentence.trim_ascii(), bound)
}
