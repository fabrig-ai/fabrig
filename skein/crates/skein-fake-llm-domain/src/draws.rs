//! Named draws and bounded answer capture (skein/docs/design/testing/fakes/drawn-agent.md, 3–4).

use alloc::boxed::Box;
use skein_lib::{List, Lru, Rng, Set, len_u64};

use crate::Config;
use crate::api::{Caching, Capture, Drawn, Finish, Key, Line, Part, Query, Role, Seeds, Turn};
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
    attempts: Lru<u64, Attempt>,
    replay: Capture,
    kept: List<Drawn>,
    bytes_left: u32,
    full: bool,
}

impl Draws {
    pub(crate) fn attempt_bytes(config: &Config) -> Option<u64> {
        Lru::<u64, Attempt>::price(config.conversations)
    }

    pub(crate) fn new(config: &Config, seeds: Seeds, replay: Capture) -> Draws {
        let (faults, parts) = match seeds {
            Seeds::One(seed) => (Rng::new(seed), None),
            Seeds::Parts { agent, faults, scheduling } => {
                (Rng::new(faults), Some(Parts { agent, scheduling: Rng::new(scheduling) }))
            }
        };
        Draws {
            faults,
            parts,
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

    pub(crate) fn replay(&self, key: Key) -> Option<&Turn> {
        for drawn in &self.replay.answers {
            if drawn.key == key {
                return Some(&drawn.answer);
            }
        }
        None
    }

    /// Count the answer even if it is cut short after being drawn.
    pub(crate) fn answered(&mut self, key: Key, parts: &[Part], finish: Finish, tokens: u64) {
        let _evicted =
            self.attempts.insert(key.conversation, Attempt { turn: key.turn, next: key.attempt.saturating_add(1) });
        if self.full || self.kept.capacity() == 0 {
            return;
        }
        let Some(price) = limits::drawn_parts(parts) else {
            self.full = true;
            return;
        };
        if self.kept.room() == 0 || price > u64::from(self.bytes_left) {
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
        self.bytes_left = self
            .bytes_left
            .checked_sub(u32::try_from(price).expect("price fits the remaining capture"))
            .expect("fits the capture");
        self.kept
            .push(Drawn { key, answer: Turn { lines: lines.into_boxed(), finish, tokens }, slots: Box::new([]) })
            .expect("room for one more answer");
    }

    pub(crate) fn capture(&self) -> Capture {
        Capture { answers: self.kept.to_boxed(), full: self.full }
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
pub(crate) fn valid(replay: &Capture) -> bool {
    let mut keys = Set::with_capacity(u32::try_from(replay.answers.len()).expect("admitted capture bytes"));
    for drawn in &replay.answers {
        let Key { conversation, turn, attempt } = drawn.key;
        if !keys.insert((conversation, turn, attempt)).expect("room for each replay key") || !valid_turn(&drawn.answer)
        {
            return false;
        }
        for slot in &drawn.slots {
            if slot.id.is_empty() || slot.value.is_empty() || slot.value.len() > 256 || !valid_class(&slot.class) {
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

fn valid_class(class: &[u8]) -> bool {
    if class.is_empty() || class.len() > 32 {
        return false;
    }
    for byte in class {
        if !byte.is_ascii_alphanumeric() && *byte != b'_' {
            return false;
        }
    }
    true
}
