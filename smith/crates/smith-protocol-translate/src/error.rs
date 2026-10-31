//! Checked wire translation failures (protocol/channel.md, sections 2–7).
use skein_channel::SchemaError;

/// A wire translation or channel operation exceeded its bounds or order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A codec body exceeds the frame range.
    Codec(smith_channel::Overflow),
    /// Skein rejected the schema or framing limits.
    Channel(SchemaError),
    /// Configured endpoint names exceed their count or byte allowance.
    Endpoints,
    /// The accepted result is outside the configured charter codec limits.
    CharterBody(smith_charter::v4::Problem),
    /// A bounded result list had no room for a field or item.
    Capacity,
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
    Write(skein_lib::Overflow),
    /// A domain answer exceeds the configured channel body limits.
    ChannelBody(smith_channel::v5::Problem),
    /// A measured application frame could not be built.
    Frame(skein_channel::FrameError),
}
