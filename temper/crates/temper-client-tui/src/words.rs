//! The TUI's phrases beside the view's (temper/docs/design/client/README.md, 6.14 and 7.1).
use crate::look;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, Hole, HoleKind, HoleShape, Invalid, Language, Phrase, PhraseId, PhraseLimits, PhraseShape, Phrasebook,
    PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
};
use temper_client_domain::Limits;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Word {
    Title,
    State,
    Quit,
    QuitKeys,
    SignIn,
    Paste,
    Link,
    LinkShape,
    NotALink,
    OtherAddress,
    SigningIn,
    Busy,
    NotReady,
    NoAnswer,
    Refused,
    Connected,
    Connecting,
    Projects,
    NotKept,
    ApprovalAsking,
    ApprovalTab,
    ApprovalCode,
    ApprovalNoTab,
    ApprovalExpires,
    ApprovalOpen,
    ApprovalCheck,
    ApprovalTunnel,
    ApprovalRefused,
    ApprovalExpired,
    ApprovalUsed,
    ApprovalUnknown,
    ApprovalFull,
    ApprovalBusy,
    ApprovalNotAskedRefused,
    ApprovalNotAskedBusy,
    ApprovalNotAskedDown,
    Separator,
    Between,
    Byline,
    LeftOff,
    NeedsYou,
    Question,
    OtherNeed,
    NoProject,
    Needing,
    NeedingCut,
    AskTemper,
    Message,
    Reply,
    Ended,
    Commands,
    Exit,
    KeyEnter,
    KeyTab,
    KeyPage,
    KeyEsc,
    KeyClose,
    Else,
    Dots,
    Numbered,
}
pub(crate) const ALL: [Word; 60] = [
    Word::Title,
    Word::State,
    Word::Quit,
    Word::QuitKeys,
    Word::SignIn,
    Word::Paste,
    Word::Link,
    Word::LinkShape,
    Word::NotALink,
    Word::OtherAddress,
    Word::SigningIn,
    Word::Busy,
    Word::NotReady,
    Word::NoAnswer,
    Word::Refused,
    Word::Connected,
    Word::Connecting,
    Word::Projects,
    Word::NotKept,
    Word::ApprovalAsking,
    Word::ApprovalTab,
    Word::ApprovalCode,
    Word::ApprovalNoTab,
    Word::ApprovalExpires,
    Word::ApprovalOpen,
    Word::ApprovalCheck,
    Word::ApprovalTunnel,
    Word::ApprovalRefused,
    Word::ApprovalExpired,
    Word::ApprovalUsed,
    Word::ApprovalUnknown,
    Word::ApprovalFull,
    Word::ApprovalBusy,
    Word::ApprovalNotAskedRefused,
    Word::ApprovalNotAskedBusy,
    Word::ApprovalNotAskedDown,
    Word::Separator,
    Word::Between,
    Word::Byline,
    Word::LeftOff,
    Word::NeedsYou,
    Word::Question,
    Word::OtherNeed,
    Word::NoProject,
    Word::Needing,
    Word::NeedingCut,
    Word::AskTemper,
    Word::Message,
    Word::Reply,
    Word::Ended,
    Word::Commands,
    Word::Exit,
    Word::KeyEnter,
    Word::KeyTab,
    Word::KeyPage,
    Word::KeyEsc,
    Word::KeyClose,
    Word::Else,
    Word::Dots,
    Word::Numbered,
];

#[derive(Clone, Copy)]
enum Slot {
    Address,
    Party,
    Projects,
    State,
    Code,
    ApprovalAddress,
    Port,
    Span,
    Number,
    Moment,
    Clock,
}
impl Slot {
    const fn kind(self) -> HoleKind {
        match self {
            Self::Number => HoleKind::Number,
            Self::Moment => HoleKind::Moment,
            Self::Address
            | Self::Party
            | Self::Projects
            | Self::State
            | Self::Code
            | Self::ApprovalAddress
            | Self::Port
            | Self::Span
            | Self::Clock => HoleKind::Name,
        }
    }
    fn bytes(self, domain: &Limits) -> u32 {
        match self {
            Self::Address => 64,
            Self::Code => 9,
            Self::ApprovalAddress => 95,
            Self::Port => 5,
            Self::Span => 32,
            Self::Number => 20,
            Self::Moment | Self::Clock => 13,
            Self::Party => domain.client.domain.text,
            Self::Projects => projects_bytes(domain),
            Self::State => 11, // "Out of date" is the longest state word.
        }
    }
}
/// Whole names and their four-byte separators, leaving room for "Projects: ".
pub(crate) fn projects_bytes(domain: &Limits) -> u32 {
    let count = u64::from(domain.client.domain.projects);
    let names = count.saturating_mul(u64::from(domain.client.domain.text));
    let joined = names.saturating_add(count.saturating_sub(1).saturating_mul(4));
    u32::try_from(joined.min(u64::from(u32::MAX.saturating_sub(10)))).expect("joined bound fits u32")
}
impl Word {
    const fn number(self) -> u16 {
        match self {
            Self::Title => 0,
            Self::State => 1,
            Self::Quit => 2,
            Self::QuitKeys => 3,
            Self::SignIn => 4,
            Self::Paste => 5,
            Self::Link => 6,
            Self::LinkShape => 7,
            Self::NotALink => 8,
            Self::OtherAddress => 9,
            Self::SigningIn => 10,
            Self::Busy => 11,
            Self::NotReady => 12,
            Self::NoAnswer => 13,
            Self::Refused => 14,
            Self::Connected => 15,
            Self::Connecting => 16,
            Self::Projects => 17,
            Self::NotKept => 18,
            Self::ApprovalAsking => 19,
            Self::ApprovalTab => 20,
            Self::ApprovalCode => 21,
            Self::ApprovalNoTab => 22,
            Self::ApprovalExpires => 23,
            Self::ApprovalOpen => 24,
            Self::ApprovalCheck => 25,
            Self::ApprovalTunnel => 26,
            Self::ApprovalRefused => 27,
            Self::ApprovalExpired => 28,
            Self::ApprovalUsed => 29,
            Self::ApprovalUnknown => 30,
            Self::ApprovalFull => 31,
            Self::ApprovalBusy => 32,
            Self::ApprovalNotAskedRefused => 33,
            Self::ApprovalNotAskedBusy => 34,
            Self::ApprovalNotAskedDown => 35,
            Self::Separator => 36,
            Self::Between => 37,
            Self::Byline => 38,
            Self::LeftOff => 39,
            Self::NeedsYou => 40,
            Self::Question => 41,
            Self::OtherNeed => 42,
            Self::NoProject => 43,
            Self::Needing => 44,
            Self::NeedingCut => 45,
            Self::AskTemper => 46,
            Self::Message => 47,
            Self::Reply => 48,
            Self::Ended => 49,
            Self::Commands => 50,
            Self::Exit => 51,
            Self::KeyEnter => 52,
            Self::KeyTab => 53,
            Self::KeyPage => 54,
            Self::KeyEsc => 55,
            Self::KeyClose => 56,
            Self::Else => 57,
            Self::Dots => 58,
            Self::Numbered => 59,
        }
    }
    const fn id(self) -> PhraseId {
        PhraseId { owner: look::OWNER, number: self.number() }
    }
    fn english(self) -> (&'static [&'static [u8]], &'static [Slot]) {
        match self {
            Self::Separator => (&[b" \xc2\xb7 "], &[]),
            Self::Between => (&[b" / "], &[]),
            Self::Byline => (&[b"written by the status agent ", b""], &[Slot::Clock]),
            Self::LeftOff => (&[b"Where you left off"], &[]),
            Self::NeedsYou => (&[b"Needs you"], &[]),
            Self::Question => (&[b"?"], &[]),
            Self::OtherNeed => (&[b"\xe2\x80\xa2"], &[]),
            Self::NoProject => (&[b"Setting temper up makes your first project: open http://", b" in a browser."], &[Slot::Address]),
            Self::Needing => (&[b"", b" need you"], &[Slot::Number]),
            Self::NeedingCut => (&[b"", b"+ need you"], &[Slot::Number]),
            Self::AskTemper => (&[b"Ask temper"], &[]),
            Self::Message => (&[b"Message"], &[]),
            Self::Reply => (&[b"Reply"], &[]),
            Self::Ended => (&[b"/new starts another chat"], &[]),
            Self::Commands => (&[b"Commands"], &[]),
            Self::Exit => (&[b"Close temper"], &[]),
            Self::KeyEnter => (&[b"Enter sends"], &[]),
            Self::KeyTab => (&[b"Tab completes a command"], &[]),
            Self::KeyPage => (&[b"PgUp and PgDn read back"], &[]),
            Self::KeyEsc => (&[b"Esc closes a list"], &[]),
            Self::KeyClose => (&[b"Ctrl+C closes temper"], &[]),
            Self::Else => (&[b"Tell it something else"], &[]),
            Self::Dots => (&[b"\xe2\x80\xa6"], &[]),
            Self::Numbered => (&[b"", b". "], &[Slot::Number]),
            Self::Title => (&[b"temper"], &[]),
            Self::State => (&[b"\xe2\x97\x8f ", b""], &[Slot::State]),
            Self::Quit => (&[b"q to quit"], &[]),
            Self::QuitKeys => (&[b"Ctrl+C to quit"], &[]),
            Self::SignIn => (&[b"Sign in this terminal to ", b""], &[Slot::Address]),
            Self::Paste => (&[b"Paste the link temper printed, then Enter."], &[]),
            Self::Link => (&[b"Link"], &[]),
            Self::LinkShape => (&[b"http://", b"/jig/v1/sign-in/link?secret=\xe2\x80\xa6"], &[Slot::Address]),
            Self::NotALink => (&[b"That is not a sign-in link: paste the whole link temper printed."], &[]),
            Self::OtherAddress => (
                &[b"That link is for ", b". To sign in there, run temper http://", b"."],
                &[Slot::Address, Slot::Address],
            ),
            Self::SigningIn => (&[b"Signing in\xe2\x80\xa6"], &[]),
            Self::Busy => (&[b"Signing in: the engine is busy; trying again."], &[]),
            Self::NotReady => (&[b"Signing in: the engine is starting or stopping; trying again."], &[]),
            Self::NoAnswer => (&[b"Signing in: no answer from the engine; trying again."], &[]),
            Self::Refused => (&[b"This link was refused. Use the fresh one temper printed."], &[]),
            Self::Connected => (&[b"Connected to ", b" as ", b""], &[Slot::Address, Slot::Party]),
            Self::Connecting => (&[b"Connecting to ", b""], &[Slot::Address]),
            Self::Projects => (&[b"Projects: ", b""], &[Slot::Projects]),
            Self::NotKept => (&[b"This sign-in could not be kept: the next temper asks for a link again."], &[]),
            Self::ApprovalAsking => (&[b"Asking temper for a code\xe2\x80\xa6"], &[]),
            Self::ApprovalTab => (&[b"A tab opened in your browser. Check it shows"], &[]),
            Self::ApprovalCode => (&[b"", b", then press Approve."], &[Slot::Code]),
            Self::ApprovalNoTab => (&[b"No tab? Open ", b""], &[Slot::ApprovalAddress]),
            Self::ApprovalExpires => (&[b"The code expires in ", b"."], &[Slot::Span]),
            Self::ApprovalOpen => (&[b"Open this address in a browser that reaches temper:"], &[]),
            Self::ApprovalCheck => (&[b"and check it shows ", b", then press Approve."], &[Slot::Code]),
            Self::ApprovalTunnel => (&[b"From another machine, tunnel port ", b" to this one first."], &[Slot::Port]),
            Self::ApprovalRefused => (&[b"The browser refused this sign-in. Press Enter to ask again."], &[]),
            Self::ApprovalExpired => (&[b"The code expired before it was approved. Press Enter to ask again."], &[]),
            Self::ApprovalUsed => (&[b"That code was already used. Press Enter to ask again."], &[]),
            Self::ApprovalUnknown => (&[b"temper doesn't know that code any more: it restarted, or the code is old. Press Enter to ask again."], &[]),
            Self::ApprovalFull => (&[b"All ", b" sign-ins are in use; the earliest ends ", b" UTC. Press Enter to ask again then."], &[Slot::Number, Slot::Moment]),
            Self::ApprovalBusy => (&[b"temper is busy; still trying\xe2\x80\xa6"], &[]),
            Self::ApprovalNotAskedRefused => (&[b"This temper doesn't sign terminals in from the browser. Paste the link it printed instead."], &[]),
            Self::ApprovalNotAskedBusy => (&[b"Too many terminals are waiting to sign in. Press Enter to ask again in a moment."], &[]),
            Self::ApprovalNotAskedDown => (&[b"temper is starting or stopping, or out of reach. Press Enter to ask again."], &[]),
        }
    }
    pub(crate) fn capacity(self, domain: &Limits) -> Option<u32> {
        let (parts, names) = self.english();
        let mut bytes = 0_u32;
        for part in parts {
            bytes = bytes.checked_add(u32::try_from(part.len()).ok()?)?;
        }
        for name in names {
            bytes = bytes.checked_add(name.bytes(domain))?;
        }
        if self == Self::Needing {
            bytes = bytes.checked_add(1)?;
        }
        Some(bytes)
    }
}

#[derive(Debug)]
pub struct Words {
    book: Phrasebook,
    view: temper_client_view::words::Words,
    pub(crate) projects_bytes: u32,
}
impl Words {
    /// The TUI's English phrases under its owner, with domain-bounded holes.
    pub fn english(view: temper_client_view::words::Words, domain: &Limits) -> Result<Words, Invalid> {
        let mut shapes = List::with_capacity(60);
        let mut phrases = List::with_capacity(60);
        let mut literal_bytes = 0_u32;
        let mut segment_count = 0_u32;
        for word in ALL {
            let (parts, names) = word.english();
            let mut holes = List::with_capacity(2);
            for name in names {
                holes.push(HoleShape { kind: name.kind(), bytes: name.bytes(domain) }).expect("at most two holes");
            }
            let mut segments = List::with_capacity(5);
            for (index, part) in parts.iter().enumerate() {
                literal_bytes = literal_bytes
                    .checked_add(u32::try_from(part.len()).ok().ok_or(Invalid::Overflow)?)
                    .ok_or(Invalid::Overflow)?;
                segments.push(Segment::Text(copy_of(part))).expect("at most five segments");
                if index < names.len() {
                    segments
                        .push(Segment::Hole {
                            index: u16::try_from(index).expect("two holes"),
                            kind: names.get(index).expect("hole has a shape").kind(),
                        })
                        .expect("at most five segments");
                }
            }
            let other = Form { class: PluralClass::Other, segments: segments.into_boxed() };
            let forms: Box<[Form]> = if word == Word::Needing {
                literal_bytes = literal_bytes.checked_add(10).ok_or(Invalid::Overflow)?;
                Box::new([
                    Form {
                        class: PluralClass::One,
                        segments: Box::new([
                            Segment::Text(copy_of(b"")),
                            Segment::Hole { index: 0, kind: HoleKind::Number },
                            Segment::Text(copy_of(b" needs you")),
                        ]),
                    },
                    other,
                ])
            } else {
                Box::new([other])
            };
            for form in &forms {
                segment_count = segment_count
                    .checked_add(u32::try_from(form.segments.len()).ok().ok_or(Invalid::Overflow)?)
                    .ok_or(Invalid::Overflow)?;
            }
            shapes
                .push(PhraseShape {
                    id: word.id(),
                    holes: holes.into_boxed(),
                    plural: if word == Word::Needing { Some(0) } else { None },
                })
                .expect("one shape per word");
            phrases.push(Phrase { id: word.id(), forms }).expect("one phrase per word");
        }
        let language = Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Clock,
            phrases: phrases.into_boxed(),
        };
        let projects_bytes = projects_bytes(domain);
        let limits = PhraseLimits {
            languages: 1,
            phrases: 60,
            holes: 2,
            segments: segment_count,
            text_bytes: literal_bytes,
            hole_bytes: projects_bytes.max(domain.client.domain.text).max(95),
        };
        let book = Phrasebook::load(shapes.into_boxed(), Box::new([language]), &limits)?;
        Ok(Words { book, view, projects_bytes })
    }
    #[must_use]
    pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(0, word.id(), holes).expect("checked TUI phrase and bounded holes").text
    }
    #[must_use]
    pub const fn view(&self) -> &temper_client_view::words::Words {
        &self.view
    }
}
