//! Worker channels in the hub's link vocabulary (jig's `domain/hosts.md`, 2,
//! 6.4 and 6.5; `domain/engine.md`, 11; `testing/jig-test-domain/src/link.rs`).
//! Payloads stay owned here until the fleet asks for or drops them. The
//! protocol layer alone reads an ended answer's outcome and encodes typed
//! assignments.
//! Review run failures follow temper/docs/design/domain/forge.md, 8.3.
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
/// are dropped and counted (jig's `domain/engine.md`, 11). Other records return
/// `None` when the root has no room for their bytes, which admission
/// ([`crate::try_step`]) hands back first, and their host sends again.
pub(crate) fn receive(domain: &mut Domain, channel: Token, link: Link) -> Option<core::Event> {
    match link {
        Link::Hello { worker, hello } => {
            domain.hub_link.connect(channel);
            Some(core::Event::Fleet(fleet::Event::Hello { channel, worker, hello }))
        }
        Link::Lost => {
            domain.hub_link.disconnect(channel);
            Some(core::Event::Fleet(fleet::Event::Lost { channel }))
        }
        Link::Up { up, parts } => {
            let bytes = if let host::LinkUp::Turn { turn, .. } = &up {
                u64::try_from(turn.body.len()).expect("bounded turn bytes")
            } else {
                0
            };
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
        WorkerInput::ResultRead { .. }
        | WorkerInput::Link { link: Link::Hello { .. } | Link::Lost, .. }
        | WorkerInput::Link {
            link:
                Link::Up {
                    up:
                        host::LinkUp::Piece { .. }
                        | host::LinkUp::Logins { .. }
                        | host::LinkUp::SignedOut { .. }
                        | host::LinkUp::Activity { .. }
                        | host::LinkUp::Unauthorized { .. }
                        | host::LinkUp::Limited { .. }
                        | host::LinkUp::AccountRefused { .. }
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
/// resources it saved (domain/forge.md). Beyond them the answer is invalid.
fn parts_within(parts: &Parts, limits: &crate::Limits) -> bool {
    let saved = match &parts.saved {
        Some(tags) => tags.len() <= usize::try_from(limits.tasks.saved_resources).expect("u32 fits usize"),
        None => true,
    };
    saved && parts.pushed.len() <= usize::try_from(limits.forge.resources_per_task).expect("u32 fits usize")
}

/// Interpret worker endings with forge parts and the change's failed-review policy.
pub(crate) fn answer(domain: &mut Domain, run: Token, attempt: Token, payload: Token) {
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

// A run that cannot judge supplies no verdict (temper/docs/design/domain/forge.md, 8.3).
fn fail_review(domain: &Domain, event: &mut core::Event) {
    let core::Event::AnswerPayload { run, attempt, payload, end, .. } = event else { return };
    if !domain.core.current_proof(run.raw(), attempt.raw()) || !current_review(domain, run.raw()) {
        return;
    }
    let Some(Payload::Answer {
        end: hub::Ending::Link(host::Ending::Failed { failure: host::Failure::Run(failure), .. }),
        ..
    }) = domain.hub_link.payload(*payload)
    else {
        return;
    };
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
    if child != task
        || row.change.last_head != Some(head)
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
        host::RunFailure::Cancelled
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
            let host_workspace = if workspace.is_some() {
                Some(host::Workspace { workstream: job.workstream, items: Token::new(job.task) })
            } else {
                None
            };
            crate::Delivery::AssignedJob {
                channel,
                workspace,
                job: host::JobAssignment {
                    run: Token::new(job.task),
                    attempt: Token::new(job.attempt),
                    kind: job.kind,
                    spec: job.spec,
                    deadline: job.deadline,
                    workspace: host_workspace,
                    grants: job.grants,
                },
            }
        }
        hub::Request::Message { channel, run, attempt, word } => {
            crate::Delivery::Link { channel, down: message(run, attempt, &word) }
        }
        hub::Request::Relayed { channel, run, attempt, call: name, answer } => {
            let answer = match answer {
                None => host::Relayed::Busy,
                Some(call) => match call.answer {
                    core::SettledAnswer::Host { error, body } => host::Relayed::Answer { error, text: body },
                    core::SettledAnswer::Delivery { .. } => {
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
/// it left on the forge kept for a lost run (domain/forge.md).
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
        tasks::MessageKind::Result(_) => (
            // The client's chat cards read this line (temper-client-domain pages/delegation.rs `results`).
            b"Result from task ".as_slice(),
            match word.from {
                tasks::Party::Task(task) => task,
                tasks::Party::Person(person) => person,
                tasks::Party::Deployment { .. } => 0,
            },
        ),
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
    let number = Decimal::of(number);
    let bytes = label.len().checked_add(number.as_bytes().len()).expect("bounded message label");
    let bytes = bytes.checked_add(2).expect("bounded message separator");
    let bytes = bytes.checked_add(word.words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(label).expect("measured label");
    writer.put(number.as_bytes()).expect("measured source");
    writer.put(b":\n").expect("measured separator");
    writer.put(&word.words).expect("measured words");
    writer.finish()
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
/// Shared by live delivery and the inbox carried at start (domain/agent.md, 5).
#[must_use]
pub fn inbox_sender(word: &tasks::Word) -> Box<[u8]> {
    let label = sender(word.from);
    let (kind, question, end) = match word.kind {
        tasks::MessageKind::Question => (b" asks (question ".as_slice(), word.number, b")".as_slice()),
        tasks::MessageKind::Answer { question } => (b" answers question ".as_slice(), question, b"".as_slice()),
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
    let bytes = label.len().checked_add(kind.len()).expect("bounded message kind");
    let bytes = bytes.checked_add(question.as_bytes().len()).expect("bounded question number");
    let bytes = bytes.checked_add(end.len()).expect("bounded message label");
    let mut writer = Writer::new(bytes);
    writer.put(&label).expect("measured sender");
    writer.put(kind).expect("measured kind");
    writer.put(question.as_bytes()).expect("measured question");
    writer.put(end).expect("measured end");
    writer.finish()
}

fn sender(value: tasks::Party) -> Box<[u8]> {
    match value {
        tasks::Party::Task(number) | tasks::Party::Person(number) => Box::from(Decimal::of(number).as_bytes()),
        tasks::Party::Deployment { .. } => Box::from(&b"deployment"[..]),
    }
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
            b"2 asks (question 4): Go ahead?\nKeep the newline."
        );
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
