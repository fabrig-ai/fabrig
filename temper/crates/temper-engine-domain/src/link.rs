//! Worker channels in the hub's link vocabulary (jig's `domain/hosts/README.md`, 2, and `domain/hosts/hub.md`, 4 and 5
//! ; `jig/docs/design/domain/engine/views.md`; `testing/jig-test-domain/src/link.rs`).
//! Payloads stay owned here until the fleet asks for or drops them. The
//! protocol layer alone reads an ended answer's outcome and encodes typed
//! assignments.
//! Review run failures follow temper/docs/design/domain/forge/change/gates.md, 1.
use crate::boundary::{Link, Parts, Payload, Request, Work, WorkerInput, WorkerRequest};
use crate::domain::Domain;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_hub_link as hub;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, List, Queue, Token, Writer};
use temper_engine_domain_forge as forge;
impl Domain {
    /// The unique resource pushed under this fenced answer's authentic writer claim.
    #[must_use]
    pub fn change_resource(&self, payload: Token) -> Option<(u16, u16, u64)> {
        let Some(Payload::Answer { task, attempt, .. }) = self.hub_link.payload(payload) else {
            return None;
        };
        let parts = self.answer_parts.get(&payload)?;
        if !self.core.current_proof(*task, *attempt) || !parts_within(parts, &self.limits) {
            return None;
        }
        let tasks::Contract::Change { connector, kind, .. } = self.core.tasks.task(*task)?.contract else {
            return None;
        };
        let heads = self.forge.reported_heads(&self.limits.forge, *task, *attempt, &parts.pushed);
        (connector == self.config.forge_connector).then_some((connector, kind, settled_resource(&heads)?))
    }
}

pub(crate) fn settled_resource(heads: &[(forge::Name, skein_forge::api::Commit)]) -> Option<u64> {
    let [(name, _)] = heads else { return None };
    Some(u64::from(name.repository))
}

/// Translate one channel input into the core's vocabulary. Unkept live pieces
/// are dropped and counted (jig's `domain/engine/views.md`). Other records return
/// `None` when the root has no room for their bytes, which admission
/// ([`crate::try_step`]) hands back first, and their host sends again.
pub(crate) fn receive(domain: &mut Domain, channel: Token, link: Link) -> Option<core::Event> {
    match link {
        Link::Mismatch => Some(core::Event::Fleet(fleet::Event::Mismatch { channel })),
        Link::Hello { worker, hello } => {
            domain.hub_link.connect(channel);
            Some(core::Event::Fleet(fleet::Event::Hello { channel, worker, hello }))
        }
        Link::Lost => {
            domain.hub_link.disconnect(channel);
            Some(core::Event::Fleet(fleet::Event::Lost { channel }))
        }
        Link::Up { up, parts } => {
            let bytes = if let host::LinkUp::Turn { turn, .. } = &up { skein_lib::len_u64(turn.body.len()) } else { 0 };
            let invalid = !parts_within(&parts, &domain.limits);
            let mut event = hub::up(&mut domain.hub_link, channel, up)?;
            domain.hold_bytes(bytes);
            if let core::Event::Fleet(fleet::Event::Answer { payload, answer, .. }) = &mut event {
                domain.answer_parts.insert(*payload, parts).expect("one application part per answer");
                if invalid {
                    *answer = fleet::Answer::Failed { turned: None };
                }
            }
            Some(event)
        }
    }
}

/// Only these protocol answers and worker records can retain a new payload.
pub(crate) fn keeps_payload(input: &WorkerInput) -> bool {
    match input {
        WorkerInput::DecodedHostCall { .. }
        | WorkerInput::RenderedHostCall { .. }
        | WorkerInput::Link {
            link:
                Link::Up {
                    up: host::LinkUp::Turn { .. } | host::LinkUp::JobAnswer { .. } | host::LinkUp::Answer { .. },
                    ..
                },
            ..
        } => true,
        WorkerInput::Opened { .. }
        | WorkerInput::ResultRead { .. }
        | WorkerInput::Link { link: Link::Hello { .. } | Link::Lost | Link::Mismatch, .. }
        | WorkerInput::Link {
            link:
                Link::Up {
                    up:
                        host::LinkUp::Piece { .. }
                        | host::LinkUp::ListRead { .. }
                        | host::LinkUp::Draining
                        | host::LinkUp::Logins { .. }
                        | host::LinkUp::SignedOut { .. }
                        | host::LinkUp::Activity { .. }
                        | host::LinkUp::Unauthorized { .. }
                        | host::LinkUp::Limited { .. }
                        | host::LinkUp::AccountRefused { .. }
                        | host::LinkUp::Ask { .. }
                        | host::LinkUp::Call { .. },
                    ..
                },
            ..
        } => false,
    }
}

/// Whether the root can retain one more payload from a worker or the protocol.
pub(crate) fn room(domain: &Domain) -> bool {
    domain.hub_link.payload_count() < crate::limits::payload_slots(&domain.limits).expect("valid payload room")
}

/// temper's parts within their bounds: the heads a run pushed and the
/// resources it saved (domain/forge/README.md). Beyond them the answer is invalid.
fn parts_within(parts: &Parts, limits: &crate::Limits) -> bool {
    let saved = match &parts.saved {
        Some(tags) => tags.len() <= skein_lib::usize_of(limits.tasks.saved_resources),
        None => true,
    };
    saved && parts.pushed.len() <= skein_lib::usize_of(limits.forge.resources_per_task)
}

/// Interpret worker endings with forge parts and the change's failed-review policy.
pub(crate) fn answer(domain: &mut Domain, run: Token, attempt: Token, payload: Token) {
    if let Some(Payload::Answer { task, end, .. }) = domain.hub_link.payload(payload) {
        match end {
            hub::Ending::Link(host::Ending::Failed { failure, detail, .. }) => {
                domain.run_diagnostics.failed.push(crate::RunDiagnostic {
                    task: *task,
                    run: run.raw(),
                    attempt: attempt.raw(),
                    failure: *failure,
                    detail: detail.clone(),
                });
            }
            hub::Ending::Link(host::Ending::Refused(reason)) => {
                domain.run_diagnostics.refused.push(crate::RunRefusalDiagnostic {
                    task: *task,
                    run: run.raw(),
                    attempt: attempt.raw(),
                    reason: reason.clone(),
                });
            }
            hub::Ending::Link(host::Ending::Ended { .. } | host::Ending::Parked { .. }) | hub::Ending::Scripted(_) => {}
            hub::Ending::Job(ending) => {
                let ending = match ending {
                    host::JobEnding::Failed { failure, detail, .. } => {
                        Some(crate::JobDiagnosticEnding::Failed { failure: *failure, detail: detail.clone() })
                    }
                    host::JobEnding::Refused(reason) => Some(crate::JobDiagnosticEnding::Refused(reason.clone())),
                    host::JobEnding::Finished { .. } => None,
                };
                if let Some(ending) = ending {
                    let executor = domain.core.tasks.executor(*task).expect("an accepted answer has a task");
                    let kind = match executor {
                        tasks::Executor::Job { kind } => Some(kind),
                        tasks::Executor::Agent { .. }
                        | tasks::Executor::Procedure { .. }
                        | tasks::Executor::Person(_) => None,
                    };
                    if let Some(kind) = kind {
                        domain.run_diagnostics.jobs.push(crate::JobDiagnostic {
                            task: *task,
                            run: run.raw(),
                            attempt: attempt.raw(),
                            kind,
                            ending,
                        });
                    }
                }
            }
        }
    }
    if match domain.answer_parts.get(&payload) {
        Some(parts) => !parts_within(parts, &domain.limits),
        None => false,
    } {
        let event = result_event(domain, payload, tasks::End::Failed(tasks::Class::Invalid)).expect("retained answer");
        domain.work.push(Work::Core(event));
        return;
    }
    route(domain, hub::Event::AnswerPayload { run, attempt, payload });
}

/// The root applies immediate outputs to its work queue and protocol door.
pub(crate) fn route(domain: &mut Domain, event: hub::Event) {
    let mut out = Queue::uncapped();
    hub::step(&mut domain.hub_link, skein_lib::Time::ZERO, event, &mut out);
    for _ in 0..out.len() {
        match out.pop().expect("link output count") {
            hub::Request::Core(mut event) => {
                fail_review(domain, &mut event);
                decorate_answer(domain, &mut event);
                domain.work.push(Work::Core(event));
            }
            hub::Request::ReadResult { payload, outcome } => {
                let request = Request::Worker(WorkerRequest::ReadResult { payload, outcome });
                assert!(crate::domain::now(domain, request), "result read door reserved");
            }
            hub::Request::AssignedJob { .. }
            | hub::Request::Assigned { .. }
            | hub::Request::Down { .. }
            | hub::Request::Relayed { .. }
            | hub::Request::Message { .. }
            | hub::Request::ConnectorCall { .. }
            | hub::Request::TurnCopy { .. } => {
                unreachable!("immediate payload routing returns a core event or protocol read")
            }
        }
    }
}

// A run that cannot judge supplies no verdict (temper/docs/design/domain/forge/change/gates.md, 1).
fn fail_review(domain: &Domain, event: &mut core::Event) {
    let core::Event::AnswerPayload { run, attempt, payload, end, .. } = event else { return };
    if !domain.core.current_proof(run.raw(), attempt.raw()) || !current_review(domain, run.raw()) {
        return;
    }
    let Some(Payload::Answer {
        end: hub::Ending::Link(host::Ending::Failed { failure: host::Failure::Run(failure), model, .. }),
        ..
    }) = domain.hub_link.payload(*payload)
    else {
        return;
    };
    if *failure == host::RunFailure::Model
        && let Some(model) = model
        && model.outage()
    {
        return;
    }
    let Some(reason) = review_failure(failure) else { return };
    *end =
        tasks::End::Finished { result: tasks::TaskResult::Failure { reason: reason.into() }, cancel_delegates: false };
}

fn current_review(domain: &Domain, task: u64) -> bool {
    let Some(task_row) = domain.core.tasks.task(task) else { return false };
    let tasks::Party::Task(parent) = task_row.requester else { return false };
    let Some(row) = domain.forge.change(parent) else { return false };
    let Some((child, temper_engine_domain_forge_change::Delegate::Gate { number, head })) = row.delegate else {
        return false;
    };
    let expected = if let temper_engine_domain_forge_change::State::Judging { target, .. } = &row.change.state {
        Some(target.prefix)
    } else {
        row.change.last_head
    };
    if child != task
        || expected != Some(head)
        || row.change.state == (temper_engine_domain_forge_change::State::CancellingGates { head })
    {
        return false;
    }
    for gate in &row.change.gates {
        if gate.number == number && gate.kind == temper_engine_domain_forge_change::GateKind::Agent {
            return true;
        }
    }
    false
}

fn review_failure(failure: &host::RunFailure) -> Option<&'static [u8]> {
    match failure {
        host::RunFailure::Model => Some(b"Review model call failed before giving a verdict"),
        host::RunFailure::Policy => Some(b"Review ended without a verdict"),
        host::RunFailure::Budget => Some(b"Review exhausted its run budget without a verdict"),
        host::RunFailure::Exceeded(_) => Some(b"Review exceeded a model call bound without a verdict"),
        host::RunFailure::Transcript
        | host::RunFailure::Cancelled
        | host::RunFailure::Stale
        | host::RunFailure::Limited { .. }
        | host::RunFailure::Refused { .. } => None,
    }
}

/// Translate link hand-outs through temper's assignment, inbox and tool rendering.
pub(crate) fn delivery(domain: &mut Domain, event: hub::Event) -> crate::Delivery {
    let mut out = Queue::uncapped();
    hub::step(&mut domain.hub_link, skein_lib::Time::ZERO, event, &mut out);
    assert!(out.len() == 1, "one delivery per link hand-out");
    match out.pop().expect("link delivery") {
        hub::Request::Assigned { channel, assignment: value } => {
            crate::Delivery::Assigned { channel, assignment: assignment(domain, value) }
        }
        hub::Request::AssignedJob { channel, job } => {
            let workspace = domain.job_workspaces.remove(&job.task);
            assert_eq!(workspace.is_some(), job.workspace.is_some(), "prepared job items");
            crate::Delivery::AssignedJob { channel, workspace, job: hub::job_assignment(job) }
        }
        hub::Request::Message { channel, run, attempt, word } => {
            crate::Delivery::Link { channel, down: message(run, attempt, &word) }
        }
        hub::Request::Relayed { channel, run, attempt, call: name, answer } => {
            let answer = match answer {
                None => host::Relayed::Busy,
                Some(mut call) => match hub::settled_answer(call.answer) {
                    Ok(answer) => answer,
                    Err(answer) => {
                        call.answer = answer;
                        return crate::Delivery::Host(Box::new(crate::HostDelivery::Answer {
                            channel,
                            task: run.raw(),
                            attempt: attempt.raw(),
                            name,
                            call,
                        }));
                    }
                },
            };
            crate::Delivery::Link { channel, down: host::LinkDown::Relayed { run, attempt, call: name, answer } }
        }
        hub::Request::Down { channel, value } => crate::Delivery::Link { channel, down: value },
        hub::Request::Core(_)
        | hub::Request::ReadResult { .. }
        | hub::Request::ConnectorCall { .. }
        | hub::Request::TurnCopy { .. } => unreachable!("link hand-out returns one delivery"),
    }
}

/// The answer's task terminal, with temper's saved work translated and what
/// it left on the forge kept for a lost run (domain/forge/README.md).
pub(crate) fn result_event(domain: &mut Domain, payload: Token, end: tasks::End) -> Option<core::Event> {
    let mut event = hub::result_event(&domain.hub_link, payload, end)?;
    decorate_answer(domain, &mut event);
    Some(event)
}

fn decorate_answer(domain: &mut Domain, event: &mut core::Event) {
    let core::Event::AnswerPayload { run, attempt, payload, saved: saved_out, invalid_saved: invalid_out, .. } = event
    else {
        return;
    };
    let (task, attempt) = (run.raw(), attempt.raw());
    let parts = domain.answer_parts.get(payload).expect("worker answer has temper parts").clone();
    let Parts { saved, pushed, preparation } = if parts_within(&parts, &domain.limits) { parts } else { Parts::none() };
    if let Some(crate::Preparation::Conflicted { tag, head }) = &preparation {
        domain.work.push(Work::Forge(temper_engine_domain_forge::Event::StartConflicted {
            child: task,
            tag: *tag,
            head: *head,
        }));
    }
    if let Some(preparation) = preparation
        && domain.facts.try_push(crate::Fact::PreparationFailed { task, attempt, preparation }).is_err()
    {
        domain.facts_lost = domain.facts_lost.saturating_add(1);
    }
    drop(domain.forge_left.insert((task, attempt), pushed));
    let (saved, invalid_saved) = match saved {
        Some(tags) => match crate::route::forge_route::saved_resources(domain.config.forge_connector, &tags) {
            Some(resources) => (Some(resources), false),
            None => (None, true),
        },
        None => (None, false),
    };
    *saved_out = saved;
    *invalid_out = invalid_saved;
}

/// Keep generic section ordering in the link and forge text in temper.
pub(crate) fn keep_sections(domain: &mut Domain, task: u64, sections: Box<[crate::BriefSection]>) {
    use crate::{BriefBody, BriefKind};
    use jig_domain_brief::GatherPlaced;
    let mut brief = List::with_capacity(u32::try_from(sections.len()).expect("bounded brief sections"));
    for section in sections {
        let placed = match section {
            crate::BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Text(text) } => {
                GatherPlaced::Core { kind, text }
            }
            crate::BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Missing(why) } => {
                GatherPlaced::CoreMissing { kind, why }
            }
            section @ crate::BriefSection { kind: BriefKind::Forge(_), .. } => {
                let token = domain.hub_link.fresh();
                let size = u32::try_from(crate::budget::section_bytes(::core::slice::from_ref(&section)))
                    .expect("bounded forge section");
                domain.forge_briefs.insert(token, section).expect("one forge section per token");
                GatherPlaced::Connector { connector: domain.config.forge_connector, kind: 0, token, size }
            }
        };
        brief.push(placed).expect("one placed section per input");
    }
    domain.hub_link.keep_brief(task, brief.into_boxed());
}

fn sections(domain: &mut Domain, brief: Box<[jig_domain_brief::GatherPlaced]>) -> Box<[crate::BriefSection]> {
    use crate::{BriefBody, BriefKind, BriefSection};
    use jig_domain_brief::GatherPlaced;
    let mut sections = List::with_capacity(u32::try_from(brief.len()).expect("bounded brief sections"));
    for placed in brief {
        let section = match placed {
            GatherPlaced::Core { kind, text } => {
                BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Text(text) }
            }
            GatherPlaced::CoreMissing { kind, why } => {
                BriefSection { kind: BriefKind::Core(kind), body: BriefBody::Missing(why) }
            }
            GatherPlaced::Connector { token, .. } => {
                domain.forge_briefs.remove(&token).expect("retained forge section")
            }
            GatherPlaced::Missing { .. } => unreachable!("forge missing sections are translated at completion"),
        };
        sections.push(section).expect("one section per placed input");
    }
    sections.into_boxed()
}

pub(crate) fn take_sections(domain: &mut Domain, task: u64) -> Option<Box<[crate::BriefSection]>> {
    let brief = domain.hub_link.take_brief(task)?;
    Some(sections(domain, brief))
}

pub(crate) fn assignment(domain: &mut Domain, assignment: hub::Assignment) -> crate::Assignment {
    let parts = domain.assignment_parts.remove(&assignment.task).expect("prepared temper assignment parts");
    crate::Assignment {
        check_timeout: crate::route::forge_route::check_timeout(domain, assignment.task),
        task: assignment.task,
        attempt: assignment.attempt,
        conversation: assignment.conversation,
        live: host::LiveAsk {
            text: assignment.live.text,
            reasoning: assignment.live.reasoning,
            input: assignment.live.input,
        },
        charter: assignment.charter,
        run: assignment.run,
        sections: sections(domain, assignment.brief),
        messages: assignment.messages,
        saved: parts.saved,
        workstream: parts.workstream,
        workspace: parts.workspace,
        transcript: assignment.transcript,
        answered: parts.answered,
        settled: assignment.answered,
        grants: assignment.grants,
    }
}

pub(crate) fn take_assignment(domain: &mut Domain, task: u64) -> Option<crate::Assignment> {
    let value = domain.hub_link.take_assignment(task)?;
    Some(assignment(domain, value))
}

/// One committed inbox word for its live host, its kind kept in its rendered line.
pub(crate) fn message(run: Token, attempt: Token, word: &tasks::Word) -> host::LinkDown {
    host::LinkDown::Message {
        run,
        attempt,
        name: Token::new(word.number),
        sender: inbox_sender(word),
        words: inbox_words(run.raw(), word),
        finish_requested: word.finish_requested(),
    }
}

/// Preserve the task-owned message kind when translating its opaque words,
/// for a live run's message and a starting run's inbox alike. `task` is
/// the receiving task, so an answer notice can distinguish its delegate's question.
#[must_use]
pub fn inbox_words(task: u64, word: &tasks::Word) -> Box<[u8]> {
    if let tasks::MessageKind::SettingsDecision { outcome, .. } = word.kind {
        return settings_decision_words(outcome, &word.words);
    }
    if let tasks::MessageKind::Escalation { task, revision, .. } = word.kind {
        return escalation_words(task, revision, &word.words);
    }

    let (label, number) = match word.kind {
        tasks::MessageKind::QuestionAnswered { question } => {
            if let tasks::Party::Task(asker) = word.from
                && asker != task
            {
                return delegated_answer_notice(asker, question, &word.words);
            }
            return question_notice(b"Person answered question ", question, b" in their inbox", &word.words);
        }
        tasks::MessageKind::QuestionEnded { question } => {
            return question_notice(b"Question ", question, b" ended", &[]);
        }
        tasks::MessageKind::SettingsDecision { .. } => unreachable!("settings decision rendered above"),
        tasks::MessageKind::Result(tasks::ResultKind::Verdict { code, .. }) => {
            return verdict_words(word, code);
        }
        tasks::MessageKind::Result(_) => (RESULT_TASK_LABEL, source_number(word.from)),
        tasks::MessageKind::ProposalDecision { proposal, accepted } => {
            (if accepted { b"Proposal accepted ".as_slice() } else { b"Proposal rejected ".as_slice() }, proposal)
        }
        tasks::MessageKind::PersonStalled { task } => (b"Person task past its waiting deadline ".as_slice(), task),
        tasks::MessageKind::EffectNotMade { entry } => (b"Effect not made ".as_slice(), entry),
        tasks::MessageKind::Met { dependency, removed } => return met_words(dependency, removed, &word.words),
        tasks::MessageKind::Words
        | tasks::MessageKind::Question
        | tasks::MessageKind::Answer { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::News { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::Escalation { .. } => return word.words.clone(),
    };
    numbered_words(label, number, &word.words)
}

const RESULT_TASK_LABEL: &[u8] = b"Result from task ";
/// A landing result's live message adds its label, separator and sender
/// (temper/docs/design/domain/agent/charter.md, 3). Both numbers can be `u64::MAX`.
pub(crate) const CHANGE_RESULT_ENVELOPE_BYTES: usize = RESULT_TASK_LABEL
    .len()
    .saturating_add(b"18446744073709551615".len())
    .saturating_add(b":\n".len())
    .saturating_add(b"18446744073709551615".len());

fn numbered_words(label: &[u8], number: u64, words: &[u8]) -> Box<[u8]> {
    let number = Decimal::of(number);
    let bytes = label.len().checked_add(number.as_bytes().len()).expect("bounded message label");
    let bytes = bytes.checked_add(2).expect("bounded message separator");
    let bytes = bytes.checked_add(words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(label).expect("measured label");
    writer.put(number.as_bytes()).expect("measured source");
    writer.put(b":\n").expect("measured separator");
    writer.put(words).expect("measured words");
    writer.finish()
}

fn source_number(from: tasks::Party) -> u64 {
    match from {
        tasks::Party::Task(task) => task,
        tasks::Party::Person(person) => person,
        tasks::Party::Deployment { .. } => 0,
    }
}

fn verdict_words(word: &tasks::Word, code: u32) -> Box<[u8]> {
    let reason = question_notice(b"verdict ", u64::from(code), b"", &word.words);
    question_notice(b"Result from task ", source_number(word.from), b"", &reason)
}

fn delegated_answer_notice(asker: u64, question: u64, words: &[u8]) -> Box<[u8]> {
    let asker = Decimal::of(asker);
    let question = Decimal::of(question);
    let separator = if words.is_empty() { b"".as_slice() } else { b":\n".as_slice() };
    let parts = [
        b"Person answered task ".as_slice(),
        asker.as_bytes(),
        b"'s question ",
        question.as_bytes(),
        b" in their inbox; task ",
        asker.as_bytes(),
        b" has the answer and does the work",
        separator,
        words,
    ];
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("bounded answer notice");
    }
    let mut writer = Writer::new(bytes);
    for part in parts {
        writer.put(part).expect("measured answer notice");
    }
    writer.finish()
}

fn question_notice(prefix: &[u8], question: u64, suffix: &[u8], words: &[u8]) -> Box<[u8]> {
    let separator = if words.is_empty() { b"".as_slice() } else { b":\n".as_slice() };
    let question = Decimal::of(question);
    let bytes = prefix.len().checked_add(question.as_bytes().len()).expect("bounded question label");
    let bytes = bytes.checked_add(suffix.len()).expect("bounded question notice");
    let bytes = bytes.checked_add(separator.len()).expect("bounded message separator");
    let bytes = bytes.checked_add(words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(prefix).expect("measured prefix");
    writer.put(question.as_bytes()).expect("measured question");
    writer.put(suffix).expect("measured suffix");
    writer.put(separator).expect("measured separator");
    writer.put(words).expect("measured words");
    writer.finish()
}

fn met_words(dependency: u64, removed: bool, words: &[u8]) -> Box<[u8]> {
    let dependency = Decimal::of(dependency);
    let line = if removed {
        b" was removed from its dependencies; the task can go on".as_slice()
    } else {
        b" is done; the task can go on".as_slice()
    };
    let separator = if words.is_empty() { b"".as_slice() } else { b":\n".as_slice() };
    let bytes = 1_usize.checked_add(dependency.as_bytes().len()).expect("bounded dependency label");
    let bytes = bytes.checked_add(line.len()).expect("bounded dependency notice");
    let bytes = bytes.checked_add(separator.len()).expect("bounded message separator");
    let bytes = bytes.checked_add(words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(b"T").expect("measured label");
    writer.put(dependency.as_bytes()).expect("measured dependency");
    writer.put(line).expect("measured notice");
    writer.put(separator).expect("measured separator");
    writer.put(words).expect("measured words");
    writer.finish()
}

/// The sender label and question reference that smith prefixes to the words.
/// Shared by live delivery and the inbox carried at start (domain/agent/README.md, 4).
#[must_use]
pub fn inbox_sender(word: &tasks::Word) -> Box<[u8]> {
    let label = word.from.sender();
    let (kind, question, end, show_answer_form) = match word.kind {
        tasks::MessageKind::Question => {
            let show_answer_form = match word.from {
                tasks::Party::Task(_) => true,
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => false,
            };
            (b" asks (question ".as_slice(), word.number, b")".as_slice(), show_answer_form)
        }
        tasks::MessageKind::Answer { question } => (b" answers question ".as_slice(), question, b"".as_slice(), false),
        tasks::MessageKind::Words
        | tasks::MessageKind::Result(_)
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::News { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::Escalation { .. } => return label,
    };
    let question = Decimal::of(question);
    let [answer, target, form, number]: [&[u8]; 4] = if show_answer_form {
        [b"; answer with message target=", &label, b", form=answer, question=", question.as_bytes()]
    } else {
        [b""; 4]
    };
    let parts: [&[u8]; 8] = [&label, kind, question.as_bytes(), answer, target, form, number, end];
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part.len()).expect("bounded message label");
    }
    let mut writer = Writer::new(bytes);
    for part in parts {
        writer.put(part).expect("measured message label");
    }
    writer.finish()
}

fn escalation_words(task: u64, revision: u64, words: &[u8]) -> Box<[u8]> {
    let task = Decimal::of(task);
    let revision = Decimal::of(revision);
    let parts: [&[u8]; 6] = [
        words,
        b"\nHeld task ",
        task.as_bytes(),
        b", hold revision ",
        revision.as_bytes(),
        b". Use read_task on that task.",
    ];
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded escalation envelope");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("sized escalation envelope");
    }
    writer.finish()
}

fn settings_decision_words(outcome: tasks::SettingsOutcome, reason: &[u8]) -> Box<[u8]> {
    let number = Decimal::of(match outcome {
        tasks::SettingsOutcome::Accepted { revision } => u64::from(revision),
        tasks::SettingsOutcome::Stale { newest } => u64::from(newest),
        tasks::SettingsOutcome::Rejected => 0,
    });
    let parts: [&[u8]; 3] = match outcome {
        tasks::SettingsOutcome::Accepted { .. } => [b"accepted: revision ", number.as_bytes(), b" made"],
        tasks::SettingsOutcome::Rejected => [b"rejected: ", reason, b""],
        tasks::SettingsOutcome::Stale { .. } => [b"withdrawn as stale: the newest is ", number.as_bytes(), b""],
    };
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded inbox words");
    }
    let mut out = Writer::new(size);
    for part in parts {
        out.put(part).expect("measured inbox words");
    }
    out.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_failures_leave_cancellation_and_account_waits_to_the_core() {
        for failure in [
            host::RunFailure::Cancelled,
            host::RunFailure::Stale,
            host::RunFailure::Limited { account: 1, reset: None },
            host::RunFailure::Refused { account: 1 },
        ] {
            assert_eq!(review_failure(&failure), None);
        }
        for failure in [host::RunFailure::Model, host::RunFailure::Policy, host::RunFailure::Budget] {
            assert!(review_failure(&failure).is_some());
        }
    }

    #[test]
    fn current_review_outages_keep_the_infrastructure_end() {
        use host::{ModelEvidence as E, ModelFailureKind as K};
        for model in [
            Some(review_model(K::Unavailable, E::Unsent)),
            Some(review_model(K::Overloaded, E::Unsent)),
            Some(review_model(K::Invalid, E::Unsent)),
            Some(review_model(K::Unavailable, E::Response)),
            None,
        ] {
            let (mut domain, task) = review_root();
            let outage = match &model {
                Some(model) => model.outage(),
                None => false,
            };
            let expected = if outage {
                tasks::End::FailedWithReason {
                    class: tasks::Class::Infrastructure,
                    reason: model.as_ref().unwrap().words(),
                }
            } else {
                tasks::End::Finished {
                    result: tasks::TaskResult::Failure {
                        reason: b"Review model call failed before giving a verdict".as_slice().into(),
                    },
                    cancel_delegates: false,
                }
            };
            let payload = domain.hub_link.retain(Payload::Answer {
                task,
                attempt: 1,
                cumulative: 0,
                read: None,
                refused: Box::new([]),
                end: hub::Ending::Link(host::Ending::Failed {
                    failure: host::Failure::Run(host::RunFailure::Model),
                    model,
                    detail: Box::new([]),
                    work: host::Work { left: None, saved: None },
                }),
            });
            domain.answer_parts.insert(payload, Parts::none()).unwrap();
            assert!(current_review(&domain, task));
            assert!(domain.core.current_proof(task, 1));
            route(&mut domain, hub::Event::AnswerPayload { run: Token::new(task), attempt: Token::new(1), payload });
            let Some(Work::Core(core::Event::AnswerPayload { end, .. })) = domain.work.pop() else {
                panic!("core answer")
            };
            assert_eq!(end, expected);
        }
    }

    fn review_model(kind: host::ModelFailureKind, evidence: host::ModelEvidence) -> host::ModelFailure {
        host::ModelFailure {
            endpoint: b"anthropic".as_slice().into(),
            model: b"claude-opus-5-5".as_slice().into(),
            kind,
            evidence,
            status: Some(403),
        }
    }

    fn review_root() -> (Domain, u64) {
        use crate::tests::{live_task, resume_fixture};
        let (source, env, task) = live_task(None, tasks::Executor::Agent { charter: 1 });
        let mut child = source.core.tasks.task(task).unwrap().clone();
        let mut parent = child.clone();
        parent.number = 7;
        child.requester = tasks::Party::Task(7);
        child.depth = 1;
        let (mut domain, _) = resume_fixture::root_with_records(
            resume_fixture::config(7),
            &env.limits,
            &[
                crate::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(
                    *source.core.tasks.funding(child.funder).unwrap(),
                ))),
                crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(parent)))),
                crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(child)))),
            ],
        );
        domain
            .core
            .proofs
            .insert(
                task,
                core::RunProof {
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
        domain.forge.restore_begin(env.wall);
        temper_engine_domain_forge::step(
            &mut domain.forge,
            &crate::translate::environment_forge(&env),
            temper_engine_domain_forge::Event::Restore {
                record: temper_engine_domain_forge::Stored::Change(Box::new(review_change(task))),
            },
            &mut Queue::uncapped(),
        );
        (domain, task)
    }

    fn review_change(task: u64) -> temper_engine_domain_forge::ChangeRow {
        use temper_engine_domain_forge as forge;
        use temper_engine_domain_forge_change as change;
        forge::ChangeRow {
            signed: Box::new([]),
            task: 7,
            repository: skein_forge::api::Repository { forge: 3, repository: 1 },
            branch: b"temper/7".as_slice().into(),
            base: b"main".as_slice().into(),
            title: b"Change".as_slice().into(),
            body: Box::new([]),
            priority: 0,
            ci: None,
            change: change::Change {
                ended_held: None,
                early: change::Early::None,
                way: change::Way::Queue,
                landing: None,
                task: 7,
                state: change::State::Gating { head: [7; 32], asked: Box::new([1]) },
                gates: Box::new([change::Gate {
                    number: 1,
                    kind: change::GateKind::Agent,
                    blocking: true,
                    freshness: change::Freshness::Clean,
                    eager: true,
                }]),
                clean: Box::new([]),
                gate_failures: Box::new([]),
                released: None,
                repairs: 0,
                resolutions: 0,
                updates: 0,
                since: skein_lib::Wall::EPOCH,
                ready_since: None,
                owns_turn: false,
                last_head: Some([7; 32]),
            },
            pull: Some(7),
            pending: None,
            effect: change::EffectResult::None,
            delegate: Some((task, change::Delegate::Gate { number: 1, head: [7; 32] })),
            delegate_status: change::Status::Unknown,
            verdicts: Box::new([]),
            goal: None,
            ending: None,
            repair_tasks: Box::new([]),
            resolution_tasks: Box::new([]),
            result: forge::ChangeResult::empty(),
            gate_remarks: Box::new([]),
            drift: None,
            base_repair: false,
            queue_repair: None,
            queue_repairs: 0,
            producing_charter: 1,
            review_charter: 1,
            producing_llm: None,
            review_llm: None,
            questions: change::Questions::default(),
            effect_account: None,
            producer_failure: None,
            report: Box::new([]),
            repairs: None,
            follows: None,
            catch_up: None,
            close: None,
            merge_result: None,
            follow_check: None,
        }
    }

    #[test]
    fn text_pieces_reach_the_core_with_their_identity_and_offsets() {
        let (mut domain, _) = crate::tests::resume_fixture::root();
        let run = Token::new(31);
        let attempt = Token::new(932);
        for offset in [0, u32::MAX] {
            let event = receive(
                &mut domain,
                Token::new(7),
                Link::Up {
                    up: host::LinkUp::Piece {
                        run,
                        attempt,
                        piece: host::Piece {
                            turn: 3,
                            retry: 2,
                            kind: host::LiveKind::Text,
                            offset: u64::from(offset),
                            bytes: Box::from(&b"words"[..]),
                        },
                    },
                    parts: Parts::none(),
                },
            );
            let Some(core::Event::RunText {
                run: actual_run,
                attempt: actual_attempt,
                turn,
                retry,
                offset: actual_offset,
                text,
                call,
            }) = event
            else {
                panic!("text reaches core")
            };
            assert_eq!((actual_run, actual_attempt, turn, retry, actual_offset), (run, attempt, 3, 2, offset));
            assert_eq!(text.as_ref(), b"words");
            assert_eq!(call, None);
        }
        assert_eq!(domain.pieces_dropped(), 0);
    }

    #[test]
    fn input_pieces_reach_the_core_with_their_provider_identity_and_offsets() {
        let (mut domain, _) = crate::tests::resume_fixture::root();
        let run = Token::new(31);
        let attempt = Token::new(932);
        for offset in [0, u32::MAX] {
            let event = receive(
                &mut domain,
                Token::new(7),
                Link::Up {
                    up: host::LinkUp::Piece {
                        run,
                        attempt,
                        piece: host::Piece {
                            turn: 3,
                            retry: 2,
                            kind: host::LiveKind::Input {
                                call: Box::from(b"call".as_slice()),
                                tool: Box::from(b"tool".as_slice()),
                            },
                            offset: u64::from(offset),
                            bytes: Box::from(b"{}".as_slice()),
                        },
                    },
                    parts: Parts::none(),
                },
            );
            let Some(core::Event::RunInput {
                run: actual_run,
                attempt: actual_attempt,
                turn,
                retry,
                offset: actual_offset,
                id,
                tool,
                input,
            }) = event
            else {
                panic!("input reaches core")
            };
            assert_eq!((actual_run, actual_attempt, turn, retry, actual_offset), (run, attempt, 3, 2, offset));
            assert_eq!(
                (id.as_ref(), tool.as_ref(), input.as_ref()),
                (b"call".as_slice(), b"tool".as_slice(), b"{}".as_slice())
            );
        }
        assert_eq!(domain.pieces_dropped(), 0);
    }

    #[test]
    fn unkept_piece_kinds_and_offsets_are_dropped_and_counted() {
        let (mut domain, _) = crate::tests::resume_fixture::root();
        for (index, (kind, offset)) in [
            (host::LiveKind::Reasoning, 0),
            (
                host::LiveKind::Input { call: Box::from(&b"call"[..]), tool: Box::from(&b"tool"[..]) },
                u64::from(u32::MAX) + 1,
            ),
            (host::LiveKind::Text, u64::from(u32::MAX) + 1),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                receive(
                    &mut domain,
                    Token::new(7),
                    Link::Up {
                        up: host::LinkUp::Piece {
                            run: Token::new(31),
                            attempt: Token::new(932),
                            piece: host::Piece { turn: 3, retry: 2, kind, offset, bytes: Box::from(&b"words"[..]) }
                        },
                        parts: Parts::none(),
                    }
                )
                .is_none()
            );
            assert_eq!(domain.pieces_dropped(), u64::try_from(index + 1).expect("three cases"));
        }
    }

    fn rendered_line(from: tasks::Party, kind: tasks::MessageKind) -> Box<[u8]> {
        let word = tasks::Word {
            number: 4,
            from,
            kind,
            words: b"Go ahead?\nKeep the newline.".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        };
        let host::LinkDown::Message { sender, words, name, .. } = message(Token::new(1), Token::new(2), &word) else {
            panic!("message translation")
        };
        assert_eq!(name, Token::new(4));
        let bytes = sender.len().checked_add(2).expect("test sender and separator");
        let bytes = bytes.checked_add(words.len()).expect("test line");
        let mut line = Writer::new(bytes);
        line.put(&sender).expect("sender fits");
        line.put(b": ").expect("separator fits");
        line.put(&words).expect("words fit");
        line.finish()
    }

    #[test]
    fn words_render_with_their_sender() {
        assert_eq!(
            rendered_line(tasks::Party::Task(2), tasks::MessageKind::Words).as_ref(),
            b"2: Go ahead?\nKeep the newline."
        );
        assert_eq!(
            rendered_line(tasks::Party::Person(1), tasks::MessageKind::Words).as_ref(),
            b"1: Go ahead?\nKeep the newline."
        );
    }

    #[test]
    fn a_question_renders_its_number_for_the_answer_form() {
        assert_eq!(
            rendered_line(tasks::Party::Task(2), tasks::MessageKind::Question).as_ref(),
            b"2 asks (question 4; answer with message target=2, form=answer, question=4): Go ahead?\nKeep the newline."
        );
    }

    #[test]
    fn non_task_questions_keep_their_number_without_a_task_answer_form() {
        for (party, expected) in [
            (tasks::Party::Person(1), b"1 asks (question 4): Go ahead?\nKeep the newline.".as_slice()),
            (
                tasks::Party::Deployment { project: 1 },
                b"deployment asks (question 4): Go ahead?\nKeep the newline.".as_slice(),
            ),
        ] {
            assert_eq!(rendered_line(party, tasks::MessageKind::Question).as_ref(), expected);
        }
    }

    #[test]
    fn an_answer_renders_the_question_it_answers() {
        assert_eq!(
            rendered_line(tasks::Party::Person(1), tasks::MessageKind::Answer { question: 3 }).as_ref(),
            b"1 answers question 3: Go ahead?\nKeep the newline."
        );
    }

    #[test]
    fn question_notices_render_their_number_and_any_answer_words() {
        let cases: &[(u64, tasks::MessageKind, &[u8], &[u8])] = &[
            (2, tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 2, fields: Box::new([]) }),
             b"bold", b"Result from task 2:\nverdict 2:\nbold"),
            (2, tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 2, fields: Box::new([]) }),
             b"", b"Result from task 2:\nverdict 2"),
            (2, tasks::MessageKind::QuestionAnswered { question: 42 }, b"", b"Person answered question 42 in their inbox"),
            (
                2, tasks::MessageKind::QuestionAnswered { question: 42 },
                b"fruit,count\nGo ahead.",
                b"Person answered question 42 in their inbox:\nfruit,count\nGo ahead.",
            ),
            (1, tasks::MessageKind::QuestionAnswered { question: 42 }, b"", b"Person answered task 2's question 42 in their inbox; task 2 has the answer and does the work"),
            (1, tasks::MessageKind::QuestionAnswered { question: 42 }, b"fruit,count\nGo ahead.", b"Person answered task 2's question 42 in their inbox; task 2 has the answer and does the work:\nfruit,count\nGo ahead."),
            (2, tasks::MessageKind::QuestionEnded { question: 42 }, b"", b"Question 42 ended"),
        ];
        for (task, kind, words, expected) in cases {
            let word = tasks::Word {
                number: 7,
                from: tasks::Party::Task(2),
                kind: kind.clone(),
                words: (*words).into(),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            };
            assert_eq!(inbox_words(*task, &word).as_ref(), *expected);
            let host::LinkDown::Message { words: live, .. } = message(Token::new(*task), Token::new(1), &word) else {
                panic!("live message");
            };
            assert_eq!(live.as_ref(), *expected);
        }
    }

    #[test]
    fn a_landing_result_fits_its_live_label_and_sender_at_the_longest_task_number() {
        const RESULT_BYTES: usize = 10_660; // The stock's result and hub event bound.
        const RESULT_WORDS: usize = 10_601; // 59 bytes for the label, two u64s and separator.
        // Both envelope rows measured 3.8 ms CPU together.
        for (source, bytes) in [(1, 10_622), (u64::MAX, RESULT_BYTES)] {
            let word = tasks::Word {
                number: 7,
                from: tasks::Party::Task(source),
                kind: tasks::MessageKind::Result(tasks::ResultKind::Change { connector: 0, kind: 1, resource: 1 }),
                words: Box::new([b'r'; RESULT_WORDS]),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            };
            let host::LinkDown::Message { words, sender, .. } = message(Token::new(2), Token::new(1), &word) else {
                panic!("live message")
            };
            assert_eq!(words.len() + sender.len(), bytes);
            assert!(bytes <= RESULT_BYTES);
            assert_eq!(RESULT_WORDS + CHANGE_RESULT_ENVELOPE_BYTES, RESULT_BYTES);
        }
    }

    #[test]
    fn met_notices_name_the_dependency_and_preserve_any_words() {
        let cases: &[(bool, &[u8], &[u8])] = &[
            (false, b"", b"T42 is done; the task can go on"),
            (true, b"", b"T42 was removed from its dependencies; the task can go on"),
            (false, b"landed", b"T42 is done; the task can go on:\nlanded"),
            (
                true,
                b"no longer needed",
                b"T42 was removed from its dependencies; the task can go on:\nno longer needed",
            ),
        ];
        for &(removed, words, expected) in cases {
            let word = tasks::Word {
                number: 2,
                from: tasks::Party::Task(7),
                kind: tasks::MessageKind::Met { dependency: 42, removed },
                words: words.into(),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            };
            assert_eq!(inbox_words(2, &word).as_ref(), expected);
        }
    }
}
