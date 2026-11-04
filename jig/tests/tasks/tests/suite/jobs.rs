use jig_domain_tasks::{
    self as tasks, Accepted, Active, Class, Contract, Control, End, Ending, Event, Executor, Funder, Hold, JobSpec,
    Key, MessageKind, Numbers, Party, Phase, ProcedureDecision, ProcedureEnd, Refusal, ResultKind, Stage, Stored,
    TaskResult, Word,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{Duration, ReplyTo, Wall};

const ROOT: u64 = 1;
const PROCEDURE: u64 = 2;
const JOB: u64 = 3;
const DEPENDENCY: u64 = 4;

fn step(world: &mut World, step: u64, decision: ProcedureDecision) -> Reply {
    let reply_to = world.to();
    let key = reply_to.into_token().raw();
    let read = world.contexts[&PROCEDURE].inbox.last().map(|word| word.number);
    world.send(Event::Procedure {
        read,
        reply_to: ReplyTo::new(skein_lib::Token::new(key)),
        task: PROCEDURE,
        step,
        decision,
    });
    world.replies[&key].clone()
}

fn job(dependencies: &[u64]) -> tasks::New {
    let mut job = task(JOB, dependencies);
    job.executor = Executor::Job { kind: 7 };
    job.spec.job = Some(Box::new(JobSpec { bytes: Box::new([0xff]), deadline: Duration::from_secs(7) }));
    job.spec.words = Box::new([]);
    job.contract = Contract::Job { bytes: 16 };
    job.numbers = Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    job.authority.budget.spend = 0;
    job.funder = Funder::Task(PROCEDURE);
    job
}

fn procedure() -> World {
    let mut world = World::new(207, LIMITS);
    world.make(Party::Person(9), vec![task(ROOT, &[])]);
    let mut procedure = task(PROCEDURE, &[]);
    procedure.executor = Executor::Procedure { connector: 1, code: 1 };
    procedure.funder = Funder::Task(ROOT);
    world.make(Party::Task(ROOT), vec![procedure]);
    world
}

fn with_job() -> World {
    let mut world = procedure();
    assert_eq!(
        step(
            &mut world,
            1,
            ProcedureDecision { batch: Box::new([job(&[])]), ..ProcedureDecision::new(ProcedureEnd::Wait) }
        ),
        Reply::Made(vec![JOB])
    );
    world
}

fn refused(reply: Reply, why: Refusal) {
    assert!(matches!(reply, Reply::Refused(problem) if problem.why == why), "expected {why:?}");
}

fn ended(world: &World, number: u64) -> &tasks::TaskRecord {
    let Stored::Ended(row) = &world.records[&Key::Ended(number)] else { panic!("ended row") };
    row
}

/// Dependencies gate a job and its bytes wake its procedure (jig/docs/design/domain/tasks.md, 5.3.1).
#[test]
fn a_procedures_job_is_due_once_its_dependency_is_done_and_ends_with_its_result() {
    let mut world = procedure();
    let mut dependency = task(DEPENDENCY, &[]);
    dependency.funder = Funder::Task(PROCEDURE);
    assert_eq!(
        step(
            &mut world,
            1,
            ProcedureDecision {
                batch: Box::new([dependency, job(&[DEPENDENCY])]),
                ..ProcedureDecision::new(ProcedureEnd::Wait)
            }
        ),
        Reply::Made(vec![DEPENDENCY, JOB])
    );
    assert_eq!(world.record(JOB).phase, Phase::Waiting);
    assert!(!world.activations.contains(&JOB));
    let before = world.records.clone();
    world.restart();
    assert_eq!(world.records, before);
    world.claim(DEPENDENCY, 1);
    world.finish(DEPENDENCY);
    world.settle(DEPENDENCY);
    assert_eq!(world.record(JOB).phase, Phase::Active(Active::Due));
    assert!(world.activations.contains(&JOB));
    assert_eq!(world.contexts[&JOB].executor, Executor::Job { kind: 7 });
    // Consume the agent's result so the job's result supplies the next wake.
    assert_eq!(step(&mut world, 2, ProcedureDecision::new(ProcedureEnd::Wait)), Reply::Done);
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Idle));
    world.claim_budget(JOB, 2, 0);
    assert_eq!(world.record(JOB).numbers.reserved, 0);
    assert_eq!(world.record(JOB).run_reserved, 0);
    assert_eq!(world.record(JOB).phase, Phase::Active(Active::Running { attempt: 2 }));
    assert_eq!(
        world.terminal(
            JOB,
            End::Finished { result: TaskResult::Job { bytes: b"result".as_slice().into() }, cancel_delegates: false }
        ),
        Reply::Acknowledged(Accepted::New)
    );
    world.settle(JOB);
    let result = TaskResult::Job { bytes: b"result".as_slice().into() };
    assert_eq!(world.results[&JOB], Ending::Done(result.clone()));
    assert_eq!(ended(&world, JOB).phase, Phase::Ended(Ending::Done(result)));
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Due));
    let result_word = world
        .record(PROCEDURE)
        .inbox
        .iter()
        .find(|word| word.kind == MessageKind::Result(ResultKind::Job))
        .expect("job result word");
    assert_eq!(result_word.from, Party::Task(JOB));
    assert_eq!(result_word.words.as_ref(), b"result");
    let before = world.records.clone();
    world.restart();
    assert_eq!(world.records, before);
}

/// A result beyond its contract spends an invalid try (jig/docs/design/domain/tasks.md, 5.3.1 and 5.5).
#[test]
fn a_job_result_past_its_contract_is_an_invalid_attempt() {
    let mut world = with_job();
    world.claim_budget(JOB, 1, 0);
    assert_eq!(
        world.terminal(
            JOB,
            End::Finished { result: TaskResult::Job { bytes: Box::new([0xff; 17]) }, cancel_delegates: false }
        ),
        Reply::Acknowledged(Accepted::New)
    );
    assert_eq!(world.record(JOB).tries.of(Class::Invalid), 1);
    assert_eq!(world.record(JOB).invalid_result, Some(tasks::InvalidResult::Words));
    assert!(matches!(world.record(JOB).phase, Phase::Active(Active::BackingOff { .. })));
    assert!(!world.results.contains_key(&JOB));
    world.advance();
    assert_eq!(world.record(JOB).phase, Phase::Active(Active::Due));
}

/// Past its run tries a job is held for its procedure (jig/docs/design/domain/tasks.md, 5.3.1 and 5.5).
#[test]
fn a_failing_job_is_retried_within_its_tries_then_held_and_its_procedure_hears_it() {
    let mut world = with_job();
    for attempt in 1..=u64::from(LIMITS.retries.run.retries) + 1 {
        world.claim_budget(JOB, attempt, 0);
        assert_eq!(world.terminal(JOB, End::Failed(Class::Run)), Reply::Acknowledged(Accepted::New));
        assert_eq!(u64::from(world.record(JOB).tries.of(Class::Run)), attempt);
        if attempt <= u64::from(LIMITS.retries.run.retries) {
            assert!(matches!(world.record(JOB).phase, Phase::Active(Active::BackingOff { .. })));
            world.advance();
            assert_eq!(world.record(JOB).phase, Phase::Active(Active::Due));
        }
    }
    assert!(matches!(world.record(JOB).phase, Phase::Held { why: Hold::Failures(Class::Run), .. }));
    let revision = world.record(JOB).escalation.revision();
    // The parent supplies policy routing; this story selects the procedure holder.
    world.send(Event::RoutedEscalation {
        task: JOB,
        revision,
        holder: tasks::EscalationHolder::Task(PROCEDURE),
        entry: 20,
    });
    assert_eq!(world.record(PROCEDURE).phase, Phase::Active(Active::Due));
    assert!(
        world.contexts[&PROCEDURE]
            .inbox
            .iter()
            .any(|word| matches!(word.kind, MessageKind::Escalation { task: JOB, .. }))
    );
    let before = world.records.clone();
    world.restart();
    assert_eq!(world.records, before);
    let reply_to = world.to();
    let key = reply_to.into_token().raw();
    world.send(Event::Control {
        reply_to: ReplyTo::new(skein_lib::Token::new(key)),
        by: Party::Task(PROCEDURE),
        task: JOB,
        control: Control::Release { again: None, message: None },
    });
    assert_eq!(world.replies[&key], Reply::Done);
    assert_eq!(world.record(JOB).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(JOB).tries, tasks::Tries::NONE);
    assert!(world.activations.contains(&JOB));
}

fn amendment() -> tasks::Amendment {
    tasks::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: Box::new([]),
    }
}

/// Messages and amendments never change a job (jig/docs/design/domain/tasks.md, 5.3.1).
#[test]
fn words_questions_and_amendments_to_a_job_are_refused_as_its_executor() {
    let mut world = with_job();
    world.send(Event::WakeProcedure { task: PROCEDURE });
    let before = world.record(JOB).clone();
    for kind in [MessageKind::Words, MessageKind::Question, MessageKind::Answer { question: 1 }] {
        let decision = ProcedureDecision {
            messages: Box::new([tasks::ProcedureMessage {
                task: JOB,
                word: Word {
                    number: 10,
                    from: Party::Task(PROCEDURE),
                    kind,
                    words: b"hello".as_slice().into(),
                    at: Wall::EPOCH,
                    hits: 1,
                    eligible: true,
                },
            }]),
            ..ProcedureDecision::new(ProcedureEnd::Wait)
        };
        refused(step(&mut world, 2, decision), Refusal::Executor);
        assert_eq!(world.record(JOB), &before);
    }
    for change in [
        tasks::Amendment { spec: Some(job(&[]).spec), ..amendment() },
        tasks::Amendment { dependencies: Some(Box::new([ROOT])), ..amendment() },
    ] {
        refused(
            step(
                &mut world,
                2,
                ProcedureDecision {
                    amendments: Box::new([tasks::ProcedureAmendment {
                        task: JOB,
                        message: 10,
                        stop_run: false,
                        amendment: change,
                    }]),
                    ..ProcedureDecision::new(ProcedureEnd::Wait)
                },
            ),
            Refusal::Executor,
        );
        assert_eq!(world.record(JOB), &before);
    }
    let reply_to = world.to();
    let key = reply_to.into_token().raw();
    world.send(Event::Message {
        reply_to: ReplyTo::new(skein_lib::Token::new(key)),
        project: 1,
        task: JOB,
        word: Word {
            number: 10,
            from: Party::Person(99),
            kind: MessageKind::Words,
            words: b"hello".as_slice().into(),
            at: Wall::EPOCH,
            hits: 1,
            eligible: true,
        },
    });
    refused(world.replies[&key].clone(), Refusal::State);
    assert_eq!(world.record(JOB), &before);
}

/// Cancellation skips a job's grace and drains its live run (jig/docs/design/domain/tasks.md, 5.3.1).
#[test]
fn a_cancel_ends_a_running_job_at_once_even_with_a_grace() {
    let mut world = with_job();
    world.claim_budget(JOB, 1, 0);
    world.send(Event::WakeProcedure { task: PROCEDURE });
    assert_eq!(
        step(
            &mut world,
            2,
            ProcedureDecision {
                cancels: Box::new([tasks::ProcedureCancel {
                    task: JOB,
                    reason: b"stop".as_slice().into(),
                    grace: Some(Duration::from_secs(5)),
                    message: Some(10)
                }]),
                ..ProcedureDecision::new(ProcedureEnd::Wait)
            }
        ),
        Reply::Done
    );
    assert!(matches!(world.record(JOB).phase, Phase::Closing(tasks::Closing { stage: Stage::Run { attempt: 1 }, .. })));
    assert!(world.stops.contains(&(JOB, 1)));
    assert!(world.record(JOB).grace.is_none());
    assert!(!world.record(JOB).inbox.iter().any(Word::finish_requested));
    world.terminal(JOB, End::Failed(Class::Run));
    world.settle(JOB);
    assert!(matches!(world.results[&JOB], Ending::Cancelled { .. }));
    assert_eq!(ended(&world, JOB).tries, tasks::Tries::NONE);

    let mut world = procedure();
    let mut dependency = task(DEPENDENCY, &[]);
    dependency.funder = Funder::Task(PROCEDURE);
    assert_eq!(
        step(
            &mut world,
            1,
            ProcedureDecision {
                batch: Box::new([dependency, job(&[DEPENDENCY])]),
                ..ProcedureDecision::new(ProcedureEnd::Wait)
            }
        ),
        Reply::Made(vec![DEPENDENCY, JOB])
    );
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Person(9),
        task: ROOT,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    assert!(matches!(world.record(JOB).phase, Phase::Closing(_)));
    assert!(world.record(JOB).grace.is_none());
    assert!(!world.record(JOB).inbox.iter().any(Word::finish_requested));
    world.settle(JOB);
    assert!(matches!(world.results[&JOB], Ending::Cancelled { .. }));
}

/// Claims require zero budget and reserve nothing (jig/docs/design/domain/tasks.md, 5.3.1).
#[test]
fn a_job_claim_reserves_nothing() {
    let mut world = with_job();
    refused(world.claim_budget_writing(JOB, 1, 1, Box::new([])), Refusal::Funding);
    assert_eq!(world.record(JOB).phase, Phase::Active(Active::Preparing));
    assert_eq!(world.record(JOB).numbers.reserved, 0);
    world.send(Event::PreparationWaiting { task: JOB });
    world.claim_budget(JOB, 1, 0);
    assert_eq!(world.record(JOB).phase, Phase::Active(Active::Running { attempt: 1 }));
    assert_eq!(world.record(JOB).numbers.reserved, 0);
    assert_eq!(world.record(JOB).run_reserved, 0);
}
