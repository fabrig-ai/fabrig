//! Committed turns use the engine's transcript codec (client/domain/contract.md, 10).
use super::{List, bytes, c, encoded, problem, w};
use smith_transcript::v6 as t;

pub(super) fn turn_bytes(value: &c::TurnRead, limits: &w::Limits) -> Result<w::TurnBytes, w::Problem> {
    let bytes = match value {
        c::TurnRead::Decoded(value) => turn(value)?,
        c::TurnRead::Undecodable { why: c::TurnProblem::Version { found } } => found.to_be_bytes().into(),
        c::TurnRead::Undecodable { why: c::TurnProblem::Malformed } => Box::new([]),
        c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes } } => {
            return Ok(w::TurnBytes::TooLarge(w::TurnTooLarge::new(limits, w::TurnTooLargeParts { bytes: *bytes })?));
        }
    };
    Ok(w::TurnBytes::Carried(w::TurnCarried::new(limits, w::TurnCarriedParts { bytes })?))
}

fn turn(value: &c::Turn) -> Result<Box<[u8]>, w::Problem> {
    let limits = &t::CEILINGS;
    let mut messages = Vec::new();
    for message in &value.messages {
        let role = match message.speaker {
            c::Speaker::Person => t::Role::User,
            c::Speaker::Assistant => t::Role::Assistant,
            c::Speaker::Tool | c::Speaker::System => return Err(problem()),
        };
        let mut blocks = Vec::new();
        for block in &message.blocks {
            let block = match block {
                c::Block::Text(value) => t::Block::Text(
                    t::Text::new(limits, t::TextParts { text: bytes::copy_of(value.bytes()), replay: None })
                        .map_err(|_| problem())?,
                ),
                c::Block::Call(value) => t::Block::Call(
                    t::Call::new(
                        limits,
                        t::CallParts {
                            id: bytes::copy_of(skein_lib::Decimal::of(value.number).as_bytes()),
                            name: bytes::copy_of(value.tool.bytes()),
                            input: bytes::copy_of(value.input.bytes()),
                            replay: None,
                        },
                    )
                    .map_err(|_| problem())?,
                ),
                c::Block::Result { call, summary, failed } => {
                    let said =
                        t::Said::new(limits, t::SaidParts { text: bytes::copy_of(summary.bytes()), error: *failed })
                            .map_err(|_| problem())?;
                    t::Block::Result(
                        t::ToolResult::new(
                            limits,
                            t::ToolResultParts {
                                id: bytes::copy_of(skein_lib::Decimal::of(*call).as_bytes()),
                                returned: t::Returned::Said(said),
                            },
                        )
                        .map_err(|_| problem())?,
                    )
                }
                c::Block::Opaque { dialect, size } => {
                    let dialect = core::str::from_utf8(dialect.bytes())
                        .map_err(|_| problem())?
                        .parse::<u32>()
                        .map_err(|_| problem())?;
                    if *size > u64::from(limits.opaque_bytes) {
                        return Err(problem());
                    }
                    t::Block::Opaque(
                        t::Opaque::new(
                            limits,
                            t::OpaqueParts {
                                dialect,
                                bytes: vec![0; usize::try_from(*size).map_err(|_| problem())?].into_boxed_slice(),
                            },
                        )
                        .map_err(|_| problem())?,
                    )
                }
            };
            blocks.push(block);
        }
        messages.push(
            t::Message::new(limits, t::MessageParts { role, blocks: List::from_boxed(blocks.into_boxed_slice()) })
                .map_err(|_| problem())?,
        );
    }
    let usage = t::Usage::new(
        limits,
        t::UsageParts {
            input: Some(value.used.input),
            output: Some(value.used.output),
            cache_read: Some(value.used.cached),
            cache_write: Some(value.used.cache_written),
            reasoning: None,
        },
    )
    .map_err(|_| problem())?;
    let turn = t::Turn::new(
        limits,
        t::TurnParts {
            kind: t::TurnKind::Ordinary,
            opening: None,
            endpoint: Box::new([]),
            model: Box::new([]),
            effort: None,
            dialect: 0,
            place: 0,
            usage,
            spent: value.spent,
            jobs: 0,
            messages: List::from_boxed(messages.into_boxed_slice()),
        },
    )
    .map_err(|_| problem())?;
    encoded(&turn, t::Turn::measure, t::Turn::encode)
}
