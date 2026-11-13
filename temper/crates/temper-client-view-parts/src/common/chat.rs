//! Shared chat items (temper/docs/design/client/view-crates.md, 3.3).
use alloc::boxed::Box;
use temper_client_domain::Domain;

/// A chat's heading is at most this many characters, its ellipsis included.
pub(crate) const SHORT_TITLE: usize = 72;

/// A chat's heading shows its title's short form (client/pages/README.md, A chat's heading is short).
pub(crate) fn short_title(title: &[u8]) -> Box<[u8]> {
    let mut start = title.len();
    for (index, byte) in title.iter().enumerate() {
        if !title_space(*byte) {
            start = index;
            break;
        }
    }
    let title = title.get(start..).expect("start is within the title");
    let mut end = title.len();
    for (index, byte) in title.iter().enumerate() {
        if *byte == b'\n' {
            end = index;
            break;
        }
        let next = index.checked_add(1).expect("title offset fits");
        if b".!?".contains(byte)
            && match title.get(next) {
                Some(byte) => title_space(*byte),
                None => true,
            }
        {
            end = next;
            break;
        }
    }
    let words = title.get(..end).expect("sentence ends within the title");
    for byte in words.iter().rev() {
        if !title_space(*byte) {
            break;
        }
        end = end.checked_sub(1).expect("trailing byte is within the words");
    }
    let words = title.get(..end).expect("trimmed words are within the title");
    let mut characters = 0_usize;
    let mut hard_end = 0;
    let mut word_end = 0;
    // Text arrived validated; each non-continuation byte starts one scalar value.
    for (index, byte) in words.iter().enumerate() {
        if byte & 0xc0 != 0x80 {
            characters = characters.checked_add(1).expect("title character count fits");
            if characters > SHORT_TITLE {
                break;
            }
            if characters == SHORT_TITLE {
                hard_end = index;
            }
            if *byte == b' ' || *byte == b'\t' {
                word_end = index;
            }
        }
    }
    if characters <= SHORT_TITLE {
        return skein_lib::bytes::copy_of(words);
    }
    let mut end = word_end;
    for byte in words.get(..word_end).expect("word boundary is within the words").iter().rev() {
        if !title_space(*byte) && !b",;:".contains(byte) {
            break;
        }
        end = end.checked_sub(1).expect("trailing byte is within the prefix");
    }
    if end == 0 {
        end = hard_end;
    }
    let prefix = words.get(..end).expect("cut is at a character boundary within the words");
    let mark = "…".as_bytes();
    let bytes = end.checked_add(mark.len()).expect("short title bytes fit");
    let mut out = skein_lib::Writer::new(bytes);
    out.put(prefix).expect("short title length counted");
    out.put(mark).expect("short title length counted");
    out.finish()
}

fn title_space(byte: u8) -> bool {
    b" \t\r\n".contains(&byte)
}

/// The delivered chat's chip decides its path and current navigation entry (client/pages/README.md).
pub(crate) fn asking(root: &Domain) -> bool {
    let Some(chat) = root.pages().chat() else { return false };
    if chat.talk().is_some() {
        return false;
    }
    let Some(task) = chat.task(root.client()) else { return false };
    let Some(me) = root.pages().me(root.client()) else { return false };
    temper_client_domain::pages::chats::asking(me, task.project, &task.chip.executor)
}
