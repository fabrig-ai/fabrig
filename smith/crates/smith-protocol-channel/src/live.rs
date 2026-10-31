//! Text pieces at the channel boundary (protocol/channel.md, sections 4 and 7).
use crate::Error;
use alloc::boxed::Box;
use skein_channel::Frame;

/// The stream selected by the service for a live piece.
#[derive(Clone, Copy, Debug)]
pub enum PieceKind<'a> {
    /// Main-conversation text before its turn commits.
    Text,
    /// One call's arguments, named by the provider's identity.
    Input { id: &'a [u8], tool: &'a [u8] },
}

/// A piece cut by the service, before its turn's record.
#[derive(Debug)]
pub struct Piece<'a> {
    pub kind: PieceKind<'a>,
    pub turn: u32,
    pub retry: u32,
    pub offset: u64,
    pub bytes: &'a [u8],
}

pub(crate) fn frame(piece: Piece<'_>, limits: &smith_channel::Limits) -> Result<Frame, Error> {
    let (kind, call) = match piece.kind {
        PieceKind::Text => (smith_channel::LiveKind::Text, None),
        PieceKind::Input { id, tool } => (
            smith_channel::LiveKind::Input,
            Some(
                smith_channel::LiveCall::new(
                    limits,
                    smith_channel::LiveCallParts { id: Box::from(id), tool: Box::from(tool) },
                )
                .map_err(Error::ChannelBody)?,
            ),
        ),
    };
    let record = smith_channel::Live::new(
        limits,
        smith_channel::LiveParts {
            turn: piece.turn,
            retry: piece.retry,
            kind,
            offset: piece.offset,
            bytes: Box::from(piece.bytes),
            call,
        },
    )
    .map_err(Error::ChannelBody)?;
    let frame = smith_channel::frame(smith_channel::Body::Live(record)).map_err(Error::Frame)?;
    Ok(frame)
}

pub(crate) fn frame_bytes(limits: &smith_channel::Limits, bytes: u32) -> Option<u32> {
    if bytes > limits.live_bytes || bytes > smith_channel::CEILINGS.live_bytes {
        return None;
    }
    let limits = smith_channel::Limits { live_bytes: bytes, ..*limits };
    u32::try_from(smith_channel::Live::encoded_bound(&limits)?).ok()?.checked_add(8)
}
