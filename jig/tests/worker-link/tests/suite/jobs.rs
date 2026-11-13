use jig_domain_host as host;
use jig_domain_link::{Form, JobBound, JobKind};
use jig_protocol_worker_link::{Event, Request};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, List, Reader, Token};

fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}

pub(super) fn answer() -> host::LinkUp {
    host::LinkUp::JobAnswer {
        run: Token::new(8),
        attempt: Token::new(3),
        answer: host::JobEnding::Finished {
            result: Box::from([0, 255, 127]),
            work: host::Work { left: Some(Token::new(10)), saved: None },
        },
    }
}
fn job_record(limits: &wire::v13::Limits) -> wire::Job {
    let mut grants = List::with_capacity(1);
    grants.push(fixtures::grant_record(fixtures::application_value())).expect("grant");
    wire::Job::new(
        limits,
        wire::JobParts {
            task: 8,
            attempt: 3,
            kind: 42,
            spec: Box::from([0, 255, 127]),
            deadline: Duration::from_secs(4),
            workspace: Some(
                wire::Workspace::new(limits, wire::WorkspaceParts { workstream: 3, items: Box::from(&b"items"[..]) })
                    .expect("workspace"),
            ),
            grants,
        },
    )
    .expect("job")
}

/// Job kinds cross whole (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_hello_carries_its_job_kinds() {
    for count in [0, 1, 2] {
        let mut limits = fixtures::limits();
        limits.sizes.models = 2;
        let mut world = World::new(&limits, 1);
        world.engine(fixtures::engine_accepting_with(&limits));
        world.run();
        let mut hello = fixtures::hello();
        hello.jobs = (0..count)
            .map(|kind| JobBound {
                kind: JobKind(kind),
                spec: kind + 3,
                result: kind + 4,
                deadline: Duration::from_nanos(u64::from(kind) + 7),
            })
            .collect();
        let expected = hello.jobs.clone();
        world.down(Event::Hello { hello });
        let records = world.records();
        let codec = wire::sizes::limits(&limits.sizes).expect("codec");
        let hello = wire::Hello::decode(&codec, &mut Reader::new(&records[0][0].body)).expect("hello");
        assert_eq!(hello.jobs().len(), count);
        for (job, expected) in hello.jobs().iter().zip(expected) {
            assert_eq!(
                (job.kind(), job.spec(), job.result(), job.deadline()),
                (expected.kind.0, expected.spec, expected.result, expected.deadline)
            );
        }
    }
}

/// Workspace bytes become parts, and grants precede the job (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_record_reaches_the_hub_as_an_assign_job() {
    let mut world = world();
    let record = job_record(&fixtures::codec());
    world.play(fixtures::frame(wire::Kind::Job.number(), &fixtures::encoded(record.measure(), |w| record.encode(w))));
    assert_eq!(world.order(), ["value", "down"]);
    let Request::Down { down: host::LinkDown::AssignJob { job }, parts } = &world.root()[1] else { panic!("job") };
    assert_eq!(
        (job.run, job.attempt, job.kind, job.deadline),
        (Token::new(8), Token::new(3), JobKind(42), Duration::from_secs(4))
    );
    assert_eq!(job.spec.as_ref(), [0, 255, 127]);
    let workspace = job.workspace.as_ref().expect("workspace");
    assert_eq!(workspace.workstream, 3);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].token, workspace.items);
    assert_eq!(parts[0].bytes.as_ref(), b"items");
    assert_eq!(job.grants.as_ref(), [fixtures::grant(Form::Application)]);
    let Request::Value { value: jig_protocol_worker_link::GrantValue::Application { secret }, .. } = &world.values()[0]
    else {
        panic!("application value")
    };
    assert_eq!(secret.expose(), fixtures::TOKEN_ONE);
}

/// The hub supplies its kept answer again after hello (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_answer_goes_up_and_again_after_a_redial() {
    let mut world = world();
    for channel in 0..2 {
        world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
        world.down(Event::Up { up: answer(), parts: fixtures::work_parts() });
        world.down(Event::Up { up: fixtures::answer(), parts: fixtures::work_parts() });
        let records = world.records();
        assert_eq!(records[channel].iter().map(|r| r.kind).collect::<Vec<_>>(), [0x0100, 0x0101, 0x010c, 0x0103]);
        let record =
            wire::JobAnswer::decode(&fixtures::codec(), &mut Reader::new(&records[channel][2].body)).expect("answer");
        assert_eq!((record.task(), record.attempt()), (8, 3));
        let wire::JobEnding::Finished(end) = record.ending() else { panic!("finished") };
        assert_eq!(end.result(), [0, 255, 127]);
        assert_eq!(end.work().left().as_deref(), Some(&b"left"[..]));
        assert!(end.work().saved().is_none());
        if channel == 0 {
            world.lose_channel();
            world.engine(fixtures::engine_accepting());
            world.advance(Duration::from_millis(200));
            world.down(Event::Hello { hello: fixtures::hello() });
        }
    }
    let record =
        wire::Acknowledge::new(&fixtures::codec(), wire::AcknowledgeParts { task: 8, attempt: 3 }).expect("ack");
    world.play(fixtures::frame(
        wire::Kind::Acknowledge.number(),
        &fixtures::encoded(record.measure(), |w| record.encode(w)),
    ));
    assert!(
        matches!(world.root().last(), Some(Request::Down { down: host::LinkDown::Acknowledge { run, attempt }, .. }) if *run == Token::new(8) && *attempt == Token::new(3))
    );
}

/// An oversized spec is refused before any parts or values leave (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_past_its_wire_bound_is_refused_as_framing() {
    let mut world = world();
    let mut larger = fixtures::codec();
    larger.job_spec += 1;
    let mut fields = job_record(&larger).into_parts();
    fields.spec = vec![0; usize::try_from(larger.job_spec).expect("small")].into();
    let record = wire::Job::new(&larger, fields).expect("larger job");
    world.play(fixtures::frame(wire::Kind::Job.number(), &fixtures::encoded(record.measure(), |w| record.encode(w))));
    assert_eq!(world.refusals(), [(3, Box::default())]);
    assert!(world.values().is_empty());
    assert!(!world.root().iter().any(|r| matches!(r, Request::Down { .. })));
}

/// Missing work and excessive result bytes are dropped as run answers are (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_answer_without_its_parts_or_past_its_bound_is_dropped() {
    let mut world = world();
    world.down(Event::Up { up: answer(), parts: Box::new([]) });
    world.down(Event::Up {
        up: host::LinkUp::JobAnswer {
            run: Token::new(8),
            attempt: Token::new(3),
            answer: host::JobEnding::Finished {
                result: vec![0; 17].into(),
                work: host::Work { left: None, saved: None },
            },
        },
        parts: Box::new([]),
    });
    assert_eq!(world.dropped_records(), 2);
    assert_eq!(world.records()[0].len(), 1);
}

/// Every job ending crosses both real translations (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn every_job_ending_crosses_without_changing_its_fields() {
    let work = host::Work { left: None, saved: None };
    let mut endings = vec![
        host::JobEnding::Refused(host::JobRefusal::Busy),
        host::JobEnding::Finished { result: Box::from([0, 255, 127]), work: work.clone() },
    ];
    for invalid in [
        host::JobInvalid::Kind,
        host::JobInvalid::Spec,
        host::JobInvalid::Deadline,
        host::JobInvalid::Grants,
        host::JobInvalid::Start { why: Box::from(&b"start refused"[..]) },
    ] {
        endings.push(host::JobEnding::Refused(host::JobRefusal::Invalid(invalid)));
    }
    let mut failures = vec![
        host::JobFailure::Unprepared(host::Preparation::Transient),
        host::JobFailure::Unprepared(host::Preparation::Permanent { resource: None }),
        host::JobFailure::Failed,
        host::JobFailure::Deadline,
        host::JobFailure::Result,
        host::JobFailure::Unstarted,
    ];
    for outcome in [
        host::DeliveryOutcome::Delivered,
        host::DeliveryOutcome::Nothing,
        host::DeliveryOutcome::Stale,
        host::DeliveryOutcome::Refused,
        host::DeliveryOutcome::Failed,
    ] {
        failures.push(host::JobFailure::Undelivered(outcome));
    }
    for reason in [host::Reason::Engine, host::Reason::Contact, host::Reason::Shutdown] {
        failures.push(host::JobFailure::Cancelled(reason));
    }
    for failure in failures {
        endings.push(host::JobEnding::Failed { failure, detail: Box::from([255, 0]), work: work.clone() });
    }
    let mut world = crate::joined_world::Joined::new(&fixtures::joined_limits(), 1);
    world.run();
    for (index, ending) in endings.into_iter().enumerate() {
        let expected = host::LinkUp::JobAnswer { run: Token::new(8), attempt: Token::new(3), answer: ending.clone() };
        world.worker_up(
            host::LinkUp::JobAnswer { run: Token::new(8), attempt: Token::new(3), answer: ending },
            Box::new([]),
        );
        world.run();
        let jig_protocol_engine_workers::Request::Up { up, parts, .. } = &world.engine_root[index + 1] else {
            panic!("answer")
        };
        assert_eq!(*up, expected);
        assert!(parts.is_empty());
    }
    world.finish();
}

/// Preparation resources precede work parts as in a run's answer (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_failure_hands_up_its_resource_and_work_parts_in_order() {
    let mut world = crate::joined_world::Joined::new(&fixtures::joined_limits(), 1);
    world.run();
    world.worker_up(
        host::LinkUp::JobAnswer {
            run: Token::new(8),
            attempt: Token::new(3),
            answer: host::JobEnding::Failed {
                failure: host::JobFailure::Unprepared(host::Preparation::Permanent { resource: Some(Token::new(12)) }),
                detail: Box::from(&b"permanent"[..]),
                work: host::Work { left: Some(Token::new(10)), saved: Some(Token::new(11)) },
            },
        },
        vec![
            wire::Part { token: Token::new(12), bytes: Box::from(&b"resource"[..]) },
            wire::Part { token: Token::new(10), bytes: Box::from(&b"left"[..]) },
            wire::Part { token: Token::new(11), bytes: Box::from(&b"saved"[..]) },
        ]
        .into(),
    );
    world.run();
    let jig_protocol_engine_workers::Request::Up {
        up:
            host::LinkUp::JobAnswer {
                answer:
                    host::JobEnding::Failed {
                        failure: host::JobFailure::Unprepared(host::Preparation::Permanent { resource }),
                        work,
                        detail,
                    },
                ..
            },
        parts,
        ..
    } = &world.engine_root[1]
    else {
        panic!("failed answer")
    };
    assert_eq!(parts.iter().map(|p| p.bytes.as_ref()).collect::<Vec<_>>(), [b"resource".as_slice(), b"left", b"saved"]);
    assert_eq!((*resource, work.left, work.saved), (Some(parts[0].token), Some(parts[1].token), Some(parts[2].token)));
    assert_eq!(detail.as_ref(), b"permanent");
    world.finish();
}
