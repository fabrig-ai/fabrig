//! Directing words awaiting a run's record (client/ctl/results.md, 1 and 2).
use super::{Cut, Text};
use jig_client::contract as c;
use skein_json::writer::Encoder;
use skein_lib::List;

pub(super) fn text(task: &c::Task, me: Option<&c::Me>, out: &mut Text) {
    if task.unread.is_empty() {
        return;
    }
    out.put(b"unread: ");
    out.number(u64::from(task.unread.len()));
    out.put(b", oldest first; no run has recorded them\n");
    for word in &task.unread {
        out.put(b"  message ");
        out.number(word.message);
        out.put(b" \xc2\xb7 from ");
        super::requester(word.by, me, out);
        out.put(b" \xc2\xb7 ");
        super::clock::deadline(word.at, out);
        out.put(if word.seen { b" \xc2\xb7 seen: " } else { b" \xc2\xb7 not yet seen: " });
        let mut head = skein_lib::bytes::copy_of(word.head.bytes());
        for byte in &mut head {
            if byte.is_ascii_control() {
                *byte = b' ';
            }
        }
        out.words(&head);
        if skein_lib::length::usize_of(word.bytes) > word.head.bytes().len() {
            out.put(b"\xe2\x80\xa6 (");
            out.number(u64::from(word.bytes));
            out.put(b" bytes; task activity ");
            out.number(task.chip.task);
            out.put(b" prints them whole)");
        }
        out.put(b"\n");
    }
}

pub(super) fn encode(words: &List<c::UnreadWords>, out: &mut Encoder, budget: &mut Cut) {
    out.key(b"unread");
    out.array_start();
    for word in words {
        out.object_start();
        out.key(b"message");
        out.unsigned(word.message);
        out.key(b"by");
        super::encode_owner_requester(&word.by, out, budget);
        out.key(b"at");
        out.unsigned(word.at.as_nanos());
        out.key(b"seen");
        out.boolean(word.seen);
        out.key(b"head");
        budget.words(word.head.bytes(), out);
        out.key(b"bytes");
        out.unsigned(u64::from(word.bytes));
        out.object_end();
    }
    out.array_end();
}
