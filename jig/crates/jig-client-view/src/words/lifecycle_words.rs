//! Checked lifecycle lifecycle sayings (client/domain/view.md, 7).
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
/// Closed lifecycle captions, composed into normal owner words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecyclePhrase {
    Attempt,
    Task,
    Number,
    Claimed,
    ClaimedAbsent,
    Started,
    StartedAbsent,
    Ended,
    EndedAbsent,
    EndAbsent,
    Phases,
    PhasesAbsent,
    Held,
    Position,
    At,
    SourceAbsent,
    EndingAbsent,
}
pub const ALL: [LifecyclePhrase; 17] = [
    LifecyclePhrase::Attempt,
    LifecyclePhrase::Task,
    LifecyclePhrase::Number,
    LifecyclePhrase::Claimed,
    LifecyclePhrase::ClaimedAbsent,
    LifecyclePhrase::Started,
    LifecyclePhrase::StartedAbsent,
    LifecyclePhrase::Ended,
    LifecyclePhrase::EndedAbsent,
    LifecyclePhrase::EndAbsent,
    LifecyclePhrase::Phases,
    LifecyclePhrase::PhasesAbsent,
    LifecyclePhrase::Held,
    LifecyclePhrase::Position,
    LifecyclePhrase::At,
    LifecyclePhrase::SourceAbsent,
    LifecyclePhrase::EndingAbsent,
];
impl LifecyclePhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig.opaque(),
            number: match self {
                Self::Attempt => 8100,
                Self::Task => 8101,
                Self::Number => 8102,
                Self::Claimed => 8103,
                Self::ClaimedAbsent => 8104,
                Self::Started => 8105,
                Self::StartedAbsent => 8106,
                Self::Ended => 8107,
                Self::EndedAbsent => 8108,
                Self::EndAbsent => 8109,
                Self::Phases => 8110,
                Self::PhasesAbsent => 8111,
                Self::Held => 8112,
                Self::Position => 8113,
                Self::At => 8114,
                Self::SourceAbsent => 8115,
                Self::EndingAbsent => 8116,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Task | Self::Number | Self::Phases | Self::Position => Some(HoleKind::Number),
            Self::Claimed | Self::Started | Self::Ended | Self::At => Some(HoleKind::Wall),
            Self::Attempt
            | Self::ClaimedAbsent
            | Self::StartedAbsent
            | Self::EndedAbsent
            | Self::EndAbsent
            | Self::PhasesAbsent
            | Self::Held
            | Self::SourceAbsent
            | Self::EndingAbsent => None,
        }
    }

    fn label(self) -> &'static [u8] {
        match self {
            Self::Attempt => b"Attempt",
            Self::Task => b"Task: ",
            Self::Number => b"Attempt number: ",
            Self::Claimed => b"Claimed: ",
            Self::ClaimedAbsent => b"Claim time not recorded",
            Self::Started => b"Started: ",
            Self::StartedAbsent => b"Start time not recorded",
            Self::Ended => b"Ended: ",
            Self::EndedAbsent => b"End time not recorded",
            Self::EndAbsent => b"Run end not recorded",
            Self::Phases => b"Phase changes: ",
            Self::PhasesAbsent => b"Phase changes not recorded",
            Self::Held => b"Held",
            Self::Position => b"Activity position: ",
            Self::At => b"Hold time: ",
            Self::SourceAbsent => b"Hold source not supplied",
            Self::EndingAbsent => b"Held ending not supplied",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(17);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed lifecycle shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(17);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(phrase.label())), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(phrase.label()))]),
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed lifecycle sayings fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
