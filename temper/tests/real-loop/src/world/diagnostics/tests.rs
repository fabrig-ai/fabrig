//! Diagnostic hooks and bounded history (temper/docs/design/testing/real-loop.md).
use super::terminals::Terminals;
use super::{Context, Judge, Operation, Operations, ROWS, Rows, Summary, Task, Tasks};
use crate::{
    calls, case,
    patience::Patience,
    world::{Kind, Proc},
};
use jig_domain_tasks as tasks_domain;
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use skein_shell::Host;
use std::collections::BTreeMap;

const START: Time = Time::from_nanos(10);
const NOW: Time = Time::from_nanos(30);

fn tasks() -> Tasks {
    let mut rows = BTreeMap::new();
    for (number, parent, reviewer, procedure) in [
        (crate::case::A, None, false, false),
        (crate::case::DESIGN, Some(crate::case::A), false, true),
        (crate::case::REVIEW, Some(crate::case::DESIGN), true, false),
    ] {
        rows.insert(
            number,
            Task {
                parent,
                phase: "running",
                closing: None,
                reviewer,
                procedure,
                live: true,
                completed: Summary::default(),
            },
        );
    }
    Tasks { rows, ..Tasks::default() }
}

fn terminal_frame(numbers: &[u64]) -> Box<[u8]> {
    // Jig owner, tasks child and Ended kind tags, followed by the u64 task.
    const TERMINAL_KEY_BYTES: u32 = 3 + 8;
    let mut body = Vec::new();
    for number in numbers {
        let key = jig_protocol_store::keys::encode(
            &jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Ended(*number)),
            TERMINAL_KEY_BYTES,
        )
        .expect("terminal key");
        body.push(1);
        body.extend_from_slice(&u32::try_from(key.len()).expect("small key").to_be_bytes());
        body.extend_from_slice(&key);
        body.extend_from_slice(&0_u32.to_be_bytes());
    }
    let mut bytes = Vec::from(b"SKVC");
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.extend_from_slice(&1_u64.to_be_bytes());
    bytes.extend_from_slice(&u32::try_from(numbers.len()).expect("small batch").to_be_bytes());
    bytes.extend_from_slice(&u32::try_from(body.len()).expect("small body").to_be_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    bytes.into_boxed_slice()
}

fn journal(numbers: &[u64]) -> Queue<Complete> {
    let bytes = terminal_frame(numbers);
    let fd = Fd::new(4);
    let mut completions = Queue::uncapped();
    completions.push(Complete {
        op: Token::new(1),
        kind: Op::Open {
            root: Fd::new(0),
            path: b"log-00000000000000000001".as_slice().into(),
            how: skein_io::kernel::OpenHow::Read,
        },
        result: Ok(Done::Fd(fd)),
    });
    let count = u32::try_from(bytes.len()).expect("small write");
    completions.push(Complete {
        op: Token::new(2),
        kind: Op::Write { fd, bytes, from: 0, at: 0 },
        result: Ok(Done::Count(count)),
    });
    completions.push(Complete { op: Token::new(3), kind: Op::Sync { fd }, result: Ok(Done::Nothing) });
    completions
}

fn end(tasks: &mut Tasks, terminals: &mut Terminals, numbers: &[u64]) {
    terminals.observe(&journal(numbers));
    tasks.observe_terminals(terminals).expect("terminal rows");
}

fn review_tree() -> (tasks_domain::Domain, Env<tasks_domain::Limits>) {
    use tasks_domain::*;
    let env = Env { limits: jig_protocol_config_fixtures::small().core.tasks, now: START, wall: Wall::EPOCH };
    let charters = [1, super::REVIEW_CHARTER].map(|number| CharterModels {
        number,
        resumes: false,
        revision: 1,
        models: Some(Box::new([0])),
        speeds: Box::default(),
    });
    let mut tree =
        Domain::new(&env.limits, Kept { revisions: 4, versions: 8 }, 91, 1, Box::new(charters), Box::new([0]));
    let reply_to = || ReplyTo::new(Token::new(1));
    let mut out = Queue::uncapped();
    for event in [
        Event::Check,
        Event::Restored,
        Event::OpenPeriod { reply_to: reply_to(), project: 1, period: 0, budget: 10_000 },
    ] {
        step(&mut tree, &env, event, &mut out);
    }
    for (number, creator, executor) in [
        (case::A, Party::Person(9), Executor::Agent { charter: 1 }),
        (case::DESIGN, Party::Task(case::A), Executor::Procedure { connector: 1, code: 1 }),
        (case::REVIEW, Party::Task(case::DESIGN), Executor::Agent { charter: super::REVIEW_CHARTER }),
    ] {
        let task = New {
            number,
            project: 1,
            executor,
            funder: Funder::Period { project: 1, period: 0 },
            deadline: Deadline::Named,
            label: 0,
            flags: Flags::PLAIN,
            llm: None,
            spec: Spec {
                job: None,
                asks: false,
                words: b"review".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: Contract::Report { words: 32 },
            authority: Authority {
                tools: Tools(0),
                models: Models::All,
                grants: Box::new([]),
                delegation: Delegation { kinds: Box::new([]), tasks: 4, depth: 4 },
                budget: Budget { spend: 100, deadline: None },
                notes: Scopes(0),
                note_resources: Box::new([]),
            },
            numbers: Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            dependencies: Box::new([]),
            holdings: Box::new([]),
            wake: WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        };
        out.clear();
        step(
            &mut tree,
            &env,
            Event::Make {
                reads_project: false,
                maker: None,
                places: Box::new([]),
                reply_to: reply_to(),
                creator,
                batch: Box::new([task]),
            },
            &mut out,
        );
        assert!(tree.task(number).is_some(), "could not make task {number}: {out:?}");
    }
    (tree, env)
}

fn finish_review(tree: &mut tasks_domain::Domain, env: &Env<tasks_domain::Limits>) {
    use tasks_domain::*;
    let task = case::REVIEW;
    let reply_to = || ReplyTo::new(Token::new(1));
    let mut out = Queue::uncapped();
    for event in [
        Event::Prepare { reply_to: reply_to(), task },
        Event::Claim {
            fresh: None,
            reply_to: reply_to(),
            task,
            attempt: 1,
            resumed: false,
            budget: 1,
            account: None,
            writes: Box::new([]),
            reasons: Reasons::FIRST,
            offered: Reasons::ALL,
            session_end: None,
        },
        Event::Activation {
            reads_project: false,
            reply_to: reply_to(),
            task,
            attempt: 1,
            end: End::Finished {
                result: TaskResult::Report { words: b"reviewed".as_slice().into() },
                cancel_delegates: false,
            },
            saved: None,
            cause: Cause::Priced { cumulative: 0 },
        },
        Event::EffectsSettled { task },
        Event::Settled { task },
    ] {
        step(tree, env, event, &mut out);
    }
    assert!(
        out.iter().any(|request| matches!(request, Request::Save { record: Stored::Ended(row) }
        if row.number == task && matches!(row.phase, Phase::Ended(Ending::Done(_))))),
        "ordinary review ended: {out:?}"
    );
    assert!(tree.task(task).is_none());
    assert!(tree.stub(task).is_none(), "ordinary delegate completion needs no stub");
}

#[test]
fn a_durable_review_terminal_reaches_the_forced_overdue_report() {
    let (mut tree, env) = review_tree();
    let mut caller = calls::Caller::new(
        vec![calls::call(calls::D1, crate::argv!["wait", "task", calls::number(case::A), "--until", "ended"])],
        std::path::PathBuf::new(),
        Fd::new(0),
    );
    caller.released = true;
    let mut procs = [Proc::new(Kind::Caller(Box::new(caller))), crate::world::tests::ctl(false)];
    // No kernel: the caller emits a spawn request; the admitted ctl and its pipe read are scripted.
    procs[0].iterate(START, Wall::EPOCH);
    procs[0].drain();
    let mut reads = Queue::uncapped();
    reads.push(Submit { op: Token::new(9), kind: Op::PipeRead { fd: Fd::new(5), buf: Box::new([0; 8]) } });
    procs[1].diagnostics.complete(&Queue::uncapped(), START, false);
    procs[1].diagnostics.submit(&reads);

    let mut engine = Operations::default();
    let mut judge = Judge::new(Patience::scripted(crate::PATIENCE, START));
    assert!(!judge.act(START, &mut procs, Context::default()));
    judge.watch(&procs, std::iter::once((1, &tree, &engine))).expect("live task snapshot");
    finish_review(&mut tree, &env);
    let terminal = journal(&[case::REVIEW]);
    let mut submissions = Queue::uncapped();
    let sync = Token::new(3);
    submissions.push(Submit { op: sync, kind: Op::Sync { fd: Fd::new(4) } });
    engine.complete(&Queue::uncapped(), START, true);
    engine.submit(&submissions);
    assert!(engine.pending.contains_key(&sync));
    let synced = Time::from_nanos(20);
    engine.complete(&terminal, synced, true);
    judge.watch(&procs, std::iter::once((2, &tree, &engine))).expect("synced task snapshot");
    // The observer's journal witness is independent of the judge's task projection.
    assert_eq!(engine.terminals.ended, [case::REVIEW]);
    assert_eq!(engine.terminals.last_sync, Some(sync));
    assert!(!engine.pending.contains_key(&sync), "completion removes the witnessed sync");

    judge.patience.force_failed();
    assert!(NOW < judge.patience.deadline().expect("armed diagnostic deadline"));
    assert!(judge.act(NOW, &mut procs, Context::default()));
    let message = judge
        .watch(&procs, std::iter::once((2, &tree, &engine)))
        .expect_err("the review terminal forces the overdue dispatch");
    for expected in [
        "the referee failed the real loop before patience",
        "task=2 state=landing",
        "wait=story-step step=0 setup=false age_ns=20",
        "kind=ctl age_ns=20",
        "child-run=ctl observed_age_ns=Some(20)",
        "operation=pipe-read token=9 age_ns=20",
        "outstanding total=5 shown=5 omitted=0",
        "caller step=0",
        "forced failed deadline figures",
    ] {
        assert!(message.contains(expected), "missing {expected}: {message}");
    }
    let design =
        message.lines().find(|line| line.starts_with("task=3 ")).unwrap_or_else(|| panic!("missing task 3: {message}"));
    assert!(design.contains("reviewed_below=1"), "missing completed reviewer: {design}");
    assert!(!message.contains("task=5 state=absent"), "the review terminal has no stub: {message}");
}

#[test]
fn task_progress_keeps_each_ending_substate_and_closing_obligation() {
    let mut tasks = tasks();
    let mut terminals = Terminals::default();
    let review = tasks.rows.remove(&crate::case::REVIEW).expect("review fixture");
    assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "delegated");
    tasks.rows.insert(crate::case::REVIEW, review);
    assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "reviewing");
    end(&mut tasks, &mut terminals, &[crate::case::REVIEW]);
    assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "landing");
    end(&mut tasks, &mut terminals, &[crate::case::DESIGN]);
    assert_eq!(tasks.state(crate::case::A, &tasks.rows[&crate::case::A]), "reporting");
    let a = tasks.rows.get_mut(&crate::case::A).expect("A");
    a.phase = "closing";
    a.closing = Some(jig_domain_tasks::Stage::Run { attempt: 7 });
    let mut report = String::new();
    tasks.describe(&mut report);
    assert!(report.contains("task=2 state=reporting phase=closing closing=Some(Run { attempt: 7 })"), "{report}");
    end(&mut tasks, &mut terminals, &[crate::case::A]);
    report.clear();
    tasks.describe(&mut report);
    assert!(report.contains("task history state=ended count=3 reclaimed=3 retained=0"), "{report}");
}

#[test]
fn grouped_short_writes_report_terminals_only_after_successful_sync() {
    let mut opening = journal(&[]);
    let open = opening.pop().expect("log open");
    let _write = opening.pop().expect("empty frame");
    let sync = opening.pop().expect("log sync");
    let mut bytes = terminal_frame(&[crate::case::REVIEW]).into_vec();
    bytes.extend_from_slice(&terminal_frame(&[crate::case::DESIGN]));
    let count = u32::try_from(bytes.len()).expect("small group");
    let prefix = count / 2;
    let mut terminals = Terminals::default();
    let mut first = Queue::uncapped();
    first.push(open);
    first.push(Complete {
        op: Token::new(2),
        kind: Op::Write { fd: Fd::new(4), bytes: bytes.clone().into_boxed_slice(), from: 0, at: 0 },
        result: Ok(Done::Count(prefix)),
    });
    terminals.observe(&first);
    assert!(terminals.ended.is_empty(), "a partial write is not a terminal witness");
    let mut suffix = Queue::uncapped();
    suffix.push(Complete {
        op: Token::new(2),
        kind: Op::Write { fd: Fd::new(4), bytes: bytes.into_boxed_slice(), from: prefix, at: 0 },
        result: Ok(Done::Count(count - prefix)),
    });
    terminals.observe(&suffix);
    assert!(terminals.ended.is_empty(), "the completed group still owes its sync");
    let mut synced = Queue::uncapped();
    synced.push(sync);
    terminals.observe(&synced);
    assert_eq!(terminals.ended, [crate::case::REVIEW, crate::case::DESIGN]);
    terminals.observe(&Queue::uncapped());
    assert!(terminals.ended.is_empty(), "a later pass cannot repeat a terminal witness");
}

#[test]
fn completed_subtrees_leave_room_for_a_new_live_blocker() {
    // Four complete output-cap batches must not consume retained task rows.
    const ROOTS: usize = ROWS * 4;
    let mut tasks = Tasks::default();
    let mut terminals = Terminals::default();
    let live = u64::try_from(ROOTS + 1).expect("small task number");
    for number in 1..=live {
        let row = Task {
            parent: None,
            phase: "running",
            closing: None,
            reviewer: false,
            procedure: false,
            live: true,
            completed: Summary::default(),
        };
        tasks.rows.insert(number, row);
        if number != live {
            end(&mut tasks, &mut terminals, &[number]);
        }
        assert!(tasks.rows.len() <= 1, "completed rows are reclaimed");
    }
    let mut report = String::new();
    tasks.describe(&mut report);
    assert!(report.contains("task history state=ended count=128 reclaimed=128 retained=1"), "{report}");
    assert!(report.contains("task=129 state=running"), "{report}");
    assert!(report.contains("task substates total=1 shown=1 omitted=0"), "{report}");
}

#[test]
fn operation_completion_and_token_reuse_reset_only_the_new_age() {
    let mut operations = Operations::default();
    let mut submissions = Queue::uncapped();
    let mut completions = Queue::uncapped();
    let token = Token::new(1);
    operations.complete(&completions, START, false);
    submissions.push(Submit { op: token, kind: Op::Wait { pidfd: Fd::new(4), reap: false } });
    operations.submit(&submissions);
    operations.now = Time::from_nanos(20);
    operations.submit(&submissions);
    let mut rows = Rows::default();
    operations.describe(0, NOW, false, &mut rows);
    let mut report = String::new();
    rows.finish("outstanding", &mut report);
    assert!(report.contains("operation=wait token=1 age_ns=20"), "{report}");
    completions.push(Complete {
        op: token,
        kind: Op::Wait { pidfd: Fd::new(4), reap: false },
        result: Ok(Done::Exit(crate::Exit::Code(0))),
    });
    operations.complete(&completions, NOW, false);
    assert!(operations.pending.is_empty());
    operations.submit(&submissions);
    assert_eq!(operations.pending[&token].since, NOW);
}

#[test]
fn the_list_reports_omissions_on_both_sides_of_its_cap() {
    for (count, expected) in [(ROWS, "total=32 shown=32 omitted=0"), (ROWS + 1, "total=33 shown=32 omitted=1")] {
        let mut rows = Rows::default();
        for _ in 0..count {
            rows.add(format_args!("pending"));
        }
        let mut report = String::new();
        rows.finish("outstanding", &mut report);
        assert!(report.contains(expected), "{report}");
        assert_eq!(report.lines().count(), ROWS + 1);
    }
}

#[test]
fn passive_listeners_cannot_hide_an_ending_workers_sync() {
    let caller = || {
        Proc::new(Kind::Caller(Box::new(crate::calls::Caller::new(Vec::new(), std::path::PathBuf::new(), Fd::new(0)))))
    };
    let mut procs = [caller(), caller()];
    for token in 1..=ROWS {
        procs[0]
            .diagnostics
            .pending
            .insert(Token::new(u64::try_from(token).expect("small list")), Operation { kind: "accept", since: START });
    }
    procs[1].diagnostics.pending.insert(Token::new(1), Operation { kind: "sync", since: START });
    let mut report = String::new();
    super::outstanding(&procs, NOW, &mut report);
    assert!(
        report.starts_with("outstanding total=33 shown=32 omitted=1\nproc=1 operation=sync token=1 age_ns=20\n"),
        "{report}"
    );
    assert_eq!(report.lines().count(), ROWS + 1);
}

#[test]
fn timer_ages_follow_the_first_observation_of_each_deadline() {
    let mut operations = Operations { now: START, ..Operations::default() };
    let first = Some(NOW.saturating_add(Duration::from_nanos(1)));
    operations.timers(first, None);
    operations.now = NOW;
    operations.timers(first, None);
    assert_eq!(operations.timers[0].as_ref().expect("timer").since, START);
    operations.timers(Some(NOW), first);
    assert_eq!(operations.timers[0].as_ref().expect("timer").since, NOW);
    operations.timers(None, None);
    assert!(operations.timers.iter().all(Option::is_none));
}
