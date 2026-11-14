//! Goal reds at a run's check start and host calls (domain/forge/reds.md, 3 and 5.1).
#![expect(clippy::match_like_matches_macro, reason = "the subset uses explicit test patterns rather than matches")]
#![expect(clippy::disallowed_methods, reason = "test assertions inspect emitted writes and host answers")]
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
    let ledger = *domain.core.tasks.funding(template.funder).unwrap();
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
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) },
        &mut Queue::uncapped(),
    );
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

fn tool_call(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    position: u32,
    call: crate::RedCall,
) -> (crate::CallAnswer, Box<[crate::Write]>) {
    use crate::route::host_route::{self, Flight, FlightRender};
    use crate::{Decision, JournalOutput, Request, WorkerRequest};
    let right = Token::new(99);
    domain.limits = env.limits;
    domain
        .call_renders
        .insert(
            right,
            Flight {
                task,
                attempt: 1,
                key: None,
                note_decoding: false,
                render: FlightRender::Plain,
                name: Box::new([u8::try_from(position).unwrap()]),
                tool: b"red".as_slice().into(),
                job: None,
                page: None,
            },
        )
        .unwrap();
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    host_route::decoded(
        domain,
        env,
        &mut decision,
        skein_lib::ReplyTo::new(right),
        crate::Call { completion: 1, position, tool: crate::Tool::Red(call) },
    );
    crate::route::route_into(domain, env, &mut decision);
    assert!(domain.start_stop().is_none(), "route start failure: {:?}", domain.start_stop());
    assert!(domain.work.is_empty(), "unrouted work: {:?}", domain.work);
    crate::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, false).unwrap();
    let writes = if let Some(JournalOutput::Commit { number, writes }) =
        crate::commit(&mut domain.journal, &env.limits.journal)
    {
        crate::committed(&mut domain.journal, number);
        writes
    } else {
        Box::default()
    };
    let mut answer = None;
    let mut out = Queue::uncapped();
    for _ in 0_u32..32 {
        crate::resume(&mut domain.journal, &mut out);
        if let Some(output) = out.pop() {
            let delivery = match output {
                JournalOutput::Deliver(delivery)
                | JournalOutput::Now(Request::Worker(WorkerRequest::Deliver(delivery))) => Some(delivery),
                JournalOutput::Now(_) | JournalOutput::Commit { .. } | JournalOutput::Stop => None,
            };
            if let Some(crate::Delivery::Host(delivery)) = delivery
                && let crate::HostDelivery::Render { answer: rendered, .. } = *delivery
            {
                answer = Some(rendered);
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
    assert!(writes.iter().filter_map(saved_red).any(|red| red.goal == 7
        && red.key.as_ref() == b"new"
        && red.by == forge::Recorder::Task(7)
        && red.at == env.wall.as_nanos()));
    assert!(writes.iter().filter_map(saved_reply).any(|kept| kept == reply));
    assert!(writes.iter().any(|write| {
        if let crate::Write::Save(crate::Record::Core(jig_domain_core::Record::Core(
            jig_domain_core::CoreRecord::Call(_),
        ))) = write
        {
            true
        } else {
            false
        }
    }));
    // Restoring the connector snapshot leaves the core call's unsettled replay unchanged.
    let payload =
        domain.forge.named_answer(forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 }).unwrap();
    domain.forge.forget_named_answer(forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 });
    restore(
        &mut domain,
        &env,
        forge::Stored::Call {
            key: forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 },
            answer: payload,
        },
    );
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
    for (task, key, expected) in [
        (7, b"a".as_slice(), None),
        (7, b"missing", Some(forge::RedRefusal::NoSuch)),
        (20, b"a", Some(forge::RedRefusal::NoGoal)),
    ] {
        let (mut domain, env) = fixture(b"main");
        let (answer, writes) = tool_call(
            &mut domain,
            &env,
            task,
            1,
            crate::RedCall::End { key: key.into(), why: b"fixed".as_slice().into() },
        );
        let crate::CallAnswer::Red(reply) = answer else { panic!("red answer") };
        let crate::RedToolAnswer::Write(reply) = *reply else { panic!("write") };
        if let Some(expected) = expected {
            assert_eq!(reply.result, Err(expected));
        } else {
            let Ok(forge::RedOutcome::Ended(red)) = reply.result else { panic!("ended") };
            assert_eq!(red.key.as_ref(), b"a");
            assert_eq!(red.why.as_ref(), b"base failure");
            assert!(writes.iter().any(|write| {
                if let crate::Write::Erase(crate::Key::Forge(forge::Key::Red { goal: 7, key })) = write {
                    key.as_ref() == b"a"
                } else {
                    false
                }
            }));
        }
        assert!(!held(domain.core.tasks.task(task).unwrap()));
    }
}
#[test]
fn red_list_is_a_fresh_goal_local_read_with_no_call_record() {
    for task in [7, 20] {
        let (mut domain, env) = fixture(b"main");
        let (answer, writes) = tool_call(&mut domain, &env, task, 1, crate::RedCall::List);
        assert!(writes.is_empty());
        assert!(
            crate::route::call_answer(&domain, crate::CallKey { task, attempt: 1, completion: 1, position: 1 })
                .is_none()
        );
        let crate::CallAnswer::Red(reply) = answer else { panic!("red list") };
        if task == 7 {
            let crate::RedToolAnswer::List { goal, bound, reds } = *reply else { panic!("list") };
            assert_eq!((goal, bound), (7, env.limits.forge.branch_reds));
            assert_eq!(reds.iter().map(|red| red.key.as_ref()).collect::<Box<[_]>>().as_ref(), [b"a".as_slice(), b"z"]);
        } else {
            assert_eq!(*reply, crate::RedToolAnswer::NoGoal { task });
        }
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
    let tasks::Phase::Held { source: Some(source), .. } = &task.phase else {
        panic!("held red goal: {:?}", task.phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("red refusal account") };
    assert_eq!(
        words.as_ref(),
        b"red: goal 7 holds its most reds (at most 2); end one, or look at what its branch carries"
    );
    assert!(writes.iter().any(|write| {
        if let crate::Write::Save(crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) =
            write
        {
            task.number == 7 && held(task)
        } else {
            false
        }
    }));
    assert!(writes.iter().find_map(saved_reply).is_some());
    assert!(!writes.iter().any(|write| if let crate::Write::Save(crate::Record::Forge { row }) = write {
        if let forge::Stored::Red(_) | forge::Stored::RedNumber { .. } = row.as_ref() { true } else { false }
    } else {
        false
    }));
}
#[test]
fn a_red_key_past_the_configured_byte_limit_decides_nothing() {
    for call in
        [record(b"new", 9), crate::RedCall::End { key: b"new".as_slice().into(), why: b"fixed".as_slice().into() }]
    {
        let (mut domain, mut env) = fixture(b"main");
        env.limits.forge.red_key_bytes = 2;
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::red_tool::call(
            &mut domain,
            &env,
            &mut decision,
            skein_lib::ReplyTo::new(Token::new(99)),
            crate::CallKey { task: 7, attempt: 1, completion: 1, position: 1 },
            call,
        );
        assert!(decision.write_free());
        assert_eq!(domain.forge.reds(7).count(), 2);
        assert!(
            domain.forge.named_answer(forge::NamedCall { task: 7, attempt: 1, completion: 1, position: 1 }).is_none()
        );
    }
}

fn saved_red(write: &crate::Write) -> Option<&forge::Red> {
    if let crate::Write::Save(crate::Record::Forge { row }) = write
        && let forge::Stored::Red(red) = row.as_ref()
    {
        Some(red)
    } else {
        None
    }
}
fn saved_reply(write: &crate::Write) -> Option<&forge::RedReply> {
    if let crate::Write::Save(crate::Record::Forge { row }) = write
        && let forge::Stored::Call { answer: forge::CallPayload::Red(reply), .. } = row.as_ref()
    {
        Some(reply)
    } else {
        None
    }
}
fn held(task: &tasks::TaskRecord) -> bool {
    if let tasks::Phase::Held { .. } = task.phase { true } else { false }
}

fn report() -> crate::wire::CheckResults {
    crate::wire::CheckResults {
        tag: forge::tag(1, 1),
        check: 2,
        tree: Box::new([0xab; 20]),
        base: None,
        verdict: crate::wire::Verdict::Passed,
        passed: Box::new([b"a".as_slice().into()]),
        failed: Box::new([]),
        flakes: Box::new([]),
    }
}
fn flake() -> crate::wire::Flake {
    crate::wire::Flake { key: b"test fabrig-x::t b".as_slice().into(), log: b"boom".as_slice().into() }
}
fn decide_report(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    results: crate::wire::CheckResults,
) -> Box<[crate::Write]> {
    domain.limits = env.limits;
    domain.work.push(crate::boundary::Work::CheckResults { task, attempt, results });
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_into(domain, env, &mut decision);
    assert!(domain.start_stop().is_none(), "{:?}", domain.start_stop());
    assert!(domain.work.is_empty());
    crate::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, false).unwrap();
    if let Some(crate::JournalOutput::Commit { number, writes }) =
        crate::commit(&mut domain.journal, &env.limits.journal)
    {
        crate::committed(&mut domain.journal, number);
        let mut out = Queue::uncapped();
        crate::resume(&mut domain.journal, &mut out);
        writes
    } else {
        Box::default()
    }
}
#[test]
fn a_reported_pass_ends_its_red_when_the_changes_landing_is_read_back() {
    use temper_engine_domain_forge_change as change;
    use temper_engine_domain_forge_landing as landing;
    let (mut domain, env) = fixture(b"temper/8/g7");
    decide_report(&mut domain, &env, 20, 1, report());
    let mut row = domain.forge.change(20).unwrap().clone();
    row.change.way = change::Way::Turn;
    row.change.landing = Some(61);
    row.change.state = change::State::Landing { head: [1; 32] };
    row.change.last_head = Some([1; 32]);
    restore(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    restore(
        &mut domain,
        &env,
        forge::Stored::Landing(forge::LandingRow {
            job: None,
            landing: landing::Landing {
                task: 61,
                project: 1,
                phase: landing::Phase::Landed,
                round: landing::Round::new(Box::new([])),
                last_left: None,
                branch: b"temper/8/g7".as_slice().into(),
                preparation: 0,
                stale_proofs: 0,
                members: Box::new([landing::Member {
                    round: None,
                    change: 20,
                    repository: landing::Repository { forge: 1, repository: 1 },
                    pull: 1,
                    head: Some([1; 32]),
                    base: None,
                    candidate: Some([2; 32]),
                    gates: Box::new([]),
                    asked: None,
                    landed: Some([2; 32]),
                    left: None,
                    unit: 0,
                    branch: b"temper/20".as_slice().into(),
                    combined: None,
                    recorded: true,
                    told: false,
                    judging: false,
                    leave_told: false,
                }]),
            },
            asks: Box::new([None]),
            entries: Box::new([]),
        }),
    );
    let event = super::super::change_step(&domain, &env, 20, 1, adopted(1).provider, b"temper/8/g7", false);
    let mut out = Queue::uncapped();
    forge::step(&mut domain.forge, &env.with(env.limits.forge), event, &mut out);
    assert_eq!(domain.forge.reds(7).map(|(_, red)| red.key.as_ref()).collect::<Box<[_]>>().as_ref(), [b"z".as_slice()]);
    assert!(out.iter().any(|request| *request
        == forge::Request::Erase { key: forge::Key::RedPass { change: 20, key: b"a".as_slice().into() } }));
}

fn saved_pass(write: &crate::Write) -> Option<&forge::RedPass> {
    if let crate::Write::Save(crate::Record::Forge { row }) = write
        && let forge::Stored::RedPass(pass) = row.as_ref()
    {
        Some(pass)
    } else {
        None
    }
}
#[test]
fn passed_reports_keep_the_latest_red_number_and_failed_reports_erase_the_pass() {
    let (mut domain, env) = fixture(b"temper/8/g7");
    for task in [20, 24] {
        let writes = decide_report(&mut domain, &env, task, 1, report());
        assert_eq!(
            writes.iter().filter_map(saved_pass).collect::<Box<[_]>>().as_ref(),
            [&forge::RedPass { change: 20, key: b"a".as_slice().into(), generation: 3 }]
        );
    }
    let mut refused = report();
    refused.verdict = crate::wire::Verdict::Refused;
    refused.failed = Box::new([b"a".as_slice().into()]);
    assert!(decide_report(&mut domain, &env, 20, 1, refused).is_empty());
    let mut results = report();
    results.passed = Box::new([]);
    results.failed = Box::new([b"a".as_slice().into()]);
    let writes = decide_report(&mut domain, &env, 20, 1, results);
    assert!(writes.iter().any(|write| *write
        == crate::Write::Erase(crate::Key::Forge(forge::Key::RedPass { change: 20, key: b"a".as_slice().into() }))));
    let writes = decide_report(&mut domain, &env, 20, 1, report());
    assert_eq!(writes.iter().filter_map(saved_pass).count(), 1);
}
#[test]
fn reports_without_a_passed_current_run_and_matching_changes_goal_keep_no_passes() {
    for (base, task, attempt, refused, other) in [
        (b"temper/8/g7".as_slice(), 20, 1, true, false),
        (b"temper/8/g7", 8, 1, false, false),
        (b"main", 20, 1, false, false),
        (b"temper/8/g7", 999, 1, false, false),
        (b"temper/8/g7", 23, 1, false, false),
        (b"temper/8/g7", 20, 2, false, false),
        (b"temper/8/g7", 20, 1, false, true),
    ] {
        let (mut domain, env) = fixture(base);
        proof(&mut domain, 999);
        let mut results = report();
        if refused {
            results.verdict = crate::wire::Verdict::Refused;
        }
        if task == 23 {
            tasks::step(
                &mut domain.core.tasks,
                &env.with(env.limits.tasks),
                tasks::Event::Cancel {
                    reply_to: skein_lib::ReplyTo::new(Token::new(200)),
                    by: tasks::Party::Deployment { project: 1 },
                    operation: tasks::Cancel { task, reason: b"ended".as_slice().into(), grace: None, message: None },
                },
                &mut Queue::uncapped(),
            );
            assert!(match domain.core.tasks.task(task).unwrap().phase {
                tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => true,
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => false,
            });
        }
        if task == 999 || task == 23 || attempt == 2 {
            results.flakes = Box::new([flake()]);
        }
        if other {
            let mut repository = adopted(1);
            repository.provider.repository = 2;
            repository.project = domain.core.tasks.task(20).unwrap().project;
            restore(&mut domain, &env, forge::Stored::Repository(repository));
            results.tag = forge::tag(1, 2);
        }
        let writes = decide_report(&mut domain, &env, task, attempt, results);
        assert!(writes.is_empty(), "task {task}, attempt {attempt}, refused {refused}, other {other}: {writes:?}");
    }
}
fn triager(domain: &mut Domain, env: &Env<Limits>) {
    let project = domain.core.tasks.task(20).unwrap().project;
    domain.forge.keep_settings(
        &crate::route::environment_forge(env),
        1,
        None,
        &[forge::settings::Asked {
            kind: 15,
            scope: project,
            number: 1,
            edit: forge::settings::Edit::Set(forge::settings::SizedValue {
                value: forge::settings::Value::ProjectTriager(forge::settings::ProjectTriager {
                    owner: forge::settings::AreaOwner::Task(8),
                }),
                size: 1,
            }),
        }],
        &mut Queue::uncapped(),
    );
}
#[test]
fn check_flakes_file_with_the_triager_and_count_silently_even_when_refused() {
    let (mut domain, mut env) = fixture(b"temper/8/g7");
    env.limits.tasks.message_bytes = 512;
    env.limits.tasks.inbox_bytes = 2048;
    triager(&mut domain, &env);
    let project = domain.core.tasks.task(20).unwrap().project;
    let mut results = report();
    results.flakes = Box::new([flake()]);
    let writes = decide_report(&mut domain, &env, 20, 1, results);
    let row = domain.forge.finding(project, 1).unwrap();
    assert_eq!(
        (row.kind, row.severity, row.area.as_deref(), row.filer, row.seen),
        (forge::FindingKind::Flake, forge::Severity::Low, None, forge::Filer::Task(20), Some(1))
    );
    assert_eq!(row.title.as_ref(), b"Flake in a delivery's check: test fabrig-x::t b");
    assert_eq!(row.body.as_ref(), b"A test failed in a delivery's check and passed when its process ran again, with nothing else of the check running.\nRepository: org/repo\nTest: test fabrig-x::t b\nSeen in task 20's check 2 (attempt 1): tree abababababababababababababababababababab, base none.\nIts log under load:\nboom");
    assert_eq!(
        row.signature,
        Some(forge::FindingSignature {
            repository: adopted(1).provider,
            source: b"check".as_slice().into(),
            ids: Box::new([flake().key]),
            first: Box::new([]),
        })
    );
    let subscription =
        domain.forge.topic_subscriber(8, &forge::Topic::Findings { project }).expect("triager subscribed");
    assert!(domain.core.tasks.task(8).unwrap().subscriptions.iter().any(|row| row.number == subscription.number));
    let news = domain
        .core
        .tasks
        .task(8)
        .unwrap()
        .inbox
        .iter()
        .find(|word| word.words.starts_with(b"Finding "))
        .expect("triager hears filed flake");
    assert_eq!(
        news.kind,
        tasks::MessageKind::News { subscription: subscription.number, class: tasks::NewsClass::Wakes }
    );
    assert!(
        news.words.starts_with(
            b"Finding 1/1 is yours: Flake in a delivery's check: test fabrig-x::t b\nflake, low, no area\n"
        )
    );
    assert!(writes.iter().find_map(saved_pass).is_some(), "one decision also keeps passes");
    let before = row.clone();
    let mut results = report();
    results.verdict = crate::wire::Verdict::Refused;
    results.check = 3;
    results.flakes = Box::new([flake()]);
    let writes = decide_report(&mut domain, &env, 20, 1, results);
    let mut counted = before;
    counted.seen = Some(2);
    assert_eq!(domain.forge.finding(project, 1), Some(&counted));
    assert_eq!(domain.forge.findings(project).count(), 1);
    assert!(
        writes.iter().all(|write| if let crate::Write::Save(crate::Record::Forge { row }) = write {
            if let forge::Stored::Finding(finding) = row.as_ref() { finding == &counted } else { false }
        } else {
            if let crate::Write::Save(crate::Record::Core(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(_),
            ))) = write
            {
                true
            } else {
                false
            }
        }),
        "counting emits no task news: {writes:?}"
    );
    let mut results = report();
    results.verdict = crate::wire::Verdict::Refused;
    results.flakes = Box::new([crate::wire::Flake { key: b"another".as_slice().into(), log: Box::new([]) }]);
    decide_report(&mut domain, &env, 20, 1, results);
    assert_eq!(domain.forge.finding(project, 2).unwrap().seen, Some(1));
}
#[test]
fn a_full_findings_project_drops_only_the_refused_flake() {
    let (mut domain, mut env) = fixture(b"temper/8/g7");
    env.limits.forge.findings_per_project = 1;
    let project = domain.core.tasks.task(20).unwrap().project;
    let mut results = report();
    results.flakes = Box::new([flake()]);
    decide_report(&mut domain, &env, 20, 1, results);
    let mut results = report();
    results.flakes = Box::new([crate::wire::Flake { key: b"new".as_slice().into(), log: Box::new([]) }, flake()]);
    let writes = decide_report(&mut domain, &env, 20, 1, results);
    assert_eq!(domain.forge.findings(project).count(), 1);
    assert_eq!(domain.forge.finding(project, 1).unwrap().seen, Some(2));
    assert!(writes.iter().find_map(saved_pass).is_some());
}
#[test]
#[expect(clippy::disallowed_macros, reason = "assertions construct expected UTF-8 words")]
fn flake_words_cut_at_character_boundaries_and_name_unnamed_repositories_by_number() {
    let (mut domain, env) = fixture(b"temper/8/g7");
    let project = domain.core.tasks.task(20).unwrap().project;
    let mut repository = domain.forge.repository_tag(project, forge::tag(1, 1)).unwrap();
    repository.owner = Box::new([]);
    restore(&mut domain, &env, forge::Stored::Repository(repository.clone()));
    let mut results = report();
    results.base = Some(Box::new([0xcd; 32]));
    results.flakes = Box::new([crate::wire::Flake {
        key: "é".repeat(100).into_bytes().into_boxed_slice(),
        log: "é".repeat(2048).into_bytes().into_boxed_slice(),
    }]);
    decide_report(&mut domain, &env, 20, 1, results);
    let row = domain.forge.finding(project, 1).unwrap();
    let title = core::str::from_utf8(&row.title).unwrap();
    assert_eq!(title, format!("Flake in a delivery's check: {}", "é".repeat(85)));
    assert_eq!(row.title.len(), 199);
    let body = core::str::from_utf8(&row.body).unwrap();
    assert!(body.contains("Repository: forge 1, repository 1\n"));
    assert!(body.contains(&format!("base {}.", "cd".repeat(32))));
    assert!(body.ends_with("é[cut]"));
    assert!((4095..=4096).contains(&row.body.len()));
}
