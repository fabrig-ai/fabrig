//! Hub-link transitions (jig/docs/design/domain/hosts/hub.md, 1–8).
use super::*;
use jig_domain_link::{Form, Grant, Scheme};
use skein_lib::Duration;
const CHANNEL: Token = Token::new(0);
const RUN: Token = Token::new(2);
const ATTEMPT: Token = Token::new(3);
fn link() -> Link {
    Link::new(Limits { assignments: 2, payloads: 4, turns: 2 }, Rng::new(7))
}
fn turn() -> host::Turn {
    host::Turn {
        turn: 1,
        spent: 9,
        read: Some(Token::new(7)),
        refused: Box::new([]),
        opens_window: true,
        body: Box::from(&b"opaque turn"[..]),
    }
}
fn answer() -> host::LinkUp {
    host::LinkUp::Answer {
        run: RUN,
        attempt: ATTEMPT,
        answer: host::RunAnswer {
            turns: 1,
            spent: 12,
            read: Some(Token::new(7)),
            refused: Box::new([]),
            ending: host::Ending::Ended {
                outcome: Box::from(&b"opaque result"[..]),
                work: host::Work { left: None, saved: None },
            },
        },
    }
}
fn run() -> Box<core::RunCharter> {
    Box::new(core::RunCharter {
        revision: 1,
        ended_by: jig_domain_tasks::EndedBy::Result,
        sources: Box::new([Some(1)]),
        accounts: Box::new([]),
        placement: jig_domain_fleet::PlacementConstraint {
            channel: Token::new(1),
            kind: jig_domain_fleet::HostKind::Worker,
        },
        budget: 37,
        contract: tasks::Contract::Report { words: 128 },
        authority: tasks::Authority {
            tools: tasks::Tools(1023),
            models: tasks::Models::Named(1),
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 100, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        policy: core::RunPolicy {
            instructions: b"Answer the person".as_slice().into(),
            waiting: Duration::from_secs(30),
            resume: true,
            turns: 8,
            time: Duration::from_secs(60),
            reserve_turns: 1,
            reserve_time: Duration::from_secs(10),
            model: core::Model {
                provider: 1,
                dialect: 3,
                endpoint: 1,
                program: None,
                name: b"small".as_slice().into(),
                cache_life_ms: 0,
                window: 1024,
                output: 128,
                input_price: 1,
                cached_price: 1,
                cache_write_price: None,
                output_price: 1,
                price_unit: 1_000_000,
                efforts: Box::new([]),
                effort: None,
            },
            alternatives: Box::new([]),
            inspect: true,
            modify: false,
            shell: false,
            agents: false,
            call_timeout: Duration::from_secs(10),
            tools: core::RunPolicy::EVERY_FAMILY,
        },
    })
}
fn assignment() -> Assignment {
    Assignment {
        live: core::LiveAsk::NONE,
        brief: Box::new([]),
        conversation: 1,
        task: RUN.raw(),
        attempt: ATTEMPT.raw(),
        charter: 1,
        run: run(),
        messages: Box::new([]),
        transcript: Box::new([b"prior turn".as_slice().into()]),
        answered: Box::new([]),
        grants: Box::new([Grant {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 2,
            valid: Duration::from_secs(30),
        }]),
    }
}

#[test]
fn an_assignment_is_relayed_and_answered() {
    let mut link = link();
    let mut out = Queue::uncapped();
    link.keep_assignment(assignment());
    step(&mut link, Time::ZERO, Event::Assign { channel: CHANNEL, run: RUN, attempt: ATTEMPT }, &mut out);
    let Request::Assigned { channel, assignment } = out.pop().expect("assignment") else { panic!("typed assignment") };
    assert_eq!(channel, CHANNEL);
    assert_eq!((assignment.task, assignment.attempt, assignment.conversation), (RUN.raw(), ATTEMPT.raw(), 1));
    assert!(link.assignment(RUN.raw()).is_none());
    step(&mut link, Time::ZERO, Event::Up { channel: CHANNEL, value: answer() }, &mut out);
    let Request::Core(core::Event::Fleet(fleet::Event::Answer { channel, run, attempt, answer, payload })) =
        out.pop().expect("answer")
    else {
        panic!("fleet answer")
    };
    assert_eq!((channel, run, attempt), (CHANNEL, RUN, ATTEMPT));
    assert_eq!(answer, fleet::Answer::Ended);
    assert!(link.payload(payload).is_some());
    assert!(out.is_empty());
}
#[test]
fn a_busy_turn_is_resent_at_its_deadline() {
    let mut link = link();
    let mut out = Queue::uncapped();
    let now = Time::from_nanos(100);
    step(&mut link, now, Event::TurnBusy { run: RUN, attempt: ATTEMPT, turn: 1, retained: true }, &mut out);
    let due = next_deadline(&link).expect("retry deadline");
    assert!(due >= now.saturating_add(Duration::from_nanos(50_000_000)));
    assert!(due <= now.saturating_add(Duration::from_nanos(100_000_000)));
    fire(&mut link, Time::from_nanos(due.as_nanos().checked_sub(1).expect("positive deadline")), &mut out);
    assert!(out.is_empty());
    fire(&mut link, due, &mut out);
    let Request::TurnCopy { run, attempt, turn: number } = out.pop().expect("copy retained turn") else {
        panic!("hub hand-out")
    };
    assert_eq!((run, attempt, number), (RUN, ATTEMPT, 1));
    step(
        &mut link,
        due,
        Event::Up { channel: CHANNEL, value: host::LinkUp::Turn { run, attempt, turn: turn() } },
        &mut out,
    );
    let Request::Core(core::Event::Fleet(fleet::Event::Turn { turn, body, .. })) = out.pop().expect("resent turn")
    else {
        panic!("fleet turn")
    };
    assert_eq!(turn, 1);
    let Some(Payload::Turn { transcript, cumulative, .. }) = link.payload(body) else {
        panic!("retained turn payload")
    };
    assert_eq!(transcript.as_ref(), b"opaque turn");
    assert_eq!(*cumulative, 9);
    assert_eq!(next_deadline(&link), None);
    assert!(out.is_empty());
}
#[test]
fn an_acknowledgement_cancels_the_busy_turn_retry() {
    let mut link = link();
    let mut out = Queue::uncapped();
    step(&mut link, Time::ZERO, Event::TurnBusy { run: RUN, attempt: ATTEMPT, turn: 1, retained: true }, &mut out);
    let due = next_deadline(&link).expect("retry deadline");
    step(&mut link, Time::ZERO, Event::TurnKept { channel: CHANNEL, run: RUN, attempt: ATTEMPT, turn: 1 }, &mut out);
    let Request::Down { channel, value: host::LinkDown::TurnKept { run, attempt, turn } } =
        out.pop().expect("acknowledgement")
    else {
        panic!("hub acknowledgement")
    };
    assert_eq!((channel, run, attempt, turn), (CHANNEL, RUN, ATTEMPT, 1));
    fire(&mut link, due, &mut out);
    assert!(out.is_empty());
    assert_eq!(next_deadline(&link), None);
}
#[test]
fn a_payload_is_handed_out_and_its_decoded_answer_is_routed() {
    let mut link = link();
    let mut out = Queue::uncapped();
    step(&mut link, Time::ZERO, Event::Up { channel: CHANNEL, value: answer() }, &mut out);
    let Request::Core(core::Event::Fleet(fleet::Event::Answer { payload, .. })) = out.pop().expect("answer") else {
        panic!("fleet answer")
    };
    step(&mut link, Time::ZERO, Event::AnswerPayload { run: RUN, attempt: ATTEMPT, payload }, &mut out);
    let Request::ReadResult { payload: handed, outcome } = out.pop().expect("protocol hand-out") else {
        panic!("opaque result")
    };
    assert_eq!(handed, payload);
    assert_eq!(outcome.as_ref(), b"opaque result");
    assert_eq!(link.payload_count(), 1);
    step(&mut link, Time::ZERO, Event::ResultRead { payload: handed, end: tasks::End::Parked }, &mut out);
    let Request::Core(core::Event::AnswerPayload { run, attempt, payload: returned, cumulative, end, read, .. }) =
        out.pop().expect("decoded answer")
    else {
        panic!("core payload")
    };
    assert_eq!((run, attempt, returned), (RUN, ATTEMPT, payload));
    assert_eq!((cumulative, end, read), (12, tasks::End::Parked, Some(7)));
    step(&mut link, Time::ZERO, Event::DropPayload { payload }, &mut out);
    assert_eq!(link.payload_count(), 0);
    step(&mut link, Time::ZERO, Event::ResultRead { payload, end: tasks::End::Parked }, &mut out);
    assert!(out.is_empty(), "a stale hand-out cannot answer another run");
}
#[test]
fn a_relay_answer_keeps_its_opaque_name_and_consumes_its_payload() {
    let mut link = link();
    let mut out = Queue::uncapped();
    let name: Box<[u8]> = Box::from(&b"opaque call"[..]);
    step(
        &mut link,
        Time::ZERO,
        Event::Up {
            channel: CHANNEL,
            value: host::LinkUp::Call {
                run: RUN,
                attempt: ATTEMPT,
                call: name.clone(),
                tool: Box::from(&b"read"[..]),
                writes: false,
                input: Box::from(&b"input"[..]),
                deadline: Duration::from_secs(1),
            },
        },
        &mut out,
    );
    let Request::Core(core::Event::Fleet(fleet::Event::Relay { channel, run, attempt, call })) =
        out.pop().expect("relay")
    else {
        panic!("fleet relay")
    };
    assert_eq!((channel, run, attempt), (CHANNEL, RUN, ATTEMPT));
    assert_eq!(call.name, name);
    let expected = core::SettledCall {
        serial: 4,
        tool: Box::from(&b"read"[..]),
        answer: core::SettledAnswer::Host { error: false, body: Box::from(&b"answer"[..]) },
    };
    let payload = link.retain(Payload::SettledCall(expected.clone()));
    step(
        &mut link,
        Time::ZERO,
        Event::Relayed { channel: CHANNEL, run: RUN, attempt: ATTEMPT, call: name.clone(), answer: payload },
        &mut out,
    );
    let Request::Relayed { channel, run, attempt, call, answer } = out.pop().expect("relay answer") else {
        panic!("protocol relay answer")
    };
    assert_eq!((channel, run, attempt), (CHANNEL, RUN, ATTEMPT));
    assert_eq!(call, name);
    assert_eq!(answer, Some(expected));
    assert!(link.payload(payload).is_none());
    assert!(link.host_calls.is_empty());
    assert!(out.is_empty());
}

#[test]
fn payloads_and_root_continuations_share_one_token_sequence() {
    let mut link = link();
    let effect = link.fresh();
    let body = link.retain(Payload::Busy);
    let judge = link.fresh();
    assert_eq!((effect.raw(), body.raw(), judge.raw()), (1, 2, 3));
    assert!(link.take_payload(effect).is_none());
    assert!(link.take_payload(judge).is_none());
    assert!(link.take_payload(body).is_some());
}

fn job() -> JobStart {
    JobStart {
        task: RUN.raw(),
        attempt: ATTEMPT.raw(),
        workstream: 8,
        kind: jig_domain_link::JobKind(7),
        spec: Box::from(&b"spec"[..]),
        deadline: Duration::from_secs(2),
        result: 16,
        grants: Box::new([]),
        workspace: Some(Box::from(&b"workspace items"[..])),
    }
}

#[test]
fn a_job_assignment_carries_its_workspace_and_grants() {
    for workspace in [None, Some(Box::from(&b"items"[..]))] {
        let mut job = job();
        job.workspace = workspace;
        job.grants = assignment().grants;
        let expected_workspace = match job.workspace {
            Some(_) => Some(host::Workspace { workstream: job.workstream, items: RUN }),
            None => None,
        };
        let expected = host::JobAssignment {
            run: RUN,
            attempt: ATTEMPT,
            kind: job.kind,
            spec: job.spec.clone(),
            deadline: job.deadline,
            workspace: expected_workspace,
            grants: job.grants.clone(),
        };
        assert_eq!(job_assignment(job), expected);
    }
}

#[test]
fn settled_answers_relay_host_bytes_and_return_delivery_evidence() {
    for error in [false, true] {
        assert_eq!(
            settled_answer(core::SettledAnswer::Host { error, body: Box::from(&b"answer"[..]) }),
            Ok(host::Relayed::Answer { error, text: Box::from(&b"answer"[..]) })
        );
    }
    let delivery =
        core::SettledAnswer::Delivery { outcome: core::DeliveryOutcome::Stale, evidence: Box::from(&b"evidence"[..]) };
    assert_eq!(settled_answer(delivery.clone()), Err(delivery));
}

#[test]
fn a_brief_plan_keeps_its_task_sections_budget_and_deadline() {
    let section = jig_domain_brief::Planned::Core {
        kind: jig_domain_brief::Core::Task,
        text: Box::from(&b"task"[..]),
        limit: 32,
        priority: 1,
        required: true,
    };
    let mut sections = skein_lib::List::uncapped();
    sections.push(section.clone()).expect("uncapped sections");
    let deadline = Time::from_nanos(91);
    let core::Event::Brief(plan) = brief_plan(RUN.raw(), sections, 64, deadline) else { panic!("brief plan") };
    assert_eq!(
        plan,
        jig_domain_brief::GatherEvent::Plan { brief: RUN, budget: 64, deadline, sections: Box::new([section]) }
    );
}

/// jig/docs/design/domain/engine/executors.md, 5: a job crosses the committed assignment door, including retries.
#[test]
fn a_job_start_is_retained_for_placement_and_busy_replacement() {
    let mut link = link();
    let mut out = Queue::uncapped();
    step(&mut link, Time::ZERO, Event::StartJob { job: job() }, &mut out);
    let Request::Core(core::Event::Fleet(fleet::Event::StartJob { run, attempt, job: bound, adopt, .. })) =
        out.pop().unwrap()
    else {
        panic!("fleet job start")
    };
    assert_eq!(
        (run, attempt, bound.kind, bound.spec, bound.result, bound.deadline, adopt),
        (RUN, ATTEMPT, job().kind, 4, 16, job().deadline, false)
    );
    for channel in [Token::new(1), Token::new(2)] {
        step(&mut link, Time::ZERO, Event::Assign { channel, run, attempt }, &mut out);
        let Request::AssignedJob { channel: placed, job: assigned } = out.pop().unwrap() else {
            panic!("assigned job")
        };
        assert_eq!((placed, assigned), (channel, job()));
    }
}

/// jig/docs/design/domain/engine/executors.md, 5: an answer becomes the tasks child's existing End.
#[test]
fn a_jobs_answer_reaches_the_core_as_its_end() {
    let work = host::Work { left: None, saved: None };
    for (ending, answer, end) in [
        (
            host::JobEnding::Finished { result: Box::from(&b"bytes"[..]), work: work.clone() },
            fleet::Answer::Ended,
            tasks::End::Finished {
                result: tasks::TaskResult::Job { bytes: Box::from(&b"bytes"[..]) },
                cancel_delegates: false,
            },
        ),
        (
            host::JobEnding::Failed { failure: host::JobFailure::Deadline, detail: Box::new([]), work: work.clone() },
            fleet::Answer::Failed { turned: None },
            tasks::End::Failed(tasks::Class::Run),
        ),
        (
            host::JobEnding::Failed {
                failure: host::JobFailure::Unprepared(host::Preparation::Permanent { resource: None }),
                detail: Box::new([]),
                work,
            },
            fleet::Answer::Failed { turned: None },
            tasks::End::Failed(tasks::Class::Permanent),
        ),
        (
            host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Kind)),
            fleet::Answer::Invalid,
            tasks::End::Failed(tasks::Class::Permanent),
        ),
        (host::JobEnding::Refused(host::JobRefusal::Busy), fleet::Answer::Busy, tasks::End::Refused),
    ] {
        let mut link = link();
        let mut out = Queue::uncapped();
        let Some(core::Event::Fleet(fleet::Event::Answer { run, attempt, answer: kind, payload, .. })) =
            up(&mut link, CHANNEL, host::LinkUp::JobAnswer { run: RUN, attempt: ATTEMPT, answer: ending })
        else {
            panic!("fleet answer")
        };
        assert_eq!(kind, answer);
        step(&mut link, Time::ZERO, Event::AnswerPayload { run, attempt, payload }, &mut out);
        let Request::Core(core::Event::AnswerPayload { run, attempt, cumulative, end: ended, read, refused, .. }) =
            out.pop().unwrap()
        else {
            panic!("core end")
        };
        assert_eq!((run, attempt, cumulative, ended, read), (RUN, ATTEMPT, 0, end, None));
        assert!(refused.is_empty());
        assert!(out.is_empty(), "job results require no protocol read");
        step(&mut link, Time::ZERO, Event::DropPayload { payload }, &mut out);
        assert_eq!(link.payload_count(), 0);
    }
}

/// jig/docs/design/domain/hosts/hub.md, 6: worker cancellations interrupt agent attempts.
#[test]
fn worker_cancellations_interrupt_agents_but_not_engine_cancellations() {
    for (reason, expected) in [
        (host::Reason::Contact, tasks::End::Recheck),
        (host::Reason::Shutdown, tasks::End::Recheck),
        (host::Reason::Engine, tasks::End::Failed(tasks::Class::Agent)),
    ] {
        let ending = host::Ending::Failed {
            failure: host::Failure::Cancelled(reason),
            detail: Box::new([]),
            work: host::Work { left: None, saved: Some(Token::new(99)) },
        };
        assert_eq!(payload::ending(&ending), expected);
    }
}

/// jig/docs/design/domain/engine/executors.md, 5: every job failure has a task retry class.
#[test]
fn job_failures_keep_their_contract_classes() {
    use host::{DeliveryOutcome as D, JobFailure as F, Preparation as P, Reason as R};
    use tasks::Class as C;
    for (failure, expected) in [
        (F::Unprepared(P::Transient), C::Infrastructure),
        (F::Unprepared(P::Permanent { resource: None }), C::Permanent),
        (F::Failed, C::Run),
        (F::Deadline, C::Run),
        (F::Unstarted, C::Agent),
        (F::Result, C::Invalid),
        (F::Undelivered(D::Stale), C::Permanent),
        (F::Undelivered(D::Refused), C::Permanent),
        (F::Undelivered(D::Failed), C::Transient),
        (F::Cancelled(R::Contact), C::Lost),
    ] {
        let ending =
            host::JobEnding::Failed { failure, detail: Box::new([]), work: host::Work { left: None, saved: None } };
        assert_eq!(payload::job_end(&ending), tasks::End::Failed(expected));
    }
    let shutdown = host::JobEnding::Failed {
        failure: F::Cancelled(R::Shutdown),
        detail: b"interrupted".as_slice().into(),
        work: host::Work { left: None, saved: None },
    };
    assert_eq!(payload::job_end(&shutdown), tasks::End::Recheck);
}

/// jig/docs/design/domain/engine/executors.md, 5: a terminal frees retained job bytes, including a lost claim's.
#[test]
fn a_job_terminal_releases_its_assignment_and_keeps_a_newer_attempts() {
    let mut link = link();
    let mut out = Queue::uncapped();
    step(&mut link, Time::ZERO, Event::StartJob { job: job() }, &mut out);
    for attempt in [ATTEMPT.raw().checked_sub(1).expect("earlier attempt"), ATTEMPT.raw()] {
        let Err(core::Held::TaskTerminalAcknowledged { task, attempt: returned }) =
            down(&mut link, core::Held::TaskTerminalAcknowledged { task: RUN.raw(), attempt })
        else {
            panic!("root terminal continues")
        };
        assert_eq!((task, returned), (RUN.raw(), attempt));
        assert_eq!(link.jobs.len(), u32::from(attempt != ATTEMPT.raw()));
    }
}

#[test]
fn core_now_translates_the_links_six_outputs_without_changing_fields() {
    const PAYLOAD: Token = Token::new(5);
    const TURN: u32 = 7;
    let Event::Relayed { channel, run, attempt, call, answer } = now(core::LinkNow::Relayed {
        channel: CHANNEL,
        run: RUN,
        attempt: ATTEMPT,
        call: Box::from(&b"call"[..]),
        answer: PAYLOAD,
    }) else {
        panic!("relayed event")
    };
    assert_eq!((channel, run, attempt, answer), (CHANNEL, RUN, ATTEMPT, PAYLOAD));
    assert_eq!(call.as_ref(), b"call");
    let Event::DropPayload { payload } = now(core::LinkNow::DropPayload { payload: PAYLOAD }) else {
        panic!("drop payload event")
    };
    assert_eq!(payload, PAYLOAD);
    let Event::TurnPayload { run, attempt, turn, body } =
        now(core::LinkNow::TurnPayload { run: RUN, attempt: ATTEMPT, turn: TURN, body: PAYLOAD })
    else {
        panic!("turn payload event")
    };
    assert_eq!((run, attempt, turn, body), (RUN, ATTEMPT, TURN, PAYLOAD));
    let Event::AnswerPayload { run, attempt, payload } =
        now(core::LinkNow::AnswerPayload { run: RUN, attempt: ATTEMPT, payload: PAYLOAD })
    else {
        panic!("answer payload event")
    };
    assert_eq!((run, attempt, payload), (RUN, ATTEMPT, PAYLOAD));
    let Event::AcceptedTurn { payload, task, attempt, turn, accepted } = now(core::LinkNow::AcceptedTurn {
        payload: PAYLOAD,
        task: RUN.raw(),
        attempt: ATTEMPT.raw(),
        turn: TURN,
        accepted: tasks::Accepted::Already,
    }) else {
        panic!("accepted turn event")
    };
    assert_eq!(
        (payload, task, attempt, turn, accepted),
        (PAYLOAD, RUN.raw(), ATTEMPT.raw(), TURN, tasks::Accepted::Already)
    );
    let problem = tasks::Problem::live_delegates(RUN.raw(), &[ATTEMPT.raw()]);
    let Event::RefusedPayload { request, problem: returned } =
        now(core::LinkNow::RefusedPayload { request: PAYLOAD, problem: problem.clone() })
    else {
        panic!("refused payload event")
    };
    assert_eq!((request, returned), (PAYLOAD, problem));
}

#[test]
fn a_refused_start_is_transient_permanent_or_busy() {
    let endings = [
        (host::Ending::Refused(host::Refusal::Busy), tasks::End::Refused, fleet::Answer::Busy),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Transcript)),
            tasks::End::Failed(tasks::Class::Transient),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
                cause: host::StartCause::Transcript,
                why: b"saved history".as_slice().into(),
            })),
            tasks::End::Failed(tasks::Class::Transient),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
                cause: host::StartCause::Other,
                why: b"invalid assignment".as_slice().into(),
            })),
            tasks::End::Failed(tasks::Class::Permanent),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
                cause: host::StartCause::Grants,
                why: b"invalid grants".as_slice().into(),
            })),
            tasks::End::Failed(tasks::Class::Permanent),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Charter)),
            tasks::End::Failed(tasks::Class::Permanent),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Grants)),
            tasks::End::Failed(tasks::Class::Permanent),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Messages)),
            tasks::End::Failed(tasks::Class::Permanent),
            fleet::Answer::Invalid,
        ),
        (
            host::Ending::Failed {
                failure: host::Failure::Run(host::RunFailure::Transcript),
                detail: Box::new([]),
                work: host::Work { left: None, saved: None },
            },
            tasks::End::Failed(tasks::Class::Transient),
            fleet::Answer::Failed { turned: None },
        ),
    ];
    for (ending, expected_end, expected) in endings {
        assert_eq!(payload::ending(&ending), expected_end, "{ending:?}");
        let mut link = link();
        let event = up(
            &mut link,
            CHANNEL,
            host::LinkUp::Answer {
                run: RUN,
                attempt: ATTEMPT,
                answer: host::RunAnswer { turns: 0, spent: 0, read: None, refused: Box::new([]), ending },
            },
        )
        .expect("terminal reaches fleet");
        let core::Event::Fleet(fleet::Event::Answer { answer, .. }) = event else {
            panic!("terminal reaches the fleet as an answer");
        };
        assert_eq!(answer, expected, "a refused start closes without an acknowledgement");
    }
}

/// jig/docs/design/domain/engine/executors.md, 5: the cause accompanies the class.
#[test]
fn a_failed_jobs_cause_is_carried_to_its_task() {
    for (detail, reason) in [
        (b"push: Broken: early EOF".as_slice(), b"push: Broken: early EOF".as_slice()),
        (b"\xffexit 127: python3 missing", b"?exit 127: python3 missing"),
    ] {
        let ending = host::JobEnding::Failed {
            failure: host::JobFailure::Undelivered(host::DeliveryOutcome::Failed),
            detail: detail.into(),
            work: host::Work { left: None, saved: None },
        };
        assert_eq!(
            payload::job_end(&ending),
            tasks::End::FailedWithReason { class: tasks::Class::Transient, reason: reason.into() }
        );
    }
}

#[test]
fn a_runs_transient_preparation_is_infrastructure_with_its_detail() {
    for (detail, expected) in [
        (
            b"fetch: TimedOut".as_slice(),
            tasks::End::FailedWithReason {
                class: tasks::Class::Infrastructure,
                reason: Box::from(b"fetch: TimedOut".as_slice()),
            },
        ),
        (b"", tasks::End::Failed(tasks::Class::Infrastructure)),
    ] {
        let ending = host::Ending::Failed {
            failure: host::Failure::Unprepared(host::Preparation::Transient),
            detail: detail.into(),
            work: host::Work { left: None, saved: None },
        };
        assert_eq!(payload::ending(&ending), expected);
    }
    let ending = host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
        detail: Box::from(b"fetch: Forbidden".as_slice()),
        work: host::Work { left: None, saved: None },
    };
    assert_eq!(payload::ending(&ending), tasks::End::Failed(tasks::Class::Permanent));
    let ending = host::JobEnding::Failed {
        failure: host::JobFailure::Unprepared(host::Preparation::Transient),
        detail: Box::from(b"fetch: TimedOut".as_slice()),
        work: host::Work { left: None, saved: None },
    };
    assert_eq!(
        payload::job_end(&ending),
        tasks::End::FailedWithReason {
            class: tasks::Class::Infrastructure,
            reason: Box::from(b"fetch: TimedOut".as_slice()),
        }
    );
}

#[test]
fn a_version_refusal_pauses_with_its_reason_and_keeps_history_usable() {
    let ending = host::Ending::Refused(host::Refusal::Version(host::Mismatch {
        wire: b"smith-charter".as_slice().into(),
        version: 5,
        lowest: 4,
        highest: 4,
    }));
    let end = tasks::End::RefusedWithReason {
        reason: b"version mismatch: smith-charter 5, the worker reads 4..4".as_slice().into(),
    };
    assert_eq!(payload::ending(&ending), end);
    assert!(!payload::unusable_history(&ending));
    let mut link = link();
    let mut out = Queue::uncapped();
    step(
        &mut link,
        Time::ZERO,
        Event::Up {
            channel: CHANNEL,
            value: host::LinkUp::Answer {
                run: RUN,
                attempt: ATTEMPT,
                answer: host::RunAnswer { turns: 0, spent: 0, read: None, refused: Box::new([]), ending },
            },
        },
        &mut out,
    );
    let Request::Core(core::Event::Fleet(fleet::Event::Answer { answer, payload, .. })) = out.pop().unwrap() else {
        panic!("fleet answer")
    };
    assert_eq!(answer, fleet::Answer::Version);
    step(&mut link, Time::ZERO, Event::AnswerPayload { run: RUN, attempt: ATTEMPT, payload }, &mut out);
    let Request::Core(core::Event::AnswerPayload { end: actual, unusable_history, .. }) = out.pop().unwrap() else {
        panic!("task terminal")
    };
    assert_eq!(actual, end);
    assert!(!unusable_history);
}
