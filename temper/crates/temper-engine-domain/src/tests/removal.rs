//! Removal cancels procedure tasks through their existing control path
//! (temper/docs/design/draft/playground-reset.md, 4).
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};
use temper_engine_domain_forge as forge;

fn procedure() -> (crate::Domain, Env<crate::Limits>, u64) {
    let (mut domain, env) = super::resume_fixture::root();
    let goal = crate::fresh(&mut domain.core.counters, crate::Family::Task).unwrap();
    let task_env = Env { now: env.now, wall: env.wall, limits: env.limits.tasks };
    let mut out = Queue::uncapped();
    for event in [
        tasks::Event::Restored,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(2)), project: 1, period: 1, budget: 100 },
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(3)),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([tasks::New {
                number: goal,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: 2 },
                llm: None,
                spec: tasks::Spec {
                    asks: false,
                    words: b"live goal".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 1 },
                deadline: tasks::Deadline::Named,
                authority: super::held_assignment().run.authority,
                numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        },
    ] {
        tasks::step(&mut domain.core.tasks, &task_env, event, &mut out);
        while out.pop().is_some() {}
    }

    (domain, env, goal)
}

#[test]
fn a_removed_changes_cancel_ends_its_procedure_even_while_held() {
    for held in [false, true] {
        let (mut domain, env, task) = procedure();
        let task_env = Env { now: env.now, wall: env.wall, limits: env.limits.tasks };
        let mut out = Queue::uncapped();
        if held {
            tasks::step(
                &mut domain.core.tasks,
                &task_env,
                tasks::Event::Hold { task, why: tasks::Hold::Effects },
                &mut out,
            );
            while out.pop().is_some() {}
        }
        let mut forge_out = Queue::uncapped();
        forge_out.push(forge::Request::ChangeDecision {
            task,
            decision: temper_engine_domain_forge_change::Decision::Cancel,
            entry: None,
            evidence: None,
        });
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::forge_route::outputs(&mut domain, &env, &mut decision, &mut forge_out);
        let Some(crate::Work::Tasks(control)) = domain.work.pop() else { panic!("task cancel routed") };
        assert!(domain.work.is_empty());
        for event in [control, tasks::Event::EffectsSettled { task }, tasks::Event::Settled { task }] {
            tasks::step(&mut domain.core.tasks, &task_env, event, &mut out);
        }
        let mut cancelled = false;
        while let Some(request) = out.pop() {
            if let tasks::Request::Save { record: tasks::Stored::Ended(row) } = request {
                assert_eq!(row.number, task);
                let tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) = row.phase else {
                    panic!("cancel ends the procedure as cancelled");
                };
                cancelled = true;
            }
        }
        assert!(cancelled, "cancel follows the task's settlement to its terminal");
    }
}
