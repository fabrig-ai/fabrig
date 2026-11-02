//! Volatile refusal observations (temper/docs/design/domain/agent.md, 8).
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
    emit(
        domain,
        crate::Fact::ReadRefused { task: key.task, attempt: key.attempt, tool, layer, reason, findings, error },
    );
}
