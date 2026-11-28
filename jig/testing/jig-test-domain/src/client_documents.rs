//! Admission and owner selection for application public reads.
use crate::routes::route_engine;
use crate::{Delivery, Domain, Limits, Work, Write};
use jig_domain_core as core;
use jig_domain_core::engine::{self, In};
use skein_lib::{Decision, Env, Queue, ReplyTo};

/// Refuse excess public work immediately (jig/docs/design/domain/people/public-reading.md, 5).
pub(crate) fn public_busy(domain: &mut Domain, reply: ReplyTo) {
    // Refusals bypass the journal's held queue and never count as admitted reads.
    domain.now.push(crate::Request::Deliver(Delivery::Core(core::Held::PublicRead {
        reply,
        answer: core::PublicAnswer::Busy,
    })));
}

pub(crate) fn public_start(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision<Write, Delivery>,
    reply: ReplyTo,
    read: core::PublicRead,
    work: &mut Queue<Work>,
) {
    if !crate::admission::public_admits(domain, &env.limits) {
        public_busy(domain, reply);
        return;
    }
    domain.public_reads = domain.public_reads.checked_add(1).expect("bounded public reads");
    // The test application's families 1 and 2 are served by connectors 1 and 2.
    let connector = match read.owner.checked_sub(1) {
        Some(1) => Some(1_u16),
        Some(2) => Some(2_u16),
        Some(_) | None => None,
    };
    if read.owner != 0 && connector.is_none() {
        crate::hold(decision, Delivery::Core(core::Held::PublicRead { reply, answer: core::PublicAnswer::Unknown }));
        return;
    }
    let read = core::PublicRead {
        owner: match connector {
            Some(number) => number.checked_add(1).expect("connector owner"),
            None => 0,
        },
        ..read
    };
    let mut out = Queue::uncapped();
    engine::step(
        &mut domain.engine,
        &env.with(env.limits.core),
        In::Core(core::Event::PublicRead { reply, read }),
        &mut out,
    );
    route_engine(domain, env, decision, out, work);
}
