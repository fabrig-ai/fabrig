use super::{Action, Live, wanted};
use crate::profile::standard;
use alloc::boxed::Box;
use skein_lib::{List, Time, Token};
use smith_domain::llm;
use smith_protocol_channel as channel;
mod room;
use room::Room;

const AT_ZERO: Time = Time::ZERO;
const PIECE_BOUND: usize = 4096;
const OWNER: Token = Token::new(7);
const RETRY_OWNER: Token = Token::new(8);
// smith/docs/design/protocol/limits.md 3.6: 16 KiB held, cut into four 4096-byte pieces.
const HELD: usize = 16_384;
const LARGE_DELTA: usize = 10_000;
static AT_BOUND: [u8; HELD] = [b'a'; HELD];
static PAST_BOUND: [u8; HELD + 1] = [b'a'; HELD + 1];
#[derive(Debug)]
struct Sent {
    turn: u32,
    retry: u32,
    offset: u64,
    bytes: Box<[u8]>,
    call: Option<smith_channel::LiveCall>,
}
fn ask(text: bool, reasoning: bool, input: bool) -> smith_channel::LiveAsk {
    smith_channel::LiveAsk::new(&smith_channel::CEILINGS, smith_channel::LiveAskParts { text, reasoning, input })
        .expect("ask")
}
fn stream(takes: bool) -> (Live, Room) {
    let mut live = Live::new(Some(ask(true, false, false)), standard().policy.live_interval);
    let mut room = Room::new(takes);
    room.act(&mut live, Action::Open { owner: OWNER, retry: 0 }, AT_ZERO);
    (live, room)
}
#[test]
fn a_stream_past_the_held_bytes_drops_the_newest_and_counts_them() {
    let (mut live, mut room) = stream(false);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'a'; LARGE_DELTA] }, AT_ZERO);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'b'; LARGE_DELTA] }, AT_ZERO);
    assert_eq!(room.lost, 3616, "20,000 provider bytes minus the held bound");
    room.takes = true;
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    assert_eq!(room.offsets().as_slice(), [(0, 4096), (4096, 4096), (8192, 4096), (12_288, 4096)]);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"c" }, AT_ZERO);
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    assert_eq!(room.sent.last().expect("later piece").offset, 20_000, "the omitted suffix leaves a gap");
}
#[test]
fn a_delta_past_the_piece_bound_goes_in_several_pieces() {
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'a'; LARGE_DELTA] }, AT_ZERO);
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    assert_eq!(room.offsets().as_slice(), [(0, 4096), (4096, 4096), (8192, 1808)]);
    let mut joined = List::with_capacity(u32::try_from(LARGE_DELTA).expect("fixture bound"));
    for piece in &room.sent {
        for byte in &piece.bytes {
            joined.push(*byte).expect("fixture payload");
        }
    }
    assert_eq!(joined.as_slice(), &[b'a'; LARGE_DELTA]);
}
#[test]
fn ending_a_turn_settles_the_held_bytes_first() {
    for (takes, pieces, lost) in [(true, 1, 0), (false, 0, 4)] {
        let (mut live, mut room) = stream(false);
        room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"held" }, AT_ZERO);
        room.takes = takes;
        room.act(&mut live, Action::End { turn: 1 }, AT_ZERO);
        assert_eq!((room.sent.len(), room.lost), (pieces, lost), "room={takes}");
        room.takes = true;
        room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
        assert_eq!(room.sent.len(), pieces, "no piece follows the ended turn");
    }
}
#[test]
fn a_delta_after_its_turn_is_dropped_and_counted() {
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::End { turn: 1 }, AT_ZERO);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"late" }, AT_ZERO);
    assert_eq!((room.sent.len(), room.lost), (0, 4));
}
#[test]
fn a_new_retry_starts_again_at_offset_zero() {
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"first" }, AT_ZERO);
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    room.takes = false;
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"held" }, AT_ZERO);
    room.act(&mut live, Action::Open { owner: RETRY_OWNER, retry: 1 }, AT_ZERO);
    room.takes = true;
    room.act(&mut live, Action::Delta { owner: RETRY_OWNER, bytes: b"again" }, AT_ZERO);
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    let mut identities = List::with_capacity(room.sent.len());
    for piece in &room.sent {
        identities.push((piece.turn, piece.retry, piece.offset)).expect("sent identities");
    }
    assert_eq!(identities.as_slice(), [(1, 0, 0), (1, 1, 0)]);
    assert_eq!(room.lost, 4);
}
#[test]
fn only_the_main_ordinary_completion_is_wanted() {
    let mut prompt = smith_protocol_events::fixture::prompt(0);
    for (text, reasoning, input, main_ordinary) in [
        (false, false, false, false),
        (true, false, false, true),
        (false, true, false, false),
        (false, false, true, true),
        (true, true, true, true),
    ] {
        let ask = ask(text, reasoning, input);
        for (asker, purpose, eligible) in [
            (llm::Asker::Main, llm::Purpose::Ordinary { guard: None }, true),
            (llm::Asker::Child, llm::Purpose::Ordinary { guard: None }, false),
            (llm::Asker::Main, llm::Purpose::Compaction { room: 1 }, false),
            (llm::Asker::Child, llm::Purpose::Compaction { room: 1 }, false),
        ] {
            prompt.asker = asker;
            prompt.purpose = purpose;
            assert_eq!(
                wanted(Some(&ask), &prompt),
                main_ordinary && eligible,
                "{asker:?}, {purpose:?}, text={text}, reasoning={reasoning}, input={input}"
            );
            assert!(!wanted(None, &prompt), "an absent ask wants no completion");
        }
    }
}
#[test]
fn the_held_bound_keeps_the_prefix_and_drops_only_the_excess() {
    for (input, lost) in [(AT_BOUND.as_slice(), 0), (PAST_BOUND.as_slice(), 1)] {
        let (mut live, mut room) = stream(false);
        room.act(&mut live, Action::Delta { owner: OWNER, bytes: input }, AT_ZERO);
        assert_eq!(room.lost, lost);
        room.takes = true;
        room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
        let mut joined = List::with_capacity(super::HELD);
        for piece in &room.sent {
            for byte in &piece.bytes {
                joined.push(*byte).expect("held prefix");
            }
        }
        assert_eq!(joined.as_slice(), &[b'a'; HELD]);
    }
}
#[test]
fn a_gap_survives_partial_draining_and_later_held_bytes() {
    let (mut live, mut room) = stream(false);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'a'; LARGE_DELTA] }, AT_ZERO);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'b'; LARGE_DELTA] }, AT_ZERO);
    room.takes = true;
    room.remaining = 1;
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"c" }, AT_ZERO);
    room.remaining = u32::MAX;
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    assert_eq!(room.offsets().as_slice(), [(0, 4096), (4096, 4096), (8192, 4096), (12_288, 4096), (20_000, 1)]);
    assert_eq!(room.lost, 3616);
}
#[test]
fn a_short_piece_goes_at_the_intervals_end() {
    let interval = standard().policy.live_interval;
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"Hel" }, AT_ZERO);
    let end = AT_ZERO.saturating_add(interval);
    room.act(&mut live, Action::Pump, Time::from_nanos(end.as_nanos() - 1));
    assert_eq!(room.sent.len(), 0, "held before the interval ends");
    assert_eq!(live.deadline(), Some(end));
    room.act(&mut live, Action::Pump, end);
    assert_eq!(room.payloads().as_slice(), [b"Hel"]);
    assert_eq!(live.deadline(), None);
}

#[test]
fn a_piece_reaching_the_bound_goes_before_the_interval() {
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'a'; PIECE_BOUND] }, AT_ZERO);
    assert_eq!(room.payloads().as_slice(), [[b'a'; PIECE_BOUND].as_slice()]);
    assert_eq!(live.deadline(), None);
}

#[test]
fn draining_a_piece_retains_the_next_bytes_arrival_time() {
    let interval = standard().policy.live_interval;
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'a'; PIECE_BOUND - 1] }, AT_ZERO);
    let end = AT_ZERO.saturating_add(interval);
    let later = Time::from_nanos(end.as_nanos() - 1);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"bc" }, later);
    assert_eq!(live.deadline(), Some(later.saturating_add(interval)));
}
#[test]
fn unnamed_refused_and_earlier_calls_arguments_are_counted() {
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 0, bytes: b"unnamed" }, AT_ZERO);
    room.act(&mut live, Action::Call { owner: OWNER, index: 0, id: &[b'x'; 257], tool: b"tool" }, AT_ZERO);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 0, bytes: b"refused" }, AT_ZERO);
    room.act(&mut live, Action::Call { owner: OWNER, index: 1, id: b"first", tool: b"tool" }, AT_ZERO);
    room.act(&mut live, Action::Call { owner: OWNER, index: 2, id: b"second", tool: b"tool" }, AT_ZERO);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 1, bytes: b"earlier" }, AT_ZERO);
    assert_eq!(room.lost, 21);
    assert!(room.sent.is_empty());
}

#[test]
fn interleaved_text_and_input_keep_separate_offsets_and_identities() {
    let (mut live, mut room) = stream(true);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"ab" }, AT_ZERO);
    room.act(&mut live, Action::Call { owner: OWNER, index: 0, id: b"first", tool: b"tool" }, AT_ZERO);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 0, bytes: b"12" }, AT_ZERO);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: b"cd" }, AT_ZERO);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 0, bytes: b"34" }, AT_ZERO);
    room.act(&mut live, Action::Call { owner: OWNER, index: 1, id: b"second", tool: b"other" }, AT_ZERO);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 1, bytes: b"56" }, AT_ZERO);
    room.act(&mut live, Action::End { turn: 1 }, AT_ZERO);
    assert_eq!(room.offsets().as_slice(), [(0, 2), (0, 2), (2, 2), (2, 2), (0, 2)]);
    let mut calls = List::with_capacity(5);
    for piece in &room.sent {
        match &piece.call {
            Some(call) => calls.push(Some(call.id())).expect("five pieces"),
            None => calls.push(None).expect("five pieces"),
        }
    }
    assert_eq!(
        calls.as_slice(),
        [None, Some(b"first".as_slice()), None, Some(b"first".as_slice()), Some(b"second".as_slice())]
    );
    assert_eq!(room.lost, 0);
}

#[test]
fn text_and_calls_share_the_held_bound() {
    let (mut live, mut room) = stream(false);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b't'; LARGE_DELTA] }, AT_ZERO);
    room.act(&mut live, Action::Call { owner: OWNER, index: 0, id: b"call", tool: b"tool" }, AT_ZERO);
    room.act(&mut live, Action::Arguments { owner: OWNER, index: 0, bytes: &[b'i'; LARGE_DELTA] }, AT_ZERO);
    assert_eq!(room.lost, 3616);
    room.takes = true;
    room.act(&mut live, Action::End { turn: 1 }, AT_ZERO);
    assert_eq!(room.lost, 3616);
    let mut bytes: usize = 0;
    for piece in &room.sent {
        bytes = bytes.checked_add(piece.bytes.len()).expect("held bound");
    }
    assert_eq!(bytes, HELD);
}

#[test]
fn new_admission_releases_the_emitted_prefix() {
    let (mut live, mut room) = stream(false);
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &AT_BOUND }, AT_ZERO);
    room.takes = true;
    room.remaining = 1;
    room.act(&mut live, Action::Pump, AT_ZERO.saturating_add(standard().policy.live_interval));
    room.takes = false;
    room.act(&mut live, Action::Delta { owner: OWNER, bytes: &[b'b'; PIECE_BOUND] }, AT_ZERO);
    let mut retained = 0_usize;
    for run in live.first.iter().chain(live.held.iter()) {
        retained = retained.checked_add(run.bytes.len()).expect("held payload bound");
    }
    assert_eq!(retained, HELD, "emitted prefixes do not occupy the held payload allowance");
}
