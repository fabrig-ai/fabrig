//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use super::retry_notice;
use crate::Limits;
use crate::translate::text_problem;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v20 as w;
use skein_lib::{Wall, bytes};

pub(crate) fn run_stage(value: &w::RunStage) -> c::RunStage {
    match value {
        w::RunStage::Idle => c::RunStage::Idle,
        w::RunStage::Due => c::RunStage::Due,
        w::RunStage::Preparing => c::RunStage::Preparing,
        w::RunStage::Claimed(row) => c::RunStage::Claimed { attempt: row.attempt() },
        w::RunStage::Running(row) => c::RunStage::Running { attempt: row.attempt() },
        w::RunStage::BackingOff(row) => c::RunStage::BackingOff { until: Wall::from_nanos(row.until()) },
    }
}

pub(crate) fn call_identity(value: &w::CallIdentity, limits: &Limits) -> Result<c::CallIdentity, w::Problem> {
    Ok(c::CallIdentity {
        owner: skein_lib::Token::new(value.owner()),
        sequence: value.sequence(),
        retry: value.retry(),
        id: match c::Text::new(bytes::copy_of(value.id()), limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
    })
}

pub(crate) fn run_work(value: &w::RunWork, limits: &Limits) -> Result<c::RunWork, w::Problem> {
    Ok(match value {
        w::RunWork::Preparing => c::RunWork::Preparing,
        w::RunWork::Starting => c::RunWork::Starting,
        w::RunWork::Active => c::RunWork::Active,
        w::RunWork::Writing => c::RunWork::Writing,
        w::RunWork::Tool(row) => c::RunWork::Tool {
            call: call_identity(row.call(), limits)?,
            tool: match c::Text::new(bytes::copy_of(row.tool()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::RunWork::Waiting => c::RunWork::Waiting,
        w::RunWork::Ending => c::RunWork::Ending,
    })
}

pub(crate) fn run_observation(value: &w::RunObservation, limits: &Limits) -> Result<c::RunObservation, w::Problem> {
    let opt0 = match value.started() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::RunObservation {
        channel: skein_lib::Token::new(value.channel()),
        work: run_work(value.work(), limits)?,
        since: Wall::from_nanos(value.since()),
        reported_at: Wall::from_nanos(value.reported_at()),
        started: opt0,
    })
}

pub(crate) fn active_run(value: &w::ActiveRun, limits: &Limits) -> Result<c::ActiveRun, w::Problem> {
    let opt0 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.placement() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match &value.retry() {
        Some(item) => Some(retry_notice(item)?),
        None => None,
    };
    let opt3 = match &value.observation() {
        Some(item) => Some(run_observation(item, limits)?),
        None => None,
    };
    Ok(c::ActiveRun {
        stage: run_stage(value.stage()),
        phase_since: Wall::from_nanos(value.phase_since()),
        attempt: opt0,
        placement: opt1,
        retry: opt2,
        observation: opt3,
    })
}
