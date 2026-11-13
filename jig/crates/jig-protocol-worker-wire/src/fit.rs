//! A resumed start's committed calls fitted to a room (jig/docs/design/protocol/channel/README.md, 8 and 10).
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Writer};

use crate::v10;

/// What fitting did to an assignment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fitted {
    /// Its frame fits as it came: nothing cut.
    Whole,
    /// `cut` host answers were cut, the oldest; its frame fits.
    Cut { cut: u32 },
    /// Past the room with every host answer cut; `bytes` is its frame so cut.
    Past { bytes: u32 },
}

pub(super) fn cut_length(bytes: u32) -> u32 {
    let digits = Decimal::of(u64::from(bytes));
    let digits = u32::try_from(digits.as_bytes().len()).expect("a u32 has at most ten decimal digits");
    64_u32.checked_add(digits).expect("the fixed words and ten digits fit u32")
}

/// The words a cut host answer's body becomes, `bytes` the body's length.
#[must_use]
pub fn cut_words(bytes: u32) -> Box<[u8]> {
    let digits = Decimal::of(u64::from(bytes));
    let mut writer = Writer::new(skein_lib::length::usize_of(cut_length(bytes)));
    writer.put(b"[this call ran; its answer of ").expect("room for the fixed words");
    writer.put(digits.as_bytes()).expect("room for the body's decimal length");
    writer.put(b" bytes was left out of this start]").expect("room for the fixed words");
    writer.finish()
}

fn saving(answer: &v10::SettledAnswer) -> u32 {
    match answer {
        v10::SettledAnswer::Host(host) => {
            let bytes = u32::try_from(host.body().len()).expect("the body's ceiling fits u32");
            bytes.saturating_sub(cut_length(bytes))
        }
        v10::SettledAnswer::Delivery(_) => 0,
    }
}

fn cut_call(call: v10::AnsweredCall, limits: &v10::Limits) -> v10::AnsweredCall {
    let mut parts = call.into_parts();
    parts.answer = match parts.answer {
        v10::SettledAnswer::Host(host) => {
            let mut host = host.into_parts();
            let bytes = u32::try_from(host.body.len()).expect("the body's ceiling fits u32");
            host.body = cut_words(bytes);
            let host = v10::HostAnswer::new(limits, host).expect("a cut body is shorter than the body it replaces");
            v10::SettledAnswer::Host(host)
        }
        v10::SettledAnswer::Delivery(delivery) => v10::SettledAnswer::Delivery(delivery),
    };
    v10::AnsweredCall::new(limits, parts).expect("only a shorter answer body changed")
}

/// `assignment` with its committed calls fitted to `room`, a frame's bytes.
#[must_use]
pub fn fit(assignment: v10::Assignment, limits: &v10::Limits, room: u32) -> (v10::Assignment, Fitted) {
    let mut bytes = u64::from(assignment.measure()).checked_add(8).expect("a u32 body and its header fit u64");
    if bytes <= u64::from(room) {
        return (assignment, Fitted::Whole);
    }

    // A body's four-byte length stays the same size, so each saving is exact.
    let mut cut = 0_u32;
    for call in assignment.answered() {
        let saved = saving(call.answer());
        if saved != 0 {
            bytes = bytes.checked_sub(u64::from(saved)).expect("each saving is within the measured body");
            cut = cut.checked_add(1).expect("the calls' count fits u32");
        }
    }
    let fitted = if bytes > u64::from(room) {
        Fitted::Past { bytes: u32::try_from(bytes).unwrap_or(u32::MAX) }
    } else {
        for call in assignment.answered().iter().rev() {
            let saved = saving(call.answer());
            if saved != 0 {
                let whole =
                    bytes.checked_add(u64::from(saved)).expect("restoring a body stays within the original frame");
                if whole > u64::from(room) {
                    break;
                }
                bytes = whole;
                cut = cut.checked_sub(1).expect("each restored answer was counted cut");
            }
        }
        Fitted::Cut { cut }
    };

    let mut parts = assignment.into_parts();
    let mut answered = List::with_capacity(parts.answered.capacity());
    let mut left = cut;
    for call in parts.answered.into_boxed() {
        let call = if left != 0 && saving(call.answer()) != 0 {
            left = left.checked_sub(1).expect("an answer remains to cut");
            cut_call(call, limits)
        } else {
            call
        };
        answered.push(call).expect("the calls keep their count and capacity");
    }
    parts.answered = answered;
    let assignment = v10::Assignment::new(limits, parts).expect("only shorter answer bodies changed");
    (assignment, fitted)
}
