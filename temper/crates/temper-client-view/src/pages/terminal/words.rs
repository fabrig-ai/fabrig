//! The terminal's own phrase table (temper/docs/design/client/README.md, 3.5).
use crate::words::Words;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{Form, Phrase, PhraseId, PhraseShape, PluralClass, Segment};
#[derive(Clone, Copy)]
pub(crate) enum TerminalWord {
    Enter,
    Star,
    Slash,
    Draft,
    Exit,
    Commands,
    Paused,
    Prompt,
}
impl TerminalWord {
    fn data(self) -> (u16, &'static [u8]) {
        match self {
            Self::Enter => (5000, b"Terminal, full screen"),
            Self::Star => (5002, "✦".as_bytes()),
            Self::Slash => (5003, b"/"),
            Self::Draft => (5004, b"Draft"),
            Self::Exit => (5005, b"Exit"),
            Self::Commands => (5006, b"/ for commands"),
            Self::Paused => (5007, "paused · a message resumes it".as_bytes()),
            Self::Prompt => (5008, "❯".as_bytes()),
        }
    }
    fn id(self) -> PhraseId {
        PhraseId { owner: crate::frame::owner(), number: self.data().0 }
    }
}
pub(crate) fn say(w: &Words, word: TerminalWord) -> Box<[u8]> {
    w.book.say(w.language, word.id(), &[]).expect("checked terminal phrase").text
}
pub(crate) fn extend(shapes: &mut List<PhraseShape>, phrases: &mut List<Phrase>) {
    for word in [
        TerminalWord::Enter,
        TerminalWord::Star,
        TerminalWord::Slash,
        TerminalWord::Draft,
        TerminalWord::Exit,
        TerminalWord::Commands,
        TerminalWord::Paused,
        TerminalWord::Prompt,
    ] {
        shapes.push(PhraseShape { id: word.id(), holes: Box::new([]), plural: None }).expect("terminal shapes fit");
        phrases
            .push(Phrase {
                id: word.id(),
                forms: Box::new([Form {
                    class: PluralClass::Other,
                    segments: Box::new([Segment::Text(copy_of(word.data().1))]),
                }]),
            })
            .expect("terminal phrases fit");
    }
}
