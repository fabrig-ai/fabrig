//! A JSON turn's text carries the same part identity and replay call as text.
use crate::jig::{
    call::{Call, number},
    render::{text_end, turn},
};
use jig_client::contract as c;
use skein_json::writer::Encoder;
pub(super) struct Window {
    task: u64,
    name: c::TurnName,
    part: u64,
    selected: Option<u64>,
    from: u64,
    limit: usize,
}
impl Window {
    pub fn reset(&mut self, name: c::TurnName) {
        self.name = name;
        self.part = 0;
    }
    pub fn new(call: &Call, task: u64, name: c::TurnName, limit: u32) -> Self {
        Self {
            task,
            name,
            part: 0,
            selected: if call.tool == 6 { number(call, 2) } else { None },
            from: if call.tool == 6 { number(call, 3).unwrap_or(0) } else { 0 },
            limit: usize::try_from(limit).expect("part bound fits"),
        }
    }
}
impl super::Cut {
    pub(super) fn part(&mut self, bytes: &[u8], out: &mut Encoder) {
        let Some(window) = &mut self.turn else {
            self.words(bytes, out);
            return;
        };
        window.part = window.part.checked_add(1).expect("bounded turn parts");
        if let Some(selected) = window.selected
            && selected != window.part
        {
            out.null();
            return;
        }
        let from = text_end(bytes, usize::try_from(window.from).unwrap_or(usize::MAX));
        let tail = bytes.get(from..).expect("part offset within text");
        let end = text_end(tail, self.room.min(window.limit));
        self.room = self.room.checked_sub(end).expect("within result text budget");
        let left = tail.len().checked_sub(end).expect("part cut within text");
        out.object_start();
        out.key(b"part");
        out.unsigned(window.part);
        out.key(b"from");
        out.unsigned(u64::try_from(from).expect("bounded offset"));
        out.key(b"text");
        out.string(tail.get(..end).expect("part cut"));
        out.key(b"bytes_left");
        out.unsigned(u64::try_from(left).expect("bounded part"));
        out.key(b"read_on");
        if left == 0 {
            out.null();
        } else {
            out.string(&turn::read_on(
                window.task,
                window.name,
                window.part,
                u64::try_from(from.checked_add(end).expect("bounded offset")).expect("bounded offset"),
            ));
        }
        out.object_end();
    }
}
