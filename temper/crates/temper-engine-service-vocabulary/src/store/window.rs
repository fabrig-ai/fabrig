//! Engine window pages (`temper/docs/design/domain/root.md`, 7).
use ::core::mem::size_of;
use alloc::boxed::Box;
use jig_domain_core::{TurnRecord, engine::store as port};
use jig_protocol_store as store;
use skein_lib::{List, Token};
use temper_engine_domain::{Record, loads::Limits};

#[derive(Debug)]
pub(super) struct Window {
    pub load: Token,
    pub task: u64,
    pub from: (u64, u32),
}

pub(super) fn request(owner: Token, window: &Window, after: Option<u64>, limits: &Limits) -> store::Request {
    store::Request::Transcript {
        owner,
        read: store::TranscriptRead::Window {
            task: window.task,
            attempt: window.from.0,
            turn: window.from.1,
            after,
            limits: Some(store::WindowPage {
                row_bytes: limits.transcript_bytes,
                bytes: limits.bytes,
                turn_overhead: u32::try_from(size_of::<Record>()).expect("record slot fits u32"),
            }),
        },
        rows: limits.rows,
        bytes: u64::from(limits.reply_bytes),
    }
}

pub(super) fn answer(turns: Box<[TurnRecord]>, cut: Option<u32>, more: bool, limits: &Limits) -> port::Answer {
    let failed = port::Answer::Failed(port::Failure::Failed);
    let Ok(rows) = u32::try_from(turns.len()) else { return failed };
    if rows > limits.rows || (rows == 0 && more) {
        return failed;
    }
    let mut loaded = List::with_capacity(rows);
    let mut bytes = u64::from(limits.bytes);
    for turn in turns {
        if let Some(length) = cut {
            if rows != 1 || !turn.transcript.is_empty() {
                return failed;
            }
            return port::Answer::Cut { turn, bytes: u64::from(length), more };
        }
        let length = skein_lib::len_u64(turn.transcript.len());
        let slot = skein_lib::len_u64(size_of::<Record>());
        let Some(cost) = slot.checked_add(length) else { return failed };
        let Some(room) = bytes.checked_sub(cost) else {
            return cut_turn(turn, length, more, rows);
        };
        if length > u64::from(limits.transcript_bytes) {
            return cut_turn(turn, length, more, rows);
        }
        bytes = room;
        let text = rendered(&turn);
        loaded.push((turn, text)).expect("one item per row");
    }
    port::Answer::Rows(port::Rows::Turns { turns: loaded.into_boxed(), more })
}

fn cut_turn(mut turn: TurnRecord, bytes: u64, more: bool, rows: u32) -> port::Answer {
    if rows != 1 {
        return port::Answer::Failed(port::Failure::Failed);
    }
    turn.transcript = Box::new([]);
    port::Answer::Cut { turn, bytes, more }
}

// Each rendering fits a tail, as jig's protocol/store/transcripts.md, 3 prices it.
fn rendered(turn: &TurnRecord) -> Option<Box<[u8]>> {
    let bound = u32::try_from(turn.transcript.len()).expect("bounded stored turn");
    let limits = jig_protocol_transcript::Limits::new(
        bound,
        jig_protocol_transcript::TurnLimits { messages: u32::MAX, blocks: u32::MAX, text: bound, outcome: bound },
    );
    let text = jig_protocol_transcript::brief_text(&limits, &turn.transcript).ok()?;
    Some(text_end(&text))
}

/// A rendering within one tail, its end kept after jig's cut marker on a UTF-8
/// boundary; the protocol rendered UTF-8.
fn text_end(text: &[u8]) -> Box<[u8]> {
    const MARKER: &[u8] = b"[earlier text cut]\n";
    let bound = usize::try_from(temper_engine_domain::TRANSCRIPT_TAIL_BYTES).expect("tail bound fits usize");
    if text.len() <= bound {
        return Box::from(text);
    }
    let mut start =
        text.len().checked_sub(bound.checked_sub(MARKER.len()).expect("marker fits a tail")).expect("oversized text");
    // A UTF-8 character has at most three continuation bytes.
    for _ in 0_u8..3 {
        match text.get(start) {
            Some(byte) if byte & 0xc0 == 0x80 => start = start.checked_add(1).expect("inside text"),
            Some(_) | None => break,
        }
    }
    let suffix = text.get(start..).expect("suffix within text");
    let mut writer = skein_lib::Writer::new(MARKER.len().checked_add(suffix.len()).expect("bounded text"));
    writer.put(MARKER).expect("measured marker");
    writer.put(suffix).expect("measured suffix");
    writer.finish()
}
