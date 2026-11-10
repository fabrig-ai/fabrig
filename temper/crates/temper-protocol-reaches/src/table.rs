//! The project's checked table (temper/docs/design/domain/forge/attention.md, 6).
use crate::{
    Class, Classes, ITEMS, Item, Kind, NAME_BYTES, PATTERN_BYTES, PATTERNS, PathSet, Refusal, SETS, Subject,
    WORDS_BYTES, Way, well_formed,
};
use alloc::boxed::Box;

impl Kind {
    pub const ALL: [Kind; 10] = [
        Self::Wording,
        Self::Retry,
        Self::Implementation,
        Self::Tests,
        Self::Dependency,
        Self::Departure,
        Self::Money,
        Self::Irreversible,
        Self::Rules,
        Self::Other,
    ];
    #[must_use]
    pub const fn fixed(self) -> bool {
        match self {
            Self::Money | Self::Irreversible | Self::Rules | Self::Other => true,
            Self::Wording | Self::Retry | Self::Implementation | Self::Tests | Self::Dependency | Self::Departure => {
                false
            }
        }
    }
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Wording => b"wording",
            Self::Retry => b"retry",
            Self::Implementation => b"implementation",
            Self::Tests => b"tests",
            Self::Dependency => b"dependency",
            Self::Departure => b"departure",
            Self::Money => b"money",
            Self::Irreversible => b"irreversible",
            Self::Rules => b"rules",
            Self::Other => b"other",
        }
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code walks names without closures")]
    pub fn named(name: &[u8]) -> Option<Kind> {
        for kind in Self::ALL {
            if kind.name() == name {
                return Some(kind);
            }
        }
        None
    }
}
impl Class {
    #[must_use]
    pub const fn strictness(self) -> u8 {
        match self {
            Self::Decide => 0,
            Self::Ask => 1,
            Self::AskAway => 2,
            Self::Wait => 3,
        }
    }
}
impl Classes {
    pub fn check(&self) -> Result<(), Refusal> {
        if self.items.len() > ITEMS {
            return Err(Refusal::Items);
        }
        if self.sets.len() > SETS {
            return Err(Refusal::Sets);
        }
        for (index, entry) in self.items.iter().enumerate() {
            let item = u8::try_from(index.saturating_add(1)).expect("bounded items");
            match entry.subject {
                Subject::Kind(kind) => {
                    if kind.fixed() {
                        return Err(Refusal::Fixed { item });
                    }
                }
                Subject::Set(_number) => {}
            }
            if entry.class == Class::Wait {
                return Err(Refusal::Wait { item });
            }
            if entry.words.len() > WORDS_BYTES {
                return Err(Refusal::Words { item });
            }
            match entry.subject {
                Subject::Kind(_kind) => {}
                Subject::Set(number) => {
                    if self.set(number).is_none() {
                        return Err(Refusal::NoSet { item });
                    }
                }
            }
            if entry.words.is_empty() {
                for earlier in self.items.get(..index).expect("earlier items") {
                    if earlier.subject == entry.subject && earlier.words.is_empty() {
                        return Err(Refusal::Twice { item });
                    }
                }
            }
        }
        for (index, entry) in self.sets.iter().enumerate() {
            let set = u8::try_from(index.saturating_add(1)).expect("bounded sets");
            if entry.number == 0 || usize::from(entry.number) > SETS {
                return Err(Refusal::Number { set });
            }
            for earlier in self.sets.get(..index).expect("earlier sets") {
                if earlier.number == entry.number {
                    return Err(Refusal::Number { set });
                }
            }
            if entry.name.is_empty() || entry.name.len() > NAME_BYTES {
                return Err(Refusal::Name { set });
            }
            if entry.patterns.is_empty() || entry.patterns.len() > PATTERNS {
                return Err(Refusal::Patterns { set });
            }
            for (index, pattern) in entry.patterns.iter().enumerate() {
                if pattern.is_empty() || pattern.len() > PATTERN_BYTES || !well_formed(pattern) {
                    return Err(Refusal::Pattern {
                        set,
                        pattern: u8::try_from(index.saturating_add(1)).expect("bounded patterns"),
                    });
                }
            }
        }
        Ok(())
    }
    #[must_use]
    pub fn stock(way: Way) -> Classes {
        match way {
            Way::YouApprove => Classes::default(),
            Way::AgentsReview | Way::DraftFirst => Classes {
                items: Box::new([
                    own(Kind::Wording, Class::Decide),
                    own(Kind::Retry, Class::Decide),
                    own(Kind::Tests, Class::Decide),
                    own(Kind::Implementation, Class::Ask),
                    own(Kind::Dependency, Class::Ask),
                    own(Kind::Departure, Class::Ask),
                ]),
                sets: Box::default(),
            },
        }
    }
    #[must_use]
    pub fn class_of(&self, subject: Subject) -> Class {
        match subject {
            Subject::Kind(kind) => {
                if kind.fixed() {
                    return Class::Wait;
                }
            }
            Subject::Set(_number) => {}
        }
        for item in &self.items {
            if item.subject == subject && item.words.is_empty() {
                return item.class;
            }
        }
        Class::AskAway
    }
    #[must_use]
    pub fn away_may_decide(&self) -> bool {
        for item in &self.items {
            if item.class == Class::Ask {
                return true;
            }
        }
        false
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code walks sets without closures")]
    pub fn set(&self, number: u8) -> Option<&PathSet> {
        for set in &self.sets {
            if set.number == number {
                return Some(set);
            }
        }
        None
    }
}
fn own(kind: Kind, class: Class) -> Item {
    Item { subject: Subject::Kind(kind), words: Box::default(), class }
}
