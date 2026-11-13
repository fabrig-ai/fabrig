#![expect(clippy::disallowed_methods, reason = "ordinary channel fixtures generate briefs and inspect their UTF-8")]
use super::fixtures;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Time, Token};
fn review_assignment(pull_bytes: usize) -> temper_engine_domain::Assignment {
    use temper_engine_domain::{BriefBody, BriefKind, BriefSection, ForgeBriefKind};
    let settings = fixtures::engine();
    let stock = crate::stock::defaults(&settings.jig.llm, settings.jig.limits.core.tasks.result_bytes);
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.charter = 4;
    assignment.run.policy = stock.charters.into_iter().find(|charter| charter.number == 4).unwrap().policy;
    assignment.run.authority.tools = assignment.run.policy.tools;
    assignment.run.contract = tasks::Contract::Verdict {
        choices: (1..=2).map(|code| tasks::Verdict { code, words: 1024, followups: 0, fields: Box::new([]) }).collect(),
    };
    let mut pull = b"Pull #10 acme/shop head recorded-head\n".to_vec();
    let remaining = pull_bytes.checked_sub(pull.len()).unwrap();
    pull.extend_from_slice("λ".repeat(remaining / 2).as_bytes());
    if !remaining.is_multiple_of(2) {
        pull.push(b'x');
    }
    assert_eq!(pull.len(), pull_bytes);
    assignment.sections = Box::new([
        BriefSection {
            kind: BriefKind::Core(jig_domain_brief::Core::Task),
            body: BriefBody::Text(b"Review L's pull #10 against its task.".as_slice().into()),
        },
        BriefSection { kind: BriefKind::Forge(ForgeBriefKind::Pull), body: BriefBody::Text(pull.into()) },
    ]);
    assignment
}

fn admitted_review(assignment: temper_engine_domain::Assignment, bound: u64) -> smith_host_domain::Start {
    let mut settings = fixtures::engine();
    settings.jig.limits.hub.charter_bytes = bound;
    settings.files = fixtures::worker_digests(&settings);
    let mut workers = crate::workers::Workers::configured(&mut settings, [7; 16]).unwrap();
    workers.configured_forges(fixtures::forge_table(&fixtures::forges()));
    assert_eq!(workers.encoding().charter_bytes, bound.min(crate::assignment::WORKER_CHARTER_BYTES));
    let event = workers
        .encoded_assignment(Time::ZERO, Token::new(8), assignment, &crate::values::Values::new(0, 0))
        .expect("review assignment encodes");
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, parts, .. } = event
    else {
        panic!("assignment event")
    };
    admit_review_wire(&assignment, &settings);
    let mut worker = fixtures::worker();
    worker.link(
        Time::ZERO,
        jig_protocol_worker_link::Request::Down { down: host::LinkDown::Assign { assignment }, parts },
    );
    let temper_worker_domain::Event::Assign { assignment, .. } = worker.engine_events().pop().unwrap() else {
        panic!("review reaches worker")
    };
    decode_review(assignment, bound)
}

fn admit_review_wire(assignment: &host::Assignment, settings: &crate::settings::Engine) {
    use jig_protocol_worker_wire as wire;
    let limits = crate::workers::limits(settings).unwrap();
    let bodies = wire::sizes::limits(&limits.sizes).unwrap();
    let run = &assignment.assignment;
    let record = wire::Assignment::new(
        &bodies,
        wire::AssignmentParts {
            task: run.run.raw(),
            attempt: run.attempt.raw(),
            identity: run.identity,
            workspace: None,
            save: run.save,
            live: wire::LiveAsk::new(&bodies, wire::LiveAskParts { text: false, reasoning: false, input: false })
                .unwrap(),
            charter: run.charter.clone(),
            transcript: Box::new([]),
            answered: skein_lib::List::with_capacity(0),
            messages: skein_lib::List::with_capacity(0),
            grants: skein_lib::List::with_capacity(0),
        },
    )
    .expect("review passes assignment wire admission");
    let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut bytes).unwrap();
    wire::Assignment::decode(&bodies, &mut skein_lib::Reader::new(&bytes.finish())).unwrap();
}

fn decode_review(assignment: host::Assignment, bound: u64) -> smith_host_domain::Start {
    let run = assignment.assignment;
    assert_eq!((run.run, run.attempt), (Token::new(7), Token::new(2)));
    assert_eq!(run.identity, jig_protocol_charter::run_identity(&[7; 16], 7, 2));
    let mut agent = fixtures::worker_agent();
    agent.host_limits.charter_bytes = bound;
    *crate::worker_route::decode(
        run.run,
        host::Start {
            live: run.live,
            workspace: None,
            charter: run.charter,
            activation: run.attempt.raw(),
            identity: run.identity,
            turns: assignment.turns,
            answered: assignment.answered,
            messages: assignment.messages,
            grants: run.grants,
        },
        &agent,
    )
    .expect("review passes receiving charter ownership admission")
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the regression verifies the retained UTF-8 prefix")]
fn a_large_pull_review_fits_before_assignment_admission() {
    let assignment = review_assignment(41_473);
    let original = temper_engine_smith::charter(&assignment);
    let start = admitted_review(assignment, 65_536);
    let charter = start.charter.value();
    assert_eq!(charter.instructions, original.instructions);
    assert_eq!(charter.grants, original.grants);
    assert_eq!(charter.outcome, original.outcome);
    assert_eq!(charter.brief.sections[0], original.brief.sections[0]);
    let pull = &charter.brief.sections[1];
    assert_eq!(pull.title.as_ref(), b"Pull request");
    assert!(pull.text.starts_with(b"Pull #10 acme/shop head recorded-head\n"));
    assert!(pull.text.ends_with(b" bytes; whole: read_pull/read_pull_files]\n"));
    assert!(skein_lib::bytes::find(&pull.text, b"[cut ").is_some());
    ::core::str::from_utf8(&pull.text).expect("a pull cut keeps complete UTF-8 characters");
}

#[test]
fn an_already_fitting_review_keeps_its_whole_charter() {
    let assignment = review_assignment(100);
    let original = temper_engine_smith::charter(&assignment);
    let start = admitted_review(assignment, 65_536);
    assert!(start.charter.value() == &original, "an already-fitting charter stays whole");
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary fixtures assemble aggregate section bodies"
)]
fn aggregate_gathered_sections_fit_in_their_existing_order() {
    use temper_engine_domain::{BriefBody, BriefKind, BriefSection, ForgeBriefKind};
    let mut assignment = review_assignment(10_000);
    let mut sections = assignment.sections.into_vec();
    for kind in [ForgeBriefKind::Home, ForgeBriefKind::Ci] {
        sections.push(BriefSection { kind: BriefKind::Forge(kind), body: BriefBody::Text(vec![b'x'; 20_000].into()) });
    }
    assignment.sections = sections.into();
    let original = temper_engine_smith::charter(&assignment);
    let start = admitted_review(assignment, 65_536);
    let sections = &start.charter.value().brief.sections;
    assert_eq!(sections.len(), 4, "earlier gathered sections remain available");
    assert_eq!(sections[0], original.brief.sections[0]);
    assert_eq!(sections[1], original.brief.sections[1]);
    assert_eq!(sections[2], original.brief.sections[2]);
    assert!(sections[3].text.ends_with(b" bytes; whole: read_job/read_checks]\n"));
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "the envelope rows measure encoded bytes and retained UTF-8"
)]
fn review_fitting_uses_the_configured_assignment_envelope() {
    // Measured CPU: 0.023 s for both envelopes and the reference's large sections.
    for bound in [49_152, 98_304] {
        let assignment = review_assignment(41_473);
        let original = temper_engine_smith::charter(&assignment);
        let start = admitted_review(assignment, bound);
        let pull = &start.charter.value().brief.sections[1];
        assert!(pull.text.len() < original.brief.sections[1].text.len());
    }
    let bound = crate::assignment::WORKER_CHARTER_BYTES;
    let mut empty = review_assignment(100);
    empty.sections[1].body = temper_engine_domain::BriefBody::Text(Box::default());
    let charter = temper_engine_smith::charter(&empty);
    let encoded =
        smith_protocol_channel::encode_charter(&charter, &smith_charter::CEILINGS, &fixtures::worker_agent().endpoints)
            .unwrap();
    let fixed = charter.owned_bytes().unwrap().max(u64::try_from(encoded.len()).unwrap());
    let room = usize::try_from(bound.checked_sub(fixed).unwrap()).unwrap();
    for (size, whole) in [(room, true), (room + 1, false)] {
        let assignment = review_assignment(size);
        let original = temper_engine_smith::charter(&assignment);
        let start = admitted_review(assignment, bound);
        let fitted = start.charter.value();
        assert!(fitted.owned_bytes().unwrap() <= bound);
        assert_eq!(fitted.brief.sections[0], original.brief.sections[0]);
        if whole {
            assert_eq!(fitted, &original);
        } else {
            let original = ::core::str::from_utf8(&original.brief.sections[1].text).unwrap();
            let text = ::core::str::from_utf8(&fitted.brief.sections[1].text).unwrap();
            let (kept, marker) = text.rsplit_once("\n[cut ").expect("a section past its room is cut");
            assert!(original.starts_with(kept));
            assert_eq!(marker, format!("{} bytes; whole: read_pull/read_pull_files]\n", size - kept.len()));
        }
    }
}

/// temper/docs/design/domain/agent/README.md, 6: startup faults keep their cause and pending words.
#[test]
fn an_asking_chat_answers_after_acceptance_and_failed_resume() {
    use fixtures::worker_peers::{ASKING_HISTORY_BYTES, AskingChat, ResumeAttempt, asking_attempt};
    let mut chat = AskingChat::new();
    let run = chat.chat(20);
    chat.accept(&run);
    assert_eq!(chat.driver.task(run.task).unwrap().settings.run, Some(1), "running work stays");
    let resumed = chat.resume(&run);
    assert_eq!(resumed.run.revision, 2, "a later run crosses the accepted revision");
    let mut history_bytes = 0_usize;
    for body in &resumed.transcript {
        history_bytes = history_bytes.checked_add(body.len()).unwrap();
    }
    assert_eq!(history_bytes, ASKING_HISTORY_BYTES);
    let up = asking_attempt(resumed.clone(), ResumeAttempt::Fault);
    assert_startup_fault(&up);
    chat.answer(up);
    let retry = chat.retry();
    assert_eq!(retry.task, run.task);
    assert_eq!(retry.transcript, resumed.transcript, "an unknown fault never discards history");
    let up = asking_attempt(retry, ResumeAttempt::Answer);
    assert_description(&up);
    chat.answer(up);
    if let tasks::Phase::Held { .. } = chat.driver.task(run.task).unwrap().phase {
        panic!("one passing startup fault never exhausts agent tries");
    }
    assert_pending_question(&resumed);
    asking_refusal_and_hold_controls(&mut chat, run.task);
}

fn asking_refusal_and_hold_controls(chat: &mut fixtures::worker_peers::AskingChat, task: u64) {
    use fixtures::worker_peers::{DESCRIPTION, ResumeAttempt, asking_attempt};
    use jig_domain_people as people;
    chat.ask(30, people::Ask::Say { project: 1, task, words: DESCRIPTION.into() });
    let run = chat.driver.last_assignment();
    let up = asking_attempt(run, ResumeAttempt::Transcript);
    chat.answer(up);
    let fresh = chat.retry();
    assert!(fresh.transcript.is_empty(), "identified transcript refusal recovers fresh");
    assert_pending_question(&fresh);
    let up = asking_attempt(fresh, ResumeAttempt::Answer);
    assert_description(&up);
    chat.answer(up);
    chat.ask(31, people::Ask::Say { project: 1, task, words: DESCRIPTION.into() });
    let mut run = chat.driver.last_assignment();
    for _ in 0..=chat.driver.env.limits.tasks.retries.agent.retries {
        let up = asking_attempt(run, ResumeAttempt::Fault);
        assert_startup_fault(&up);
        chat.answer(up);
        if let tasks::Phase::Held { .. } = chat.driver.task(task).unwrap().phase {
            break;
        }
        run = chat.retry();
    }
    let row = chat.driver.task(task).unwrap();
    let tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Agent), .. } = row.phase else {
        panic!("persistent unidentified faults keep the agent-failure hold");
    };
    let hold = row.revisions.hold;
    assert_eq!(hold, 1, "release hold 1 from the walk");
    chat.ask(32, people::Ask::Release { project: 1, task, revision: hold, again: Some(true) });
    let released = chat.driver.last_assignment();
    let up = asking_attempt(released, ResumeAttempt::Answer);
    assert_description(&up);
    chat.answer(up);
    let control = chat.chat(33);
    assert_ne!(control.task, task, "a fresh chat remains a working control");
    let up = asking_attempt(control, ResumeAttempt::Answer);
    assert_description(&up);
    chat.answer(up);
}

fn assert_startup_fault(up: &fixtures::worker_peers::AskingAnswer) {
    let host::LinkUp::Answer { answer, .. } = &up.terminal else { panic!("startup answer") };
    assert_eq!((answer.turns, answer.read), (0, None), "an unadmitted attempt took no turn or words");
    assert_eq!(
        answer.ending,
        host::Ending::Failed {
            failure: host::Failure::Agent(host::AgentFailure::Exited),
            detail: b"resume channel ended before admission".as_slice().into(),
            work: host::Work { left: None, saved: None },
            model: None,
        }
    );
}

fn assert_description(up: &fixtures::worker_peers::AskingAnswer) {
    let Some(host::LinkUp::Turn { turn, .. }) = &up.turn else { panic!("description turn") };
    assert!(skein_lib::bytes::find(&turn.body, b"running work stays on its revision").is_some());
    let host::LinkUp::Answer { answer, .. } = &up.terminal else { panic!("description terminal") };
    let host::Ending::Parked { .. } = answer.ending else { panic!("the description parks the chat") };
}

fn assert_pending_question(run: &temper_engine_domain::Assignment) {
    for message in &run.messages {
        if message.words.as_ref() == fixtures::worker_peers::DESCRIPTION {
            return;
        }
    }
    panic!("the person's pending question remains whole");
}
