//! The box's lines (temper/docs/design/client/tui/box.md).
use alloc::boxed::Box;
use skein_lib::{
    Secret,
    bytes::{concat, copy_of, find},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoxLine {
    Link { url: Secret },
    Log { line: Box<[u8]> },
    Restarting { step: Box<[u8]> },
    Running { step: Box<[u8]> },
    GaveUp { step: Box<[u8]> },
}
impl BoxLine {
    /// The line's bound, including the line feed that `read` never sees.
    pub const MOST: usize = 8192;

    /// One line without its line feed; unreadable lines are dropped.
    #[must_use]
    pub fn read(line: &[u8]) -> Option<BoxLine> {
        if line.len() >= Self::MOST {
            return None;
        }
        let space = find(line, b" ")?;
        let (word, rest) = line.split_at_checked(space)?;
        let field = rest.get(1..)?;
        if field.is_empty() {
            return None;
        }
        match word {
            b"link" => Some(Self::Link { url: Secret::new(copy_of(field)) }),
            b"log" => Some(Self::Log { line: copy_of(field) }),
            b"restarting" => Some(Self::Restarting { step: copy_of(field) }),
            b"running" => Some(Self::Running { step: copy_of(field) }),
            b"gave-up" => Some(Self::GaveUp { step: copy_of(field) }),
            _ => None,
        }
    }

    /// The line without its line feed, as `read` takes it back.
    #[must_use]
    pub fn print(&self) -> Box<[u8]> {
        let (word, field): (&[u8], &[u8]) = match self {
            Self::Link { url } => (b"link", url.expose()),
            Self::Log { line } => (b"log", line),
            Self::Restarting { step } => (b"restarting", step),
            Self::Running { step } => (b"running", step),
            Self::GaveUp { step } => (b"gave-up", step),
        };
        concat(&[word, b" ", field])
    }
}
