//! Harness failures retain their kind and exact detail, never persisted kind data.
//! Preparation labels follow the ends in smith/docs/design/benchmarks.md, 8.3.

use crate::isolate::SetupStep;

/// What kind of failure a harness step met; the fault's detail says which.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    /// A file, directory or process call failed.
    Io,
    /// A record, document or stream did not encode or decode.
    Format,
    /// An authored input, a pin, a configuration or an argument was refused.
    Input,
    /// What was read disagrees with what was pinned or written.
    Mismatch,
    /// A size, time or count bound was passed, or its arithmetic overflowed.
    Bound,
    /// A login is absent, lapsed or changed.
    Credential,
    /// An agent's event stream or its collection failed.
    Stream,
    /// A preparation step refused the attempt before inference.
    Setup(SetupStep),
}

/// One failure of the harness: its kind and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    pub detail: String,
}

impl Fault {
    /// Attach a kind without changing the failure's words.
    pub fn new(kind: FaultKind, detail: impl Into<String>) -> Self {
        Self { kind, detail: detail.into() }
    }

    /// Label a preparation call, preserving a more specific preparation step.
    pub fn at(step: SetupStep) -> impl FnOnce(Self) -> Self {
        move |fault| match fault.kind {
            FaultKind::Setup(_) => fault,
            _ => Self { kind: FaultKind::Setup(step), ..fault },
        }
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for Fault {}

macro_rules! from_errors {
    ($kind:ident: $($error:ty),+ $(,)?) => {
        $(impl From<$error> for Fault {
            fn from(error: $error) -> Self {
                Self::new(FaultKind::$kind, error.to_string())
            }
        })+
    };
}

from_errors!(Io: std::io::Error);
from_errors!(Format: serde_json::Error, toml::de::Error, toml::ser::Error,
    std::string::FromUtf8Error, std::str::Utf8Error, std::num::ParseIntError, std::num::ParseFloatError,
    time::error::ComponentRange, time::error::Format);
from_errors!(Bound: std::num::TryFromIntError);
from_errors!(Input: crate::Refusal);
from_errors!(Stream: crate::agent::codex::ParseError);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_the_detail_alone() {
        let fault = Fault::new(FaultKind::Setup(SetupStep::Seed), "seed refused: exact detail\n");
        assert_eq!(fault.to_string(), fault.detail);
    }

    #[test]
    fn a_missing_file_is_io() {
        let scratch = skein_scratch::Scratch::new("bench-fault");
        let error = std::fs::read(scratch.path().join("missing")).expect_err("missing file");
        assert_eq!(Fault::from(error).kind, FaultKind::Io);
    }
}
