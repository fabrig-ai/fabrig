# Contracts of goal 19433: the fake model's slots, named draws and capture

2026-11-27. The shapes the goal's changes share while it runs (`temper/docs/design/draft/design-economy.md`, 4.3): `skein-fake-llm-domain`'s types and entry points that smith's session world builds on. They bind as [drawn-agent.md](drawn-agent.md) does, which says what they mean. The goal's last change deletes this document once the code holds them.

## 1. The menu's slots

In `skein_fake_llm_domain::api`, beside `Menu`:

```rust
/// A placeholder a menu body holds (drawn-agent.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Slot {
    pub name: Box<[u8]>,
    pub before: Box<[u8]>,
    pub after: Box<[u8]>,
    /// The value the world never issues.
    pub unissued: Box<[u8]>,
    /// Per mille: the never-issued value though candidates exist.
    pub weight: u32,
    /// `Some` for a numeric slot.
    pub numeric: Option<Bounds>,
}

/// A numeric slot's inclusive bounds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Bounds {
    pub low: u64,
    pub high: u64,
}

pub struct Menu {
    pub arguments: Box<[Box<[u8]>]>,
    pub invalid: Box<[InvalidInput]>,
    /// At most 8, declared in the order overlap is settled by.
    pub slots: Box<[Slot]>,
}
```

## 2. The seeds

In `skein_fake_llm_domain::api`:

```rust
/// What a fake draws from (drawn-agent.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Seeds {
    /// One seed: one generator draws answers, faults and timing, as before.
    One(u64),
    /// Three parts: `Parties`' `agent/NAME`, `Faults`' `llm`, `Scheduling`'s `llm`.
    Parts { agent: u64, faults: u64, scheduling: u64 },
}
```

## 3. The capture

In `skein_fake_llm_domain::api`:

```rust
/// An answer's key (drawn-agent.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key {
    pub conversation: u64,
    pub turn: u32,
    pub attempt: u32,
}

/// Whether a slot's value came from the conversation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Source {
    Conversation,
    Unissued,
}

/// One slot's draw for one call, beside its answer.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Filled {
    /// The provider id the answer gave the call.
    pub id: Box<[u8]>,
    /// The slot's index in `Menu::slots`.
    pub slot: u32,
    pub value: Box<[u8]>,
    pub source: Source,
    /// The world's class; empty until it gives one.
    pub class: Box<[u8]>,
}

/// One answer the agent drew or a capture replayed.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Drawn {
    pub key: Key,
    /// Lines of `Line::Call` or one `Line::Text`, its finish and its tokens.
    pub answer: Turn,
    pub slots: Box<[Filled]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Capture {
    /// In the order made.
    pub answers: Box<[Drawn]>,
    /// An answer or a class didn't fit its bounds, and nothing after it was kept.
    pub full: bool,
}

impl Capture {
    /// No answer: what a fake that replays nothing is given.
    pub fn none() -> Capture;
}
```

## 4. The configuration and the entry points

In `skein_fake_llm_domain::domain`:

```rust
pub struct Config {
    // ...the fields it has, then:
    /// Conversations whose attempts are counted at once; zero counts none.
    pub conversations: u32,
    /// Answers the capture keeps; zero keeps no capture.
    pub capture_answers: u32,
    /// Owned bytes the capture keeps: its answers, their slots' draws and their classes.
    pub capture_bytes: u32,
}

impl Domain {
    /// `played_by_input` is `drawn(config, Seeds::One(seed), scripts, menu, rules, Capture::none())`.
    pub fn drawn(
        config: &Config,
        seeds: Seeds,
        scripts: Scripts,
        menu: Menu,
        rules: Rules,
        replay: Capture,
    ) -> Result<Domain, Error>;

    /// A copy of what the capture holds.
    pub fn capture(&self) -> Capture;

    /// Gives the captured draw of `slot`, by name, for the call `id` its class;
    /// whether one was found and kept.
    pub fn classify(&mut self, id: &[u8], slot: &[u8], class: &[u8]) -> bool;
}
```

The constructors that take one `seed: u64` keep their signatures and give `Seeds::One(seed)`.
