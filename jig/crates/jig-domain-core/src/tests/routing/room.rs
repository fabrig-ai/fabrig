//! The procedure refusal and its durable room condition leave the core together.
use crate::tests::*;

#[test]
fn a_refused_procedure_batch_keeps_its_room_wait_write_with_the_refusal() {
    let (mut core, mut limits) = tree_core();
    let mut parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    parent.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    let blocker = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Person(1),
            batch: Box::new([parent, blocker]),
        },
    );
    limits.tasks.tasks = 2;
    let child = new_row(&core, 3, tasks::Funder::Task(1), None);
    let (_, _, step_number) = core.tasks.procedure_due(1).unwrap();
    let read = core.tasks.procedure_offered(1);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = step(
        &mut core,
        &env,
        Event::Tasks(tasks::Event::Procedure {
            reply_to: ReplyTo::new(Token::new(1)),
            task: 1,
            step: step_number,
            read,
            decision: tasks::ProcedureDecision {
                batch: Box::new([child]),
                ..tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait)
            },
        }),
    );
    let wait = tasks::RoomWait { room: tasks::LiveRoom::Deployment, order: 1 };
    let mut saved = 0_u32;
    let mut refused = 0_u32;
    while let Some(request) = out.pop() {
        match request {
            Request::Write(Write::Save(Record::Tasks(tasks::Stored::Live(row)))) if row.number == 1 => {
                assert_eq!(row.room_wait, Some(wait));
                assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle));
                saved = saved.checked_add(1).unwrap();
            }
            Request::Ask { connector: 1, ask: Ask::StepRefused { task: 1, step, why } } => {
                assert_eq!(step, step_number);
                let StepRefusal::Refused(problem) = why else { panic!("task refusal") };
                assert_eq!(problem.freeing_count(), Some(tasks::LiveRoom::Deployment));
                refused = refused.checked_add(1).unwrap();
            }
            Request::ProjectMade { .. }
            | Request::Write(_)
            | Request::Ask { .. }
            | Request::Held(_)
            | Request::Now(_) => {}
        }
    }
    assert_eq!((saved, refused), (1, 1));
    assert_eq!(core.tasks.task(1).unwrap().room_wait, Some(wait));
    assert!(core.tasks.procedure_due(1).is_none());
}
