use jig_domain_host as host;
use jig_domain_link::{Form, JobBound, JobKind};
use jig_engine_workers_world::{World, fixtures};
use jig_protocol_engine_workers::{Event, GrantValue, Request};
use jig_protocol_worker_wire as wire;
use skein_lib::{Duration, List, Reader, Secret, Token, Writer};

fn codec() -> wire::v13::Limits {
    wire::sizes::limits(&fixtures::SIZES).expect("codec")
}
pub(super) fn assignment(channel: Token) -> Event {
    Event::Down {
        channel,
        down: host::LinkDown::AssignJob {
            job: host::JobAssignment {
                run: Token::new(8),
                attempt: Token::new(3),
                kind: JobKind(42),
                spec: Box::from([0, 255, 127]),
                deadline: Duration::from_secs(4),
                workspace: Some(host::Workspace { workstream: 9, items: Token::new(500) }),
                grants: Box::new([fixtures::grant(Form::Application)]),
            },
        },
        parts: fixtures::parts(),
        values: Box::new([Some(GrantValue::Application { secret: Secret::new(fixtures::TOKEN_TWO.into()) })]),
    }
}
fn encoded(record: &wire::JobAnswer) -> skein_channel::Frame {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("small"));
    record.encode(&mut writer).expect("answer");
    fixtures::frame(wire::Kind::JobAnswer.number(), &writer.finish())
}
pub(super) fn answer_frame() -> skein_channel::Frame {
    encoded(&answer(wire::JobEnding::Finished(
        wire::JobFinished::new(&codec(), wire::JobFinishedParts { result: Box::from([0, 255, 127]), work: work() })
            .expect("finished"),
    )))
}
fn answer(ending: wire::JobEnding) -> wire::JobAnswer {
    wire::JobAnswer::new(&codec(), wire::JobAnswerParts { task: 8, attempt: 3, ending }).expect("answer")
}
fn work() -> wire::Work {
    wire::Work::new(&codec(), wire::WorkParts { left: Some(Box::from(&b"left"[..])), saved: None }).expect("work")
}

/// Jobs carry their workspace and application grants (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_is_encoded_with_its_workspace_and_grants() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, channel) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    world.down(assignment(channel));
    world.run();
    let records = world.observed(connection);
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].kind, wire::Kind::Job.number());
    let job = wire::Job::decode(&codec(), &mut Reader::new(&records[1].body)).expect("job");
    assert_eq!((job.task(), job.attempt(), job.kind(), job.deadline()), (8, 3, 42, Duration::from_secs(4)));
    assert_eq!(job.spec(), [0, 255, 127]);
    let workspace = job.workspace().as_ref().expect("workspace");
    assert_eq!((workspace.workstream(), workspace.items()), (9, &b"items"[..]));
    let grant = job.grants().get(0).expect("grant");
    assert_eq!((grant.number(), grant.generation(), grant.valid()), (1, 4, Duration::from_secs(30)));
    let wire::GrantValue::Application(value) = grant.value() else { panic!("application") };
    assert_eq!(value.secret().expose(), fixtures::TOKEN_TWO);
}

/// All three job endings reach the fleet, with work as parts (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_answer_reaches_the_fleet_as_link_up() {
    let stale = wire::JobFailure::Undelivered(
        wire::DeliveryOutcomeValue::new(
            &codec(),
            wire::DeliveryOutcomeValueParts { value: wire::DeliveryOutcome::Stale },
        )
        .expect("stale"),
    );
    let invalid = wire::JobRefusal::Invalid(
        wire::JobInvalidValue::new(&codec(), wire::JobInvalidValueParts { value: wire::JobInvalid::Kind })
            .expect("invalid"),
    );
    let records = [
        answer(wire::JobEnding::Finished(
            wire::JobFinished::new(&codec(), wire::JobFinishedParts { result: Box::from([0, 255, 127]), work: work() })
                .expect("finished"),
        )),
        answer(wire::JobEnding::Failed(
            wire::JobFailed::new(
                &codec(),
                wire::JobFailedParts { failure: stale, detail: Box::from(&b"stale"[..]), work: work() },
            )
            .expect("failed"),
        )),
        answer(wire::JobEnding::Refused(
            wire::JobRefusalValue::new(&codec(), wire::JobRefusalValueParts { value: invalid }).expect("refused"),
        )),
    ];
    for (index, record) in records.into_iter().enumerate() {
        let mut peer = fixtures::reporting_worker(fixtures::WORKER_A, fixtures::SECRET_A);
        peer.play(encoded(&record));
        let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
        world.connect(peer);
        world.run();
        assert_eq!(world.root().len(), 2);
        let Request::Up { up: host::LinkUp::JobAnswer { run, attempt, answer }, parts, .. } = &world.root()[1] else {
            panic!("job answer")
        };
        assert_eq!((*run, *attempt), (Token::new(8), Token::new(3)));
        let work = host::Work { left: parts.first().map(|p| p.token), saved: None };
        let expected = match index {
            0 => host::JobEnding::Finished { result: Box::from([0, 255, 127]), work },
            1 => host::JobEnding::Failed {
                failure: host::JobFailure::Undelivered(host::DeliveryOutcome::Stale),
                detail: Box::from(&b"stale"[..]),
                work,
            },
            2 => host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Kind)),
            _ => unreachable!(),
        };
        assert_eq!(*answer, expected);
        if index < 2 {
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].bytes.as_ref(), b"left");
        } else {
            assert!(parts.is_empty());
        }
    }
}

/// The hello's declared kinds keep every bound (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_hello_carries_its_job_kinds() {
    let mut fields = fixtures::hello_record().into_parts();
    fields.jobs = List::with_capacity(1);
    fields
        .jobs
        .push(
            wire::JobBound::new(
                &codec(),
                wire::JobBoundParts { kind: 42, spec: 12, result: 13, deadline: Duration::from_nanos(123) },
            )
            .expect("kind"),
        )
        .expect("room");
    let record = wire::Hello::new(&codec(), fields).expect("hello");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("small"));
    record.encode(&mut writer).expect("hello");
    let mut peer = fixtures::worker(fixtures::WORKER_A, fixtures::SECRET_A, fixtures::APPLICATION_VERSION);
    peer.play(fixtures::frame(wire::Kind::Hello.number(), &writer.finish()));
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    world.connect(peer);
    world.run();
    let Request::Hello { hello, .. } = &world.root()[0] else { panic!("hello") };
    assert_eq!(
        hello.jobs.as_ref(),
        [JobBound { kind: JobKind(42), spec: 12, result: 13, deadline: Duration::from_nanos(123) }]
    );
}

/// Bounds and missing parts refuse jobs as assignments do (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_past_its_bounds_or_missing_its_parts_is_dropped() {
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let (connection, channel) = world.reporting(fixtures::WORKER_A, fixtures::SECRET_A);
    for oversized in [false, true] {
        let Event::Down { channel, down: host::LinkDown::AssignJob { mut job }, mut parts, values } =
            assignment(channel)
        else {
            panic!("job")
        };
        if oversized {
            job.spec = vec![0; 17].into();
        } else {
            parts = Box::new([]);
        }
        world.down(Event::Down { channel, down: host::LinkDown::AssignJob { job }, parts, values });
        world.run();
    }
    assert_eq!(world.dropped_down(), 2);
    assert_eq!((world.observed(connection)).len(), 1);
}

/// An oversized answer never reaches the fleet (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_answer_past_its_bound_is_refused_as_framing() {
    let mut larger = codec();
    larger.job_finished_result += 1;
    let record = wire::JobAnswer::new(
        &larger,
        wire::JobAnswerParts {
            task: 8,
            attempt: 3,
            ending: wire::JobEnding::Finished(
                wire::JobFinished::new(&larger, wire::JobFinishedParts { result: vec![0; 17].into(), work: work() })
                    .expect("larger result"),
            ),
        },
    )
    .expect("answer");
    let mut peer = fixtures::reporting_worker(fixtures::WORKER_A, fixtures::SECRET_A);
    peer.play(encoded(&record));
    let mut world = World::new(&fixtures::limits(), &fixtures::workers(), 1);
    let connection = world.connect(peer);
    world.run();
    assert_eq!(world.refusal(connection), Some((3, Box::default())));
    assert!(!world.root().iter().any(|r| matches!(r, Request::Up { .. })));
}
