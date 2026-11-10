//! Public admission and the forge handoff (temper/docs/design/client/forge/README.md, 9.1–9.2).
use super::{Decision, Delivery, Domain, Env, Limits, core, emit};
use crate::boundary::Work;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, ReplyTo};
use temper_client_protocol_wire as wire;
use temper_engine_domain_forge::{PublicAsk, PublicTask};

pub(crate) fn start(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    reply: ReplyTo,
    mut read: core::PublicRead,
) {
    domain.public_reads = domain.public_reads.checked_add(1).expect("bounded public reads");
    let forge = wire::number(wire::Owner::Forge).checked_add(1).expect("family owner");
    if read.owner == forge {
        read.owner = domain.config.forge_connector.checked_add(1).expect("connector owner");
    } else if read.owner != 0 {
        emit(decision, &env.limits, Delivery::Public { reply, answer: core::PublicAnswer::Unknown, admitted: true });
        return;
    }
    domain.work.push(Work::Core(core::Event::PublicRead { reply, read }));
}

pub(crate) fn ask(domain: &mut Domain, env: &Env<Limits>, connector: u16, ask: core::Ask) {
    let core::Ask::Public { reply, kind, version, project, task, public } = ask else { unreachable!("public ask") };
    let answer = if connector == domain.config.forge_connector {
        let public = public_tasks(domain, &public);
        match domain
            .forge
            .public(&PublicAsk { kind, version, project, task, public: &public, now: env.wall }, &env.limits.forge)
        {
            Some(made) => Some(core::PublicApplication { tasks: made.tasks, record: made.record }),
            None => None,
        }
    } else {
        None
    };
    domain.work.push(Work::Core(core::Event::PublicAnswered { reply, answer }));
}
pub(crate) fn public_tasks(domain: &Domain, public: &[u64]) -> Box<[PublicTask]> {
    let mut tasks = List::with_capacity(u32::try_from(public.len()).expect("core public tasks bound"));
    for task in public {
        tasks.push(PublicTask { task: *task, goal: public_goal(domain, *task) }).expect("public tasks room");
    }
    tasks.into_boxed()
}
pub(crate) fn public_goal(domain: &Domain, task: u64) -> Option<u64> {
    let mut current = domain.core.tasks.task(task)?;
    let mut goal = None;
    for _ in 0..=current.depth {
        if current.tracked.is_some() {
            goal = Some(current.number);
        }
        match current.requester {
            tasks::Party::Task(parent) => current = domain.core.tasks.task(parent)?,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return goal,
        }
    }
    None
}

pub(crate) fn busy(domain: &mut Domain, reply: ReplyTo) {
    let mut out = skein_lib::Queue::uncapped();
    out.push(crate::Request::Party(crate::PartyRequest::Deliver(Delivery::Public {
        reply,
        answer: core::PublicAnswer::Busy,
        admitted: false,
    })));
    crate::domain::stage_now(domain, &mut out);
}
