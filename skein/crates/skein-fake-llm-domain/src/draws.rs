//! Named draws and bounded answer capture (skein/docs/design/testing/fakes/drawn-agent.md, 3–4).

use skein_lib::{List, Lru, Rng, Set, len_u64, usize_of};

use crate::Config;
use crate::api::{
    Caching, Capture, Drawn, Filled, Finish, Key, Line, Menu, Part, Query, Role, Seeds, Slot, Source, Turn,
};
use crate::limits;

#[derive(Debug)]
struct Parts {
    agent: u64,
    scheduling: Rng,
}

#[derive(Debug)]
struct Attempt {
    turn: u32,
    next: u32,
}

/// The shared or split generators, the current turn of each retained conversation,
/// the supplied replay, and the prefix this fake kept.
#[derive(Debug)]
pub(crate) struct Draws {
    faults: Rng,
    parts: Option<Parts>,
    pub(crate) menu_bound: Option<limits::MenuBound>,
    attempts: Lru<u64, Attempt>,
    replay: Capture,
    kept: List<Drawn>,
    bytes_left: u32,
    full: bool,
}

/// Slot evidence for one answer, copied only within the remaining capture budget.
pub(crate) struct SlotCapture {
    kept: List<Filled>,
    remaining: Option<u32>,
}

impl SlotCapture {
    pub(crate) fn active(&self) -> bool {
        self.remaining.is_some()
    }

    pub(crate) fn record(&mut self, call: u32, id: &[u8], slot: u32, value: &[u8], source: Source, class: &[u8]) {
        self.remaining = slot_room(self.remaining, id, value, class);
        if self.active() {
            let draw = Filled { call, id: id.into(), slot, value: value.into(), source, class: class.into() };
            self.kept.push(draw).expect("at least one byte per kept slot draw");
        }
    }

    pub(crate) fn replay(&mut self, slots: &[Filled], parts: &[Part]) {
        for filled in slots {
            if !self.active() {
                break;
            }
            let Some(Part::ToolCall { id, .. }) = parts.get(usize_of(filled.call)) else {
                unreachable!("replay admission checks the call position");
            };
            self.record(filled.call, id, filled.slot, &filled.value, filled.source, &filled.class);
        }
    }
}

fn slot_room(remaining: Option<u32>, id: &[u8], value: &[u8], class: &[u8]) -> Option<u32> {
    let price = u32::try_from(limits::filled(id, value, class)?).ok()?;
    remaining?.checked_sub(price)
}

impl Draws {
    pub(crate) fn attempt_bytes(config: &Config) -> Option<u64> {
        if config.conversations == 0 {
            // With no entries, neither index ever allocates a node.
            return Some(0);
        }
        Lru::<u64, Attempt>::price(config.conversations)
    }

    pub(crate) fn new(config: &Config, seeds: Seeds, menu: &Menu, replay: Capture) -> Draws {
        let (faults, parts) = match seeds {
            Seeds::One(seed) => (Rng::new(seed), None),
            Seeds::Parts { agent, faults, scheduling } => {
                (Rng::new(faults), Some(Parts { agent, scheduling: Rng::new(scheduling) }))
            }
        };
        Draws {
            faults,
            parts,
            menu_bound: limits::MenuBound::new(menu),
            attempts: Lru::with_capacity(config.conversations),
            replay,
            kept: List::with_capacity(config.capture_answers),
            bytes_left: config.capture_bytes,
            full: false,
        }
    }

    pub(crate) fn faults(&mut self) -> &mut Rng {
        &mut self.faults
    }

    pub(crate) fn scheduling(&mut self) -> &mut Rng {
        match &mut self.parts {
            Some(parts) => &mut parts.scheduling,
            None => &mut self.faults,
        }
    }

    pub(crate) fn agent(&self, key: Key) -> Option<Rng> {
        let parts = self.parts.as_ref()?;
        let opening = mix(parts.agent ^ key.conversation);
        let turn = mix(opening ^ u64::from(key.turn));
        Some(Rng::new(mix(turn ^ u64::from(key.attempt))))
    }

    pub(crate) fn key(&mut self, query: &Query) -> Key {
        let conversation = opening(query);
        let mut turn = 0_u32;
        for message in &query.messages {
            match message.role {
                Role::Assistant => turn = turn.checked_add(1).expect("query admission bounds the message count"),
                Role::User => {}
            }
        }
        let attempt = match self.attempts.get(conversation) {
            Some(entry) if entry.turn == turn => entry.next,
            Some(_) | None => 0,
        };
        Key { conversation, turn, attempt }
    }

    #[expect(clippy::manual_find, reason = "step code searches with an explicit loop, not a closure")]
    pub(crate) fn replay(&self, key: Key) -> Option<&Drawn> {
        for drawn in &self.replay.answers {
            if drawn.key == key {
                return Some(drawn);
            }
        }
        None
    }

    pub(crate) fn slot_capture(&self) -> SlotCapture {
        let remaining = if self.full || self.kept.room() == 0 { None } else { Some(self.bytes_left) };
        SlotCapture { kept: List::with_capacity(self.bytes_left), remaining }
    }

    /// Count the answer even if it is cut short after being drawn.
    pub(crate) fn answered(&mut self, key: Key, parts: &[Part], finish: Finish, tokens: u64, slots: SlotCapture) {
        let _evicted =
            self.attempts.insert(key.conversation, Attempt { turn: key.turn, next: key.attempt.saturating_add(1) });
        if self.full || self.kept.capacity() == 0 {
            return;
        }
        let Some(remaining) = slots.remaining else {
            self.full = true;
            return;
        };
        let Some(price) = limits::drawn_parts(parts) else {
            self.full = true;
            return;
        };
        if price > u64::from(remaining) {
            self.full = true;
            return;
        }
        let mut lines = List::with_capacity(u32::try_from(parts.len()).expect("admitted capture lines"));
        for part in parts {
            let line = match part {
                Part::ToolCall { name, arguments, .. } => {
                    Line::Call { name: name.clone(), arguments: arguments.clone() }
                }
                Part::Text { text } => Line::Text { text: text.clone() },
                Part::Opaque { .. } | Part::ToolOutput { .. } => unreachable!("only drawn calls or text are captured"),
            };
            lines.push(line).expect("room for every part");
        }
        self.bytes_left = remaining
            .checked_sub(u32::try_from(price).expect("price fits the remaining capture"))
            .expect("fits the capture");
        self.kept
            .push(Drawn {
                key,
                answer: Turn { lines: lines.into_boxed(), finish, tokens },
                slots: slots.kept.into_boxed(),
            })
            .expect("room for one more answer");
    }

    pub(crate) fn capture(&self) -> Capture {
        Capture { answers: self.kept.to_boxed(), full: self.full }
    }

    pub(crate) fn classify(&mut self, id: &[u8], slot: u32, class: &[u8]) -> bool {
        if self.full || !crate::take::valid_name(class) {
            return false;
        }
        let added = u32::try_from(class.len()).expect("validated class length");
        for drawn in self.kept.iter_mut() {
            for filled in &mut drawn.slots {
                if &*filled.id != id || filled.slot != slot {
                    continue;
                }
                let old = u32::try_from(filled.class.len()).expect("admitted class length");
                let available = self.bytes_left.checked_add(old).expect("old class was charged to the capture");
                let Some(remaining) = available.checked_sub(added) else {
                    self.full = true;
                    return false;
                };
                filled.class = class.into();
                self.bytes_left = remaining;
                return true;
            }
        }
        false
    }
}

fn mix(seed: u64) -> u64 {
    Rng::new(seed).next_u64()
}

fn opening(query: &Query) -> u64 {
    let basis = 0xcbf2_9ce4_8422_2325;
    let mut hash = match &query.caching {
        Caching::Scope(scope) => field(basis, scope),
        Caching::Unscoped | Caching::Marks(_) => basis,
    };
    hash = field(hash, &query.system);
    if let Some(first) = query.messages.first() {
        for part in &first.parts {
            match part {
                Part::Text { text } => hash = field(hash, text),
                Part::ToolOutput { output, .. } => hash = field(hash, output),
                Part::Opaque { .. } | Part::ToolCall { .. } => {}
            }
        }
    }
    hash
}

fn field(mut hash: u64, bytes: &[u8]) -> u64 {
    let length = len_u64(bytes.len()).to_le_bytes();
    for byte in &length {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3);
    }
    for byte in bytes {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Check the capture's shapes and unique keys after its aggregate bytes have been admitted.
pub(crate) fn valid(replay: &Capture, slots: &[Slot]) -> bool {
    let mut keys = Set::with_capacity(u32::try_from(replay.answers.len()).expect("admitted capture bytes"));
    for drawn in &replay.answers {
        let Key { conversation, turn, attempt } = drawn.key;
        if !keys.insert((conversation, turn, attempt)).expect("room for each replay key") || !valid_turn(&drawn.answer)
        {
            return false;
        }
        for slot in &drawn.slots {
            let line = drawn.answer.lines.get(usize_of(slot.call));
            if !is_call(line)
                || usize_of(slot.slot) >= slots.len()
                || slot.id.is_empty()
                || slot.value.is_empty()
                || slot.value.len() > 256
                || (!slot.class.is_empty() && !crate::take::valid_name(&slot.class))
            {
                return false;
            }
        }
    }
    true
}

fn valid_turn(turn: &Turn) -> bool {
    if turn.lines.is_empty() {
        return false;
    }
    for line in &turn.lines {
        let valid = match line {
            Line::Call { .. } => turn.finish == Finish::ToolCalls,
            Line::Text { .. } => turn.finish == Finish::Stop && turn.lines.len() == 1,
            Line::Cut { .. } | Line::Take { .. } | Line::Fail { .. } | Line::Pieces { .. } | Line::Opaque { .. } => {
                false
            }
        };
        if !valid {
            return false;
        }
    }
    true
}

fn is_call(line: Option<&Line>) -> bool {
    match line {
        Some(Line::Call { .. }) => true,
        Some(
            Line::Text { .. }
            | Line::Cut { .. }
            | Line::Take { .. }
            | Line::Fail { .. }
            | Line::Pieces { .. }
            | Line::Opaque { .. },
        )
        | None => false,
    }
}
