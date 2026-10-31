//! Created batches routed to their owner, retaining accepted proposal evidence
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::{
    CallPart, Core, CoreRecord, Limits, PersonProposalRoute, Record, Request, Write,
    routing::{children::append_people, flight::Flight, run::calls::call_decision},
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo};

pub(crate) fn made(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    tasks: Box<[u64]>,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let request = reply_to.into_token();
    let flight = core.land_task(request);
    if let Some(Flight::PersonProposal(PersonProposalRoute::Accepting { person, proposer, proposal, .. })) = &flight {
        let first = *tasks.first().expect("an accepted batch made tasks");
        let last = *tasks.last().expect("an accepted batch made tasks");
        let count = match last.checked_sub(first) {
            Some(count) => count.checked_add(1),
            None => None,
        };
        assert_eq!(count, u64::try_from(tasks.len()).ok(), "batch numbers are consecutive");
        let project = core.tasks.task(*proposer).expect("accepted proposer").project;
        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::ProposalDecision(
            crate::ProposalDecisionRecord {
                project,
                proposer: people::Proposer::Task(*proposer),
                proposal: *proposal,
                kind: tasks::ProposalKind::Batch,
                by: *person,
                at: env.wall,
                choice: people::ProposalChoice::Accepted,
                created: Some((first, last)),
            },
        )))));
    }
    match core.made_flight(flight, tasks) {
        crate::MadeRoute::Internal => {}
        crate::MadeRoute::Tasks(event) => {
            let mut follow = Queue::uncapped();
            crate::routing::step::step_tasks(core, env, *event, &mut follow);
            for _ in 0..follow.len() {
                pending.push(follow.pop().expect("made output count"));
            }
        }
        crate::MadeRoute::Person { event, stubs } => {
            for stub in stubs {
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(core, env, tasks::Event::RememberStub { stub }, &mut follow);
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("stub output count"));
                }
            }
            append_people(core, env, *event, out);
        }
        crate::MadeRoute::Delegated { request, key, tasks, stubs, given } => {
            for stub in stubs {
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(core, env, tasks::Event::RememberStub { stub }, &mut follow);
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("stub output count"));
                }
            }
            call_decision(
                core,
                out,
                ReplyTo::new(request),
                key,
                CallPart::Delegated(crate::admission::as_allowed::made(&tasks, given, env.limits.tasks.batch)),
            );
        }
    }
}
