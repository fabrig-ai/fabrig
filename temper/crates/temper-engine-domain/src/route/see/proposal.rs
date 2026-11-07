//! Proposal address reads (temper/docs/design/domain/agent.md, 8.4.1).
use super::{Decision, Domain, Env, Limits, ReplyTo, SeeAnswer};
use alloc::boxed::Box;

#[expect(clippy::too_many_arguments, reason = "the proposal read carries its call and proposal identity")]
pub(super) fn read(
    _domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    _key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
    _proposer: jig_domain_people::Proposer,
    _number: u64,
) {
    super::deliver(env, decision, to, line, from, SeeAnswer::NotYet);
}

#[expect(dead_code, reason = "constructed by the task see-proposal of ui-ask-m1")]
pub(crate) fn answered(domain: &mut Domain, to: ReplyTo, _answer: Box<jig_domain_core::ProposalRead>) {
    let right = to.into_token();
    let flight = domain.call_renders.get(&right).expect("see proposal flight");
    let super::FlightRender::Settings { see: Some(see), from, .. } = &flight.render else {
        unreachable!("see proposal context")
    };
    let request = crate::boundary::delivery_output(super::Delivery::Host(Box::new(crate::HostDelivery::See {
        to: ReplyTo::new(right),
        line: see.line.clone(),
        from: *from,
        answer: Box::new(SeeAnswer::NotYet),
    })));
    assert!(crate::domain::now(domain, request), "see proposal render door reserved");
}
