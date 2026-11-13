use crate::read_support::request;
use jig_core_world::reads;
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core::{self as core, documents};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

#[test]
fn a_partys_request_after_the_stop_is_answered_not_ready() {
    let mut world = World::new(Setup { seed: 160, ..Setup::default() });
    world.stop();
    let commits = world.store.applied;
    let later_chat = jig_core_world::fixtures::chat(&world, 1, b"later".as_slice().into());
    request(&mut world, 55, later_chat);
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Again(people::Again::NotReady)));
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_turn_after_the_stop_is_answered_busy_and_not_committed() {
    let mut world = World::new(Setup { seed: 161, ..Setup::default() });
    world.stop();
    let commits = world.store.applied;
    world.turn(1, 1, None, b"a turn the engine will not take");
    assert_eq!(world.store.applied, commits, "nothing is committed after the stop");
    assert_eq!(world.busy_turns, [(world.task(), world.attempt(), 1)]);
}

#[test]
fn an_answer_after_the_stop_is_left_unacknowledged() {
    let mut world = World::new(Setup { seed: 162, ..Setup::default() });
    world.stop();
    let commits = world.store.applied;
    world.end(tasks::End::Parked, 0);
    assert!(!world.stop_deliveries.iter().any(|held| matches!(held, core::Held::Acknowledge { .. })));
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_call_after_the_stop_is_answered_busy() {
    let mut world = World::new(Setup { seed: 163, ..Setup::default() });
    world.stop();
    let commits = world.store.applied;
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Relay {
        channel: Token::new(7),
        run: Token::new(world.task()),
        attempt: Token::new(world.attempt()),
        call: fleet::Call {
            name: b"call".as_slice().into(),
            tool: b"tool".as_slice().into(),
            writes: true,
            input: Box::new([]),
            deadline: Duration::from_secs(2),
        },
    })));
    assert_eq!(world.busy_calls, [Box::<[u8]>::from(b"call".as_slice())]);
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_new_watch_after_the_stop_is_ended_as_stopping() {
    let mut world = World::new(Setup { seed: 164, ..Setup::default() });
    world.stop();
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(801),
        sign_in: world.sign_in.expect("signed in"),
        subject: documents::Watched::Task { task: world.task() },
        window: documents::Window { rows: 1, after: None },
    }));
    assert!(world.stop_deliveries.iter().any(
        |held| matches!(held, core::Held::WatchEnded { watcher, why: views::End::Stopping } if watcher.raw() == 801)
    ));
}

#[test]
fn no_run_is_placed_after_the_stop() {
    let mut world = World::new(Setup { seed: 165, ..Setup::default() });
    world.stop();
    let assignments = world.assignments.len();
    let commits = world.store.applied;
    world.send(root::Event::Timer(core::Timer::Tasks));
    world.send(root::Event::Timer(core::Timer::Fleet));
    let replacement_chat = jig_core_world::fixtures::chat(&world, 1, b"later".as_slice().into());
    request(&mut world, 56, replacement_chat);
    assert_eq!(world.assignments.len(), assignments);
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_read_after_the_stop_is_answered_as_before() {
    let mut world = crate::read_support::world(166);
    world.stop();
    let task = world.task();
    reads::ask(&mut world, 1, core::reads::Read::Task { task });
    assert!(matches!(reads::latest(&world), core::reads::Answer::Document(_)));
}

#[test]
fn a_connector_event_after_the_stop_is_handed_back_busy() {
    let mut world = World::new(Setup { seed: 167, ..Setup::default() });
    world.stop();
    let commits = world.store.applied;
    world.send(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::ApplicationDrop { owner: Token::new(9) },
    });
    assert!(matches!(world.busy_events.as_slice(), [root::Event::Connector { number: 1, .. }]));
    assert_eq!(world.store.applied, commits);
}

#[test]
fn every_open_watch_ends_saying_the_engine_is_stopping() {
    let mut world = jig_core_world::watches::world(168, true);
    world.stop();
    assert_eq!(
        world.watches.as_ref().expect("watch parties").endings,
        [(1, views::End::Stopping), (2, views::End::Stopping), (3, views::End::Stopping), (4, views::End::Stopping)]
    );
}

#[test]
fn drained_is_released_once_after_the_last_commit_is_durable() {
    let mut world = World::new(Setup { seed: 169, ..Setup::default() });
    let task = world.task();
    let commits = world.store.applied;
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    request(&mut world, 57, people::Ask::Say { project: 1, task, words: b"before the stop".as_slice().into() });
    world.stop();
    assert!(!world.stop_deliveries.iter().any(|held| matches!(held, core::Held::Drained)));
    let number = world.store.release_held().expect("the pending commit");
    assert!(number > commits);
    world.send(root::Event::Committed { number, fill: 0 });
    assert_eq!(world.stop_deliveries.iter().filter(|held| matches!(held, core::Held::Drained)).count(), 1);
    world.drain();
    assert_eq!(world.store.applied, number);
    assert_eq!(world.stop_deliveries.iter().filter(|held| matches!(held, core::Held::Drained)).count(), 1);
}

#[test]
fn a_second_stop_says_drained_no_second_time() {
    let mut world = World::new(Setup { seed: 170, ..Setup::default() });
    world.stop();
    world.stop();
    assert_eq!(world.stop_deliveries.iter().filter(|held| matches!(held, core::Held::Drained)).count(), 1);
}

#[test]
fn settling_commits_turns_and_answers_but_leaves_a_waiting_task_due() {
    let mut world = World::new(Setup { seed: 171, ..Setup::default() });
    let first = (world.task(), world.attempt());
    let chat = jig_core_world::fixtures::chat(&world, 1, b"waiting for the slot".as_slice().into());
    request(&mut world, 58, chat);
    let waiting = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if row.number != first.0 => {
                Some(row.number)
            }
            _ => None,
        })
        .expect("second task");
    assert_eq!(world.assignments.len(), 1, "the worker has one slot");
    let commits = world.store.applied;
    for _ in 0..2 {
        world.send(root::Event::Core(core::Event::Settling));
        assert!(world.domain.core().settling());
        assert!(!world.domain.core().stopping());
        assert_eq!(world.store.applied, commits, "settling is idempotent and writes nothing");
    }
    world.turn(1, 1, None, b"a turn while settling");
    assert!(world.store.applied > commits);
    assert_eq!(world.busy_turns.len(), 0);
    assert!(world.stop_deliveries.iter().any(|held| matches!(held,
        core::Held::AcknowledgeTurn { run, attempt, turn: 1, .. } if (run.raw(), attempt.raw()) == first
    )));
    let commits = world.store.applied;
    jig_core_world::fixtures::finish_chat(&mut world, 1, b"finished while settling");
    assert!(world.store.applied > commits, "the answer commits before it is acknowledged");
    assert!(world.stop_deliveries.iter().any(|held| matches!(held,
        core::Held::Acknowledge { run, attempt, .. } if (run.raw(), attempt.raw()) == first
    )));
    assert_eq!(world.assignments, [first], "the freed slot places no run");
    assert_eq!(
        world.domain.core().tasks.task(waiting).expect("waiting task").phase,
        tasks::Phase::Active(tasks::Active::Due)
    );
    world.stop();
    world.stop();
    assert_eq!(world.stop_deliveries.iter().filter(|held| matches!(held, core::Held::Drained)).count(), 1);
}

#[test]
fn settling_leaves_a_queued_job_unplaced_when_the_host_frees_its_slot() {
    use jig_core_world::fixtures::{job, job_world, live_row};
    use jig_domain_link::{Hello, Hosting, JobBound, JobKind, Phase};
    let (mut world, previous) = job_world(172, true);
    let (step, connector, code, _) = world.procedure_inputs.last().expect("fixture procedure");
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: previous,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep::new(core::StepEnd::Result(tasks::TaskResult::Report { words: Box::new([]) })),
    }));
    // The queued job belongs to a separate root, so finishing the hosted chat
    // does not await that job as one of its own delegates.
    let mut member = jig_test_profile::fixtures::people_member(b"job owner", 20);
    member.executor = people::TaskExecutor::Procedure { connector: 1, code: 1 };
    member.llm = None;
    member.authority.delegation =
        people::Delegation { kinds: Box::new([people::Executor::Job(7)]), tasks: 1, depth: 1 };
    request(&mut world, 60, people::Ask::Create { project: 1, batch: Box::new([member]) });
    let procedure = world.procedure_inputs.last().expect("independent procedure").3.task;
    assert_ne!(procedure, previous);
    let first = (world.task(), world.attempt());
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: Token::new(7) })));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(8),
        hello: Hello {
            jobs: Box::new([JobBound { kind: JobKind(7), spec: 64, result: 32, deadline: Duration::from_secs(60) }]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([Hosting {
                run: Token::new(first.0),
                attempt: Token::new(first.1),
                phase: Phase::Active,
            }]),
        },
    })));
    let mut child = job(&world, 7, Box::new([0; 64]), Duration::from_secs(60));
    child.authority = live_row(&world, procedure).authority.clone();
    child.authority.budget.spend = 0;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let (step, connector, code, _) = world.procedure_inputs.last().expect("procedure started");
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task: procedure,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep { batch: Box::new([child]), ..core::ProcedureStep::new(core::StepEnd::Wait) },
    }));
    assert_eq!(live_row(&world, procedure).delegates.len(), 1, "the procedure admitted its job");
    let task = live_row(&world, procedure).delegates[0];
    let phase = live_row(&world, task).phase.clone();
    assert!(
        matches!(phase, tasks::Phase::Active(tasks::Active::Claimed { .. })),
        "a claimed job is waiting in the fleet"
    );
    assert!(world.jobs.is_empty(), "the job is queued behind the hosted run");
    world.send(root::Event::Core(core::Event::Settling));
    world.send(root::Event::Answer {
        channel: Token::new(8),
        task: first.0,
        attempt: first.1,
        cumulative: 0,
        read: None,
        refused: Box::new([]),
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"free the slot".as_slice().into() },
            cancel_delegates: false,
        },
    });
    assert!(
        world.stop_deliveries.iter().any(|held| matches!(held,
            core::Held::Acknowledge { run, attempt, .. } if (run.raw(), attempt.raw()) == first
        )),
        "the hosted run's answer is acknowledged and its slot frees"
    );
    assert!(world.jobs.is_empty(), "settling assigns no queued job");
    assert_eq!(live_row(&world, task).phase, phase, "no Placed follows the answer");
    world.stop();
    assert_eq!(world.stop_deliveries.iter().filter(|held| matches!(held, core::Held::Drained)).count(), 1);
}

#[test]
fn settling_does_not_activate_a_backoff_that_passes() {
    let mut world = World::new(Setup { seed: 173, ..Setup::default() });
    world.end(tasks::End::Failed(tasks::Class::Transient), 0);
    let task = world.task();
    let phase = world.domain.core().tasks.task(task).expect("backing off").phase.clone();
    assert!(matches!(phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
    let deadline = world.domain.core().tasks.next_deadline().expect("backoff timer");
    world.send(root::Event::Core(core::Event::Settling));
    world.advance(Duration::from_secs(60));
    world.send(root::Event::Timer(core::Timer::Tasks));
    assert_eq!(world.assignments.len(), 1);
    assert_eq!(world.domain.core().tasks.task(task).expect("still due for retry").phase, phase);
    assert_eq!(world.domain.core().tasks.next_deadline(), Some(deadline));
    let env = world.env();
    assert_ne!(world.domain.core().due(env.now, env.wall), Some(core::Timer::Tasks), "the held timer is not rearmed");
}

#[test]
fn settling_holds_a_procedure_wake_until_the_next_start() {
    use jig_core_world::fixtures::{delegate, member};
    let seed = 174;
    let (configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    let mut world = World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() });
    world.hold_procedures = true;
    let run = world.assignments[0];
    let child = member(&world, tasks::Executor::Procedure { connector: 1, code: 1 }, b"waiting procedure");
    let task = delegate(&mut world, run, 1, child);
    let (step, connector, code, _) = world.procedure_inputs.last().expect("procedure started");
    world.send(root::Event::Core(core::Event::ProcedureStep {
        task,
        step: *step,
        connector: *connector,
        code: u32::from(*code),
        decision: core::ProcedureStep::new(core::StepEnd::Wait),
    }));
    let steps = world.procedure_inputs.len();
    world.send(root::Event::Core(core::Event::Settling));
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task })));
    assert_eq!(world.procedure_inputs.len(), steps);
    assert_eq!(
        world.domain.core().tasks.task(task).expect("due procedure").phase,
        tasks::Phase::Active(tasks::Active::Due)
    );
    assert!(world.domain.core_mut().take_due().is_none(), "a root cannot take held work");
    world.restart(seed, false);
    assert!(world.procedure_inputs.len() > steps, "the next start steps it afresh");
}

#[test]
fn a_placement_decided_before_settling_is_still_delivered() {
    let mut limits = jig_test_profile::limits();
    limits.core.fleet.slots = 2;
    let mut world = World::new(Setup { seed: 175, limits, ..Setup::default() });
    let assigned = world.assignments.len();
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    let chat = jig_core_world::fixtures::chat(&world, 1, b"placed before settling".as_slice().into());
    request(&mut world, 59, chat);
    let mut placed = false;
    for _ in 0..16 {
        if world.domain.core().tasks.view_tasks().iter().any(|task| {
            task.number != world.task()
                && matches!(
                    world.domain.core().tasks.task(task.number).expect("view task").phase,
                    tasks::Phase::Active(tasks::Active::Running { .. })
                )
        }) {
            placed = true;
            break;
        }
        world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
        let number = world.store.release_held().expect("a preparation or claim decision is held");
        world.send(root::Event::Committed { number, fill: 0 });
    }
    assert!(placed, "the new run's placement decision is held");
    assert_eq!(world.assignments.len(), assigned, "the decision is not yet released");
    world.offer(root::Event::Core(core::Event::Settling));
    let number = world.store.release_held().expect("the placement decision");
    world.send(root::Event::Committed { number, fill: 0 });
    assert_eq!(world.assignments.len(), assigned + 1, "a durable placement is delivered even while settling");
}
