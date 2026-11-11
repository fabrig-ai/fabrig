//! Smith payloads are translated in the worker layer (jig's
//! protocol/channel/README.md, 5.1 and 5.4).
use alloc::boxed::Box;
use jig_domain_host as hub;
use jig_protocol_worker_agents::route;
use skein_lib::{List, Reader, Token};
use smith_host_domain as smith;
use temper_worker_domain::{Encodable, Encoded};

pub use route::activity;

#[must_use]
pub fn encode(value: Encodable, agent: &crate::settings::Agent) -> Encoded {
    match value {
        Encodable::Turn(turn) => match route::encode_turn(turn, &agent.limits, &agent.endpoints) {
            Some(turn) => Encoded::Turn(turn),
            None => Encoded::Undecodable,
        },
        Encodable::CallName(name) => match route::encode_call_name(name, &agent.limits) {
            Some(name) => Encoded::CallName(name),
            None => Encoded::Undecodable,
        },
        Encodable::Result(result) => match route::encode_result(result, &agent.limits) {
            Some(result) => Encoded::Result(result),
            None => Encoded::Undecodable,
        },
    }
}

/// Decode the hub start, including its live ask (domain/worker/README.md, 4.1 and 6).
pub fn decode(
    run: Token,
    source: hub::Start,
    agent: &crate::settings::Agent,
) -> Result<Box<smith::Start>, hub::Undecoded> {
    route::decode(run, source, &agent.limits, &agent.host_limits, &agent.endpoints, route::Evidence::Reply)
}

#[must_use]
pub fn process(event: smith_host_protocol::ProcessEvent) -> Option<temper_worker_domain::Event> {
    match route::process(event)? {
        route::Processed::Live { agent, piece } => Some(temper_worker_domain::Event::Live { agent, piece }),
        route::Processed::Process(event) => Some(temper_worker_domain::Event::Process(event)),
    }
}

/// Workspace items named by the assignment, decoded before entering the root.
pub fn items(
    assignment: &hub::Assignment,
    parts: &[jig_protocol_worker_wire::Part],
    limits: &temper_protocol_worker_wire::v1::Limits,
) -> Result<crate::worker::translate::Assigned, temper_worker_domain::wire::Undecodable> {
    let Some(workspace) = &assignment.assignment.workspace else {
        return Ok(crate::worker::translate::workspace(None));
    };
    let bytes = part(workspace.items, parts).ok_or(temper_worker_domain::wire::Undecodable)?;
    let mut reader = Reader::new(bytes);
    let items = temper_protocol_worker_wire::v1::Items::decode(limits, &mut reader)
        .or(Err(temper_worker_domain::wire::Undecodable))?;
    if !reader.is_empty() {
        return Err(temper_worker_domain::wire::Undecodable);
    }
    Ok(crate::worker::translate::workspace(Some(&items)))
}
/// A job's workspace uses the run's items family (temper/docs/design/domain/worker/jobs.md).
pub fn job_items(
    job: &hub::JobAssignment,
    parts: &[jig_protocol_worker_wire::Part],
    limits: &temper_protocol_worker_wire::v1::Limits,
) -> Result<temper_worker_domain::wire::Workspace, temper_worker_domain::wire::Undecodable> {
    let Some(workspace) = &job.workspace else {
        return Ok(crate::worker::translate::workspace(None));
    };
    let bytes = part(workspace.items, parts).ok_or(temper_worker_domain::wire::Undecodable)?;
    let items = temper_protocol_worker_wire::v1::Items::decode(limits, &mut Reader::new(bytes))
        .or(Err(temper_worker_domain::wire::Undecodable))?;
    Ok(crate::worker::translate::workspace(Some(&items)))
}
/// Decode the job's spec at the protocol boundary (temper/docs/design/protocol/tooling.md, 2).
pub fn job_spec(
    bytes: &[u8],
    limits: &temper_protocol_worker_wire::v1::Limits,
) -> Result<temper_worker_domain::wire::JobSpec, temper_worker_domain::wire::Undecodable> {
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(limits, &mut Reader::new(bytes))
        .or(Err(temper_worker_domain::wire::Undecodable))?
        .into_parts();
    Ok(temper_worker_domain::wire::JobSpec { command: spec.command, request: spec.request })
}
fn part(token: Token, parts: &[jig_protocol_worker_wire::Part]) -> Option<&[u8]> {
    for part in parts {
        if part.token == token {
            return Some(&part.bytes);
        }
    }
    None
}

/// An application's granted value, decoded and checked once before being kept.
#[must_use]
pub fn identity(
    value: &skein_lib::Secret,
    limits: &temper_protocol_worker_wire::v1::Limits,
) -> Option<crate::identities::Value> {
    let mut reader = Reader::new(value.expose());
    let identity = temper_protocol_worker_wire::v1::Identity::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    crate::worker::translate::value(&identity)
}

/// Parts only under tokens an answer names; refusals have no application parts.
#[must_use]
pub fn answer_parts(
    answer: &hub::RunAnswer,
    work: &temper_worker_domain::wire::Work,
    preparation: Option<&temper_worker_domain::wire::Preparation>,
) -> Box<[jig_protocol_worker_wire::Part]> {
    let (names, resource) = match &answer.ending {
        hub::Ending::Refused(_) => return Box::new([]),
        hub::Ending::Ended { work, .. } | hub::Ending::Parked { work } => (work, None),
        hub::Ending::Failed { failure, work, .. } => {
            let resource = match failure {
                hub::Failure::Unprepared(hub::Preparation::Permanent { resource }) => *resource,
                hub::Failure::Unprepared(hub::Preparation::Transient)
                | hub::Failure::Run(_)
                | hub::Failure::Agent(_)
                | hub::Failure::Cancelled(_) => None,
            };
            (work, resource)
        }
    };
    resource_parts(names, resource, work, preparation)
}

/// Encode a job's application parts under precisely the tokens its ending names.
#[must_use]
pub(crate) fn job_answer_parts(
    answer: &hub::JobEnding,
    work: &temper_worker_domain::wire::Work,
    preparation: Option<&temper_worker_domain::wire::Preparation>,
) -> Box<[jig_protocol_worker_wire::Part]> {
    let (names, resource) = match answer {
        hub::JobEnding::Refused(_) => return Box::new([]),
        hub::JobEnding::Finished { work, .. } => (work, None),
        hub::JobEnding::Failed { failure, work, .. } => (
            work,
            match failure {
                hub::JobFailure::Unprepared(hub::Preparation::Permanent { resource }) => *resource,
                hub::JobFailure::Unprepared(hub::Preparation::Transient)
                | hub::JobFailure::Failed
                | hub::JobFailure::Deadline
                | hub::JobFailure::Result
                | hub::JobFailure::Unstarted
                | hub::JobFailure::Undelivered(_)
                | hub::JobFailure::Cancelled(_) => None,
            },
        ),
    };
    resource_parts(names, resource, work, preparation)
}

fn resource_parts(
    names: &hub::Work,
    resource: Option<Token>,
    work: &temper_worker_domain::wire::Work,
    preparation: Option<&temper_worker_domain::wire::Preparation>,
) -> Box<[jig_protocol_worker_wire::Part]> {
    use jig_protocol_worker_wire::Part;
    use temper_protocol_worker_wire as payload;
    let mut parts = List::with_capacity(3);
    let (left, saved) = crate::worker::translate::work(work);
    if let Some(token) = names.left {
        parts.push(Part { token, bytes: payload::left(&left) }).expect("three parts");
    }
    if let Some(token) = names.saved
        && let Some(saved) = saved
    {
        parts.push(Part { token, bytes: payload::saved(&saved) }).expect("three parts");
    }
    if let Some(token) = resource
        && let Some(preparation) = preparation
        && let Some(preparation) = crate::worker::translate::preparation(preparation)
    {
        parts.push(Part { token, bytes: payload::preparation(&preparation) }).expect("three parts");
    }
    parts.into_boxed()
}

/// Fit a prepared start within the receiving run and admitted charter bounds.
/// Directories carry the resolved host paths that the agent will receive.
#[must_use]
pub fn fit_start(mut start: smith::Start, limits: &smith_domain_run::Limits, charter_bytes: u64) -> smith::Start {
    let workspace = if start.directories.is_empty() {
        None
    } else {
        let mut directories =
            List::with_capacity(u32::try_from(start.directories.len()).expect("admitted directories"));
        for directory in &start.directories {
            directories
                .push(smith_domain_run::Directory {
                    name: directory.name.clone(),
                    host: directory.host.clone(),
                    root: directory.root,
                    writable: directory.writable,
                    git: directory.git,
                    conflicts: directory.conflicts.clone(),
                })
                .expect("one entry per prepared directory");
        }
        Some(smith_domain_run::Workspace { directories: directories.into_boxed() })
    };
    let mut charter = start.charter.into_value();
    temper_protocol_brief::fit_brief(&mut charter, workspace.as_ref(), limits);
    start.charter =
        smith::Charter::new(charter, charter_bytes).expect("fitted charter within the admitted ownership bound");
    start
}
