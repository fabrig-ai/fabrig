//! Closed owner budget/ledger sayings (client/domain/view.md, section 7).
//! IDs 6000..6012 are independent of the shared Funder sayings.
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};

/// Exact accounting labels and explicit ledger state/timing labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetPhrase {
    Budget,
    Spent,
    SpentBelow,
    Reserved,
    Left,
    Funder,
    Deadline,
    Source,
    Parent,
    Open,
    Closed,
    Began,
    Next,
}
/// Stable independent phrase order for normal Words composition.
pub const ALL: [BudgetPhrase; 13] = [
    BudgetPhrase::Budget,
    BudgetPhrase::Spent,
    BudgetPhrase::SpentBelow,
    BudgetPhrase::Reserved,
    BudgetPhrase::Left,
    BudgetPhrase::Funder,
    BudgetPhrase::Deadline,
    BudgetPhrase::Source,
    BudgetPhrase::Parent,
    BudgetPhrase::Open,
    BudgetPhrase::Closed,
    BudgetPhrase::Began,
    BudgetPhrase::Next,
];
impl BudgetPhrase {
    /// Identity available to the normal checked Words composition.
    #[must_use]
    pub fn id(self) -> PhraseId {
        let number = match self {
            Self::Budget => 6000,
            Self::Spent => 6001,
            Self::SpentBelow => 6002,
            Self::Reserved => 6003,
            Self::Left => 6004,
            Self::Funder => 6005,
            Self::Deadline => 6006,
            Self::Source => 6007,
            Self::Parent => 6008,
            Self::Open => 6009,
            Self::Closed => 6010,
            Self::Began => 6011,
            Self::Next => 6012,
        };
        PhraseId { owner: Owner::Jig.opaque(), number }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Budget | Self::Spent | Self::SpentBelow | Self::Reserved | Self::Left => Some(HoleKind::Amount),
            Self::Deadline | Self::Began | Self::Next => Some(HoleKind::Wall),
            Self::Funder | Self::Source | Self::Parent | Self::Open | Self::Closed => None,
        }
    }
}

/// Budget-only closed shapes; shared Funder shapes are composed by their owner.
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut shapes = List::with_capacity(13);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        shapes.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed budget shapes fit");
    }
    shapes.into_boxed()
}

/// Budget-only English data for selected-language composition, never built by rendering.
#[must_use]
pub fn language() -> Language {
    let mut phrases = List::with_capacity(13);
    for phrase in ALL {
        let text: &[u8] = match phrase {
            BudgetPhrase::Budget => b"Budget: ",
            BudgetPhrase::Spent => b"Spent directly: ",
            BudgetPhrase::SpentBelow => b"Settled spend below: ",
            BudgetPhrase::Reserved => b"Reserved for funded work: ",
            BudgetPhrase::Left => b"Left: ",
            BudgetPhrase::Funder => b"Funded by",
            BudgetPhrase::Deadline => b"Deadline: ",
            BudgetPhrase::Source => b"Ledger source",
            BudgetPhrase::Parent => b"Parent funder",
            BudgetPhrase::Open => b"Ledger is open",
            BudgetPhrase::Closed => b"Ledger is closed",
            BudgetPhrase::Began => b"Period began: ",
            BudgetPhrase::Next => b"Next period begins: ",
        };
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(text)), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(text))]),
        };
        phrases
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed budget phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: phrases.into_boxed(),
    }
}
