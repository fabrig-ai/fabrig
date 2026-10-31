//! Best effort main-conversation text and tool input (smith/docs/design/protocol/channel.md 7; smith/docs/design/protocol/limits.md 3.6).
use skein_channel::Lower;
use skein_lib::{Duration, Map, Queue, Time, Token};
use smith_domain::llm;
use smith_protocol_channel::{self as channel, Piece};

/// Held payload bound, independent of provider delta size (smith/docs/design/protocol/limits.md 3.6).
const HELD: u32 = 16_384;
const PIECE: u32 = 4096;

pub(crate) trait Room {
    fn takes(&self, kind: channel::PieceKind<'_>, bytes: u32) -> bool;
    fn send(&mut self, piece: Piece<'_>);
    fn lose(&mut self, bytes: u64);
}

pub(crate) struct Connection<'a> {
    pub channel: &'a mut channel::Component,
    pub above: &'a mut Queue<channel::OpenEvent>,
    pub below: &'a mut Queue<Lower>,
    pub next: &'a mut u64,
    pub failed: bool,
    pub room: channel::EdgeRoom,
    pub ending: bool,
}

impl Room for Connection<'_> {
    fn takes(&self, kind: channel::PieceKind<'_>, bytes: u32) -> bool {
        let room = self.room;
        let entrances = if self.ending { 2 } else { 1 };
        self.above.room() >= room.to_domain.saturating_mul(entrances)
            && self.below.room() >= room.below.saturating_mul(entrances)
            && self.channel.takes_live(kind, bytes)
    }
    fn send(&mut self, piece: Piece<'_>) {
        let token = Token::new(*self.next);
        *self.next = self.next.checked_add(1).expect("one run does not exhaust send names");
        if self.channel.send_live(piece, token, self.above, self.below).is_err() {
            self.failed = true;
        }
    }
    fn lose(&mut self, bytes: u64) {
        self.channel.lose_live(bytes);
    }
}

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
struct HeldByte {
    byte: u8,
    arrived: Time,
    call: Option<u32>,
    offset: u64,
}

#[derive(Debug)]
pub(crate) struct Live {
    ask: Option<smith_channel::LiveAsk>,
    last_turn: u32,
    stream: Option<Stream>,
    held: Queue<HeldByte>,
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
            held: Queue::with_capacity(HELD),
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
    pub(crate) fn act<R: Room>(&mut self, action: Action<'_>, now: Time, room: &mut R) {
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
    fn open<R: Room>(&mut self, owner: Token, turn: u32, retry: u32, room: &mut R) {
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
        if id.len() > 256 || tool.len() > 64 {
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
    fn arguments<R: Room>(&mut self, owner: Token, index: u32, bytes: &[u8], now: Time, room: &mut R) {
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
    fn delta<R: Room>(&mut self, owner: Token, bytes: &[u8], now: Time, room: &mut R) {
        match &self.ask {
            Some(ask) if ask.text() => self.append(owner, None, bytes, now, room),
            Some(_) | None => {}
        }
    }
    fn append<R: Room>(&mut self, owner: Token, call: Option<u32>, bytes: &[u8], now: Time, room: &mut R) {
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
        let next = match call {
            Some(_) => &mut self.input_next,
            None => &mut self.next,
        };
        let count = bytes.len().min(usize::try_from(self.held.room()).expect("held bound"));
        for (position, byte) in bytes.get(..count).expect("admitted prefix").iter().enumerate() {
            self.held.push(HeldByte {
                byte: *byte,
                arrived: now,
                call,
                offset: next.checked_add(u64::try_from(position).expect("held bound")).expect("stream bound"),
            });
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
        Some(self.held.iter().next()?.arrived.saturating_add(self.interval))
    }
    fn pump<R: Room>(&mut self, now: Time, room: &mut R) {
        let Some(stream) = &self.stream else {
            return;
        };
        if stream.ended {
            return;
        }
        for _ in 0..HELD {
            let Some(first) = self.held.iter().next() else {
                break;
            };
            let offset = first.offset;
            let call = first.call;
            let due = now >= first.arrived.saturating_add(self.interval);
            let mut count: u32 = 0;
            for byte in &self.held {
                if count == PIECE
                    || byte.call != call
                    || byte.offset != offset.checked_add(u64::from(count)).expect("stream bound")
                {
                    break;
                }
                count = count.checked_add(1).expect("piece bound");
            }
            // A stream boundary flushes the preceding short piece without mixing streams.
            if count < PIECE && count == self.held.len() && !due {
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
            let mut bytes = [0; 4096];
            for byte in bytes.get_mut(..usize::try_from(count).expect("piece bound")).expect("piece bound") {
                *byte = self.held.pop().expect("held byte").byte;
            }
            room.send(Piece {
                kind,
                turn: stream.turn,
                retry: stream.retry,
                offset,
                bytes: bytes.get(..usize::try_from(count).expect("piece bound")).expect("piece bound"),
            });
            if let Some(index) = call {
                let identity = self.calls.get_mut(&index).expect("held identity");
                identity.held = identity.held.checked_sub(count).expect("held count");
                if identity.held == 0 && self.current != Some(index) {
                    self.calls.remove(&index);
                }
            }
        }
    }
    fn end<R: Room>(&mut self, room: &mut R) {
        self.pump(Time::from_nanos(u64::MAX), room);
        room.lose(u64::from(self.held.len()));
        for _ in 0..HELD {
            if self.held.pop().is_none() {
                break;
            }
        }
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
