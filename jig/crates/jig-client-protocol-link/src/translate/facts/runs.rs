//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::Limits;
use crate::generated::client;
use crate::translate::text_problem;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;

pub(crate) fn call_identity(value: &w::CallIdentity, limits: &Limits) -> Result<c::CallIdentity, w::Problem> {
    call_identity_to_domain(value.clone(), limits)
}

fn call_identity_to_domain(value: w::CallIdentity, limits: &Limits) -> Result<c::CallIdentity, w::Problem> {
    Ok(c::CallIdentity {
        owner: skein_codec::convert::token_to_domain(value.owner),
        sequence: value.sequence,
        retry: value.retry,
        id: match c::Text::new(value.id, limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
    })
}

pub(crate) fn run_work_to_domain(value: w::RunWork, limits: &Limits) -> Result<c::RunWork, w::Problem> {
    Ok(match value {
        w::RunWork::Preparing => c::RunWork::Preparing,
        w::RunWork::Starting => c::RunWork::Starting,
        w::RunWork::Active => c::RunWork::Active,
        w::RunWork::Writing => c::RunWork::Writing,
        w::RunWork::Tool(row) => c::RunWork::Tool {
            call: call_identity_to_domain(row.call, limits)?,
            tool: match c::Text::new(row.tool, limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::RunWork::Waiting => c::RunWork::Waiting,
        w::RunWork::Ending => c::RunWork::Ending,
    })
}

pub(crate) fn active_run(value: &w::ActiveRun, limits: &Limits) -> Result<c::ActiveRun, w::Problem> {
    client::active_run_to_domain(value.clone(), limits)
}
