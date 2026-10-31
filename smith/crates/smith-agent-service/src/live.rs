//! Holds admitted runs of main-conversation text and tool input until a piece or interval ends.
//! Never chooses a completion or retains provider bytes past the held-byte bound.
//! `act` advances the buffer through a concrete host connection; `deadline` names its oldest hold.
//! Contracts: smith/docs/design/protocol/channel.md, 7; smith/docs/design/protocol/limits.md, 3.6.
use crate::service::link::HostLink;
use alloc::boxed::Box;
use skein_lib::{Duration, Map, Queue, Time, Token, Writer};
use smith_domain::llm;
use smith_protocol_channel::{self as channel, Piece};

/// Held payload bound, independent of provider delta size (smith/docs/design/protocol/limits.md 3.6).
const HELD: u32 = 16_384;
const PIECE: u32 = smith_channel::CEILINGS.live_bytes;

pub(crate) enum Action<'a> {
    Open { owner: Token, retry: u32 },
    Delta { owner: Token, bytes: &'a [u8] },
    Call { owner: Token, index: u32, id: &'a [u8], tool: &'a [u8] },
    Arguments { owner: Token, index: u32, bytes: &'a [u8] },
    End { turn: u32 },
    Pump,
}

#[derive(Debug)]
struct Stream {
    owner: Token,
    turn: u32,
    retry: u32,
    ended: bool,
}

// At most one identity per held byte, plus the currently open empty call.
const IDENTITIES: u32 = HELD.checked_add(1).expect("held identities bound");

#[derive(Debug)]
struct Call {
    identity: smith_channel::LiveCall,
    held: u32,
}

#[derive(Debug)]
struct HeldRun {
    bytes: Box<[u8]>,
    used: u32,
    arrived: Time,
    call: Option<u32>,
    offset: u64,
}

#[derive(Debug)]
pub(crate) struct Live {
    ask: Option<smith_channel::LiveAsk>,
    last_turn: u32,
    stream: Option<Stream>,
    first: Option<HeldRun>,
    held: Queue<HeldRun>,
    held_bytes: u32,
    interval: Duration,
    calls: Map<u32, Call>,
    current: Option<u32>,
    next: u64,
    input_next: u64,
}

pub(crate) fn wanted(ask: Option<&smith_channel::LiveAsk>, prompt: &llm::Prompt) -> bool {
    let selected = match ask {
        Some(ask) => ask.text() || ask.input(),
        None => false,
    };
    let ordinary = match prompt.purpose {
        llm::Purpose::Ordinary { .. } => true,
        llm::Purpose::Compaction { .. } => false,
    };
    selected && prompt.asker == llm::Asker::Main && ordinary
}

impl Live {
    pub(crate) fn new(ask: Option<smith_channel::LiveAsk>, live_interval: Duration) -> Self {
        Self {
            ask,
            last_turn: 0,
            stream: None,
            first: None,
            held: Queue::with_capacity(HELD),
            held_bytes: 0,
            interval: live_interval,
            calls: Map::with_capacity(IDENTITIES),
            current: None,
            next: 0,
            input_next: 0,
        }
    }

    pub(crate) fn ask(&mut self, ask: Option<smith_channel::LiveAsk>) {
        self.ask = ask;
    }

    pub(crate) fn wants(&self, prompt: &llm::Prompt) -> bool {
        wanted(self.ask.as_ref(), prompt)
    }

    pub(crate) fn deltas(&self, prompt: &llm::Prompt, observed: bool) -> smith_protocol_llm::Deltas {
        let eligible = self.wants(prompt);
        let (text, input) = match &self.ask {
            Some(ask) => (ask.text(), ask.input()),
            None => (false, false),
        };
        smith_protocol_llm::Deltas { text: observed || (eligible && text), input: eligible && input }
    }

    pub(crate) fn act(&mut self, action: Action<'_>, now: Time, room: &mut HostLink<'_>) {
        match action {
            Action::Open { owner, retry } => {
                self.open(owner, self.last_turn.checked_add(1).expect("bounded turns"), retry, room);
            }
            Action::Delta { owner, bytes } => self.delta(owner, bytes, now, room),
            Action::Call { owner, index, id, tool } => self.call(owner, index, id, tool),
            Action::Arguments { owner, index, bytes } => self.arguments(owner, index, bytes, now, room),
            Action::End { turn } => {
                self.end(room);
                self.last_turn = turn;
            }
            Action::Pump => self.pump(now, room),
        }
    }

    fn open(&mut self, owner: Token, turn: u32, retry: u32, room: &mut HostLink<'_>) {
        self.end(room);
        self.stream = Some(Stream { owner, turn, retry, ended: false });
        self.next = 0;
        self.input_next = 0;
    }

    fn call(&mut self, owner: Token, index: u32, id: &[u8], tool: &[u8]) {
        match &self.stream {
            Some(stream) if stream.owner == owner && !stream.ended => {}
            Some(_) | None => return,
        }
        if let Some(previous) = self.current.take()
            && self.calls.get(&previous).expect("open call").held == 0
        {
            self.calls.remove(&previous);
        }
        self.input_next = 0;
        // Validate before allocating the bounded identity, including UTF-8.
        if id.len() > usize::try_from(smith_channel::CEILINGS.live_call_id).expect("identity bound")
            || tool.len() > usize::try_from(smith_channel::CEILINGS.live_call_tool).expect("tool name bound")
        {
            return;
        }
        let Ok(identity) = smith_channel::LiveCall::new(
            &smith_channel::CEILINGS,
            smith_channel::LiveCallParts { id: id.into(), tool: tool.into() },
        ) else {
            return;
        };
        self.calls.insert(index, Call { identity, held: 0 }).expect("one identity per held byte plus open call");
        self.current = Some(index);
    }

    fn arguments(&mut self, owner: Token, index: u32, bytes: &[u8], now: Time, room: &mut HostLink<'_>) {
        match &self.stream {
            Some(stream) if stream.owner == owner => {}
            Some(_) | None => return,
        }
        if self.current != Some(index) {
            room.lose(u64::try_from(bytes.len()).expect("delta length"));
            return;
        }
        self.append(owner, Some(index), bytes, now, room);
    }

    fn delta(&mut self, owner: Token, bytes: &[u8], now: Time, room: &mut HostLink<'_>) {
        match &self.ask {
            Some(ask) if ask.text() => self.append(owner, None, bytes, now, room),
            Some(_) | None => {}
        }
    }

    fn append(&mut self, owner: Token, call: Option<u32>, bytes: &[u8], now: Time, room: &mut HostLink<'_>) {
        let Some(stream) = &self.stream else {
            return;
        };
        if stream.owner != owner {
            return;
        }
        let length = u64::try_from(bytes.len()).expect("provider delta length fits u64");
        if stream.ended {
            room.lose(length);
            return;
        }
        // Release an emitted prefix before allocating another admitted run.
        if let Some(first) = &mut self.first
            && first.used > 0
        {
            first.bytes = Box::from(
                first.bytes.get(usize::try_from(first.used).expect("run cursor")..).expect("retained suffix"),
            );
            first.offset = first.offset.checked_add(u64::from(first.used)).expect("stream bound");
            first.used = 0;
        }
        let next = match call {
            Some(_) => &mut self.input_next,
            None => &mut self.next,
        };
        let count = bytes
            .len()
            .min(usize::try_from(HELD.checked_sub(self.held_bytes).expect("held bound")).expect("held bound"));
        if count > 0 {
            let run = HeldRun {
                bytes: Box::from(bytes.get(..count).expect("admitted prefix")),
                used: 0,
                arrived: now,
                call,
                offset: *next,
            };
            if self.first.is_none() {
                self.first = Some(run);
            } else {
                self.held.push(run);
            }
            self.held_bytes =
                self.held_bytes.checked_add(u32::try_from(count).expect("held bound")).expect("held bound");
        }
        if let Some(index) = call {
            let identity = self.calls.get_mut(&index).expect("open identity");
            identity.held = identity.held.checked_add(u32::try_from(count).expect("held bound")).expect("held bound");
        }
        *next = next.checked_add(length).expect("bounded stream bytes");
        room.lose(u64::try_from(bytes.len().saturating_sub(count)).expect("delta length"));
        self.pump(now, room);
    }

    pub(crate) fn deadline(&self) -> Option<Time> {
        Some(self.first.as_ref()?.arrived.saturating_add(self.interval))
    }

    fn pump(&mut self, now: Time, room: &mut HostLink<'_>) {
        let Some(stream) = &self.stream else {
            return;
        };
        if stream.ended {
            return;
        }
        for _ in 0..HELD {
            let Some(first) = &self.first else {
                break;
            };
            let offset = first.offset.checked_add(u64::from(first.used)).expect("stream bound");
            let call = first.call;
            let due = now >= first.arrived.saturating_add(self.interval);
            let mut count = 0_u32;
            for run in self.first.iter().chain(self.held.iter()) {
                let start = run.offset.checked_add(u64::from(run.used)).expect("stream bound");
                if count == PIECE
                    || run.call != call
                    || start != offset.checked_add(u64::from(count)).expect("stream bound")
                {
                    break;
                }
                let remaining =
                    u32::try_from(run.bytes.len()).expect("held bound").checked_sub(run.used).expect("run cursor");
                count = count
                    .checked_add(remaining.min(PIECE.checked_sub(count).expect("piece bound")))
                    .expect("piece bound");
            }
            // A stream boundary flushes the preceding short piece without mixing streams.
            if count < PIECE && count == self.held_bytes && !due {
                break;
            }
            let kind = match call {
                Some(index) => {
                    let identity = &self.calls.get(&index).expect("held identity").identity;
                    channel::PieceKind::Input { id: identity.id(), tool: identity.tool() }
                }
                None => channel::PieceKind::Text,
            };
            if !room.takes(kind, count) {
                break;
            }
            let mut bytes = Writer::new(usize::try_from(count).expect("piece bound"));
            let mut copied = 0_u32;
            // Each pass consumes a nonempty run or finishes this piece.
            for _ in 0..count {
                if copied == count {
                    break;
                }
                let mut run = self.first.take().expect("held run");
                let remaining =
                    u32::try_from(run.bytes.len()).expect("held bound").checked_sub(run.used).expect("run cursor");
                let length = remaining.min(count.checked_sub(copied).expect("piece cursor"));
                let end = run.used.checked_add(length).expect("run cursor");
                let target_end = copied.checked_add(length).expect("piece cursor");
                bytes
                    .put(
                        run.bytes
                            .get(
                                usize::try_from(run.used).expect("run cursor")
                                    ..usize::try_from(end).expect("run cursor"),
                            )
                            .expect("run slice"),
                    )
                    .expect("measured piece");
                copied = target_end;
                run.used = end;
                self.first = if length == remaining { self.held.pop() } else { Some(run) };
            }
            self.held_bytes = self.held_bytes.checked_sub(count).expect("held count");
            let bytes = bytes.finish();
            room.send(Piece { kind, turn: stream.turn, retry: stream.retry, offset, bytes: &bytes });
            if let Some(index) = call {
                let identity = self.calls.get_mut(&index).expect("held identity");
                identity.held = identity.held.checked_sub(count).expect("held count");
                if identity.held == 0 && self.current != Some(index) {
                    self.calls.remove(&index);
                }
            }
        }
    }

    fn end(&mut self, room: &mut HostLink<'_>) {
        self.pump(Time::from_nanos(u64::MAX), room);
        room.lose(u64::from(self.held_bytes));
        self.first = None;
        self.held.clear();
        self.held_bytes = 0;
        for _ in 0..IDENTITIES {
            let index = match self.calls.iter().next() {
                Some((index, _)) => *index,
                None => break,
            };
            self.calls.remove(&index);
        }
        self.current = None;
        if let Some(stream) = &mut self.stream {
            stream.ended = true;
        }
    }
}

#[cfg(test)]
mod tests;
