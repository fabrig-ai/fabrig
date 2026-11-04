use jig_domain_tasks::{self as tasks, Event, Executor, KeptStart, Party, ProcedureDecision, ProcedureEnd, Refusal};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::ReplyTo;

const PROCEDURE: u64 = 1;
const DEPENDENCY: u64 = 2;
const OTHER: u64 = 3;
const REQUESTER: Party = Party::Person(9);

fn pending(dependencies: &[u64]) -> World {
    let mut world = World::new(894, LIMITS);
    world.set_kept_starts(Box::new([KeptStart { connector: 1, code: 1 }]));
    let mut procedure = task(PROCEDURE, dependencies);
    procedure.executor = Executor::Procedure { connector: 1, code: 1 };
    assert_eq!(
        world.make(REQUESTER, vec![procedure, task(DEPENDENCY, &[]), task(OTHER, &[])]),
        Reply::Made(vec![1, 2, 3])
    );
    world
}

fn response(world: &mut World, event: impl FnOnce(ReplyTo) -> Event) -> Reply {
    let call = world.to().into_token();
    world.send(event(ReplyTo::new(call)));
    world.replies[&call.raw()].clone()
}

fn step(world: &mut World, decision: ProcedureDecision) -> Reply {
    let step = world.record(PROCEDURE).attempt + 1;
    let read = world.contexts[&PROCEDURE].inbox.last().map(|word| word.number);
    response(world, |reply_to| Event::Procedure { reply_to, task: PROCEDURE, step, read, decision })
}

fn mark(world: &mut World, dependencies: &[u64]) {
    assert_eq!(
        step(
            world,
            ProcedureDecision { started_from: dependencies.into(), ..ProcedureDecision::new(ProcedureEnd::Wait) }
        ),
        Reply::Done
    );
}

fn amendment(dependencies: &[u64]) -> tasks::Amendment {
    tasks::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: Some(dependencies.into()),
        authority: None,
        llm: None,
        reason: Box::new([1]),
    }
}

fn amend(world: &mut World, dependencies: &[u64]) -> Reply {
    let message = world.record(PROCEDURE).last_message + 1;
    response(world, |reply_to| Event::Amend {
        reply_to,
        by: REQUESTER,
        task: PROCEDURE,
        message,
        places: Box::new([]),
        stop_run: false,
        funder: None,
        amendment: amendment(dependencies),
    })
}

fn refuses(reply: Reply, blocked: &[u64]) {
    let Reply::Refused(problem) = reply else { panic!("expected dependencies refusal, got {reply:?}") };
    assert_eq!(problem, tasks::Problem::naming(Some(PROCEDURE), Refusal::Dependencies, blocked));
}

fn done(world: &mut World, number: u64) {
    world.claim_fresh(number);
    world.finish(number);
    world.settle(number);
}

#[test]
fn a_step_marks_a_pending_dependency_started_from_and_an_amendment_dropping_it_is_refused() {
    let mut world = pending(&[DEPENDENCY]);
    mark(&mut world, &[DEPENDENCY]);
    assert_eq!(world.record(PROCEDURE).started_from.as_ref(), [DEPENDENCY]);
    world.restart();
    let before = world.records.clone();
    refuses(amend(&mut world, &[]), &[DEPENDENCY]);
    assert_eq!(world.records, before);
}

#[test]
fn an_amendment_keeping_the_marked_dependency_and_adding_another_is_admitted() {
    let mut world = pending(&[DEPENDENCY]);
    mark(&mut world, &[DEPENDENCY]);
    assert_eq!(amend(&mut world, &[DEPENDENCY, OTHER]), Reply::Done);
    assert_eq!(world.record(PROCEDURE).dependencies.as_ref(), [DEPENDENCY, OTHER]);
    assert_eq!(amend(&mut world, &[DEPENDENCY]), Reply::Done);
    assert_eq!(world.record(PROCEDURE).started_from.as_ref(), [DEPENDENCY]);
}

#[test]
fn a_mark_naming_no_dependency_refuses_the_step_whole() {
    let mut world = pending(&[DEPENDENCY]);
    let before = world.records.clone();
    refuses(
        step(
            &mut world,
            ProcedureDecision {
                started_from: Box::new([DEPENDENCY, OTHER]),
                batch: Box::new([task(4, &[])]),
                messages: Box::new([tasks::ProcedureMessage {
                    task: DEPENDENCY,
                    word: tasks::Word {
                        number: 1,
                        from: Party::Task(PROCEDURE),
                        kind: tasks::MessageKind::Words,
                        words: Box::new([1]),
                        at: skein_lib::Wall::EPOCH,
                        hits: 1,
                        eligible: true,
                    },
                }]),
                ..ProcedureDecision::new(ProcedureEnd::Hold(tasks::Hold::Drift))
            },
        ),
        &[OTHER],
    );
    assert_eq!(world.records, before);
}

#[test]
fn a_marked_dependency_done_still_refuses_its_removal() {
    let mut world = pending(&[DEPENDENCY]);
    mark(&mut world, &[DEPENDENCY]);
    done(&mut world, DEPENDENCY);
    assert!(world.record(PROCEDURE).waiting_on.is_empty());
    world.restart();
    refuses(amend(&mut world, &[]), &[DEPENDENCY]);
}

#[test]
fn marks_keep_their_first_order_once_and_can_name_a_done_dependency() {
    let mut world = pending(&[DEPENDENCY, OTHER]);
    mark(&mut world, &[OTHER, OTHER]);
    world.send(Event::WakeProcedure { task: PROCEDURE });
    mark(&mut world, &[OTHER]);
    assert_eq!(world.record(PROCEDURE).started_from.as_ref(), [OTHER]);
    done(&mut world, DEPENDENCY);
    world.send(Event::WakeProcedure { task: PROCEDURE });
    mark(&mut world, &[DEPENDENCY, OTHER, DEPENDENCY]);
    assert_eq!(world.record(PROCEDURE).started_from.as_ref(), [OTHER, DEPENDENCY]);
    refuses(amend(&mut world, &[]), &[OTHER, DEPENDENCY]);
    refuses(amend(&mut world, &[DEPENDENCY]), &[OTHER]);
}

#[test]
fn a_procedure_amendment_cannot_drop_a_delegates_marked_dependency() {
    let mut world = World::new(895, LIMITS);
    world.set_kept_starts(Box::new([KeptStart { connector: 1, code: 1 }]));
    let mut parent = task(4, &[]);
    parent.executor = Executor::Procedure { connector: 1, code: 1 };
    world.make(REQUESTER, vec![parent]);
    let mut child = task(PROCEDURE, &[DEPENDENCY]);
    child.executor = Executor::Procedure { connector: 1, code: 1 };
    world.make(Party::Task(4), vec![child, task(DEPENDENCY, &[])]);
    mark(&mut world, &[DEPENDENCY]);
    let before = world.records.clone();
    let read = world.contexts[&4].inbox.last().map(|word| word.number);
    refuses(
        response(&mut world, |reply_to| Event::Procedure {
            reply_to,
            task: 4,
            step: 1,
            read,
            decision: ProcedureDecision {
                amendments: Box::new([tasks::ProcedureAmendment {
                    task: PROCEDURE,
                    message: 1,
                    stop_run: false,
                    amendment: amendment(&[]),
                }]),
                ..ProcedureDecision::new(ProcedureEnd::Wait)
            },
        }),
        &[DEPENDENCY],
    );
    assert_eq!(world.records, before);
}
