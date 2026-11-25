//! Job refusals shared by client views (jig/docs/design/domain/engine/fleet.md, 2).

use super::Word;
use crate::contract::{Figures, JobShort, JobWhy};
use alloc::boxed::Box;
use skein_lib::{Writer, bytes::copy_of};
use skein_ui::phrases::{Hole, HoleKind, Phrasebook, Segment};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobWord {
    Worker,
    Engine,
    Kind,
    Spec,
    Result,
    Deadline,
    Untranslated,
    CannotTake,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Needs,
    Amount,
}

impl JobWord {
    pub(super) const fn data(self) -> (u16, &'static [u8], &'static [HoleKind]) {
        use HoleKind::Number;
        let (id, holes): (u16, &[HoleKind]) = match self {
            Self::Worker => (156, &[Number]),
            Self::Engine => (157, &[]),
            Self::Kind => (158, &[Number]),
            Self::Spec => (159, &[Number, Number]),
            Self::Result => (160, &[Number, Number]),
            Self::Deadline => (161, &[Number]),
            Self::Untranslated => (162, &[]),
            Self::CannotTake => (163, &[Number]),
            Self::Hours => (164, &[Number]),
            Self::Minutes => (165, &[Number]),
            Self::Seconds => (166, &[Number]),
            Self::Milliseconds => (167, &[Number]),
            Self::Needs => (168, &[]),
            Self::Amount => (169, &[Number]),
        };
        (id, b"", holes)
    }

    pub(super) fn segments(self) -> Box<[Segment]> {
        match self {
            Self::Worker => Box::new([text(b"worker "), number(0)]),
            Self::Engine => Box::new([text(b"the engine")]),
            Self::Kind | Self::CannotTake => Box::new([
                text(if self == Self::Kind { b" runs no job of kind " } else { b" cannot take job kind " }),
                number(0),
            ]),
            Self::Spec | Self::Result => Box::new([
                text(b" takes job kind "),
                number(0),
                text(if self == Self::Spec { b"'s spec up to " } else { b"'s result up to " }),
                number(1),
                text(b" bytes"),
            ]),
            Self::Deadline => Box::new([text(b" takes job kind "), number(0), text(b" for up to ")]),
            Self::Untranslated => Box::new([text(b"'s layouts cannot keep this job")]),
            Self::Hours => Box::new([number(0), text(b" h")]),
            Self::Minutes => Box::new([number(0), text(b" min")]),
            Self::Seconds => Box::new([number(0), text(b" s")]),
            Self::Milliseconds => Box::new([number(0), text(b" ms")]),
            Self::Needs => Box::new([text(b"; this job's kind needs ")]),
            Self::Amount => Box::new([number(0)]),
        }
    }
}

fn text(bytes: &[u8]) -> Segment {
    Segment::Text(copy_of(bytes))
}

const fn number(index: u16) -> Segment {
    Segment::Hole { index, kind: HoleKind::Number }
}

/// Say a derived job wait in the selected vocabulary.
#[must_use]
pub fn job_wait(
    book: &Phrasebook,
    language: u16,
    worker: Option<u32>,
    kind: u32,
    why: JobWhy,
    figures: Option<Figures>,
) -> Box<[u8]> {
    use JobShort as S;
    use JobWord as W;
    let host = match worker {
        Some(worker) => say(book, language, W::Worker, &[Hole::Number(u64::from(worker))]),
        None => say(book, language, W::Engine, &[]),
    };
    let kind = Hole::Number(u64::from(kind));
    match why {
        JobWhy::Untranslated => join(&[&host, &say(book, language, W::Untranslated, &[])]),
        JobWhy::Short(S::Kind) => join(&[&host, &say(book, language, W::Kind, &[kind])]),
        JobWhy::Short(short @ (S::Spec | S::Result | S::Deadline)) => {
            let Some(figures) = figures else {
                return join(&[&host, &say(book, language, W::CannotTake, &[kind])]);
            };
            let needs = say(book, language, W::Needs, &[]);
            match short {
                S::Spec | S::Result => {
                    let word = if short == S::Spec { W::Spec } else { W::Result };
                    let offer = say(book, language, word, &[kind, Hole::Number(figures.offered)]);
                    let required = say(book, language, W::Amount, &[Hole::Number(figures.required)]);
                    join(&[&host, &offer, &needs, &required])
                }
                S::Deadline => join(&[
                    &host,
                    &say(book, language, W::Deadline, &[kind]),
                    &span(book, language, figures.offered),
                    &needs,
                    &span(book, language, figures.required),
                ]),
                S::Kind => unreachable!("kind has no figures"),
            }
        }
    }
}

fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("job wait phrase length");
    }
    let mut joined = Writer::new(length);
    for part in parts {
        joined.put(part).expect("measured job wait phrase");
    }
    joined.finish()
}

fn span(book: &Phrasebook, language: u16, millis: u64) -> Box<[u8]> {
    for (unit, word) in [(3_600_000, JobWord::Hours), (60_000, JobWord::Minutes), (1_000, JobWord::Seconds)] {
        if millis != 0 && millis.is_multiple_of(unit) {
            return say(book, language, word, &[Hole::Number(millis.div_euclid(unit))]);
        }
    }
    say(book, language, JobWord::Milliseconds, &[Hole::Number(millis)])
}

fn say(book: &Phrasebook, language: u16, word: JobWord, holes: &[Hole]) -> Box<[u8]> {
    book.say(language, Word::Job(word).id(), holes).expect("checked job words").text
}
