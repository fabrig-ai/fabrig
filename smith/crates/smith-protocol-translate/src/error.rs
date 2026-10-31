//! Checked wire translation failures (protocol/channel.md, sections 2–7).
use skein_channel::SchemaError;

/// A checked channel cannot be built from the supplied limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A codec body exceeds the frame range.
    Codec(smith_channel::Overflow),
    /// Skein rejected the schema or framing limits.
    Channel(SchemaError),
    /// Configured endpoint names exceed their count or byte allowance.
    Endpoints,
    /// The accepted result is outside the configured charter codec limits.
    Result(smith_charter::v4::Problem),
    /// A bounded result list had no room for a field or item.
    ResultCapacity,
    /// A saved answer decodes but fails the domain's sealed result bounds.
    InvalidSavedAnswer,
    /// A result or delivery record does not form the concrete typed value.
    InvalidResult,
    /// A concrete turn cannot be written to the durable transcript format.
    Transcript(smith_domain_session::record::Refusal),
    /// A concrete turn exceeds the transcript codec bounds.
    TranscriptBody(smith_transcript::v6::Problem),
    /// The service sent admission or an answer out of order.
    Order,
    /// A live host operation name is repeated, unknown, or beyond capacity.
    Calls,
    /// The host's outstanding turn or byte window has no remaining credit.
    Window,
    /// A credential generation is stale, repeated, or exceeds the table.
    Grants,
    /// The measured result did not fit its output writer.
    ResultWrite(skein_lib::Overflow),
    /// A domain answer exceeds the configured channel body limits.
    Answer(smith_channel::v5::Problem),
    /// A measured application frame could not be built.
    Frame(skein_channel::FrameError),
}

impl From<smith_channel::v5::Problem> for Error {
    fn from(problem: smith_channel::v5::Problem) -> Error {
        Error::Answer(problem)
    }
}

impl From<smith_transcript::v6::Problem> for Error {
    fn from(problem: smith_transcript::v6::Problem) -> Error {
        Error::TranscriptBody(problem)
    }
}

impl From<skein_channel::FrameError> for Error {
    fn from(problem: skein_channel::FrameError) -> Error {
        Error::Frame(problem)
    }
}

impl From<smith_charter::v4::Problem> for Error {
    fn from(problem: smith_charter::v4::Problem) -> Error {
        Error::Result(problem)
    }
}

impl From<skein_lib::Overflow> for Error {
    fn from(problem: skein_lib::Overflow) -> Error {
        Error::ResultWrite(problem)
    }
}
