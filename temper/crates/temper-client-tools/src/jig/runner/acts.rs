//! Offered acts are read once, then kept by the shared client's request table.
pub(crate) mod choice;
mod goal;
mod subject;
use super::{Called, Out, Runner, finish};
use crate::jig::{
    Limits,
    call::{Call, Value, Values, number, value},
    render::{self, outcome::Operation},
    rows,
};
use alloc::boxed::Box;
use jig_client::{Client, Event, Handle, Key, Latest, Readable, Readout, contract as c, shapes};
use skein_lib::{Decimal, Env, List, Queue, Writer};
#[derive(Debug)]
pub(super) struct Reading {
    called: Called,
    document: Option<c::ReadValue>,
}
#[derive(Debug)]
pub(super) struct Sent {
    pub(super) called: Called,
    pub(super) operation: Operation,
    pub(super) holder: Option<c::AskAccountHolder>,
    pub(super) dispatched: bool,
    alternate: Option<c::Ask>,
}
/// A durable visibility answer waits within its call for me to carry its page
/// (client/ctl/tools.md, 3).
#[derive(Debug)]
pub(super) struct Visibility {
    called: Called,
    project: u32,
    public: bool,
}
pub(crate) fn me<'a>(runner: &Runner, client: &'a Client) -> Option<&'a c::Me> {
    match client.document(runner.me())?.latest? {
        Latest::Jig(c::Delivery::Me(me)) => Some(me),
        Latest::Jig(_) | Latest::Family(_) => None,
    }
}
pub(super) fn start(runner: &mut Runner, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.call.tool == rows::CHANGE_SETTINGS_AT || called.call.tool == rows::PROPOSE_SETTINGS_AT {
        let Some(me) = me(runner, client) else { return };
        let request = settings_request(&called.call, &env.limits, me);
        prepared(runner, client, env, called, request, out);
        return;
    }
    if let Some(request) = immediate(&called.call, &env.limits) {
        prepared(runner, client, env, called, request, out);
        return;
    }
    if called.call.tool == rows::PROJECT_FUND_AT
        || called.call.tool == rows::PROJECT_AMEND_AT
        || called.call.tool == rows::PROJECT_DELETE_AT
    {
        let Some(me) = me(runner, client) else { return };
        let request = if called.call.tool == rows::PROJECT_AMEND_AT {
            project_amend(&called.call, me)
        } else if called.call.tool == rows::PROJECT_DELETE_AT {
            project_delete(&called.call, me)
        } else {
            project_fund(&called.call, me)
        };
        prepared(runner, client, env, called, request, out);
        return;
    }
    if called.call.tool == rows::CHAT_CREATE_AT || called.call.tool == rows::GOAL_START_AT {
        let Some(me) = me(runner, client) else { return };
        let ask = creation(&called.call, me, &env.limits);
        prepared(runner, client, env, called, ask, out);
        return;
    }
    let read = act_read(&called.call);
    out.push(Out::Client {
        event: Event::Read {
            handle: Handle { asker: runner.asker, number: (u64::from(called.call_number) << 32u32) | 1 },
            read: Readable::Jig(read),
        },
    });
    runner.acts.insert(called.call_number, Reading { called, document: None });
}
fn immediate(call: &Call, limits: &Limits) -> Option<Prepared> {
    let request = match call.tool {
        rows::PROJECT_MAKE_AT => project_make(call, limits),
        rows::LIFT_AT => Ok((
            Request::Ask(c::Ask::Lift {
                hold: number(call, 0).expect("hold argument"),
                revision: c::Revision { value: number(call, 1).expect("revision argument") },
                with_tasks: value(call, 2).is_some(),
            }),
            None,
        )),
        rows::RESTORE_AT => restore_request(call, limits),
        rows::MOVE_TO_NEWEST_AT => {
            let revision = u32::try_from(number(call, 0).expect("revision argument")).expect("revision fits u32");
            Ok((Request::Ask(c::Ask::MoveToNewest(c::MoveToNewest { revision })), None))
        }
        _ => return None,
    };
    Some(request)
}
fn project_make(call: &Call, limits: &Limits) -> Prepared {
    let name = words(call, 0, limits)?;
    Ok((Request::Ask(c::Ask::MakeProject { name, policy: 1 }), None))
}
fn restore_request(call: &Call, limits: &Limits) -> Prepared {
    let reason = words(call, 1, limits)?;
    let revision = u32::try_from(number(call, 0).expect("revision argument")).expect("revision fits u32");
    Ok((Request::Ask(c::Ask::Restore { revision, reason }), None))
}
fn settings_request(call: &Call, limits: &Limits, me: &c::Me) -> Prepared {
    let settings = match crate::jig::settings_file::request(call, limits) {
        Ok(settings) => settings,
        Err(why) => return Err(Failed::Detail(why)),
    };
    if let Some(why) = choice::settings_effort(settings.items.as_slice(), me) {
        return Err(Failed::Detail(why));
    }
    let ask = if call.tool == rows::PROPOSE_SETTINGS_AT {
        c::Ask::Propose {
            project: 0,
            reason: words(call, 1, limits)?,
            action: c::ProposeAction::Settings(c::ProposeSettings { against: settings.against, items: settings.items }),
        }
    } else {
        c::Ask::ChangeSettings(settings)
    };
    Ok((Request::Ask(ask), None))
}
fn act_read(call: &Call) -> c::Read {
    let task = number(call, 0).unwrap_or(0);
    match call.tool {
        rows::DECIDE_AT => {
            let Some(Value::Proposal(proposal)) = value(call, 0) else { unreachable!("proposal form") };
            c::Read::Proposal { proposal: *proposal }
        }
        rows::TASK_DECIDE_AT => c::Read::Escalation { task },
        rows::ANSWER_AT if call.form != 0 => c::Read::PersonTask { task },
        rows::TAKE_AT | rows::HAND_BACK_AT => c::Read::PersonTask { task },
        rows::ANSWER_AT => c::Read::Question { task, message: number(call, 1).expect("question") },
        _ => c::Read::Task { task },
    }
}
pub(crate) enum Failed {
    Refused(&'static [u8]),
    Fixed(&'static [u8]),
    Unknown { what: &'static [u8], name: Box<[u8]> },
    NotOffered { name: Box<[u8]> },
    SubAgentNamedTwice { name: Box<[u8]> },
    Detail(Box<[u8]>),
}
pub(crate) type Prepared = Result<(Request, Option<c::Recipient>), Failed>;
fn project_amend(call: &Call, me: &c::Me) -> Prepared {
    let project = super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let Some(Value::Word(visibility)) = value(call, 1) else { unreachable!("visibility argument") };
    let public = *visibility == 0;
    Ok((Request::Ask(c::Ask::SetVisibility { project, public }), None))
}
fn project_delete(call: &Call, me: &c::Me) -> Prepared {
    let project = super::reads::project(call, me).ok_or(Failed::Refused(b"unknown project"))?;
    Ok((Request::Ask(c::Ask::RemoveProject { project, grace: None }), None))
}
fn project_fund(call: &Call, me: &c::Me) -> Prepared {
    match super::reads::project(call, me) {
        Some(project) => {
            let period_spend = number(call, 1).expect("amount argument");
            let edit = match number(call, 2) {
                Some(role) => c::PolicyEdit::RoleSpend { role: u32::try_from(role).expect("role fits"), period_spend },
                None => c::PolicyEdit::ProjectSpend { period_spend },
            };
            Ok((Request::Ask(c::Ask::ChangePolicy { project, edit }), None))
        }
        None => Err(Failed::Fixed(b"unknown project")),
    }
}
fn creation(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    if call.tool == rows::GOAL_START_AT { goal::start(call, me, limits) } else { chat(call, me, limits) }
}
fn chat(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    let project = super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let template = number(call, 1);
    let words = words(call, 2, limits)?;
    let shaped = match template {
        Some(template) => match u32::try_from(template) {
            Ok(template) => shapes::chat_from(me, project, template, words),
            Err(_) => return Err(chat_template_refusal(me, project, template)),
        },
        None => shapes::chat(me, project, words),
    };
    let mut ask = match shaped {
        Ok(ask) => ask,
        Err(why) => {
            if why == shapes::NotOffered::Template
                && let Some(template) = template
            {
                return Err(chat_template_refusal(me, project, template));
            }
            return Err(Failed::Fixed(b"chat not offered"));
        }
    };
    let c::Ask::Create { tasks, .. } = &mut ask else { unreachable!("chat creates") };
    let task = tasks.get_mut(0).expect("chat creates one task");
    task.llm = choice::selection(call, 3, 4, 5, me, project, task.llm.as_ref())?;
    let mut creating = None;
    for membership in &me.projects {
        if membership.project == project {
            creating = membership.creating.as_ref();
        }
    }
    let creating = creating.ok_or(Failed::Fixed(b"chat not offered"))?;
    let draft = shapes::GoalDraft {
        words: task.spec.words.clone(),
        executor: task.executor,
        budget: task.authority.spend,
        deadline: task.authority.deadline,
        priority: task.tracked,
        llm: task.llm.clone(),
    };
    match shapes::fall(creating, &draft, task.deadline) {
        shapes::Fall::Within => Ok((Request::Ask(ask), None)),
        shapes::Fall::Proposable(to) => {
            let c::Ask::Create { tasks, .. } = ask else { unreachable!("chat creates") };
            Ok((
                Request::Ask(c::Ask::Propose {
                    project,
                    action: c::ProposeAction::Batch { tasks },
                    reason: empty(limits),
                }),
                Some(to),
            ))
        }
        shapes::Fall::Refused(_) | shapes::Fall::NotOffered(_) => Err(Failed::Fixed(b"chat exceeds offered bounds")),
    }
}
// client/ctl/tools.md, 3: name the chat templates offered here.
fn chat_template_refusal(me: &c::Me, project: u32, template: u64) -> Failed {
    let number = Decimal::of(template);
    let prefix = b"template ";
    let middle = b" is no chat template of this project; its chat templates: ";
    let mut length = prefix
        .len()
        .checked_add(number.as_bytes().len())
        .expect("template number fits")
        .checked_add(middle.len())
        .expect("refusal prefix fits");
    let mut any = false;
    for membership in &me.projects {
        if membership.project == project {
            for template in &membership.templates {
                if template.flags.conversation {
                    let digits = Decimal::of(u64::from(template.number));
                    length = length
                        .checked_add(if any { 2 } else { 0 })
                        .expect("separator fits")
                        .checked_add(digits.as_bytes().len())
                        .expect("template number fits")
                        .checked_add(1)
                        .expect("space fits")
                        .checked_add(template.name.bytes().len())
                        .expect("template name fits");
                    any = true;
                }
            }
        }
    }
    if !any {
        length = length.checked_add(b"none".len()).expect("empty list fits");
    }
    let mut out = Writer::new(length);
    out.put(prefix).expect("measured prefix");
    out.put(number.as_bytes()).expect("measured template number");
    out.put(middle).expect("measured refusal");
    let mut any = false;
    for membership in &me.projects {
        if membership.project == project {
            for template in &membership.templates {
                if template.flags.conversation {
                    if any {
                        out.put(b", ").expect("measured separator");
                    }
                    let digits = Decimal::of(u64::from(template.number));
                    out.put(digits.as_bytes()).expect("measured number");
                    out.put(b" ").expect("measured space");
                    out.put(template.name.bytes()).expect("measured name");
                    any = true;
                }
            }
        }
    }
    if !any {
        out.put(b"none").expect("measured empty list");
    }
    Failed::Detail(out.finish())
}
pub(crate) fn words(call: &Call, index: usize, limits: &Limits) -> Result<c::Text, Failed> {
    let bytes = match value(call, index) {
        Some(Value::Text(bytes)) => bytes.as_ref(),
        None => b"",
        _ => return Err(Failed::Fixed(b"text not loaded")),
    };
    match c::Text::new(Box::from(bytes), limits.words) {
        Ok(text) => Ok(text),
        Err(_) => Err(Failed::Fixed(b"text limit")),
    }
}
fn empty(limits: &Limits) -> c::Text {
    c::Text::new(Box::default(), limits.words).expect("empty words fit")
}
pub(super) fn page(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    handle: Handle,
    page: Readout,
    out: &mut Queue<Out>,
) {
    let prefix = u32::try_from(handle.number >> 32u32).expect("high half");
    let Some(mut reading) = runner.acts.remove(&prefix) else { return };
    if let Some(document) = reading.document.take() {
        let called = reading.called;
        let ask = match &page {
            Readout::Jig(c::ReadResult::Value(value)) => match render::read_task(value) {
                Some((task, _)) if subject::routes_through(&called.call, &document, task.chip.task) => {
                    let Some(me) = me(runner, client) else { return };
                    build(
                        &called.call,
                        &document,
                        me,
                        &env.limits,
                        Some(task.project),
                        runner.parts.expect("host vocabulary"),
                    )
                }
                Some(_) | None => Err(Failed::Fixed(b"task project unavailable")),
            },
            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
                Err(Failed::Fixed(b"task project unavailable"))
            }
        };
        prepared(runner, client, env, called, ask, out);
        return;
    }
    if let Readout::Jig(c::ReadResult::Value(document)) = &page
        && let Ok(subject::Scope::Task(task)) = subject::scope(&reading.called.call, document)
    {
        out.push(Out::Client { event: Event::Read { handle, read: Readable::Jig(c::Read::Task { task }) } });
        reading.document = Some(document.clone());
        runner.acts.insert(prefix, reading);
        return;
    }
    let called = reading.called;
    let Some(me) = me(runner, client) else {
        let printed = render::failure(b"signed out", called.json, 1);
        finish(called, printed, out);
        return;
    };
    if called.call.tool == rows::AMEND_AT
        && let Readout::Jig(c::ReadResult::Value(document)) = &page
        && let Some((task, _)) = render::read_task(document)
        && task.chip.task == number(&called.call, 0).expect("task")
    {
        out.push(Out::DraftAmendment { called: Box::new(called), task: Box::new(task.clone()) });
        return;
    }
    let ask = match &page {
        Readout::Jig(c::ReadResult::Value(document)) => {
            build(&called.call, document, me, &env.limits, None, runner.parts.expect("host vocabulary"))
        }
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
            let printed = render::read(
                &page,
                client,
                runner.me(),
                called.json,
                &env.limits,
                runner.parts.expect("host vocabulary"),
            );
            finish(called, printed, out);
            return;
        }
    };
    prepared(runner, client, env, called, ask, out);
}
fn settings_history(call: &Call, document: &c::ReadValue) -> bool {
    if call.tool == rows::DECIDE_AT
        && let c::ReadValue::Ended(history) = document
        && let Some(Value::Proposal(key)) = value(call, 0)
        && render::ended_proposal(history) == Some(*key)
        && let c::EndedValue::Proposal(proposal) = &history.value
        && match proposal.action {
            c::RetainedAction::Settings(_) => true,
            c::RetainedAction::Batch(_)
            | c::RetainedAction::Effect { .. }
            | c::RetainedAction::ProcedureEffect { .. }
            | c::RetainedAction::Amend { .. }
            | c::RetainedAction::Widen { .. }
            | c::RetainedAction::Release { .. } => false,
        }
    {
        return true;
    }
    false
}

fn settings_action(action: &c::ProposalAction) -> bool {
    match action {
        c::ProposalAction::Settings(_) => true,
        c::ProposalAction::Batch(_)
        | c::ProposalAction::Effect { .. }
        | c::ProposalAction::ProcedureEffect { .. }
        | c::ProposalAction::Amend { .. }
        | c::ProposalAction::Widen { .. }
        | c::ProposalAction::Release { .. } => false,
    }
}

fn build(
    call: &Call,
    document: &c::ReadValue,
    me: &c::Me,
    limits: &Limits,
    project: Option<u32>,
    vocabulary: &dyn render::Parts,
) -> Prepared {
    if call.tool == rows::AMEND_AT {
        return Err(Failed::Fixed(b"unexpected amendment document"));
    }
    if call.tool == rows::CLOSE_AT {
        return close_request(call, document, me, limits);
    }
    if settings_history(call, document) {
        return Ok((Request::Ask(c::Ask::Decide { project: 0, on: decide(call, 0, limits, true)? }), None));
    }
    let project = match subject::scope(call, document)? {
        subject::Scope::Project(project) => project,
        subject::Scope::Task(_) => project.ok_or(Failed::Fixed(b"project unavailable"))?,
    };
    if call.tool == rows::RELEASE_AT
        && value(call, 2).is_some()
        && let Some((task, _)) = render::read_task(document)
        && let Some(why) = vocabulary.release_again_refusal(task)
    {
        return Err(Failed::Refused(why));
    }
    let offers = subject::offers(document);
    let act = kind(call);
    let mut admission = None;
    if let Some(offers) = offers {
        for offer in offers {
            if offer.act == act {
                admission = Some(&offer.admission);
            }
        }
    }
    let to = match admission {
        Some(c::Admission::Proposed { to }) => Some(to.clone()),
        Some(c::Admission::Bounded { .. }) => return Err(Failed::Fixed(b"act has no bounded draft")),
        // A replay must reach the engine even when its subject no longer
        // offers this act. The engine alone refuses a fresh key.
        Some(c::Admission::Allowed | c::Admission::Refused(_)) | None => None,
    };
    let settings =
        if let c::ReadValue::Proposal(proposal) = document { settings_action(&proposal.action) } else { false };
    let task = number(call, 0).unwrap_or(0);
    let ask = match call.tool {
        rows::MESSAGE_AT => c::Ask::Message { project, task, words: words(call, 1, limits)? },
        rows::ANSWER_AT => c::Ask::Answer { project, on: answer(call, limits)? },
        rows::TAKE_AT => c::Ask::Take { project, task },
        rows::HAND_BACK_AT => c::Ask::HandBack { project, task },
        rows::DECIDE_AT | rows::TASK_DECIDE_AT => c::Ask::Decide { project, on: decide(call, task, limits, settings)? },
        rows::RELEASE_AT => c::Ask::Release {
            project,
            task,
            revision: c::Revision { value: number(call, 1).expect("hold") },
            again: if value(call, 2).is_some() { Some(true) } else { None },
        },
        rows::STOP_AT => c::Ask::Stop { project, task },
        rows::CANCEL_AT => c::Ask::Cancel {
            project,
            task,
            reason: words(call, 2, limits)?,
            grace: match value(call, 1) {
                Some(Value::Duration(duration)) => Some(*duration),
                _ => None,
            },
        },
        _ => unreachable!("act tool"),
    };
    let phase = subject::ended(document);
    Ok((Request::Act { ask, phase, asked: asked(call, document) }, to))
}
/// Past what the read contract asks, the refusal names the bound (client/ctl/areas.md).
fn asked(call: &Call, document: &c::ReadValue) -> Option<render::outcome::Asked> {
    if call.tool != rows::ANSWER_AT {
        return None;
    }
    let c::ReadValue::PersonTask(task) = document else { return None };
    let (bound, choice, index) = match call.form {
        1 => {
            let c::TaskContractDocument::Verdict { choices, .. } = &task.contract else { return None };
            let code = number(call, 2)?;
            let mut bound = None;
            for choice in choices {
                if u64::from(choice.code) == code {
                    bound = Some((choice.words, Some(choice.code), 3));
                    break;
                }
            }
            bound?
        }
        2 => {
            let c::TaskContractDocument::Report { words } = &task.contract else { return None };
            (*words, None, 2)
        }
        _ => return None,
    };
    let Some(Value::Text(text)) = value(call, index) else { return None };
    if text.len() > skein_lib::length::usize_of(bound) { Some(render::outcome::Asked { bound, choice }) } else { None }
}
fn answer(call: &Call, limits: &Limits) -> Result<c::AnswerOn, Failed> {
    let task = number(call, 0).expect("task");
    if call.form == 0 {
        return Ok(c::AnswerOn::Question {
            task,
            message: number(call, 1).expect("question"),
            words: words(call, 2, limits)?,
        });
    }
    let answer = match call.form {
        1 => c::PersonAnswer::Choice {
            option: u32::try_from(number(call, 2).expect("choice")).or(Err(Failed::Fixed(b"choice code limit")))?,
            reason: words(call, 3, limits)?,
        },
        2 => c::PersonAnswer::Words(words(call, 2, limits)?),
        3 => c::PersonAnswer::Approved {
            state: crate::jig::argv::approval_state(call).ok_or(Failed::Fixed(b"approval state not loaded"))?,
            words: if value(call, 3).is_some() { Some(words(call, 3, limits)?) } else { None },
        },
        4 => c::PersonAnswer::Cannot { reason: words(call, 2, limits)? },
        _ => unreachable!("answer form"),
    };
    Ok(c::AnswerOn::Person { task, revision: c::Revision { value: number(call, 1).expect("revision") }, answer })
}
fn decide(call: &Call, task: u64, limits: &Limits, settings: bool) -> Result<c::DecideOn, Failed> {
    Ok(if call.tool == rows::DECIDE_AT {
        let Some(Value::Proposal(proposal)) = value(call, 0) else { unreachable!("proposal form") };
        let decision = decision(call, limits)?;
        let running = match value(call, 3) {
            Some(Value::Word(0)) => Some(c::Running::Stays),
            Some(Value::Word(1)) => Some(c::Running::Moves),
            None if settings && decision == c::Decision::Accepted => Some(c::Running::Stays),
            Some(_) | None => None,
        };
        c::DecideOn::Proposal { proposal: *proposal, decision, running }
    } else {
        c::DecideOn::Escalation {
            task,
            revision: c::Revision { value: number(call, 1).expect("hold") },
            decision: decision(call, limits)?,
        }
    })
}

fn chip_project(chip: &c::Chip) -> u32 {
    chip.details.details.project
}
fn kind(call: &Call) -> c::ActKind {
    match call.tool {
        rows::MESSAGE_AT => c::ActKind::Message,
        rows::CLOSE_AT | rows::CANCEL_AT => c::ActKind::Cancel,
        rows::ANSWER_AT => c::ActKind::Answer,
        rows::TAKE_AT => c::ActKind::Take,
        rows::HAND_BACK_AT => c::ActKind::HandBack,
        rows::DECIDE_AT | rows::TASK_DECIDE_AT => match value(call, if call.tool == rows::DECIDE_AT { 1 } else { 2 }) {
            Some(Value::Word(0)) => {
                if call.tool == rows::DECIDE_AT {
                    c::ActKind::Accept
                } else {
                    c::ActKind::Release
                }
            }
            Some(Value::Word(1)) => {
                if call.tool == rows::DECIDE_AT {
                    c::ActKind::Reject
                } else {
                    c::ActKind::LeaveHeld
                }
            }
            _ => c::ActKind::PassUp,
        },
        rows::RELEASE_AT => c::ActKind::Release,
        rows::STOP_AT => c::ActKind::Stop,
        _ => unreachable!("act tool"),
    }
}
fn decision(call: &Call, limits: &Limits) -> Result<c::Decision, Failed> {
    let index = if call.tool == rows::DECIDE_AT { 1 } else { 2 };
    let reason = words(call, if call.tool == rows::DECIDE_AT { 2 } else { 3 }, limits)?;
    Ok(match value(call, index) {
        Some(Value::Word(0)) => {
            if call.tool == rows::DECIDE_AT {
                c::Decision::Accepted
            } else {
                c::Decision::Released { again: None }
            }
        }
        Some(Value::Word(1)) => {
            if call.tool == rows::DECIDE_AT {
                c::Decision::Rejected { reason }
            } else {
                c::Decision::LeftHeld { reason }
            }
        }
        _ => c::Decision::PassedUp,
    })
}
pub(crate) enum Request {
    MoveOut { holder: c::AskAccountHolder, account: u32 },
    Written { holder: c::AskAccountHolder, account: u32, number: u32 },
    MoveIn,
    Ask(c::Ask),
    Amend { ask: c::Ask, alternate: Option<c::Ask> },
    Act { ask: c::Ask, phase: Option<c::Phase>, asked: Option<render::outcome::Asked> },
    Account { ask: c::Ask, context: render::outcome::Account },
    AddKey { holder: c::AskAccountHolder, provider: u32, join: Option<u32> },
    ReplaceKey { holder: c::AskAccountHolder, source: u32 },
    DeclareProgram { holder: c::AskAccountHolder, provider: u32, join: Option<u32>, name: Box<[u8]> },
    CheckProgram { holder: c::AskAccountHolder, source: u32 },
}
pub(crate) fn prepared(
    runner: &mut Runner,
    client: &Client,
    _env: &Env<Limits>,
    called: Called,
    request: Prepared,
    out: &mut Queue<Out>,
) {
    let (request, to) = match request {
        Ok(request) => request,
        Err(why) => {
            let printed = match why {
                Failed::Refused(why) => render::failure(why, called.json, 2),
                Failed::Fixed(why) => render::failure(why, called.json, 1),
                Failed::Detail(why) => render::failure(&why, called.json, 1),
                Failed::Unknown { what, name } => choice::named(b"unknown ", what, &name, called.json),
                Failed::NotOffered { name } => choice::named(b"model not offered", b"", &name, called.json),
                Failed::SubAgentNamedTwice { name } => choice::named(b"sub-agent named twice", b"", &name, called.json),
            };
            finish(called, printed, out);
            return;
        }
    };
    send(runner, client, called, request, to, out);
}
struct Context {
    request: Request,
    account: Option<render::outcome::Account>,
    ended_task: Option<c::Phase>,
    asked: Option<render::outcome::Asked>,
}
fn context(request: Request) -> Context {
    match request {
        Request::Act { ask, phase, asked } => {
            Context { request: Request::Ask(ask), account: None, ended_task: phase, asked }
        }
        Request::Account { ask, context } => {
            Context { request: Request::Ask(ask), account: Some(context), ended_task: None, asked: None }
        }
        request @ (Request::MoveOut { .. }
        | Request::Written { .. }
        | Request::MoveIn
        | Request::Ask(_)
        | Request::Amend { .. }
        | Request::AddKey { .. }
        | Request::ReplaceKey { .. }
        | Request::DeclareProgram { .. }
        | Request::CheckProgram { .. }) => Context { request, account: None, ended_task: None, asked: None },
    }
}
fn amendment_request(request: Request) -> (Request, Option<c::Ask>, bool) {
    match request {
        Request::Amend { ask, alternate } => {
            let retry = alternate.is_none();
            (Request::Ask(ask), alternate, retry)
        }
        request @ (Request::MoveOut { .. }
        | Request::Written { .. }
        | Request::MoveIn
        | Request::Ask(_)
        | Request::Act { .. }
        | Request::Account { .. }
        | Request::AddKey { .. }
        | Request::ReplaceKey { .. }
        | Request::DeclareProgram { .. }
        | Request::CheckProgram { .. }) => (request, None, false),
    }
}

fn send(
    runner: &mut Runner,
    client: &Client,
    mut called: Called,
    request: Request,
    to: Option<c::Recipient>,
    out: &mut Queue<Out>,
) {
    let (request, alternate, retry) = amendment_request(request);
    let mut key = crate::jig::key::local(runner.asker, called.call_number);
    if retry {
        key.number = (u64::from(called.call_number) << 32u32) | 2;
    }
    if client.pending(key).is_some() {
        let printed = render::failure(b"tools' bug; client key collision; report it as a finding", called.json, 1);
        finish(called, printed, out);
        return;
    }
    draw_name(runner, &mut called);
    let name = match &called.call.key {
        Some(name) => Some(crate::jig::key::named(runner.salt, name)),
        None => None,
    };
    let Context { request, account, ended_task, asked } = context(request);
    let mut holder = None;
    let mut exported = None;
    let event = match request {
        Request::MoveOut { holder: h, account } => {
            holder = Some(h);
            jig_client::accounts::move_out(key, name.expect("move name"), h, account)
        }
        Request::Written { holder, account, number } => {
            exported = Some((account, number));
            jig_client::accounts::written(key, name.expect("written name"), holder, number)
        }
        Request::MoveIn => jig_client::accounts::move_in(
            key,
            name.expect("import name"),
            c::AskAccountHolder::Own,
            called.value.take().expect("record supplied"),
        ),
        Request::Account { .. } | Request::Act { .. } | Request::Amend { .. } => {
            unreachable!("request context extracted")
        }
        Request::DeclareProgram { holder, provider, join, name: program } => {
            jig_client::accounts::declare_program(key, name.expect("declaration name"), holder, provider, join, program)
        }
        Request::CheckProgram { holder, source } => {
            jig_client::accounts::check_program(key, name.expect("check name"), holder, source)
        }
        Request::Ask(ask) => Event::Send { key, name, request: jig_client::Ask::Jig(Box::new(ask)) },
        Request::AddKey { holder, provider, join } => jig_client::accounts::add_key(
            key,
            name.expect("key request name"),
            holder,
            provider,
            join,
            called.value.take().expect("admitted value"),
        ),
        Request::ReplaceKey { holder, source } => jig_client::accounts::replace_key(
            key,
            name.expect("key request name"),
            holder,
            source,
            called.value.take().expect("admitted value"),
        ),
    };
    let (Event::Send { request, .. } | Event::Carry { request, .. }) = &event else { unreachable!("act event") };
    let mut operation = Operation::for_call(request, render::outcome::act_tool(called.call.tool));
    operation.to = to;
    operation.exported = exported;
    if called.call.tool == rows::EXPORT_AT {
        let Some(Value::File(file)) = value(&called.call, 1) else { unreachable!("export file") };
        operation.file = Some(file.clone());
    }
    operation.account = account;
    operation.ended_task = ended_task;
    operation.asked = asked;
    if called.call.tool != rows::EXPORT_AT
        && called.call.tool != rows::CHANGE_SETTINGS_AT
        && called.call.tool != rows::PROPOSE_SETTINGS_AT
    {
        operation.retry = Some(render::outcome::retry(&called.call));
    }
    if called.call.tool == rows::ADD_KEY_AT
        || called.call.tool == rows::REPLACE_KEY_AT
        || called.call.tool == rows::IMPORT_AT
    {
        operation.value_name.clone_from(&called.call.key);
    }
    if let Some(me) = me(runner, client) {
        operation.party = Some(me.party.number);
        operation.removed_project_name = removal_name(&called.call, me);
    }
    operation.word = operation_word(called.call.tool, operation.word);
    if (rows::CONNECT_AT..=rows::RETRY_AT).contains(&called.call.tool)
        || called.call.tool == rows::DECLARE_AT
        || called.call.tool == rows::VERIFY_AT
        || called.call.tool == rows::EXPORT_AT
        || called.call.tool == rows::IMPORT_AT
    {
        // Printed operation words stay unchanged until the calls-rendering task.
        operation.word = render::outcome::account_word(called.call.tool);
    }
    runner.sent.insert(key, Sent { called, operation, holder, dispatched: false, alternate });
    out.push(Out::Client { event });
}
fn removal_name(call: &Call, me: &c::Me) -> Option<c::DocumentText> {
    if call.tool != rows::PROJECT_DELETE_AT {
        return None;
    }
    let project = super::reads::project(call, me).expect("removal project resolved");
    for membership in &me.projects {
        if membership.project == project {
            return Some(membership.name.clone());
        }
    }
    None
}
pub(super) fn answered(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    key: Key,
    outcome: &jig_client::Outcome,
    out: &mut Queue<Out>,
) {
    if let Some(mut sent) = runner.sent.remove(&key) {
        if amendment_conflict(outcome)
            && let Some(alternate) = sent.alternate.take()
        {
            send(runner, client, sent.called, Request::Amend { ask: alternate, alternate: None }, None, out);
            return;
        }
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::Proposed { to, .. },
        )))) = outcome
        {
            let Some(project) = sent.operation.proposal_project else {
                let printed = render::failure(
                    b"tools' bug; proposal answer lacks request scope; report it as a finding",
                    sent.called.json,
                    1,
                );
                finish(sent.called, printed, out);
                return;
            };
            sent.operation.to = Some(c::Recipient::Role { project, role: *to });
        }
        if sent.called.call.tool == rows::EXPORT_AT {
            super::moves::answered(runner, client, env, sent, outcome, out);
            return;
        }
        if sent.called.call.tool == rows::PROJECT_FUND_AT
            && let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::PolicyChanged { project },
            )))) = outcome
        {
            // Read the committed project once, resolving names to the answered identity.
            *sent.called.call.values.get_mut(0).expect("project argument") =
                Some(Values::One(Value::Project(crate::jig::call::Project::Number(*project))));
            super::reads::start(runner, client, env, sent.called, out);
            return;
        }
        if sent.called.call.tool == rows::PROJECT_AMEND_AT
            && let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::VisibilitySet { project, public },
            )))) = outcome
        {
            visibility_complete(
                runner,
                client,
                Visibility { called: sent.called, project: *project, public: *public },
                out,
            );
            return;
        }
        let mut printed = render::outcome::print(
            &sent.operation,
            outcome,
            sent.called.json,
            me(runner, client),
            &env.limits,
            runner.parts.expect("host vocabulary").diagnostic_prefix(),
        );
        if sent.called.call.tool == rows::IMPORT_AT
            && let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountMovedIn { .. },
            )))) = outcome
        {
            printed.file = super::FileEnd::Remove;
        }
        finish(sent.called, printed, out);
    } else if let Some(operation) = runner.kept.remove(&key) {
        let line = render::outcome::kept(
            &operation,
            outcome,
            runner.kept_json,
            me(runner, client),
            &env.limits,
            runner.parts.expect("host vocabulary").diagnostic_prefix(),
        );
        out.push(Out::Kept { line });
    }
}

fn amendment_conflict(outcome: &jig_client::Outcome) -> bool {
    if let jig_client::Outcome::Held = outcome {
        return true;
    }
    if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::KeyConflict,
        ..
    })) = outcome
    {
        return true;
    }
    false
}

fn close_request(call: &Call, document: &c::ReadValue, me: &c::Me, limits: &Limits) -> Prepared {
    let (task, _) = render::read_task(document).ok_or(Failed::Fixed(b"unexpected close document"))?;
    if task.chip.task != number(call, 0).expect("chat argument") {
        return Err(Failed::Fixed(b"unexpected close task"));
    }
    // Only immutable routing scope enters the ask. Terminal history is kept
    // as answer context: an archived task can be Unknown to a fresh act.
    let phase = subject::ended(document);
    Ok((Request::Act { ask: close(call, me, task.project, limits)?, phase, asked: None }, None))
}

fn close(call: &Call, me: &c::Me, project: u32, limits: &Limits) -> Result<c::Ask, Failed> {
    let moves = match call.values.get(1) {
        Some(Some(Values::Many(moves))) => Some(moves),
        _ => None,
    };
    let count = match moves {
        Some(moves) => u32::try_from(moves.len()).expect("bounded moves"),
        None => 0,
    };
    let mut delegates = List::with_capacity(count);
    if let Some(moves) = moves {
        for moved in moves {
            let Value::Number(task) = moved else { unreachable!("move argument") };
            delegates.push(shapes::Delegate { task: *task, fate: shapes::Fate::Move }).expect("one per move");
        }
    }
    // Cancelling the chat cancels its remaining tree in the engine. Only
    // explicit moves precede it, so changing delegates cannot change the ask.
    match shapes::close_chat(
        me,
        project,
        number(call, 0).expect("chat argument"),
        delegates.as_slice(),
        words(call, 2, limits)?,
    ) {
        Ok(ask) => Ok(ask),
        Err(_) => Err(Failed::Fixed(b"close not offered")),
    }
}

fn draw_name(runner: &mut Runner, called: &mut Called) {
    // Tool 33 draws a key because check_program requires one even when verify omits --key.
    if (called.value.is_some() || called.call.tool == rows::VERIFY_AT || called.call.tool == rows::EXPORT_AT)
        && called.call.key.is_none()
    {
        let mut bytes = [0; 32];
        for chunk in bytes.chunks_mut(16) {
            let hex = crate::jig::digest::Digest::from_bytes(runner.names.next_u64().to_le_bytes()).hex();
            for (slot, byte) in chunk.iter_mut().zip(hex) {
                *slot = byte;
            }
        }
        called.call.key = Some(crate::jig::key::Name::new(&bytes).expect("drawn hexadecimal name"));
    }
}

fn operation_word(tool: u8, word: &'static [u8]) -> &'static [u8] {
    match tool {
        rows::GOAL_START_AT => b"goal",
        rows::CLOSE_AT => b"close",
        _ => word,
    }
}

fn visibility_complete(runner: &mut Runner, client: &Client, waiting: Visibility, out: &mut Queue<Out>) {
    if let Some(me) = me(runner, client) {
        for project in &me.projects {
            if project.project == waiting.project && project.public.is_some() == waiting.public {
                let printed = render::visibility::print(waiting.project, project.public, waiting.called.json);
                finish(waiting.called, printed, out);
                return;
            }
        }
    }
    runner.visibility.insert(waiting.called.call_number, waiting);
}
pub(super) fn visibility_changed(runner: &mut Runner, client: &Client, out: &mut Queue<Out>) {
    // Visit each waiting call once, including those still awaiting agreement.
    let waiting = runner.visibility.split_off(&0);
    for (_, waiting) in waiting {
        visibility_complete(runner, client, waiting, out);
    }
}
