//! Recurring batches, ended tasks, and connector cleanup
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::{Ask, Core, CoreKey, Held, Key, Limits, Now, Request, Write};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};

pub(crate) fn recurring_due(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    period: u64,
    members: u32,
    pending: &mut Queue<tasks::Request>,
) {
    let Some(context) = core.tasks.delegation(task) else { return };
    let Some(template) = core.tasks.recurring_template(task) else { return };
    if members != u32::try_from(template.batch.len()).expect("bounded template") {
        return;
    }
    let mut asked = List::with_capacity(env.limits.tasks.batch);
    for member in &template.batch {
        let executor = match member.executor {
            tasks::Executor::Agent { charter } => authority::Executor::Charter(charter),
            tasks::Executor::Procedure { code, .. } => authority::Executor::Procedure(code),
            tasks::Executor::Person(_) => unreachable!("person template refused at admission"),
        };
        asked
            .push(authority::Delegate {
                executor,
                authority: crate::translate::tasks_authority(&member.authority),
                symbolic: Box::new([]),
            })
            .expect("bounded template");
    }
    let checked = core.connector_batch_admit(
        context.project,
        &context.authority,
        tasks::Numbers { budget: context.authority.budget.spend, spent: 0, spent_below: 0, reserved: 0 },
        context.tasks_left,
        asked.into_boxed(),
        &mut Queue::uncapped(),
    );
    if checked.answer != authority::Answer::Allow {
        return;
    }
    // No per-occurrence refusal outcome exists. Invalid account hints become
    // typed next-run waits on the created task (accounts.md, 4.3).
    if let Err(why) = core.recurring_accounts(context.project, &template.batch) {
        if let tasks::Refusal::Account { .. } = why {
            // Keep this occurrence; run_choice rechecks the same facts for its wait.
        } else {
            return;
        }
    }
    let mut numbers = List::with_capacity(env.limits.tasks.batch);
    for _ in 0..members {
        let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else { break };
        numbers.push(number).expect("bounded recurring batch");
    }
    if numbers.len() == members {
        let mut follow = Queue::uncapped();
        crate::routing::step::step_tasks(
            core,
            env,
            tasks::Event::RecurringBatch { task, period, numbers: numbers.into_boxed() },
            &mut follow,
        );
        for _ in 0..follow.len() {
            pending.push(follow.pop().expect("recurring output count"));
        }
    }
}

pub(crate) fn adopt(core: &mut Core, env: &Env<Limits>, task: u64, attempt: u64, kept: u32) {
    core.adopt(env, task, attempt, kept);
}

pub(crate) fn ended(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    requester: tasks::Party,
    ending: tasks::Ending,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let _pending = core.pending_note_briefs.remove(&task);
    let position = core.ending_positions.remove(&task).expect("ended row assigned its result position");
    let mut ended = crate::routing::view::routes::route_view(
        core,
        env,
        jig_domain_views::Event::RunEnded { run: Token::new(task) },
    );
    for _ in 0..ended.len() {
        let request = ended.pop().expect("ended run outputs");
        out.push(request);
    }
    for key in core.retire_calls(task, u64::MAX, u32::MAX, env.limits.call_records) {
        out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Call(key)))));
    }
    drop(core.proofs.remove(&task));
    out.push(Request::Write(Write::Erase(Key::Core(CoreKey::RunProof(task)))));
    match requester {
        tasks::Party::Person(person) => {
            out.push(Request::Held(Box::new(Held::Result { person, task, words: super::ending_words(ending) })));
        }
        tasks::Party::Task(parent) => {
            let (kind, words) = super::result_notice(ending);
            let mut follow = Queue::uncapped();
            crate::routing::step::step_tasks(
                core,
                env,
                tasks::Event::DelegateResult {
                    task: parent,
                    word: tasks::Word {
                        number: position,
                        from: tasks::Party::Task(task),
                        kind: tasks::MessageKind::Result(kind),
                        words,
                        at: env.wall,
                        hits: 1,
                        eligible: false,
                    },
                },
                &mut follow,
            );
            for _ in 0..follow.len() {
                pending.push(follow.pop().expect("delegate result output count"));
            }
        }
        tasks::Party::Deployment { .. } => {}
    }
}

pub(crate) fn close(core: &mut Core, task: u64, ending: tasks::Ending, out: &mut Queue<Request>) {
    let ancestor_number = core.tasks.root(task).unwrap_or(task);
    for &connector in core.connectors.as_ref() {
        core.closing_connectors.insert((task, connector), ()).expect("bounded closing connectors");
        out.push(Request::Ask { connector, ask: Ask::Close { task, root: ancestor_number, ending: ending.clone() } });
    }
}

pub(crate) fn release(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    ending: tasks::Ending,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let ancestor_number = core.tasks.root(task).unwrap_or(task);
    let mut entries = List::with_capacity(u32::try_from(core.connectors.len()).expect("connector count fits u32"));
    for &connector in core.connectors.as_ref() {
        let Some(entry) = crate::fresh(&mut core.counters, crate::Family::ConnectorRow) else {
            let mut follow = Queue::uncapped();
            crate::routing::step::step_tasks(
                core,
                env,
                tasks::Event::Hold { task, why: tasks::Hold::Budget },
                &mut follow,
            );
            for _ in 0..follow.len() {
                pending.push(follow.pop().expect("release budget output count"));
            }
            break;
        };
        entries.push((connector, entry)).expect("one outbox entry per connector");
    }
    if entries.len() != u32::try_from(core.connectors.len()).expect("connector count fits u32") {
        return;
    }
    for &(connector, entry) in &entries {
        out.push(Request::Ask {
            connector,
            ask: Ask::Release { task, root: ancestor_number, ending: ending.clone(), entry },
        });
    }
}

pub(crate) fn activate(core: &mut Core, context: Box<tasks::RunContext>, out: &mut Queue<Request>) {
    if core.pending_note_briefs.get(&context.task) == Some(&crate::routing::NoteBrief::Activation) {
        let _pending = core.pending_note_briefs.remove(&context.task);
        crate::routing::children::remember_due(core, context);
        return;
    }
    out.push(Request::Now(Box::new(Now::Activate { context })));
}
