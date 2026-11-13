/// Why a channel, or a dial, ended (protocol/channel/README.md, 8 and 9.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Loss {
    /// As skein's channel ended it: the stream, a framing error, a refusal
    /// sent or received, or the owner.
    Closed(skein_channel::Closed),
    /// Nothing heard for the silence deadline.
    Silence,
    /// Output queued with no frame drained for the stall deadline.
    Stall,
    /// The opening did not finish within the handshake's deadline.
    Handshake,
    /// The output cap filled: the channel refused a record as full.
    Full,
    /// The engine's busy answers outran the table that keeps them.
    Busy,
    /// The root faulted the link (`Event::SourceFault`).
    Fault,
    /// The dial failed (`Event::Unreachable`).
    Unreachable,
    /// The older engine cannot state the agreed versions of the worker's ranges.
    EngineStatesNoWires,
}
