//! One call's bounded read set, with watches closed at their first delivery.
use super::{Called, Out, Printed, Runner};
use crate::jig::{
    Limits,
    call::{Call, Project, Value, cursor, number, value},
    render, rows,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{
    Client, Event, Handle, Latest, Readable, Readout, Watchable, Why, contract as c,
    thread::{self, Thread},
};
use skein_lib::{Env, Queue, Writer};
#[derive(Debug)]
pub(crate) struct Reading {
    pub called: Called,
    pub command_prefix: &'static [u8],
    pub pending: u8,
    pub pages: BTreeMap<u32, Readout>,
    pub documents: BTreeMap<u32, c::Delivery>,
    pub text: Option<c::RunText>,
    pub project: Option<u32>,
    pub specification: Option<super::specification::Specification>,
    pub thread: Option<Thread>,
    pub talk: Option<super::talk::Reply>,
    pub questions: BTreeMap<(u64, u64), Option<Box<c::Question>>>,
    pub question: Option<(u64, u64)>,
    pub person: Option<Box<c::PersonTask>>,
    pub person_answerer: Option<u64>,
    pub proposals: BTreeMap<c::ProposalKey, Option<c::DocumentText>>,
    pub proposal: Option<c::ProposalKey>,
    pub met_inbox: Option<Printed>,
    pub parts: &'static dyn render::Parts,
    pub application: super::application::Reading,
}
pub(super) fn handle(runner: &Runner, prefix: u32, local: u32) -> Handle {
    Handle { asker: runner.asker, number: (u64::from(prefix) << 32u32) | u64::from(local) }
}
fn read(runner: &Runner, reading: &mut Reading, local: u32, what: c::Read, out: &mut Queue<Out>) {
    // Supplemental review reads (slots 8 and up) run one at a time and share bit 7.
    reading.pending |= 1u8.checked_shl(local.min(7)).expect("fixed read slot");
    out.push(Out::Client {
        event: Event::Read { handle: handle(runner, reading.called.call_number, local), read: Readable::Jig(what) },
    });
}
fn watch(runner: &Runner, reading: &mut Reading, local: u32, subject: c::Subject, out: &mut Queue<Out>) {
    reading.pending |= 1u8.checked_shl(local).expect("fixed watch slot");
    out.push(Out::Client {
        event: Event::Watch {
            handle: handle(runner, reading.called.call_number, local),
            subject: Watchable::Jig(subject),
        },
    });
}
pub(super) fn start(runner: &mut Runner, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    start_reading(runner, client, env, called, None, out);
}

// A met owner inbox wait uses the plain read, retaining its delivery on failure.
pub(crate) fn met_inbox(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    mut called: Called,
    fallback: Printed,
    out: &mut Queue<Out>,
) {
    // The inbox form ends with since, until and timeout.
    for value in called.call.values.iter_mut().skip(4) {
        *value = None;
    }
    start_reading(runner, client, env, called, Some(After::Inbox(fallback)), out);
}

enum After {
    Inbox(Printed),
    Talk(super::talk::Reply),
}
pub(crate) fn talk_reply(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    called: Called,
    reply: super::talk::Reply,
    out: &mut Queue<Out>,
) {
    start_reading(runner, client, env, called, Some(After::Talk(reply)), out);
}
fn start_reading(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    called: Called,
    after: Option<After>,
    out: &mut Queue<Out>,
) {
    let (met_inbox, talk) = match after {
        Some(After::Inbox(fallback)) => (Some(fallback), None),
        Some(After::Talk(reply)) => (None, Some(reply)),
        None => (None, None),
    };
    let reading = Reading {
        called,
        command_prefix: runner.parts.expect("host vocabulary").command_prefix(),
        pending: 0,
        pages: BTreeMap::new(),
        documents: BTreeMap::new(),
        text: None,
        project: None,
        specification: None,
        thread: None,
        talk,
        questions: BTreeMap::new(),
        question: None,
        person: None,
        person_answerer: None,
        proposals: BTreeMap::new(),
        proposal: None,
        met_inbox,
        application: super::application::Reading::new(),
        parts: runner.parts.expect("host vocabulary"),
    };
    if reading.called.call.tool <= 3 {
        start_overview(runner, client, env, reading, out);
    } else {
        start_detail(runner, client, env, reading, out);
    }
}

fn start_overview(runner: &mut Runner, client: &Client, env: &Env<Limits>, mut reading: Reading, out: &mut Queue<Out>) {
    let task = number(&reading.called.call, 0).unwrap_or(0);
    match reading.called.call.tool {
        1 => {
            if !inbox(runner, client, &mut reading, out) {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            }
            if render::document::subjects::inbox_eligible(&reading.called.call) {
                // Pages may omit updates the watch holds, or carry a group
                // continuation that is not the window's `more` marker.
                watch(runner, &mut reading, 3, c::Subject::Inbox { window: env.limits.rows }, out);
            }
        }
        2 => {
            let Some(document) = client.document(runner.me()) else { return };
            let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest else { return };
            let Some(_) = project(&reading.called.call, me) else {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            };
            let call = &reading.called.call;
            let ended = value(call, 4).is_some();
            let query = task_query(call, me).expect("resolved project");
            let after = cursor(call, 6);
            let (live, ended) = if ended { (None, after) } else { (after, None) };
            read(runner, &mut reading, 1, c::Read::Tasks { query, live, ended }, out);
        }
        3 => {
            read(runner, &mut reading, 1, c::Read::Task { task }, out);
            if value(&reading.called.call, 1).is_some() {
                watch(runner, &mut reading, 3, c::Subject::Tree { task, window: env.limits.rows }, out);
            }
            if number(&reading.called.call, 2).is_some() {
                read(runner, &mut reading, 2, c::Read::Activity { task, before: None }, out);
            }
            if value(&reading.called.call, 3).is_some() {
                watch(runner, &mut reading, 4, c::Subject::Run { task, turn: None, offset: 0 }, out);
            }
        }
        _ => unreachable!("overview read"),
    }
    runner.active.insert(reading.called.call_number, reading);
}

fn start_detail(runner: &mut Runner, client: &Client, env: &Env<Limits>, mut reading: Reading, out: &mut Queue<Out>) {
    let task = number(&reading.called.call, 0).unwrap_or(0);
    match reading.called.call.tool {
        4 | rows::ROLE_TALK_AT => thread_start(runner, env, &mut reading, task, out),
        5 => {
            let before = cursor(&reading.called.call, 1);
            read(runner, &mut reading, 1, c::Read::Activity { task, before }, out);
        }
        rows::ROLE_LIST_AT => {
            let Some(me) = super::acts::me(runner, client) else { return };
            let Some(project) = project(&reading.called.call, me) else {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            };
            let query = c::TaskQuery {
                project: Some(project),
                requester: None,
                root: None,
                parent: None,
                label: Some(3),
                phase: None,
                executor: None,
                tracked: None,
                order: c::QueryOrder::Number,
            };
            let live = cursor(&reading.called.call, 1);
            read(runner, &mut reading, 1, c::Read::Tasks { query, live, ended: None }, out);
        }
        rows::ROLE_SHOW_AT => read(runner, &mut reading, 1, c::Read::Task { task }, out),
        rows::TASK_TEXT_AT => {
            let what = task_text_read(&reading.called.call);
            read(runner, &mut reading, 1, what, out);
        }
        rows::TASK_NOTES_AT => {
            let Some(me) = super::acts::me(runner, client) else { return };
            let call = &reading.called.call;
            let Some(project) = selected_project(value(call, 1), me) else {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            };
            let what = c::Read::Notes {
                scope: c::NoteScope::Task { project, task },
                description: None,
                after: cursor(call, 2),
            };
            read(runner, &mut reading, 1, what, out);
        }
        6 => {
            let Some(Value::Turn(name)) = value(&reading.called.call, 1) else { return };
            let what = c::Read::Turn { task, attempt: name.attempt, turn: name.number };
            read(runner, &mut reading, 1, what, out);
        }
        22 | 23 | 25 | 27..=30 | 33 | rows::EXPORT_AT => {
            let Some(document) = client.document(runner.me()) else { return };
            let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest else { return };
            if reading.called.call.tool == 23 && model_project(&reading.called.call, me).is_err() {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            }
            watch(runner, &mut reading, 3, c::Subject::Accounts, out);
            if me.deployment_role == Some(c::HeldRole::Owner) {
                watch(runner, &mut reading, 4, c::Subject::Deployment, out);
            }
        }
        rows::PROJECT_SHOW_AT | rows::PROJECT_FUND_AT => {
            let Some(me) = super::acts::me(runner, client) else { return };
            let Some(project) = project(&reading.called.call, me) else {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            };
            reading.project = Some(project);
            read(runner, &mut reading, 1, c::Read::Project { project }, out);
        }
        rows::PROPOSAL_AT => {
            let Some(Value::Proposal(proposal)) = value(&reading.called.call, 0) else { unreachable!("proposal form") };
            let what = c::Read::Proposal { proposal: *proposal };
            read(runner, &mut reading, 1, what, out);
        }
        rows::SETTINGS_AT..=rows::REVISIONS_AT => {
            if !reading.parts.prepare_setting(&mut reading.called.call, super::acts::me(runner, client)) {
                let printed = render::failure(b"unknown project", reading.called.json, 1);
                finish(reading, printed, out);
                return;
            }
            settings_start(runner, &mut reading, out);
        }
        rows::TREE_AT => watch(runner, &mut reading, 3, c::Subject::Tree { task, window: env.limits.rows }, out),
        7 => watch(runner, &mut reading, 4, c::Subject::Run { task, turn: None, offset: 0 }, out),
        _ => unreachable!("read tool"),
    }
    runner.active.insert(reading.called.call_number, reading);
}

fn task_text_read(call: &Call) -> c::Read {
    let field = match value(call, 1) {
        Some(Value::Word(1)) => c::TaskTextField::Instructions,
        Some(Value::Word(2)) => c::TaskTextField::Result,
        _ => c::TaskTextField::Specification,
    };
    c::Read::TaskText { task: number(call, 0).unwrap_or(0), field, from: number(call, 2).unwrap_or(0) }
}

fn thread_start(runner: &Runner, env: &Env<Limits>, reading: &mut Reading, task: u64, out: &mut Queue<Out>) {
    let mut thread = Thread::new(
        task,
        &thread::Limits {
            lines: env.limits.rows,
            bytes: env.limits.ask_bytes,
            text: env.limits.text,
            live: env.limits.text,
        },
    );
    let next = match if reading.talk.is_some() { None } else { cursor(&reading.called.call, 1) } {
        Some(cursor) => {
            thread.lines.edges.after = thread::window::Edge::More(cursor);
            thread.older()
        }
        None => thread.moved(1),
    };
    if let Some(next) = next {
        read(runner, reading, 1, next, out);
    }
    reading.thread = Some(thread);
    if reading.talk.is_none() {
        watch(runner, reading, 4, c::Subject::Run { task, turn: None, offset: 0 }, out);
    }
}
pub(crate) fn project(call: &Call, me: &c::Me) -> Option<u32> {
    selected_project(value(call, 0), me)
}
fn selected_project(selected: Option<&Value>, me: &c::Me) -> Option<u32> {
    match selected {
        Some(Value::Project(Project::Number(number))) => Some(*number),
        Some(Value::Project(Project::Name(name))) => {
            for project in &me.projects {
                if project.name.bytes() == name.as_ref() {
                    return Some(project.project);
                }
            }
            None
        }
        _ => None,
    }
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
    let local = u32::try_from(handle.number & u64::from(u32::MAX)).expect("low half");
    let Some(mut reading) = runner.active.remove(&prefix) else { return };
    if local == super::application::SLOT && reading.application.query.is_some() {
        application_page(runner, client, env, reading, page, out);
        return;
    }
    // Joined setting reads share the last bit; inbox question details use slot 6.
    reading.pending &= !1u8.checked_shl(local.min(7)).expect("fixed read slot");
    if reading.called.call.tool == rows::REVISION_AT && local == 8 {
        super::application::history_received(&mut reading, &page);
        complete(runner, client, env, reading, out);
        return;
    }
    if reading.called.call.tool == rows::TASK_AT && local == 8 {
        person_page(runner, client, env, reading, page, out);
        return;
    }
    if (reading.called.call.tool == rows::TASK_AT
        || reading.called.call.tool == rows::TASK_TEXT_AT
        || reading.called.call.tool == rows::ROLE_SHOW_AT)
        && local == 7
    {
        super::specification::accept(&mut reading, &page);
        complete(runner, client, env, reading, out);
        return;
    }
    if reading.called.call.tool == 1 && (local == 6 || local == 7) {
        inbox_detail(&mut reading, page, local);
        complete(runner, client, env, reading, out);
        return;
    }
    // An opening Unknown can mean an ended task, but proves no identity or
    // authority. Only the authorized task read may establish that end.
    if local == 6 && (reading.called.call.tool == 3 || reading.called.call.tool == 4) {
        terminal_read(runner, client, env, reading, &page, out);
        return;
    }
    accept_page(runner, client, env, reading, local, page, out);
}
fn accept_page(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    mut reading: Reading,
    local: u32,
    page: Readout,
    out: &mut Queue<Out>,
) {
    if reading.called.call.tool == rows::TASK_AT
        && local == 1
        && let Readout::Jig(c::ReadResult::Value(document)) = &page
        && let Some((task, _)) = render::read_task(document)
        && task.person_revision.is_some()
    {
        read(runner, &mut reading, 8, c::Read::PersonTask { task: task.chip.task }, out);
    }
    if setting_page(runner, &mut reading, local, &page, out) {
        complete(runner, client, env, reading, out);
        return;
    }
    if let Some(reply) = &mut reading.talk {
        let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = &page else {
            let printed = render::read(&page, client, runner.me(), reading.called.json, &env.limits, reading.parts);
            finish(reading, printed, out);
            return;
        };
        match reply.page(reading.thread.as_mut().expect("talk thread"), *page.clone(), &env.limits) {
            Ok(Some(next)) => read(runner, &mut reading, local, next, out),
            Ok(None) => {}
            Err(printed) => {
                finish(reading, printed, out);
                return;
            }
        }
    } else if let Some(thread) = &mut reading.thread
        && let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = &page
    {
        let mut next = thread.accept(*page.clone());
        // The engine's page can be smaller than ctl's conversation window.
        // Read older pages until that bounded window is filled, including
        // when closing activity occupies the newest page.
        if next.is_none() && thread.lines.rows.len() < env.limits.rows {
            next = thread.older();
        }
        if let Some(next) = next {
            read(runner, &mut reading, local, next, out);
        }
    }
    let more_inbox = match &page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) => page.after.is_some(),
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => true,
    };
    reading.pages.insert(local, page);
    if reading.called.call.tool == 1 && local == 1 && !more_inbox && value(&reading.called.call, 1).is_none() {
        let call = &reading.called.call;
        let Some(document) = client.document(runner.me()) else { return };
        let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest else { return };
        let what = c::Read::Inbox {
            group: c::InboxReadGroup::Updates,
            project: project(call, me),
            kind: None,
            goal: number(call, 2),
            after: None,
        };
        read(runner, &mut reading, 5, what, out);
    }
    complete(runner, client, env, reading, out);
}
// An omitted setting revision means the newest authorized inventory revision.
fn setting_page(runner: &Runner, reading: &mut Reading, local: u32, page: &Readout, out: &mut Queue<Out>) -> bool {
    if reading.called.call.tool == rows::SETTING_AT
        && local == 6
        && let Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(settings))) = &page
    {
        let setting = match reading.parts.setting_selector(&reading.called.call) {
            Some(setting) => setting,
            None => super::super::settings_file::selector(&reading.called.call),
        };
        if let c::Setting::Application { .. } = setting {
            super::application::inventory(reading, settings, setting);
            super::application::read(runner, reading, super::application::Query::Setting(setting), out);
            return true;
        }
        let what = c::Read::Setting(c::ReadSetting { setting, revision: settings.newest, proposal: None });
        read(runner, reading, 1, what, out);
        return true;
    }
    false
}
fn person_page(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    mut reading: Reading,
    page: Readout,
    out: &mut Queue<Out>,
) {
    if let Readout::Jig(c::ReadResult::Value(document)) = &page
        && let Some(person) = render::person_task(document)
        && person.task.task == number(&reading.called.call, 0).expect("task")
    {
        reading.person = Some(Box::new(person.clone()));
        reading.person_answerer = render::person_answerer(document);
        complete(runner, client, env, reading, out);
    } else {
        let printed = render::read(
            &page,
            client,
            runner.me(),
            reading.called.json,
            &env.limits,
            runner.parts.expect("host vocabulary"),
        );
        close(runner, &reading, out);
        finish(reading, printed, out);
    }
}
pub(super) fn changed(runner: &mut Runner, client: &Client, env: &Env<Limits>, handle: Handle, out: &mut Queue<Out>) {
    let prefix = u32::try_from(handle.number >> 32u32).expect("high half");
    let local = u32::try_from(handle.number & u64::from(u32::MAX)).expect("low half");
    let Some(mut reading) = runner.active.remove(&prefix) else { return };
    let bit = 1u8.checked_shl(local).expect("fixed watch slot");
    if reading.pending & bit != 0
        && let Some(document) = client.document(handle)
        && (document.state == jig_client::State::Live || document.state == jig_client::State::Ended)
        && let Some(Latest::Jig(delivery)) = document.latest
    {
        reading.pending &= !bit;
        reading.documents.insert(local, delivery.clone());
        if local == 4 {
            reading.text = client.text(handle).cloned();
            if let Some(c::Delivery::RunState(run)) = reading.documents.get_mut(&local)
                && let Some(text) = &reading.text
            {
                run.writing = Some(text.clone());
            }
        }
        out.push(Out::Client { event: Event::Unwatch { handle } });
    }
    complete(runner, client, env, reading, out);
}
pub(super) fn ended(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    handle: Handle,
    why: Why,
    out: &mut Queue<Out>,
) {
    let prefix = u32::try_from(handle.number >> 32u32).expect("high half");
    let local = u32::try_from(handle.number & u64::from(u32::MAX)).expect("low half");
    let Some(mut reading) = runner.active.remove(&prefix) else { return };
    // A first delivery already filled this slot; its closure cannot cancel
    // the other documents still needed by the call.
    if reading.documents.contains_key(&local) {
        runner.active.insert(prefix, reading);
        return;
    }
    // A chat or task read can outlive its live run. Its kept documents
    // still answer the call; the ended run contributes no provisional text.
    if local == 4 && (reading.called.call.tool == 3 || reading.called.call.tool == 4) && why == Why::SubjectEnded {
        reading.pending &= !1u8.checked_shl(local).expect("fixed watch slot");
        complete(runner, client, env, reading, out);
        return;
    }
    if local == 4 && (reading.called.call.tool == 3 || reading.called.call.tool == 4) && why == Why::Unknown {
        reading.pending &= !1u8.checked_shl(local).expect("fixed watch slot");
        let task = number(&reading.called.call, 0).expect("task argument");
        read(runner, &mut reading, 6, c::Read::Task { task }, out);
        complete(runner, client, env, reading, out);
        return;
    }
    let status = match why {
        Why::Unknown | Why::NotAllowed | Why::SubjectEnded => 2,
        Why::NotServed | Why::SignedOut | Why::Limit => 1,
    };
    close(runner, &reading, out);
    let printed = render::failure(b"watch ended", reading.called.json, status);
    finish(reading, printed, out);
}
fn close(runner: &Runner, reading: &Reading, out: &mut Queue<Out>) {
    for local in [3, 4] {
        if reading.pending & 1u8.checked_shl(local).expect("fixed watch slot") != 0 {
            out.push(Out::Client {
                event: Event::Unwatch { handle: handle(runner, reading.called.call_number, local) },
            });
        }
    }
}
fn complete(runner: &mut Runner, client: &Client, env: &Env<Limits>, mut reading: Reading, out: &mut Queue<Out>) {
    if reading.called.call.tool == 1
        && let Some((local, what)) = next_inbox_detail(&mut reading, env.limits.rows)
    {
        read(runner, &mut reading, local, what, out);
        runner.active.insert(reading.called.call_number, reading);
        return;
    }

    if reading.pending != 0 {
        runner.active.insert(reading.called.call_number, reading);
        return;
    }
    if let Some((local, what)) = super::role::next(&reading) {
        read(runner, &mut reading, local, what, out);
        runner.active.insert(reading.called.call_number, reading);
        return;
    }
    if let Some(what) = super::specification::next(&mut reading, &env.limits) {
        read(runner, &mut reading, 7, what, out);
        runner.active.insert(reading.called.call_number, reading);
        return;
    }
    if reading.called.call.tool == rows::PROPOSAL_AT
        && let Some((local, what)) = super::proposal::next(&reading)
    {
        read(runner, &mut reading, local, what, out);
        runner.active.insert(reading.called.call_number, reading);
        return;
    }
    if let Some(what) = super::application::next_history(&mut reading) {
        read(runner, &mut reading, 8, what, out);
        runner.active.insert(reading.called.call_number, reading);
        return;
    }
    if (reading.called.call.tool == rows::PROPOSAL_AT || reading.called.call.tool == rows::REVISION_AT)
        && let Some(query) = super::application::next(&reading)
    {
        super::application::read(runner, &mut reading, query, out);
        runner.active.insert(reading.called.call_number, reading);
        return;
    }
    join_thread(&mut reading);
    if reading.called.call.tool == 25
        || (27..=30).contains(&reading.called.call.tool)
        || reading.called.call.tool == 33
        || reading.called.call.tool == rows::EXPORT_AT
    {
        super::accounts::resolved(runner, client, env, reading, out);
        return;
    }
    let printed = if let Some(reply) = &reading.talk {
        render::roles::reply(reply, reading.called.json, &env.limits)
    } else if reading.called.call.tool == rows::ROLE_SHOW_AT {
        let now = reading.pages.remove(&8);
        let report = reading.called.call.values.get_mut(1).expect("report flag").take();
        reading.called.call.tool = rows::TASK_AT;
        let task =
            render::reads(&reading, client, runner.me(), &env.limits, runner.parts.expect("host vocabulary"), env.wall);
        reading.called.call.tool = rows::ROLE_SHOW_AT;
        *reading.called.call.values.get_mut(1).expect("report flag") = report;
        if let Some(now) = now {
            reading.pages.insert(8, now);
        }
        render::roles::show(task, &reading, super::acts::me(runner, client), &env.limits)
    } else {
        render::reads(&reading, client, runner.me(), &env.limits, runner.parts.expect("host vocabulary"), env.wall)
    };
    finish(reading, printed, out);
}

fn inbox(runner: &Runner, client: &Client, reading: &mut Reading, out: &mut Queue<Out>) -> bool {
    let call = &reading.called.call;
    let kind = match value(call, 1) {
        Some(Value::Group(kind)) => Some(*kind),
        _ => None,
    };
    let project = match client.document(runner.me()) {
        Some(document) => match document.latest {
            Some(Latest::Jig(c::Delivery::Me(me))) => project(call, me),
            _ => None,
        },
        None => None,
    };
    if value(call, 0).is_some() && project.is_none() {
        return false;
    }
    let goal = number(call, 2);
    let after = match value(call, 3) {
        Some(Value::Cursor(bytes)) => Some(bytes.as_ref()),
        _ => None,
    };
    let tag = match after {
        Some(bytes) => bytes.first().copied(),
        None => None,
    };
    let group = match tag {
        Some(1) => c::InboxReadGroup::Updates,
        Some(0) => c::InboxReadGroup::Needs,
        None => match kind {
            Some(kind) => c::InboxReadGroup::of(kind),
            None => c::InboxReadGroup::Needs,
        },
        Some(_) => unreachable!("parsed inbox group"),
    };
    let cursor = match after {
        Some(bytes) => match bytes.get(1..) {
            Some(bytes) if !bytes.is_empty() => Some(c::Cursor::new(bytes.into(), u32::MAX).expect("parsed cursor")),
            Some(_) | None => None,
        },
        None => None,
    };
    let what = c::Read::Inbox { group, project, kind, goal, after: cursor };
    read(runner, reading, if group == c::InboxReadGroup::Needs { 1 } else { 5 }, what, out);
    true
}

pub(crate) fn task_query(call: &Call, me: &c::Me) -> Option<c::TaskQuery> {
    let project = project(call, me)?;
    let ended = value(call, 4).is_some();
    Some(c::TaskQuery {
        project: Some(project),
        requester: if value(call, 3).is_some() { Some(c::Requester::Person(me.party.number)) } else { None },
        root: None,
        parent: number(call, 7),
        label: match value(call, 1) {
            Some(Value::Word(0)) => Some(jig_client::shapes::CHAT),
            Some(Value::Word(1)) => Some(jig_client::shapes::GOAL),
            _ => None,
        },
        phase: if ended { Some(c::PhaseClass::Ended) } else { None },
        executor: None,
        tracked: if value(call, 2).is_some() { Some(true) } else { None },
        order: match value(call, 5) {
            Some(Value::Order(order)) => *order,
            _ => c::QueryOrder::Number,
        },
    })
}

// Numeric project arguments require membership too; omission includes every
// project in the supplied me document, never an engine-wide catalogue.
pub(crate) fn model_project(call: &Call, me: &c::Me) -> Result<Option<u32>, ()> {
    if value(call, 0).is_none() {
        return Ok(None);
    }
    let selected = project(call, me).ok_or(())?;
    for held in &me.projects {
        if held.project == selected {
            return Ok(Some(selected));
        }
    }
    Err(())
}

fn settings_start(runner: &Runner, reading: &mut Reading, out: &mut Queue<Out>) {
    let task = number(&reading.called.call, 0).unwrap_or(0);
    match reading.called.call.tool {
        rows::SETTINGS_AT => read(runner, reading, 1, c::Read::Settings, out),
        rows::SETTING_AT => setting_start(runner, reading, out),
        rows::REVISION_AT => {
            let number = u32::try_from(task).expect("parsed revision");
            read(runner, reading, 1, c::Read::Revision(c::ReadRevision { number }), out);
        }
        rows::REVISIONS_AT => match number(&reading.called.call, 0) {
            Some(n) => {
                let before = Some(u32::try_from(n).expect("parsed revision"));
                read(runner, reading, 1, c::Read::Revisions(c::ReadRevisions { before }), out);
            }
            None => read(runner, reading, 1, c::Read::Revisions(c::ReadRevisions { before: None }), out),
        },
        _ => unreachable!("settings read tool"),
    }
}

fn setting_start(runner: &Runner, reading: &mut Reading, out: &mut Queue<Out>) {
    let call = &reading.called.call;
    let setting = match reading.parts.setting_selector(call) {
        Some(setting) => setting,
        None => super::super::settings_file::selector(call),
    };
    if let c::Setting::Application { .. } = setting {
        if let Some(revision) = number(call, if call.form == 2 || call.form == 7 { 3 } else { 2 }) {
            super::application::historical(
                runner,
                reading,
                setting,
                u32::try_from(revision).expect("parsed revision"),
                out,
            );
        } else {
            read(runner, reading, 6, c::Read::Settings, out);
        }
        return;
    }
    match number(call, 2) {
        Some(n) => {
            let revision = u32::try_from(n).expect("parsed revision");
            read(runner, reading, 1, c::Read::Setting(c::ReadSetting { setting, revision, proposal: None }), out);
        }
        None => read(runner, reading, 6, c::Read::Settings, out),
    }
}

fn edge_cursor(edge: &thread::window::Edge) -> Option<c::Cursor> {
    match edge {
        thread::window::Edge::More(cursor) => Some(cursor.clone()),
        thread::window::Edge::End | thread::window::Edge::Dropped => None,
    }
}

fn terminal_read(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    reading: Reading,
    page: &Readout,
    out: &mut Queue<Out>,
) {
    let expected = number(&reading.called.call, 0).expect("task argument");
    let terminal = match page {
        Readout::Jig(c::ReadResult::Value(value)) => match render::read_task(value) {
            Some((task, _)) => task.chip.task == expected && render::terminal_phase(task.chip.phase),
            None => false,
        },
        Readout::Jig(c::ReadResult::None(_)) | Readout::Limit | Readout::NotServed { .. } | Readout::Family { .. } => {
            false
        }
    };
    if terminal {
        complete(runner, client, env, reading, out);
    } else {
        close(runner, &reading, out);
        let printed = match page {
            Readout::Jig(c::ReadResult::Value(_)) => {
                render::failure(b"watch ended; task is not the requested ended task", reading.called.json, 2)
            }
            Readout::Jig(c::ReadResult::None(_))
            | Readout::Limit
            | Readout::NotServed { .. }
            | Readout::Family { .. } => render::read(
                page,
                client,
                runner.me(),
                reading.called.json,
                &env.limits,
                runner.parts.expect("host vocabulary"),
            ),
        };
        finish(reading, printed, out);
    }
}

fn next_inbox_detail(reading: &mut Reading, maximum: u32) -> Option<(u32, c::Read)> {
    if reading.question.is_some() || reading.proposal.is_some() {
        return None;
    }
    if let Some((task, message)) = next_question(reading, maximum) {
        reading.question = Some((task, message));
        return Some((6, c::Read::Question { task, message }));
    }
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) = reading.pages.get(&1)? else {
        return None;
    };
    for row in page.rows.as_slice().iter().take(usize::try_from(maximum).expect("rows fit")) {
        if let c::InboxObject::Proposal { proposer, number } = row.object {
            let proposal = c::ProposalKey { proposer, number };
            if !reading.proposals.contains_key(&proposal) {
                reading.proposal = Some(proposal);
                return Some((7, c::Read::Proposal { proposal }));
            }
        }
    }
    None
}

fn inbox_detail(reading: &mut Reading, page: Readout, local: u32) {
    if local == 6 {
        question_page(reading, page);
        return;
    }
    let key = reading.proposal.take().expect("one proposal read in flight");
    let title = match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Proposal(proposal))) if proposal.key == key => {
            Some(proposal.reason)
        }
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
    };
    reading.proposals.insert(key, title);
}

// At most the displayed needs page is joined, one read in flight at a time.
fn next_question(reading: &Reading, maximum: u32) -> Option<(u64, u64)> {
    let mut keys = skein_lib::List::with_capacity(maximum);
    match reading.pages.get(&1) {
        Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page)))) => {
            for row in page.rows.as_slice().iter().take(usize::try_from(maximum).expect("rows fit")) {
                if let c::InboxObject::Question { message } = row.object {
                    keys.push((row.task.number(), message)).expect("one bounded page");
                }
            }
        }
        _ => return None,
    }
    for key in &keys {
        if !reading.questions.contains_key(key) {
            return Some(*key);
        }
    }
    None
}

fn finish(reading: Reading, mut printed: Printed, out: &mut Queue<Out>) {
    if let Some(fallback) = reading.met_inbox {
        let prefix = if printed.status == 0 {
            b"".as_slice()
        } else {
            fallback.stdout.strip_suffix(b"met\n").expect("met wait suffix")
        };
        let length = prefix.len().checked_add(printed.stdout.len()).expect("two bounded documents");
        let mut stdout = Writer::new(length.checked_add(b"met\n".len()).expect("met suffix"));
        stdout.put(prefix).expect("measured fallback");
        stdout.put(&printed.stdout).expect("measured read");
        stdout.put(b"met\n").expect("measured suffix");
        printed.stdout = stdout.finish();
        printed.status = fallback.status;
    }
    super::finish(reading.called, printed, out);
}

fn question_page(reading: &mut Reading, page: Readout) {
    let key = reading.question.take().expect("one question read in flight");
    let question = match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(question)))
            if (question.task.task, question.message) == key =>
        {
            Some(question)
        }
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(value))) => match value.value {
            c::EndedValue::Question { question, .. } if (question.task.task, question.message) == key => {
                Some(Box::new(question))
            }
            c::EndedValue::Task { .. }
            | c::EndedValue::Proposal(_)
            | c::EndedValue::PersonProposal { .. }
            | c::EndedValue::Escalation { .. }
            | c::EndedValue::Question { .. }
            | c::EndedValue::PersonTask { .. } => None,
        },
        Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
    };
    reading.questions.insert(key, question);
}

fn join_thread(reading: &mut Reading) {
    if let Some(thread) = &mut reading.thread
        && let Some(c::Delivery::RunState(run)) = reading.documents.get(&4)
    {
        thread.run(run, reading.text.as_ref());
    }
    if let Some(thread) = &reading.thread
        && let Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(_)))) = reading.pages.get(&1)
    {
        // JSON and text expose the same kept window, not just the final
        // transport page used to fill it. Thread rows are newest first.
        let mut rows = skein_lib::List::with_capacity(thread.lines.rows.len());
        for row in thread.lines.rows.as_slice().iter().rev() {
            rows.push(row.clone()).expect("bounded conversation rows");
        }
        reading.pages.insert(
            1,
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                cut: thread.page_cut,
                before: edge_cursor(&thread.lines.edges.after),
                after: edge_cursor(&thread.lines.edges.before),
            })))),
        );
    }
}

fn application_page(
    runner: &mut Runner,
    client: &Client,
    env: &Env<Limits>,
    mut reading: Reading,
    page: Readout,
    out: &mut Queue<Out>,
) {
    reading.pending &= !(1 << super::application::SLOT);
    if let Err(page) = super::application::received(&mut reading, page, &env.limits) {
        let printed = render::read(&page, client, runner.me(), reading.called.json, &env.limits, reading.parts);
        finish(reading, printed, out);
        return;
    }
    complete(runner, client, env, reading, out);
}
