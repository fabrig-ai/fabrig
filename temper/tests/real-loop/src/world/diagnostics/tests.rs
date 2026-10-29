//! Diagnostic hooks and bounded history (temper/docs/design/domain/testing.md, 2.4).
use super::terminals::Terminals;
use super::{Operation, Operations, ROWS, Rows, Summary, Task, Tasks};
use crate::world::{Kind, Proc};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Duration, Queue, Time, Token};
use skein_world::{Referee, real::World};
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

struct Forced {
    judge: crate::world::Judge,
}
impl Referee<Proc> for Forced {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        self.judge.act(now, procs);
    }
    fn observe(&mut self, now: Time, procs: &[Proc]) {
        self.judge.observe(now, procs);
        if procs[1].diagnostics.terminals.ended.contains(&crate::case::REVIEW) {
            // An independent terminal witness triggers the fake verdict, even if
            // the judge's task-observation hook has been disconnected.
            let Kind::Service(host) = &procs[1].kind else { panic!("engine fixture") };
            let engine = host.service.as_ref().expect("prepared engine").engine_domain().expect("engine domain");
            assert!(engine.tasks().stub(crate::case::REVIEW).is_none(), "ordinary delegate completion needs no stub");
            let sync = procs[1].diagnostics.terminals.last_sync.expect("terminal sync");
            assert!(!procs[1].diagnostics.pending.contains_key(&sync), "completion hook removes the finished sync");
            self.judge.patience.force_failed();
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        self.judge.next_deadline()
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.judge.overdue(now)
    }
    fn passed(&self) -> bool {
        self.judge.passed()
    }
}

#[test]
fn forced_overdue_observes_real_tasks_operations_and_dispatch() {
    use crate::world::{self, ForgePeer};
    const WORKERS: &[&str] = &["ws", "s1"];
    let scratch: Vec<_> = ["engine", "ws", "s1"].map(crate::Scratch::new).into();
    let state = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut world = World::new_controlled(|controls| Forced {
        judge: world::cluster_judge(controls, WORKERS, Some(state.clone()), crate::PATIENCE),
    });
    world::cluster_roots(
        &mut world,
        WORKERS,
        &[],
        &scratch,
        || crate::llm::scripted(Box::new([])),
        Some((crate::case::fixture(), crate::case::scripts::step_4_1, crate::case::scripts::step_4_1_rules)),
        crate::case::acts::step_4_1,
    );
    world.spawn(|| {
        let (forge, peer) = crate::forge::State::new(crate::case::fixture());
        *state.borrow_mut() = Some(forge);
        Proc::new(Kind::Forge(Box::new(ForgePeer::new(peer))))
    });
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _outcome = world.run(&skein_shell::Clock::new(), crate::PATIENCE);
    }))
    .expect_err("the review terminal forces the overdue dispatch");
    crate::git::share(None);
    let message = panic.downcast_ref::<String>().expect("formatted failure");
    for expected in [
        "the referee failed the real loop before patience",
        "task=2 state=landing",
        "task=3",
        "reviewed_below=1",
        "wait=story-step",
        "kind=ctl age_ns=",
        "child-run=ctl",
        "operation=pipe-read",
        "observed_age_ns=",
        "outstanding total=",
        "shown=",
        "omitted=",
        "caller step=",
        "forced failed deadline figures",
    ] {
        assert!(message.contains(expected), "missing {expected}: {message}");
    }
    assert!(!message.contains("task=5 state=absent"), "the review terminal has no stub: {message}");
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
    tasks.observe_terminals(terminals);
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
    operations.complete(&completions, START);
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
    operations.complete(&completions, NOW);
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
