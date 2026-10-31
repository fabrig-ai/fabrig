//! Retained delegate endings and funding records (client/domain/contract.md,
//! 5–6; pages.md, 5.2 and 6.5). Rows retain their owner's source fields and
//! both opaque page edges; no result details or amount left is inferred.
use super::{Chip, ChipEnding, Cursor, Funder, Numbers};
use skein_lib::{List, Wall};

/// One ended delegate with its supplied ending and independent optional time.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndedDelegate {
    pub chip: Chip,
    pub ended: ChipEnding,
    pub at: Option<Wall>,
}
/// A whole retained delegate page under the original ended read.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndedPage {
    pub rows: List<EndedDelegate>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub cut: bool,
}
/// The opening and optional next boundary supplied by a period's owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PeriodTimes {
    pub began: Wall,
    pub next: Option<Wall>,
}
/// A retained funding row, distinct from live spend and pool ledgers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FundingRecord {
    pub period: Option<PeriodTimes>,
    pub funder: Funder,
    pub parent: Option<Funder>,
    pub numbers: Numbers,
    pub made: u32,
    pub closed: bool,
}
/// A whole retained spend page under the original project spend read.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpendPage {
    pub rows: List<FundingRecord>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub cut: bool,
}
