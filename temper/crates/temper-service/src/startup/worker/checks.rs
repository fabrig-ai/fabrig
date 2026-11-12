//! Worker checks, also called by paired startup (protocol/services.md, 5).
use crate::settings;
use jig_protocol_config::{self as config, Path};
use temper_protocol_worker_wire as payload;
use temper_service_core::startup::problems::{Problems, What};
use temper_service_core::startup::{at, own, short};
pub(crate) fn bounded<E>(
    bounds: &payload::Bounds,
    engine: bool,
    problems: &mut Problems<What<E>>,
) -> Option<payload::v1::Limits> {
    let ceilings = payload::v1::CEILINGS;
    let repositories: &[u8] = b"checkout.repositories";
    let names: &[u8] = if engine { b"forge.name_bytes" } else { b"checkout.name_bytes" };
    for (name, bytes, bound) in [
        (
            repositories,
            bounds.repositories,
            ceilings.items_repositories.min(ceilings.left_landed).min(ceilings.saved_repositories),
        ),
        (
            names,
            bounds.name_bytes,
            ceilings
                .repository_name
                .min(ceilings.merge_branch)
                .min(ceilings.branch_branch)
                .min(ceilings.writable_push)
                .min(ceilings.writable_landing_some)
                .min(ceilings.repository_save_some),
        ),
        (b"checkout.name_bytes", bounds.remote_bytes, ceilings.repository_remote),
        (b"checkout.conflicts", bounds.conflicts, ceilings.files_files),
        (b"checkout.path_bytes", bounds.path_bytes, ceilings.files_files_item),
        (b"identity.token_bytes", bounds.token_bytes, ceilings.token_token),
        (b"forges.installations", bounds.entries, ceilings.entries_entries),
        (b"forge_component.token_bytes", bounds.entry_token_bytes, ceilings.entry_token),
        (
            b"identity.text_bytes",
            bounds.text_bytes,
            ceilings
                .identity_name
                .min(ceilings.identity_login)
                .min(ceilings.identity_email)
                .min(ceilings.entry_account_login),
        ),
    ] {
        if bytes > bound {
            problems.push(own(name), What::Past { bytes: u64::from(bytes), bound: u64::from(bound) });
        }
    }
    payload::limits(bounds)
}

pub(crate) fn channel<E>(bounds: &payload::Bounds, problems: &mut Problems<What<E>>) {
    if let Some(side) = bounded(bounds, false, problems) {
        let _largest = largest(&side, b"workers", problems);
    }
}
pub(crate) fn largest<E>(
    side: &payload::v1::Limits,
    prefix: &[u8],
    problems: &mut Problems<What<E>>,
) -> Option<payload::Largest> {
    let largest = payload::largest(side)?;
    let ceilings = jig_protocol_worker_wire::CEILINGS;
    for (name, bytes, bound) in [
        (b"workspace_items".as_slice(), largest.items, ceilings.workspace_items),
        (b"work_left_some", largest.left, ceilings.work_left_some),
        (b"resource_resource_some", largest.resource, ceilings.resource_resource_some),
        (b"application_value_secret", largest.grant, ceilings.application_value_secret),
    ] {
        if bytes > bound {
            problems.push(
                Path::root().member(prefix).member(b"limits").member(name),
                What::Past { bytes: u64::from(bytes), bound: u64::from(bound) },
            );
        }
    }
    Some(largest)
}
/// Only an agent running locally must hold the answer in its tool payload.
pub(crate) fn agent_answer<E>(agent: &settings::Agent, problems: &mut Problems<What<E>>) {
    short(
        problems,
        at(0, Path::root().member(b"workers"))
            .member(b"agent")
            .member(b"profile")
            .member(b"declared")
            .member(b"tool_payload"),
        u64::from(agent.tool_payload()),
        b"temper's documentation answer (read_docs)",
        Some(u64::from(temper_service_core::DOC_ANSWER)),
    );
}

pub(crate) fn turns<E>(workers: &[config::Worker], agent: &settings::Agent, problems: &mut Problems<What<E>>) {
    for (index, worker) in workers.iter().enumerate() {
        let path = at(index, Path::root().member(b"workers")).member(b"limits").member(b"overrides");
        short(
            problems,
            path.member(b"hub.outcome_bytes"),
            worker.limits.hub.outcome_bytes,
            b"the agent's outcome (agent.json profile.declared.tool_payload)",
            Some(agent.outcome_bytes()),
        );
        short(
            problems,
            path.member(b"hub.turn_bytes"),
            worker.limits.hub.turn_bytes,
            b"the agent's largest turn",
            Some(u64::from(agent.turn_body())),
        );
    }
}
/// Check standalone's raw agent path before jig joins a configuration path.
pub(crate) fn raw_worker<E>(document: &skein_json::Document, at: u32, path: &Path, problems: &mut Problems<What<E>>) {
    if let Ok(mut worker) = config::read::Object::new(document, at)
        && let Some(agent_at) = worker.member(document, b"agent")
        && let Ok(mut agent) = config::read::Object::new(document, agent_at)
        && let Some(at) = agent.member(document, b"configuration")
        && let Ok(value) = config::read::text(document, at, 4096)
        && !value.starts_with(b"/")
    {
        problems.push(path.member(b"agent").member(b"configuration"), What::Absolute);
    }
}
