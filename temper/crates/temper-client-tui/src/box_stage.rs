//! The box's stages and recording words (temper/docs/design/client/tui/box.md, The box's stages).
use alloc::boxed::Box;
use skein_lib::{
    Decimal,
    bytes::{concat, copy_of, decimal, find},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoxStage {
    Finding,
    Fetching { done: u64, total: Option<u64> },
    Starting,
    Running,
    Stopping,
    Ended { end: BoxEnd },
    Failed { why: Box<[u8]> },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxEnd {
    Exited { code: u8 },
    Killed { signal: u8 },
}
impl BoxStage {
    /// The recording's words, without a line feed.
    #[must_use]
    pub fn print(&self) -> Box<[u8]> {
        match self {
            Self::Finding => copy_of(b"finding"),
            Self::Fetching { done, total } => {
                let done = Decimal::of(*done);
                let total = match total {
                    Some(total) => copy_of(Decimal::of(*total).as_bytes()),
                    None => copy_of(b"-"),
                };
                concat(&[b"fetching ", done.as_bytes(), b" ", &total])
            }
            Self::Starting => copy_of(b"starting"),
            Self::Running => copy_of(b"running"),
            Self::Stopping => copy_of(b"stopping"),
            Self::Ended { end: BoxEnd::Exited { code } } => {
                concat(&[b"ended exited ", Decimal::of(u64::from(*code)).as_bytes()])
            }
            Self::Ended { end: BoxEnd::Killed { signal } } => {
                concat(&[b"ended killed ", Decimal::of(u64::from(*signal)).as_bytes()])
            }
            Self::Failed { why } => concat(&[b"failed ", why]),
        }
    }
    /// The words `print` gives back; `None` for any other.
    #[must_use]
    pub fn read(words: &[u8]) -> Option<BoxStage> {
        match words {
            b"finding" => return Some(Self::Finding),
            b"starting" => return Some(Self::Starting),
            b"running" => return Some(Self::Running),
            b"stopping" => return Some(Self::Stopping),
            _ => {}
        }
        if let Some(why) = words.strip_prefix(b"failed ") {
            return Some(Self::Failed { why: copy_of(why) });
        }
        if let Some(fields) = words.strip_prefix(b"fetching ") {
            let at = find(fields, b" ")?;
            let (done, rest) = fields.split_at_checked(at)?;
            let total = rest.strip_prefix(b" ")?;
            return Some(Self::Fetching {
                done: number(done)?,
                total: if total == b"-" { None } else { Some(number(total)?) },
            });
        }
        if let Some(code) = words.strip_prefix(b"ended exited ") {
            return Some(Self::Ended { end: BoxEnd::Exited { code: u8::try_from(number(code)?).ok()? } });
        }
        if let Some(signal) = words.strip_prefix(b"ended killed ") {
            return Some(Self::Ended { end: BoxEnd::Killed { signal: u8::try_from(number(signal)?).ok()? } });
        }
        None
    }
}
fn number(words: &[u8]) -> Option<u64> {
    let value = decimal(words)?;
    if Decimal::of(value).as_bytes() == words { Some(value) } else { None }
}
