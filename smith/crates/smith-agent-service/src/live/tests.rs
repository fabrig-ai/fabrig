use super::{Live, Room as ChannelRoom, wanted};
use crate::profile::standard;
use alloc::boxed::Box;
use skein_lib::{List, Time, Token};
use smith_domain::llm;
use smith_protocol_channel::Piece;

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
struct Room {
    takes: bool,
    remaining: u32,
    lost: u64,
    sent: List<Sent>,
}
impl Room {
    fn new(takes: bool) -> Self {
        Self { takes, remaining: u32::MAX, lost: 0, sent: List::with_capacity(super::HELD) }
    }
    fn payloads(&self) -> List<&[u8]> {
        let mut pieces = List::with_capacity(self.sent.len());
        for piece in &self.sent {
            pieces.push(piece.bytes.as_ref()).expect("sent piece count");
        }
        pieces
    }
    fn offsets(&self) -> List<(u64, usize)> {
        let mut offsets = List::with_capacity(self.sent.len());
        for piece in &self.sent {
            offsets.push((piece.offset, piece.bytes.len())).expect("sent piece count");
        }
        offsets
    }
}
impl ChannelRoom for Room {
    fn takes(&self, _: smith_protocol_channel::PieceKind<'_>, _: u32) -> bool {
        self.takes && self.remaining > 0
    }
    fn send(&mut self, piece: Piece<'_>) {
        self.sent
            .push(Sent {
                turn: piece.turn,
                retry: piece.retry,
                offset: piece.offset,
                bytes: Box::from(piece.bytes),
                call: match piece.kind {
                    smith_protocol_channel::PieceKind::Text => None,
                    smith_protocol_channel::PieceKind::Input { id, tool } => Some(
                        smith_channel::LiveCall::new(
                            &smith_channel::CEILINGS,
                            smith_channel::LiveCallParts { id: id.into(), tool: tool.into() },
                        )
                        .expect("valid input identity"),
                    ),
                },
            })
            .expect("piece bound");
        self.remaining = self.remaining.saturating_sub(1);
    }
    fn lose(&mut self, bytes: u64) {
        self.lost = self.lost.saturating_add(bytes);
    }
}
fn ask(text: bool, reasoning: bool, input: bool) -> smith_channel::LiveAsk {
    smith_channel::LiveAsk::new(&smith_channel::CEILINGS, smith_channel::LiveAskParts { text, reasoning, input })
        .expect("ask")
}
fn stream(takes: bool) -> (Live, Room) {
    let mut live = Live::new(Some(ask(true, false, false)), standard().policy.live_interval);
    let mut room = Room::new(takes);
    live.open(OWNER, 1, 0, &mut room);
    (live, room)
}
#[test]
fn a_stream_past_the_held_bytes_drops_the_newest_and_counts_them() {
    let (mut live, mut room) = stream(false);
    live.delta(OWNER, &[b'a'; LARGE_DELTA], AT_ZERO, &mut room);
    live.delta(OWNER, &[b'b'; LARGE_DELTA], AT_ZERO, &mut room);
    assert_eq!(room.lost, 3616, "20,000 provider bytes minus the held bound");
    room.takes = true;
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
    assert_eq!(room.offsets().as_slice(), [(0, 4096), (4096, 4096), (8192, 4096), (12_288, 4096)]);
    live.delta(OWNER, b"c", AT_ZERO, &mut room);
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
    assert_eq!(room.sent.last().expect("later piece").offset, 20_000, "the omitted suffix leaves a gap");
}
#[test]
fn a_delta_past_the_piece_bound_goes_in_several_pieces() {
    let (mut live, mut room) = stream(true);
    live.delta(OWNER, &[b'a'; LARGE_DELTA], AT_ZERO, &mut room);
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
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
        live.delta(OWNER, b"held", AT_ZERO, &mut room);
        room.takes = takes;
        live.end(&mut room);
        assert_eq!((room.sent.len(), room.lost), (pieces, lost), "room={takes}");
        room.takes = true;
        live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
        assert_eq!(room.sent.len(), pieces, "no piece follows the ended turn");
    }
}
#[test]
fn a_delta_after_its_turn_is_dropped_and_counted() {
    let (mut live, mut room) = stream(true);
    live.end(&mut room);
    live.delta(OWNER, b"late", AT_ZERO, &mut room);
    assert_eq!((room.sent.len(), room.lost), (0, 4));
}
#[test]
fn a_new_retry_starts_again_at_offset_zero() {
    let (mut live, mut room) = stream(true);
    live.delta(OWNER, b"first", AT_ZERO, &mut room);
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
    room.takes = false;
    live.delta(OWNER, b"held", AT_ZERO, &mut room);
    live.open(RETRY_OWNER, 1, 1, &mut room);
    room.takes = true;
    live.delta(RETRY_OWNER, b"again", AT_ZERO, &mut room);
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
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
        live.delta(OWNER, input, AT_ZERO, &mut room);
        assert_eq!(room.lost, lost);
        room.takes = true;
        live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
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
    live.delta(OWNER, &[b'a'; LARGE_DELTA], AT_ZERO, &mut room);
    live.delta(OWNER, &[b'b'; LARGE_DELTA], AT_ZERO, &mut room);
    room.takes = true;
    room.remaining = 1;
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
    live.delta(OWNER, b"c", AT_ZERO, &mut room);
    room.remaining = u32::MAX;
    live.pump(AT_ZERO.saturating_add(standard().policy.live_interval), &mut room);
    assert_eq!(room.offsets().as_slice(), [(0, 4096), (4096, 4096), (8192, 4096), (12_288, 4096), (20_000, 1)]);
    assert_eq!(room.lost, 3616);
}
#[test]
fn a_short_piece_goes_at_the_intervals_end() {
    let interval = standard().policy.live_interval;
    let (mut live, mut room) = stream(true);
    live.delta(OWNER, b"Hel", AT_ZERO, &mut room);
    let end = AT_ZERO.saturating_add(interval);
    live.pump(Time::from_nanos(end.as_nanos() - 1), &mut room);
    assert_eq!(room.sent.len(), 0, "held before the interval ends");
    assert_eq!(live.deadline(), Some(end));
    live.pump(end, &mut room);
    assert_eq!(room.payloads().as_slice(), [b"Hel"]);
    assert_eq!(live.deadline(), None);
}

#[test]
fn a_piece_reaching_the_bound_goes_before_the_interval() {
    let (mut live, mut room) = stream(true);
    live.delta(OWNER, &[b'a'; PIECE_BOUND], AT_ZERO, &mut room);
    assert_eq!(room.payloads().as_slice(), [[b'a'; PIECE_BOUND].as_slice()]);
    assert_eq!(live.deadline(), None);
}

#[test]
fn draining_a_piece_retains_the_next_bytes_arrival_time() {
    let interval = standard().policy.live_interval;
    let (mut live, mut room) = stream(true);
    live.delta(OWNER, &[b'a'; PIECE_BOUND - 1], AT_ZERO, &mut room);
    let end = AT_ZERO.saturating_add(interval);
    let later = Time::from_nanos(end.as_nanos() - 1);
    live.delta(OWNER, b"bc", later, &mut room);
    assert_eq!(live.deadline(), Some(later.saturating_add(interval)));
}
#[test]
fn unnamed_refused_and_earlier_calls_arguments_are_counted() {
    let (mut live, mut room) = stream(true);
    live.arguments(OWNER, 0, b"unnamed", AT_ZERO, &mut room);
    live.call(OWNER, 0, &[b'x'; 257], b"tool");
    live.arguments(OWNER, 0, b"refused", AT_ZERO, &mut room);
    live.call(OWNER, 1, b"first", b"tool");
    live.call(OWNER, 2, b"second", b"tool");
    live.arguments(OWNER, 1, b"earlier", AT_ZERO, &mut room);
    assert_eq!(room.lost, 21);
    assert!(room.sent.is_empty());
}

#[test]
fn interleaved_text_and_input_keep_separate_offsets_and_identities() {
    let (mut live, mut room) = stream(true);
    live.delta(OWNER, b"ab", AT_ZERO, &mut room);
    live.call(OWNER, 0, b"first", b"tool");
    live.arguments(OWNER, 0, b"12", AT_ZERO, &mut room);
    live.delta(OWNER, b"cd", AT_ZERO, &mut room);
    live.arguments(OWNER, 0, b"34", AT_ZERO, &mut room);
    live.call(OWNER, 1, b"second", b"other");
    live.arguments(OWNER, 1, b"56", AT_ZERO, &mut room);
    live.end(&mut room);
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
    live.delta(OWNER, &[b't'; LARGE_DELTA], AT_ZERO, &mut room);
    live.call(OWNER, 0, b"call", b"tool");
    live.arguments(OWNER, 0, &[b'i'; LARGE_DELTA], AT_ZERO, &mut room);
    assert_eq!(room.lost, 3616);
    room.takes = true;
    live.end(&mut room);
    assert_eq!(room.lost, 3616);
    let mut bytes: usize = 0;
    for piece in &room.sent {
        bytes = bytes.checked_add(piece.bytes.len()).expect("held bound");
    }
    assert_eq!(bytes, HELD);
}
