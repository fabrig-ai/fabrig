//! Tasks with scripted executors and an atomic durable parent at the retained
//! production boundary (domain/tasks.md, 11; domain/engine.md, 7).
use jig_domain_tasks::{Event, Party, ProcedureDecision, Refusal};
use skein_lib::ReplyTo;

pub mod accounting_referee;
pub mod coverage;
pub mod referee;
pub mod stories;
pub mod trace;
mod world;
pub use world::{
    EFFORTS, Frozen, LIMITS, RETRY, Reply, World, authority, charters, domain, run_story, run_story_choices,
    run_story_facts, run_story_traced, task,
};

/// The tasks world's project and first task.
pub const PROJECT: u32 = 1;
pub const TASK: u64 = 1;

/// The tasks world's person and their party.
pub const PERSON: u64 = 9;
pub const PERSON_PARTY: Party = Party::Person(PERSON);

/// An owner task and its member.
pub const OWNER: u64 = 1;
pub const MEMBER: u64 = 2;

/// Send a request and read its terminal reply (jig/docs/design/domain/tasks.md, 7).
pub fn response(world: &mut World, event: impl FnOnce(ReplyTo) -> Event) -> Reply {
    let to = world.to();
    let call = to.into_token();
    world.send(event(ReplyTo::new(call)));
    world.replies[&call.raw()].clone()
}

/// Finish an agent task and settle its effects (jig/docs/design/domain/tasks.md, 5.1).
pub fn done(world: &mut World, number: u64) {
    world.claim_fresh(number);
    world.finish(number);
    world.settle(number);
}

/// Check the task, cause and dependencies of a refusal (jig/docs/design/domain/tasks.md, 7).
pub fn refused(reply: Reply, task: u64, why: Refusal, blocked: &[u64]) {
    let Reply::Refused(problem) = reply else { panic!("expected refusal, got {reply:?}") };
    assert_eq!(problem.task, Some(task));
    assert_eq!(problem.why, why);
    assert_eq!(problem.blocked_by.as_deref(), Some(blocked));
}

/// Step a procedure with its current inbox (jig/docs/design/domain/tasks.md, 5.3).
pub fn step(world: &mut World, task: u64, decision: ProcedureDecision) -> Reply {
    let step = world.record(task).attempt + 1;
    let read = world.contexts[&task].inbox.last().map(|word| word.number);
    response(world, |reply_to| Event::Procedure { reply_to, task, step, read, decision })
}
