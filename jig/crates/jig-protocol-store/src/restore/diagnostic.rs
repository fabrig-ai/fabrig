//! Key-only restore diagnostics (jig/docs/design/protocol/store.md, 7.1 and 9).
use crate::{Family, Phase, Store};
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Failure {
    pub cause: Cause,
    pub row: Option<RowSite>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RowSite {
    pub family: Option<Family>,
    pub key: Box<[u8]>,
    pub key_bytes: u64,
    pub truncated: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Cause {
    Phase { found: Phase },
    Range(RangeFailure),
    Walk(WalkFailure),
    Decode(DecodeFailure),
    Join { target: RowSite, failure: JoinFailure },
    Arithmetic(Arithmetic),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeFailure {
    Encoding,
    Cursor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WalkFailure {
    NoTerminal,
    UnexpectedTerminal,
    Refused(skein_kv::Refusal),
    Failed(skein_kv::Failure),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecodeFailure {
    Key,
    Value,
    KeyValueDisagree,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JoinFailure {
    KeyEncoding,
    NoTerminal,
    UnexpectedTerminal,
    Refused(skein_kv::Refusal),
    Failed(skein_kv::Failure),
    Decode(DecodeFailure),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arithmetic {
    RangeCount,
    CursorLength,
    RowBytes,
    PageBytes,
    JoinedBytes,
}

impl Failure {
    pub(crate) fn rowless(cause: Cause) -> Self {
        Self { cause, row: None }
    }
    pub(crate) fn at(cause: Cause, row: RowSite) -> Self {
        Self { cause, row: Some(row) }
    }
}

pub(crate) fn ready(store: &Store) -> Result<(), Failure> {
    match store.phase {
        Phase::Ready { .. } => Ok(()),
        Phase::Closed | Phase::Stopped => Err(Failure::rowless(Cause::Phase { found: store.phase })),
    }
}

pub(crate) fn site(store: &Store, key: &[u8]) -> RowSite {
    let allowance = usize::try_from(store.limits.store.key).expect("map key allowance fits usize");
    let shown = key.get(..key.len().min(allowance)).expect("bounded key prefix");
    let family = match key {
        [crate::keys::COMPONENT, ..] => Some(Family::Component),
        [crate::keys::APPLICATION, ..] => Some(Family::Application),
        [crate::keys::JIG, 0, 8, ..] => Some(Family::Settings),
        [crate::keys::JIG, 0, ..] => Some(Family::Core),
        [crate::keys::JIG, 1, ..] => Some(Family::Tasks),
        [crate::keys::JIG, 2, ..] => Some(Family::People),
        [crate::keys::JIG, 3, ..] => Some(Family::Notes),
        [crate::keys::JIG, crate::keys::ACCOUNTS, ..] => Some(Family::Accounts),
        _ => None,
    };
    RowSite {
        family,
        key: Box::from(shown),
        key_bytes: u64::try_from(key.len()).expect("key length fits u64"),
        truncated: key.len() > shown.len(),
    }
}
