//! Forge address reads (temper/docs/design/domain/agent.md, 8.4.1).
use super::{Decision, Domain, Env, Limits, ReplyTo, SeeAnswer};
use alloc::boxed::Box;

pub(super) fn landing(
    _domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    _key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
) {
    super::deliver(env, decision, to, line, from, SeeAnswer::NotYet);
}
#[expect(clippy::too_many_arguments, reason = "the forge read carries its call and repository")]
#[expect(clippy::boxed_local, reason = "see-forge keeps the owned repository address")]
pub(super) fn gates(
    _domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    _key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
    _repository: Box<[u8]>,
) {
    super::deliver(env, decision, to, line, from, SeeAnswer::NotYet);
}
#[expect(clippy::too_many_arguments, reason = "the file read carries its call and address members")]
#[expect(clippy::boxed_local, reason = "see-forge keeps the owned file address")]
pub(super) fn file(
    _domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    _key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
    _repository: Box<[u8]>,
    _path: Box<[u8]>,
    _branch: Option<Box<[u8]>>,
) {
    super::deliver(env, decision, to, line, from, SeeAnswer::NotYet);
}

pub(crate) fn answered(domain: &mut Domain, to: ReplyTo, _answer: crate::CallAnswer) {
    let right = to.into_token();
    let flight = domain.call_renders.get(&right).expect("see forge flight");
    let super::FlightRender::Settings { see: Some(see), from, .. } = &flight.render else {
        unreachable!("see forge context")
    };
    let request = crate::boundary::delivery_output(super::Delivery::Host(Box::new(crate::HostDelivery::See {
        to: ReplyTo::new(right),
        line: see.line.clone(),
        from: *from,
        answer: Box::new(SeeAnswer::NotYet),
    })));
    assert!(crate::domain::now(domain, request), "see forge render door reserved");
}
