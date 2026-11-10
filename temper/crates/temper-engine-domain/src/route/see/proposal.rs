//! Proposal address reads (temper/docs/design/domain/agent/ask-temper.md, 1).
use super::{
    Decision, Domain, Env, FlightRender, Limits, ReplyTo, SeeAnswer, SeeFlight, SeeProposal, SeeReading, Work,
};
use alloc::boxed::Box;
use jig_domain_core as core;

#[expect(clippy::too_many_arguments, reason = "the proposal read carries its call and proposal identity")]
pub(super) fn read(
    domain: &mut Domain,
    _env: &Env<Limits>,
    _decision: &mut Decision,
    to: ReplyTo,
    key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
    proposer: jig_domain_people::Proposer,
    number: u64,
) {
    let right = to.into_token();
    domain.call_renders.get_mut(&right).expect("decoded see flight").render = FlightRender::Settings {
        see: Some(Box::new(SeeFlight { line, reading: SeeReading::Proposal })),
        from,
        requested: None,
        asked: None,
    };
    domain.work.push(Work::Core(core::Event::NamedAction {
        to: ReplyTo::new(right),
        key,
        action: core::NamedAction::ReadProposal { proposer, number },
    }));
}

pub(crate) fn answered(domain: &mut Domain, to: ReplyTo, answer: Box<core::ProposalRead>) {
    let right = to.into_token();
    let flight = domain.call_renders.get(&right).expect("see proposal flight");
    let FlightRender::Settings { see: Some(see), from, .. } = &flight.render else {
        unreachable!("see proposal context")
    };
    let request = crate::boundary::delivery_output(super::Delivery::Host(Box::new(crate::HostDelivery::See {
        to: ReplyTo::new(right),
        line: see.line.clone(),
        from: *from,
        answer: Box::new(SeeAnswer::Proposal(Box::new(match *answer {
            core::ProposalRead::Waiting(proposal) => SeeProposal::Waiting(proposal),
            core::ProposalRead::Decided(proposal) => SeeProposal::Decided(proposal),
            core::ProposalRead::Unknown => SeeProposal::Unknown,
        }))),
    })));
    assert!(crate::domain::now(domain, request), "see proposal render door reserved");
}
