//! Goal reds at a run's check start and host calls (domain/forge/reds.md, 3 and 5.1).
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "test assertions inspect emitted writes and host answers")]
use super::{adopted, change_row, change_start_fixture};
use crate::{Domain, Limits, RunRed, reds_for_run};
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Token};
use temper_engine_domain_forge as forge;

fn proof(domain: &mut Domain, task: u64) {
    domain
        .core
        .proofs
        .insert(
            task,
            jig_domain_core::RunProof {
                task,
                attempt: 1,
                conversation: 1,
                credentials: Box::new([]),
                accounts: Box::new([]),
                placement: Some(Token::new(7)),
                account_failure: None,
                account_terminal: false,
                window: None,
                crossed: false,
                host: jig_domain_fleet::HostKind::Worker,
                offered: None,
                outstanding: Box::new([]),
                turn: None,
                terminal: None,
            },
        )
        .unwrap();
}
fn restore(domain: &mut Domain, env: &Env<Limits>, row: forge::Stored) {
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: row },
        &mut Queue::uncapped(),
    );
}
fn hold(domain: &mut Domain, env: &Env<Limits>, task: u64, branch: &[u8]) {
    restore(
        domain,
        env,
        forge::Stored::Hold(forge::Hold {
            name: forge::Name {
                forge: 1,
                repository: 1,
                what: super::super::branch_what(branch, env.limits.forge.name_bytes).unwrap(),
            },
            task,
            writer: None,
            design_writers: Box::new([]),
            drift: None,
        }),
    );
}
fn fixture(base: &[u8]) -> (Domain, Env<Limits>) {
    let (mut domain, mut env, context) = change_start_fixture();
    env.limits.tasks.tasks = 8;
    env.limits.tasks.project_tasks = 8;
    env.limits.tasks.tree_tasks = 8;
    env.limits.tasks.delegates = 8;
    env.limits.tasks.depth = 3;
    domain.core.proofs = skein_lib::Map::with_capacity(8);
    let template = domain.core.tasks.task(context.task).unwrap().clone();
    let ledger = domain.core.tasks.funding(template.funder).unwrap().clone();
    domain.core.tasks = tasks::Domain::new(
        &env.limits.tasks,
        tasks::Kept { revisions: env.limits.settings.revisions, versions: env.limits.settings.versions },
        7,
        0,
        Box::new([]),
        Box::new([]),
    );
    for (number, parent, depth) in [
        (7, None, 0),
        (8, None, 0),
        (20, Some(8), 1),
        (21, Some(20), 2),
        (22, Some(20), 2),
        (23, Some(20), 2),
        (24, Some(23), 3),
    ] {
        let mut row = template.clone();
        row.authority.tools.0 |= 1;
        row.number = number;
        row.root = if number == 7 { 7 } else { 8 };
        row.depth = depth;
        if let Some(parent) = parent {
            row.requester = tasks::Party::Task(parent);
        }
        tasks::step(
            &mut domain.core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
            &mut Queue::uncapped(),
        );
        assert!(domain.core.tasks.task(number).is_some(), "task {number}: {:?}", domain.core.tasks.restore_failure());
        proof(&mut domain, number);
    }
    tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) }, &mut Queue::uncapped());
    tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), tasks::Event::Check, &mut Queue::uncapped());
    assert_eq!(domain.core.tasks.restore_failure(), None);
    tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), tasks::Event::Restored, &mut Queue::uncapped());
    assert_eq!(domain.core.tasks.restore_failure(), None);
    hold(&mut domain, &env, 7, b"temper/8/g7");
    hold(&mut domain, &env, 8, b"temper/8/g8");
    for (goal, key, byte) in [(7, b"z", 3), (8, b"a", 8), (7, b"a", 4)] {
        domain
            .forge
            .record_red(
                &env.limits.forge,
                forge::RecordRed {
                    goal,
                    key: key.as_slice().into(),
                    head: Box::new([byte; 20]),
                    why: b"base failure".as_slice().into(),
                    by: forge::Recorder::Task(goal),
                    at: 9,
                },
                &mut Queue::uncapped(),
            )
            .unwrap();
    }
    let mut change = change_row(None);
    change.task = 20;
    change.change.task = 20;
    change.repository = adopted(1).provider;
    change.base = base.into();
    change.goal = Some(8); // The destination's holding, not this historical hint, selects the reds.
    restore(&mut domain, &env, forge::Stored::Change(Box::new(change)));
    (domain, env)
}
fn expected() -> Box<[RunRed]> {
    Box::new([
        RunRed { key: b"a".as_slice().into(), head: Box::new([4; 20]) },
        RunRed { key: b"z".as_slice().into(), head: Box::new([3; 20]) },
    ])
}
#[test]
fn change_producer_repair_and_resolution_read_the_goal_holding_their_changes_base() {
    let (domain, _) = fixture(b"temper/8/g7");
    for task in [20, 21, 22, 23, 24] {
        assert_eq!(reds_for_run(&domain, task, 1), expected(), "task {task}");
    }
}
#[test]
fn a_goals_own_run_reads_its_own_reds_even_when_its_change_lands_in_main() {
    let (mut domain, env) = fixture(b"main");
    let mut change = change_row(None);
    change.task = 7;
    change.change.task = 7;
    change.repository = adopted(1).provider;
    restore(&mut domain, &env, forge::Stored::Change(Box::new(change)));
    assert_eq!(reds_for_run(&domain, 7, 1), expected());
    assert_eq!(
        reds_for_run(&domain, 8, 1).as_ref(),
        &[RunRed { key: b"a".as_slice().into(), head: Box::new([8; 20]) }]
    );
}
#[test]
fn reads_for_main_nonlive_and_stale_attempts_are_empty() {
    let (mut domain, env) = fixture(b"main");
    // A holder of main who also holds a goal branch is not main's goal.
    hold(&mut domain, &env, 8, b"main");
    assert!(reds_for_run(&domain, 22, 1).is_empty());
    proof(&mut domain, 999);
    assert!(reds_for_run(&domain, 999, 1).is_empty());
    assert!(reds_for_run(&domain, 7, 2).is_empty());
    assert!(reds_for_run(&domain, 500, 1).is_empty());
}

fn tool_call(domain: &mut Domain, env: &Env<Limits>, task: u64, position: u32, call: crate::RedCall) -> (crate::CallAnswer, Box<[crate::Write]>) {
    use crate::route::host_route::{self, Flight, FlightRender};
    use crate::{Decision, JournalOutput, Request, WorkerRequest};
    let right = Token::new(99);
    domain.limits = env.limits;
    assert!(domain.call_renders.insert(right, Flight {
        task, attempt: 1, key: None, note_decoding: false, render: FlightRender::Plain,
        name: Box::new([u8::try_from(position).unwrap()]), tool: b"red".as_slice().into(), job: None, page: None,
    }).is_ok());
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    host_route::decoded(domain, env, &mut decision, skein_lib::ReplyTo::new(right), crate::Call { completion: 1, position, tool: crate::Tool::Red(call) });
    crate::route::route_into(domain, env, &mut decision);
    assert!(domain.start_stop().is_none(), "route start failure: {:?}", domain.start_stop());
    assert!(domain.work.is_empty(), "unrouted work: {:?}", domain.work);
    crate::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, false).unwrap();
    let writes = if let Some(JournalOutput::Commit { number, writes }) = crate::commit(&mut domain.journal, &env.limits.journal) {
        crate::committed(&mut domain.journal, number);
        writes
    } else { Box::default() };
    let mut answer = None;
    let mut out = Queue::uncapped();
    for _ in 0..32 {
        crate::resume(&mut domain.journal, &mut out);
        if let Some(output) = out.pop() {
            let delivery = match output {
                JournalOutput::Deliver(delivery) => Some(delivery),
                JournalOutput::Now(Request::Worker(WorkerRequest::Deliver(delivery))) => Some(delivery),
                JournalOutput::Now(_) | JournalOutput::Commit { .. } | JournalOutput::Stop => None,
            };
            if let Some(crate::Delivery::Host(delivery)) = delivery {
                if let crate::HostDelivery::Render { answer: rendered, .. } = *delivery { answer = Some(rendered); }
            }
        }
    }
    drop(domain.call_renders.remove(&right));
    (answer.expect("red host answer"), writes)
}
fn record(key: &[u8], byte: u8) -> crate::RedCall {
    crate::RedCall::Record { key: key.into(), at: Box::new([byte; 20]), why: b"inherited".as_slice().into() }
}
#[test]
fn a_goals_red_call_keeps_its_answer_with_its_record_and_replays_once() {
    let (mut domain, env) = fixture(b"main");
    let (answer, writes) = tool_call(&mut domain, &env, 7, 1, record(b"new", 9));
    let crate::CallAnswer::Red(reply) = &answer else { panic!("red answer: {answer:?}") };
    let crate::RedToolAnswer::Write(reply) = reply.as_ref() else { panic!("red write") };
    assert_eq!((reply.goal, reply.key.as_ref()), (7, b"new".as_slice()));
    let Ok(forge::RedOutcome::Recorded { recorded, head }) = &reply.result else { panic!("recorded") };
    assert_eq!(head.as_ref(), &[9; 20]);
    assert_eq!(recorded.replaced, None);
    let generation = recorded.generation;
    assert!(writes.iter().any(|write| matches!(write, crate::Write::Save(crate::Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Red(red) if red.goal == 7 && red.key.as_ref() == b"new" && red.by == forge::Recorder::Task(7) && red.at == env.wall.as_nanos()))));
    assert!(writes.iter().any(|write| matches!(write, crate::Write::Save(crate::Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Call { answer: forge::CallPayload::Red(kept), .. } if kept.as_ref() == reply))));
    assert!(writes.iter().any(|write| matches!(write, crate::Write::Save(crate::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(_)))))));
    // Restored connector and core call parts give the same replay before rendering settles it.
    let payload = domain.forge.named_answer(forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 }).unwrap();
    domain.forge.forget_named_answer(forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 });
    restore(&mut domain, &env, forge::Stored::Call { key: forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 }, answer: payload });
    let (repeated, writes) = tool_call(&mut domain, &env, 7, 1, record(b"new", 10));
    assert_eq!(repeated, answer);
    assert!(writes.is_empty(), "replay records nothing");
    assert_eq!(domain.forge.reds(7).find(|(_, row)| row.key.as_ref() == b"new").unwrap().1.generation, generation);
    let (replacement, _) = tool_call(&mut domain, &env, 7, 2, record(b"new", 11));
    let crate::CallAnswer::Red(reply) = replacement else { panic!("red replacement") };
    let crate::RedToolAnswer::Write(reply) = *reply else { panic!("write") };
    let Ok(forge::RedOutcome::Recorded { recorded, .. }) = reply.result else { panic!("recorded") };
    assert!(recorded.generation > generation);
    assert_eq!(recorded.replaced.unwrap().head.as_ref(), &[9; 20]);
}
#[test]
fn red_end_keeps_the_ended_snapshot_and_refuses_missing_and_nongoals_without_holding() {
    for (task, key, expected) in [(7, b"a".as_slice(), None), (7, b"missing", Some(forge::RedRefusal::NoSuch)), (20, b"a", Some(forge::RedRefusal::NoGoal))] {
        let (mut domain, env) = fixture(b"main");
        let (answer, writes) = tool_call(&mut domain, &env, task, 1, crate::RedCall::End { key: key.into(), why: b"fixed".as_slice().into() });
        let crate::CallAnswer::Red(reply) = answer else { panic!("red answer") };
        let crate::RedToolAnswer::Write(reply) = *reply else { panic!("write") };
        if let Some(expected) = expected { assert_eq!(reply.result, Err(expected)); } else {
            let Ok(forge::RedOutcome::Ended(red)) = reply.result else { panic!("ended") };
            assert_eq!(red.key.as_ref(), b"a");
            assert_eq!(red.why.as_ref(), b"base failure");
            assert!(writes.iter().any(|write| matches!(write, crate::Write::Erase(crate::Key::Forge(forge::Key::Red { goal: 7, key })) if key.as_ref() == b"a")));
        }
        assert!(!matches!(domain.core.tasks.task(task).unwrap().phase, tasks::Phase::Held { .. }));
    }
}
#[test]
fn red_list_is_a_fresh_goal_local_read_with_no_call_record() {
    for task in [7, 20] {
        let (mut domain, env) = fixture(b"main");
        let (answer, writes) = tool_call(&mut domain, &env, task, 1, crate::RedCall::List);
        assert!(writes.is_empty());
        assert!(crate::route::call_answer(&domain, crate::CallKey { task, attempt: 1, completion: 1, position: 1 }).is_none());
        let crate::CallAnswer::Red(reply) = answer else { panic!("red list") };
        if task == 7 {
            let crate::RedToolAnswer::List { goal, bound, reds } = *reply else { panic!("list") };
            assert_eq!((goal, bound), (7, env.limits.forge.branch_reds));
            assert_eq!(reds.iter().map(|red| red.key.as_ref()).collect::<Vec<_>>(), [b"a", b"z"]);
        } else { assert_eq!(*reply, crate::RedToolAnswer::NoGoal { task }); }
    }
}
#[test]
fn a_red_past_the_bound_holds_the_goal_in_the_refusals_decision() {
    let (mut domain, mut env) = fixture(b"main");
    env.limits.forge.branch_reds = 2;
    let (answer, writes) = tool_call(&mut domain, &env, 7, 1, record(b"new", 9));
    let crate::CallAnswer::Red(reply) = answer else { panic!("red answer") };
    let crate::RedToolAnswer::Write(reply) = *reply else { panic!("write") };
    assert_eq!(reply.result, Err(forge::RedRefusal::Full { bound: 2 }));
    assert_eq!(domain.forge.reds(7).count(), 2);
    let task = domain.core.tasks.task(7).unwrap();
    let tasks::Phase::Held { source: Some(source), .. } = &task.phase else { panic!("held red goal: {:?}", task.phase) };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("red refusal account") };
    assert_eq!(words.as_ref(), b"red: goal 7 holds its most reds (at most 2); end one, or look at what its branch carries");
    assert!(writes.iter().any(|write| matches!(write, crate::Write::Save(crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) if task.number == 7 && matches!(task.phase, tasks::Phase::Held { .. }))));
    assert!(writes.iter().any(|write| matches!(write, crate::Write::Save(crate::Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Call { answer: forge::CallPayload::Red(_), .. }))));
    assert!(!writes.iter().any(|write| matches!(write, crate::Write::Save(crate::Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Red(_) | forge::Stored::RedNumber { .. }))));
}
#[test]
fn a_red_key_past_the_configured_byte_limit_decides_nothing() {
    for call in [record(b"new", 9), crate::RedCall::End { key: b"new".as_slice().into(), why: b"fixed".as_slice().into() }] {
        let (mut domain, mut env) = fixture(b"main");
        env.limits.forge.red_key_bytes = 2;
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::red_tool::call(&mut domain, &env, &mut decision, skein_lib::ReplyTo::new(Token::new(99)), crate::CallKey { task: 7, attempt: 1, completion: 1, position: 1 }, call);
        assert!(decision.write_free());
        assert_eq!(domain.forge.reds(7).count(), 2);
        assert!(domain.forge.named_answer(forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 }).is_none());
    }
}
