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

/// Keep what a report says (accounts.md, 3.7). Save or erase only a changed record.
pub(crate) fn report(
    domain: &mut crate::Domain,
    env: &skein_lib::Env<crate::Limits>,
    account: u32,
    source: u32,
    origin: u16,
    said: crate::Said,
    out: &mut skein_lib::Queue<crate::Request>,
) {
    let held = domain.account_facts(account);
    let mut facts = skein_lib::List::with_capacity(FACTS);
    for fact in held {
        if origin != ORIGIN_SIGN_IN || (fact.origin != ORIGIN_SIGN_IN && fact.origin != ORIGIN_REFRESH) {
            facts.push(fact.clone()).expect("held facts fit their bound");
        }
    }
    for (kind, text) in [(KIND_EMAIL, said.email), (KIND_NAME, said.name)] {
        let Some(text) = text else { continue };
        if !readable(&text) {
            continue;
        }
        let value = FactValue::Text(text);
        let mut seen = env.wall;
        for fact in held {
            if fact.kind == kind
                && fact.label.is_empty()
                && fact.value == value
                && fact.source == Some(source)
                && fact.origin == origin
            {
                seen = fact.seen;
            }
        }
        let next = AccountFact { kind, label: Box::new([]), value, seen, source: Some(source), origin };
        let mut replaced = false;
        for fact in facts.iter_mut() {
            if fact.kind == kind && fact.label.is_empty() {
                *fact = next.clone();
                replaced = true;
            }
        }
        if !replaced {
            drop(facts.push(next));
        }
    }
    keep(domain, account, facts.into_boxed(), out);
}

fn readable(text: &[u8]) -> bool {
    if text.is_empty() || text.len() > skein_lib::length::usize_of(FACT_TEXT) {
        return false;
    }
    for byte in text {
        if byte.is_ascii_control() {
            return false;
        }
    }
    true
}

/// Take the facts that came through a source gone.
pub(crate) fn source_gone(
    domain: &mut crate::Domain,
    account: u32,
    source: u32,
    out: &mut skein_lib::Queue<crate::Request>,
) {
    let mut facts = skein_lib::List::with_capacity(FACTS);
    for fact in domain.account_facts(account) {
        if fact.source != Some(source) {
            facts.push(fact.clone()).expect("removing facts stays within their bound");
        }
    }
    keep(domain, account, facts.into_boxed(), out);
}

fn keep(
    domain: &mut crate::Domain,
    account: u32,
    facts: Box<[AccountFact]>,
    out: &mut skein_lib::Queue<crate::Request>,
) {
    if domain.account_facts(account) == facts.as_ref() {
        return;
    }
    if facts.is_empty() {
        drop(domain.said.remove(&account));
        out.push(crate::Request::Erase { key: crate::Key::Facts { account } });
    } else {
        assert!(domain.said.insert(account, facts.clone()).is_ok(), "one facts record per admitted account");
        out.push(crate::Request::Save { record: crate::Stored::Facts { account, facts } });
    }
}
