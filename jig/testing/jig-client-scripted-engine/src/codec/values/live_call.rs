//! Wire projections of live call facts (client/domain/testing.md, 4).
use super::super::{bytes, c, w};
use super::wire_call;
pub(in crate::codec) fn wire_call_identity(value: &c::CallIdentity) -> w::CallIdentity {
    w::CallIdentity {
        owner: value.owner.raw(),
        sequence: value.sequence,
        retry: value.retry,
        id: bytes::copy_of(value.id.bytes()),
    }
}

pub(in crate::codec) fn wire_live_call(value: &c::LiveCall) -> Result<w::LiveCall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::LiveCall::Decoded(value) => w::LiveCall::Decoded(w::LiveCallDecoded { value: wire_call(value)? }),
        c::LiveCall::Raw(value) => w::LiveCall::Raw(w::LiveCallRaw { value: wire_current_call(value) }),
    })
}

pub(in crate::codec) fn wire_current_call(value: &c::CurrentCall) -> w::CurrentCall {
    w::CurrentCall {
        identity: wire_call_identity(&value.identity),
        tool: bytes::copy_of(value.tool.bytes()),
        input: bytes::copy_of(value.input.bytes()),
        cut: value.cut,
        began: value.began.as_ref().map(|item| (item).as_nanos()),
    }
}
