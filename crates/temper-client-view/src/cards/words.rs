//! Temper conversation and request phrases, separate from jig decision words.
use super::super::words::Words;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{Form, Hole, HoleKind, HoleShape, Phrase, PhraseId, PhraseShape, PluralClass, Segment};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardWord {
    CardLeft,
    Spend,
    Tools,
    Delegation,
    Charter,
    ProcedureCode,
    PersonRole,
    AuthorityBefore,
    AuthorityAfter,
    PatternSegment,
    PatternExact,
    PatternOpen,
    Cut,
    ParameterNumber,
    ParameterBytes,
    ParameterResource,
    Input,
    NoteBits,

    ProposedTasks,
    NoHeldRole,
    ConversationLog,
    WordsBeingSent,
    WordsSent,
    WordsRead,
    Provisional,
    TurnCut,
    TurnUnreadVersion,
    TurnUnreadMalformed,
    TurnUnreadTooLarge,
    You,
    Temper,
    Person,
    Agent,
    CallCompleted,
    CallFailed,
    TurnInstructions,
    OpaqueDialect,
    OpaqueBytes,
    LookAgain,
    ProposeInstead,
    Unknown,
    RequestNotKept,
    RequestsInFlight,
    RequestUnconfirmed,
    HoldRevision,
    InFlight,
    Retrying,
    AnswersFull,
    Parked,
    Done,
    Failed,
    Cancelled,
    Accepted,
    Rejected,
    Answered,
    Released,
    LeftHeld,
    Taken,
    HandedBack,
    Applied,
    AccountSignInStarted,
    AccountSignedOut,
    ToPerson,
    ToRole,
    ToTask,
    ProposalTask,
    ProposalAmend,
    ProposalBudget,
    Result,
    Authority,
}
const ALL: &[CardWord] = &[
    CardWord::CardLeft,
    CardWord::Spend,
    CardWord::Tools,
    CardWord::Delegation,
    CardWord::Charter,
    CardWord::ProcedureCode,
    CardWord::PersonRole,
    CardWord::AuthorityBefore,
    CardWord::AuthorityAfter,
    CardWord::PatternSegment,
    CardWord::PatternExact,
    CardWord::PatternOpen,
    CardWord::Cut,
    CardWord::ParameterNumber,
    CardWord::ParameterBytes,
    CardWord::ParameterResource,
    CardWord::Input,
    CardWord::NoteBits,
    CardWord::ProposedTasks,
    CardWord::NoHeldRole,
    CardWord::ConversationLog,
    CardWord::WordsBeingSent,
    CardWord::WordsSent,
    CardWord::WordsRead,
    CardWord::Provisional,
    CardWord::TurnCut,
    CardWord::TurnUnreadVersion,
    CardWord::TurnUnreadMalformed,
    CardWord::TurnUnreadTooLarge,
    CardWord::You,
    CardWord::Temper,
    CardWord::Person,
    CardWord::Agent,
    CardWord::CallCompleted,
    CardWord::CallFailed,
    CardWord::TurnInstructions,
    CardWord::OpaqueDialect,
    CardWord::OpaqueBytes,
    CardWord::LookAgain,
    CardWord::ProposeInstead,
    CardWord::Unknown,
    CardWord::RequestNotKept,
    CardWord::RequestsInFlight,
    CardWord::RequestUnconfirmed,
    CardWord::HoldRevision,
    CardWord::InFlight,
    CardWord::Retrying,
    CardWord::AnswersFull,
    CardWord::Parked,
    CardWord::Done,
    CardWord::Failed,
    CardWord::Cancelled,
    CardWord::Accepted,
    CardWord::Rejected,
    CardWord::Answered,
    CardWord::Released,
    CardWord::LeftHeld,
    CardWord::Taken,
    CardWord::HandedBack,
    CardWord::Applied,
    CardWord::AccountSignInStarted,
    CardWord::AccountSignedOut,
    CardWord::ToPerson,
    CardWord::ToRole,
    CardWord::ToTask,
    CardWord::ProposalTask,
    CardWord::ProposalAmend,
    CardWord::ProposalBudget,
    CardWord::Result,
    CardWord::Authority,
];
impl CardWord {
    fn data(self) -> (u16, &'static [&'static [u8]], &'static [HoleKind]) {
        use HoleKind::{Name, Number, Wall};
        match self {
            Self::CardLeft => (65, &[b"This proposal is no longer waiting in this chat."], &[]),
            Self::Spend => (48, &[b"May spend ", b""], &[Number]),
            Self::Tools => (49, &[b"Tool permissions ", b""], &[Number]),
            Self::Delegation => (50, &[b"May delegate ", b" tasks, depth ", b""], &[Number, Number]),
            Self::Charter => (51, &[b"Agent charter ", b""], &[Number]),
            Self::ProcedureCode => (52, &[b"Procedure ", b""], &[Number]),
            Self::PersonRole => (53, &[b"Person role ", b""], &[Number]),
            Self::AuthorityBefore => (54, &[b"Authority before"], &[]),
            Self::AuthorityAfter => (55, &[b"Authority after"], &[]),
            Self::PatternSegment => (56, &[b"Resource segment: ", b""], &[Name]),
            Self::PatternExact => (57, &[b"Resource ending: ", b""], &[Name]),
            Self::PatternOpen => (58, &[b"Resource prefix: ", b""], &[Name]),
            Self::Cut => (59, &[b"Some facts were omitted from this document."], &[]),
            Self::ParameterNumber => (60, &[b"Parameter ", b": ", b""], &[Number, Number]),
            Self::ParameterBytes => (61, &[b"Parameter ", b": ", b""], &[Number, Name]),
            Self::ParameterResource => {
                (62, &[b"Parameter ", b": resource ", b" on connector ", b""], &[Number, Number, Number])
            }
            Self::Input => (63, &[b"Input task ", b""], &[Number]),
            Self::NoteBits => (64, &[b"Note permissions ", b""], &[Number]),

            Self::ProposedTasks => (46, &[b"Proposed tasks"], &[]),
            Self::NoHeldRole => (47, &[b"No held role was supplied"], &[]),
            Self::ConversationLog => (0, &[b"Conversation"], &[]),
            Self::WordsBeingSent => (1, &[b"Being sent"], &[]),
            Self::WordsSent => (2, &[b"Sent"], &[]),
            Self::WordsRead => (3, &[b"Read"], &[]),
            Self::Provisional => (4, &[b"Writing..."], &[]),
            Self::TurnCut => (5, &[b"Earlier text omitted"], &[]),
            Self::TurnUnreadVersion => (6, &[b"Cannot read turn version ", b""], &[Number]),
            Self::TurnUnreadMalformed => (7, &[b"Cannot read this turn"], &[]),
            Self::TurnUnreadTooLarge => (8, &[b"This turn is too large to show"], &[]),
            // 68..=79 are occupied by frame words 268..=279.
            Self::You => (80, &[b"You"], &[]),
            Self::Temper => (81, &[b"Temper"], &[]),
            Self::Person => (9, &[b"Person"], &[]),
            Self::Agent => (10, &[b"Agent"], &[]),
            Self::CallCompleted => (11, &[b"Tool completed"], &[]),
            Self::CallFailed => (12, &[b"Tool failed"], &[]),
            Self::TurnInstructions => (13, &[b"Instructions"], &[]),
            Self::OpaqueDialect => (14, &[b"Content in ", b""], &[Name]),
            Self::OpaqueBytes => (15, &[b"", b" bytes"], &[Number]),
            Self::LookAgain => (16, &[b"Look again before deciding."], &[]),
            Self::ProposeInstead => (17, &[b"This needs a proposal to"], &[]),
            Self::Unknown => (18, &[b"The engine no longer knows this request."], &[]),
            Self::RequestNotKept => (19, &[b"The request was not kept."], &[]),
            Self::RequestsInFlight => (20, &[b"Too many requests are pending."], &[]),
            Self::RequestUnconfirmed => (21, &[b"Sign in before sending this request."], &[]),
            Self::HoldRevision => (22, &[b"Hold revision ", b""], &[Number]),
            Self::InFlight => (23, &[b"Pending"], &[]),
            Self::Retrying => (24, &[b"Retrying"], &[]),
            Self::AnswersFull => {
                (45, &[b"The engine holds its ", b" answers of the last day; room again at ", b""], &[Number, Wall])
            }
            Self::Parked => (25, &[b"Waiting to send"], &[]),
            Self::Done => (26, &[b"Done"], &[]),
            Self::Failed => (27, &[b"Failed"], &[]),
            Self::Cancelled => (28, &[b"Cancelled"], &[]),
            Self::Accepted => (29, &[b"Accepted"], &[]),
            Self::Rejected => (30, &[b"Rejected"], &[]),
            Self::Answered => (31, &[b"Answered"], &[]),
            Self::Released => (32, &[b"Released"], &[]),
            Self::LeftHeld => (33, &[b"Left held"], &[]),
            Self::Taken => (34, &[b"Taken"], &[]),
            Self::HandedBack => (35, &[b"Handed back"], &[]),
            Self::Applied => (36, &[b"Applied"], &[]),
            Self::ToPerson => (37, &[b"Person ", b""], &[Number]),
            Self::ToRole => (38, &[b"Role ", b" in project ", b""], &[Number, Number]),
            Self::ToTask => (39, &[b"Task ", b""], &[Number]),
            Self::ProposalTask => (40, &[b"Make task ", b" yours"], &[Number]),
            Self::ProposalAmend => (41, &[b"Amend task ", b""], &[Number]),
            Self::ProposalBudget => (42, &[b"Budget ", b""], &[Number]),
            Self::Result => (43, &[b"Result"], &[]),
            Self::Authority => (44, &[b"Authority"], &[]),
            Self::AccountSignInStarted => (66, &[b"LLM account sign-in started"], &[]),
            Self::AccountSignedOut => (67, &[b"LLM account signed out"], &[]),
        }
    }
    #[must_use]
    pub fn id(self) -> PhraseId {
        PhraseId {
            owner: crate::frame::owner(),
            number: 200_u16.checked_add(self.data().0).expect("closed card phrase ids fit"),
        }
    }
}
#[must_use]
pub fn say(words: &Words, phrase: CardWord, holes: &[Hole]) -> Box<[u8]> {
    words.book.say(words.language, phrase.id(), holes).expect("checked temper card phrase").text
}
pub fn extend(text: u32, shapes: &mut List<PhraseShape>, phrases: &mut List<Phrase>) {
    for word in ALL {
        let (_, literals, kinds) = word.data();
        let mut holes = List::with_capacity(3);
        let mut segments = List::with_capacity(7);
        for (index, literal) in literals.iter().enumerate() {
            segments.push(Segment::Text(copy_of(literal))).expect("card segments fit");
            if let Some(kind) = kinds.get(index) {
                holes
                    .push(HoleShape { kind: *kind, bytes: if *kind == HoleKind::Name { text } else { 0 } })
                    .expect("card holes fit");
                segments
                    .push(Segment::Hole { index: u16::try_from(index).expect("card hole index fits"), kind: *kind })
                    .expect("card segments fit");
            }
        }
        // "May delegate 1 task, depth 1": the delegation's count picks its form.
        let counted = *word == CardWord::Delegation;
        shapes
            .push(PhraseShape {
                id: word.id(),
                holes: holes.into_boxed(),
                plural: if counted { Some(0) } else { None },
            })
            .expect("card shapes fit");
        let other = Form { class: PluralClass::Other, segments: segments.into_boxed() };
        let forms: Box<[Form]> = if counted {
            Box::new([
                Form {
                    class: PluralClass::One,
                    segments: Box::new([
                        Segment::Text(copy_of(b"May delegate ")),
                        Segment::Hole { index: 0, kind: HoleKind::Number },
                        Segment::Text(copy_of(b" task, depth ")),
                        Segment::Hole { index: 1, kind: HoleKind::Number },
                    ]),
                },
                other,
            ])
        } else {
            Box::new([other])
        };
        phrases.push(Phrase { id: word.id(), forms }).expect("card phrases fit");
    }
}
