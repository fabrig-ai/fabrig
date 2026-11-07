//! What a provider says of an account (jig/docs/design/domain/accounts.md, 3.7).
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

pub const FACTS: u32 = 24;
pub const FACT_TEXT: u32 = 255;
pub const FACT_LABEL: u32 = 32;
pub const SHARE_WHOLE: u16 = 10_000;

pub const KIND_EMAIL: u16 = 1;
pub const KIND_NAME: u16 = 2;
pub const KIND_PLAN: u16 = 3;
pub const KIND_WORKSPACE: u16 = 4;
pub const KIND_RENEWS: u16 = 5;
pub const KIND_ENDS: u16 = 6;
pub const KIND_USAGE: u16 = 7;
pub const ORIGIN_SIGN_IN: u16 = 1;
pub const ORIGIN_REFRESH: u16 = 2;
pub const ORIGIN_ANSWER: u16 = 3;
pub const ORIGIN_LIMIT: u16 = 4;
pub const ORIGIN_PROGRAM: u16 = 5;
pub const ORIGIN_USAGE_READ: u16 = 6;
pub const WINDOW_OPEN: u16 = 1;
pub const WINDOW_WARNED: u16 = 2;
pub const WINDOW_LIMITED: u16 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Audience {
    Holder,
    Account,
}

/// A kind's audience; none for a kind this build does not register.
#[must_use]
pub const fn audience(kind: u16) -> Option<Audience> {
    if kind == KIND_EMAIL || kind == KIND_NAME || kind == KIND_RENEWS || kind == KIND_ENDS {
        Some(Audience::Holder)
    } else if kind == KIND_PLAN || kind == KIND_WORKSPACE || kind == KIND_USAGE {
        Some(Audience::Account)
    } else {
        None
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AccountFact {
    pub kind: u16,
    pub label: Box<[u8]>,
    pub value: FactValue,
    pub seen: Wall,
    pub source: Option<u32>,
    pub origin: u16,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FactValue {
    Text(Box<[u8]>),
    Count(u64),
    Instant(Wall),
    Span(Duration),
    Share(u16),
    Window(FactWindow),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FactWindow {
    pub length: Option<Duration>,
    pub used: Option<u16>,
    pub resets: Option<Wall>,
    pub state: u16,
}
impl AccountFact {
    /// Whether this build registers its kind, its origin and, for a window, its state.
    #[must_use]
    pub fn registered(&self) -> bool {
        let state = match &self.value {
            FactValue::Window(window) => (WINDOW_OPEN..=WINDOW_LIMITED).contains(&window.state),
            FactValue::Text(_)
            | FactValue::Count(_)
            | FactValue::Instant(_)
            | FactValue::Span(_)
            | FactValue::Share(_) => true,
        };
        audience(self.kind).is_some() && (ORIGIN_SIGN_IN..=ORIGIN_USAGE_READ).contains(&self.origin) && state
    }
    /// Whether a reader may see it: registered, and of an account's kind or read by its holder.
    #[must_use]
    pub fn shown_to(&self, holder: bool) -> bool {
        self.registered()
            && match audience(self.kind) {
                Some(Audience::Holder) => holder,
                Some(Audience::Account) => true,
                None => false,
            }
    }
}
