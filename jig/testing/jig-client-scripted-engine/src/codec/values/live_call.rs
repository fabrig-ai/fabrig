//! Wire projections of live call facts (client/domain/testing.md, 4).
use super::super::{bytes, c, w};
use super::wire_call;
pub(in crate::codec) fn wire_call_identity(
    value: &c::CallIdentity,
    limits: &w::Limits,
) -> Result<w::CallIdentity, w::Problem> {
    w::CallIdentity::new(
        limits,
        w::CallIdentityParts {
            owner: value.owner.raw(),
            sequence: value.sequence,
            retry: value.retry,
            id: bytes::copy_of(value.id.bytes()),
        },
    )
}

pub(in crate::codec) fn wire_live_call(value: &c::LiveCall, limits: &w::Limits) -> Result<w::LiveCall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::LiveCall::Decoded(value) => w::LiveCall::Decoded(w::LiveCallDecoded::new(
            limits,
            w::LiveCallDecodedParts { value: wire_call(value, limits)? },
        )?),
        c::LiveCall::Raw(value) => w::LiveCall::Raw(w::LiveCallRaw::new(
            limits,
            w::LiveCallRawParts { value: wire_current_call(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_current_call(
    value: &c::CurrentCall,
    limits: &w::Limits,
) -> Result<w::CurrentCall, w::Problem> {
    w::CurrentCall::new(
        limits,
        w::CurrentCallParts {
            identity: wire_call_identity(&value.identity, limits)?,
            tool: bytes::copy_of(value.tool.bytes()),
            input: bytes::copy_of(value.input.bytes()),
            cut: value.cut,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}
