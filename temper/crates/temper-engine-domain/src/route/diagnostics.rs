//! Volatile refusal observations (temper/docs/design/domain/agent/agents.md).
//! No admission decision depends on diagnostic room.
use super::{CallKey, Domain};
use alloc::boxed::Box;

/// Recover the declared tool from its current decoded flight.
pub(super) fn tool(domain: &Domain, key: CallKey) -> Box<[u8]> {
    for (_, flight) in &domain.call_renders {
        if flight.key == Some(key) {
            return flight.tool.clone();
        }
    }
    Box::new([])
}

/// A job read's own arguments, retained for both refusal words and diagnostics.
pub(super) fn job_attempt(read: &skein_forge::api::Read) -> Option<skein_forge::api::JobAttempt> {
    use skein_forge::api::Read;
    match read {
        Read::Job { attempt, .. } => Some(*attempt),
        Read::Locate { .. }
        | Read::Items { .. }
        | Read::Item { .. }
        | Read::Pull { .. }
        | Read::PullFor { .. }
        | Read::Reviews { .. }
        | Read::Statuses { .. }
        | Read::Remarks { .. }
        | Read::Branch { .. }
        | Read::Branches { .. }
        | Read::PullFiles { .. }
        | Read::Compare { .. }
        | Read::Checks { .. }
        | Read::File { .. }
        | Read::Protection { .. }
        | Read::Settings
        | Read::Collaborators { .. }
        | Read::Permission { .. }
        | Read::Runs { .. }
        | Read::Run { .. }
        | Read::Tree { .. }
        | Read::Artifact { .. }
        | Read::Commits { .. } => None,
    }
}

/// Keep an observation without making room part of the decision.
pub(super) fn emit(domain: &mut Domain, fact: crate::Fact) {
    if domain.facts.try_push(fact).is_err() {
        domain.facts_lost = domain.facts_lost.saturating_add(1);
    }
}

/// Record a read's refusal without changing the answer or its admission.
pub(super) fn read(
    domain: &mut Domain,
    key: CallKey,
    layer: &'static str,
    reason: &'static str,
    error: Option<skein_forge::api::Error>,
    findings: Box<[jig_domain_authority::Finding]>,
) {
    let tool = tool(domain, key);
    let mut job = None;
    for (_, flight) in &domain.call_renders {
        if flight.key == Some(key) {
            job = flight.job;
            break;
        }
    }
    emit(
        domain,
        crate::Fact::ReadRefused { task: key.task, attempt: key.attempt, tool, layer, reason, findings, error, job },
    );
}
