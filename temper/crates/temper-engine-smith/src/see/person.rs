//! Person reads in temper's words (temper/docs/design/domain/agent/ask-temper.md, 1).

mod changes;
mod runs;
mod words;
use self::words::{executor, kind, number, state, task_name, time};
use super::{join, join_parts};
use alloc::boxed::Box;
use jig_domain_core::{
    documents as d,
    reads::{Answer, EndedGroup, PageRow},
};
use skein_lib::List;
use temper_engine_domain::{SeeNames, SeePerson, SeeRead, SeeWords};

pub(super) fn text(person: &SeePerson, catalogue: &[jig_domain_core::settings::CatalogueModel]) -> Box<[u8]> {
    let names = &person.names;
    let mut lines = List::uncapped();
    match &person.read {
        SeeRead::Tasks { tasks } => {
            let Answer::Tasks(page) = tasks else { return b"tasks: not read\n".as_slice().into() };
            lines
                .push(join(&[
                    b"live tasks of ",
                    &project_name(names),
                    b": ",
                    &count(page.live.len(), page.live_next.is_some()),
                    b", the latest active first\n",
                ]))
                .expect("heading");
            for row in &page.live {
                lines
                    .push(join(&[
                        &task_name(b"task", &row.chip),
                        " · ".as_bytes(),
                        &kind(row.label, None, row.chip.details.executor),
                        " · ".as_bytes(),
                        &executor(row.chip.details.executor, None, names, catalogue),
                        " · ".as_bytes(),
                        &state(&row.chip, &row.waits_on, names),
                        " · ".as_bytes(),
                        &time(row.last_activity),
                        b"\n",
                    ]))
                    .expect("task row");
            }
        }
        SeeRead::Me { person, project, needs, words } => me(*person, project, needs, words, names, &mut lines),
        SeeRead::Project { project, held, repositories } => {
            project_text(project, held, repositories, names, catalogue, &mut lines);
        }
        SeeRead::Goals { ended, goals, reports } => goals_text(*ended, goals, reports, names, &mut lines),
        SeeRead::Task { number, task, report } => {
            task_text(*number, task, report.as_ref(), names, catalogue, &mut lines);
        }
        SeeRead::Tree { number, live, ended } => tree(*number, live, ended, names, catalogue, &mut lines),
        SeeRead::Inbox { needs, words } => inbox(needs, words, names, &mut lines),
        SeeRead::Activity { number, activity } => runs::activity(*number, activity, names, &mut lines),
        SeeRead::Turn { number, attempt, turn, read } => runs::turn(*number, *attempt, *turn, read, &mut lines),
        SeeRead::Change { number, panel } => changes::change(*number, panel, &mut lines),
        SeeRead::Changes { changes, landings } => {
            changes::queues(changes.as_deref(), landings.as_deref(), names, &mut lines);
        }
    }
    join_parts(&lines)
}

fn project_name(names: &SeeNames) -> Box<[u8]> {
    join(&[b"project ", &number(names.project.number), b", ", &words::short(&names.project.name, false)])
}
fn count(value: u32, more: bool) -> Box<[u8]> {
    join(&[&number(u64::from(value)), if more { b" or more" } else { b"" }])
}

fn project_document(answer: &Answer) -> Option<&d::Project> {
    let Answer::Document(document) = answer else { return None };
    let d::Document::Project(value) = document.as_ref() else { return None };
    Some(value)
}
fn me(
    person: u64,
    project: &Answer,
    needs: &Answer,
    words: &[SeeWords],
    names: &SeeNames,
    lines: &mut List<Box<[u8]>>,
) {
    let Some(project) = project_document(project) else {
        lines.push(b"me: not read\n".as_slice().into()).expect("unread");
        return;
    };
    let mut member = None;
    for candidate in &project.members {
        if candidate.party == person {
            member = Some(candidate);
            break;
        }
    }
    let role = match member {
        Some(member) => words::role(member.role),
        None => b"role not read".as_slice().into(),
    };
    lines
        .push(join(&[
            b"you: ",
            &words::person(person, names),
            " · ".as_bytes(),
            &role,
            b" in ",
            &project_name(names),
            b"\n",
        ]))
        .expect("person heading");
    let current_period = match &project.current_period {
        Some(ledger) => match ledger.source {
            jig_domain_tasks::Funder::Period { period, .. } => Some(period),
            jig_domain_tasks::Funder::Task(_)
            | jig_domain_tasks::Funder::Pool { .. }
            | jig_domain_tasks::Funder::Recurring { .. } => None,
        },
        None => None,
    };
    let mut pool = None;
    for ledger in &project.pools {
        match ledger.source {
            jig_domain_tasks::Funder::Pool { person: owner, period, .. }
                if owner == person && Some(period) == current_period && !ledger.closed =>
            {
                pool = Some(ledger);
                break;
            }
            jig_domain_tasks::Funder::Pool { .. }
            | jig_domain_tasks::Funder::Task(_)
            | jig_domain_tasks::Funder::Recurring { .. }
            | jig_domain_tasks::Funder::Period { .. } => {}
        }
    }
    let pool_text = match pool {
        Some(pool) => join(&[
            b"your pool this period: ",
            &words::money(pool.numbers.budget),
            b", ",
            &words::money(pool.left),
            b" left\n",
        ]),
        None => b"your pool this period: none open\n".as_slice().into(),
    };
    lines.push(pool_text).expect("pool");
    let Answer::Page(page) = needs else {
        lines.push(b"waiting on you here: not read\n".as_slice().into()).expect("unread needs");
        return;
    };
    let mut counts = [0_u32; 4];
    for row in &page.rows {
        let PageRow::Inbox(entry) = row else { continue };
        let index = match entry.object {
            d::InboxObject::Question { .. } => 0,
            d::InboxObject::Proposal { .. } => 1,
            d::InboxObject::Escalation { .. } => 2,
            d::InboxObject::PersonTask => 3,
            d::InboxObject::Reply { .. } | d::InboxObject::Result { .. } => continue,
        };
        let value = counts.get_mut(index).expect("four kinds");
        *value = value.checked_add(1).expect("bounded page");
    }
    let mut heading = join(&[b"waiting on you here: ", &count(page.rows.len(), page.next.is_some() || page.cut)]);
    for (value, (singular, plural)) in counts.into_iter().zip([
        (b"question".as_slice(), b"questions".as_slice()),
        (b"proposal", b"proposals"),
        (b"escalation", b"escalations"),
        (b"person task", b"person tasks"),
    ]) {
        if value != 0 {
            heading = join(&[
                &heading,
                " · ".as_bytes(),
                &number(u64::from(value)),
                b" ",
                if value == 1 { singular } else { plural },
            ]);
        }
    }
    lines.push(join(&[&heading, b"\n"])).expect("needs heading");
    inbox_rows(page, words, lines);
}

fn project_text(
    project: &Answer,
    held: &Answer,
    repositories: &[temper_engine_domain::RepositoryIdentity],
    names: &SeeNames,
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    lines: &mut List<Box<[u8]>>,
) {
    lines.push(join(&[&project_name(names), b"\n"])).expect("project heading");
    let Some(project) = project_document(project) else {
        lines.push(b"project: not read\n".as_slice().into()).expect("unread");
        return;
    };
    lines
        .push(join(&[b"repositories: ", &number(skein_lib::len_u64(repositories.len())), b"\n"]))
        .expect("repositories heading");
    for repository in repositories {
        lines
            .push(join(&[
                &words::short(&repository.host, false),
                b"/",
                &words::short(&repository.owner, false),
                b"/",
                &words::short(&repository.name, false),
                b"\n",
            ]))
            .expect("repository");
    }
    lines.push(join(&[b"people: ", &count(project.members.len(), project.cut), b"\n"])).expect("people heading");
    for member in &project.members {
        lines
            .push(join(&[&words::person(member.party, names), b", ", &words::role(member.role), b"\n"]))
            .expect("member");
    }
    match &project.current_period {
        Some(period) => {
            let period_number = match period.source {
                jig_domain_tasks::Funder::Period { period, .. } => number(period),
                jig_domain_tasks::Funder::Pool { .. }
                | jig_domain_tasks::Funder::Task(_)
                | jig_domain_tasks::Funder::Recurring { .. } => b"not read".as_slice().into(),
            };
            let began = match period.began {
                Some(wall) => time(wall),
                None => b"not read".as_slice().into(),
            };
            let next = match period.next {
                Some(wall) => time(wall),
                None => b"not read".as_slice().into(),
            };
            lines
                .push(join(&[
                    b"this period: ",
                    &period_number,
                    b", from ",
                    &began,
                    b" to ",
                    &next,
                    " · budget ".as_bytes(),
                    &words::money(period.numbers.budget),
                    " · reserved ".as_bytes(),
                    &words::money(period.numbers.reserved),
                    " · spent ".as_bytes(),
                    &words::spent(&period.numbers),
                    " · left ".as_bytes(),
                    &words::money(period.left),
                    b"\n",
                ]))
                .expect("period");
        }
        None => lines.push(b"this period: none open\n".as_slice().into()).expect("no period"),
    }
    let Answer::Tasks(held) = held else {
        lines.push(b"held tasks: not read\n".as_slice().into()).expect("unread held");
        return;
    };
    lines
        .push(join(&[b"held tasks: ", &count(held.live.len(), held.live_next.is_some()), b"\n"]))
        .expect("held heading");
    for row in &held.live {
        lines
            .push(join(&[
                &task_name(b"task", &row.chip),
                " · ".as_bytes(),
                &state(&row.chip, &row.waits_on, names),
                " · ".as_bytes(),
                &executor(row.chip.details.executor, None, names, catalogue),
                " · ".as_bytes(),
                &chip_hold(&row.chip),
                b"\n",
            ]))
            .expect("held task");
    }
}

fn task_rows(answer: &Answer, ended: bool) -> Option<(&[d::TaskLine], bool)> {
    let Answer::Tasks(page) = answer else { return None };
    if ended {
        match &page.ended {
            EndedGroup::Rows { rows, next, cut } => Some((rows.as_slice(), next.is_some() || *cut)),
            EndedGroup::None => Some((&[], false)),
        }
    } else {
        Some((page.live.as_slice(), page.live_next.is_some()))
    }
}
fn goals_text(ended: bool, goals: &Answer, reports: &Answer, names: &SeeNames, lines: &mut List<Box<[u8]>>) {
    let Some((rows, more)) = task_rows(goals, ended) else {
        lines.push(b"goals: not read\n".as_slice().into()).expect("unread goals");
        return;
    };
    lines
        .push(join(&[
            if ended { b"ended goals of " } else { b"goals of " },
            &project_name(names),
            b": ",
            &count(u32::try_from(rows.len()).expect("bounded rows"), more),
            if ended { b", the latest ended first\n" } else { b", by priority\n" },
        ]))
        .expect("goals heading");
    if rows.is_empty() {
        match reports {
            Answer::None(_) => lines.push(b"report: not read\n".as_slice().into()).expect("unread reports"),
            Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::Page(_) => {}
        }
    }
    for row in rows {
        let priority = match row.tracked {
            Some(priority) => join(&[" · priority ".as_bytes(), &number(u64::from(priority))]),
            None => b"".as_slice().into(),
        };
        lines
            .push(join(&[
                &task_name(b"goal", &row.chip),
                &priority,
                " · ".as_bytes(),
                &state(&row.chip, &row.waits_on, names),
                " · ".as_bytes(),
                &report_head(row.chip.number, reports, names.project.number),
                b"\n",
            ]))
            .expect("goal row");
    }
}
fn goal_note(answer: &Answer, task: u64, project: u64) -> Option<&d::Note> {
    let Answer::Page(page) = answer else { return None };
    let mut newest: Option<&d::Note> = None;
    for row in &page.rows {
        let PageRow::Note(note) = row else { continue };
        if !note.description.bytes.starts_with(b"report: ") {
            continue;
        }
        match note.scope {
            d::NoteLocation::Goal { task: owner, project: owner_project }
                if owner == task && u64::from(owner_project) == project =>
            {
                let replace = match newest {
                    Some(previous) => note.changed > previous.changed,
                    None => true,
                };
                if replace {
                    newest = Some(note);
                }
            }
            d::NoteLocation::Goal { .. }
            | d::NoteLocation::Deployment
            | d::NoteLocation::Project(_)
            | d::NoteLocation::Resources { .. }
            | d::NoteLocation::Party { .. } => {}
        }
    }
    newest
}
fn report_head(task: u64, reports: &Answer, project: u64) -> Box<[u8]> {
    match reports {
        Answer::None(_) => return b"report: not read".as_slice().into(),
        Answer::TaskText(_)
        | Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::Tasks(_)
        | Answer::Document(_)
        | Answer::Ended(_)
        | Answer::Page(_) => {}
    }
    match goal_note(reports, task, project) {
        Some(note) => words::short(&note.description.bytes, note.description.cut),
        None => match reports {
            Answer::Page(page) if page.next.is_some() || page.cut => b"report: not read".as_slice().into(),
            Answer::Page(_)
            | Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Document(_)
            | Answer::Ended(_)
            | Answer::None(_) => b"no report yet".as_slice().into(),
        },
    }
}

fn tree(
    number_value: u64,
    live: &Answer,
    ended: &Answer,
    names: &SeeNames,
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    lines: &mut List<Box<[u8]>>,
) {
    let live = task_rows(live, false);
    let ended = task_rows(ended, true);
    let live_count = match live {
        Some((rows, more)) => count(u32::try_from(rows.len()).expect("rows"), more),
        None => b"not read".as_slice().into(),
    };
    let ended_count = match ended {
        Some((rows, more)) => count(u32::try_from(rows.len()).expect("rows"), more),
        None => b"not read".as_slice().into(),
    };
    lines
        .push(join(&[
            b"delegates of task ",
            &number(number_value),
            b": ",
            &live_count,
            b" live, ",
            &ended_count,
            b" ended\n",
        ]))
        .expect("tree heading");
    for (label, group) in [(b"live delegates".as_slice(), live), (b"ended delegates".as_slice(), ended)] {
        let Some((rows, _)) = group else {
            lines.push(join(&[label, b": not read\n"])).expect("unread delegates");
            continue;
        };
        for row in rows {
            lines
                .push(join(&[
                    &task_name(b"task", &row.chip),
                    " · ".as_bytes(),
                    &executor(row.chip.details.executor, None, names, catalogue),
                    " · ".as_bytes(),
                    &state(&row.chip, &row.waits_on, names),
                    " · spent ".as_bytes(),
                    &words::spent(&row.budget.numbers),
                    " · delegates: ".as_bytes(),
                    &number(u64::from(row.delegates_total)),
                    b"\n",
                ]))
                .expect("delegate");
        }
    }
}
fn inbox(needs: &Answer, words: &[SeeWords], names: &SeeNames, lines: &mut List<Box<[u8]>>) {
    let Answer::Page(page) = needs else {
        lines.push(b"inbox: not read\n".as_slice().into()).expect("unread inbox");
        return;
    };
    lines
        .push(join(&[
            b"waiting on you in ",
            &project_name(names),
            b": ",
            &count(page.rows.len(), page.next.is_some() || page.cut),
            b", oldest first; each is answered on Needs you\n",
        ]))
        .expect("inbox heading");
    inbox_rows(page, words, lines);
    if page.next.is_some() || page.cut {
        lines.push(b"more wait on you\n".as_slice().into()).expect("more needs");
    }
}
fn inbox_rows(page: &jig_domain_core::reads::Page, words: &[SeeWords], lines: &mut List<Box<[u8]>>) {
    for row in &page.rows {
        let PageRow::Inbox(entry) = row else { continue };
        let kind = match entry.object {
            d::InboxObject::Question { .. } => b"question".as_slice(),
            d::InboxObject::Proposal { .. } => b"proposal",
            d::InboxObject::Escalation { .. } => b"escalation",
            d::InboxObject::PersonTask => b"person task",
            d::InboxObject::Reply { .. } => b"reply",
            d::InboxObject::Result { .. } => b"result",
        };
        let task = match &entry.task {
            d::TaskLink::Live(chip) | d::TaskLink::Retained { chip, .. } => task_name(b"task", chip),
            d::TaskLink::Stored { number: n, .. } => join(&[b"task ", &number(*n)]),
        };
        let mut line = join(&[kind, " · ".as_bytes(), &task]);
        if let Some(text) = inbox_words(entry, words) {
            line = join(&[&line, " · ".as_bytes(), &text]);
        }
        lines.push(join(&[&line, b"\n"])).expect("inbox row");
    }
}

fn inbox_words(entry: &d::InboxEntry, words: &[SeeWords]) -> Option<Box<[u8]>> {
    let message = match entry.object {
        d::InboxObject::Question { message } => message,
        d::InboxObject::Proposal { .. }
        | d::InboxObject::Escalation { .. }
        | d::InboxObject::PersonTask
        | d::InboxObject::Reply { .. }
        | d::InboxObject::Result { .. } => return None,
    };
    let task = match &entry.task {
        d::TaskLink::Live(chip) | d::TaskLink::Retained { chip, .. } => chip.number,
        d::TaskLink::Stored { number, .. } => *number,
    };
    for read in words {
        if read.task != task || read.message != message {
            continue;
        }
        return match &read.words {
            Answer::Document(document) => match document.as_ref() {
                d::Document::Question(question) => Some(words::short(&question.words.bytes, question.words.cut)),
                d::Document::Frame(_)
                | d::Document::Chips(_)
                | d::Document::Run(_)
                | d::Document::Tree(_)
                | d::Document::Lines(_)
                | d::Document::Proposal(_)
                | d::Document::Escalation(_)
                | d::Document::PersonTask(_)
                | d::Document::Result(_)
                | d::Document::Project(_)
                | d::Document::Note(_)
                | d::Document::Inbox(_)
                | d::Document::Terminal { .. }
                | d::Document::Accounts(_)
                | d::Document::Deployment(_)
                | d::Document::Settings(_)
                | d::Document::Task(_) => None,
            },
            Answer::Ended(ended) => match &ended.value {
                jig_domain_core::reads::EndedValue::Question { question, .. } => {
                    Some(words::short(&question.words.bytes, question.words.cut))
                }
                jig_domain_core::reads::EndedValue::Task { .. }
                | jig_domain_core::reads::EndedValue::Proposal(_)
                | jig_domain_core::reads::EndedValue::PersonProposal { .. }
                | jig_domain_core::reads::EndedValue::Escalation { .. }
                | jig_domain_core::reads::EndedValue::PersonTask { .. }
                | jig_domain_core::reads::EndedValue::Reply(_)
                | jig_domain_core::reads::EndedValue::WithheldTurn(_)
                | jig_domain_core::reads::EndedValue::Turn(_) => None,
            },
            Answer::TaskText(_)
            | Answer::Setting(_)
            | Answer::Revision(_)
            | Answer::Revisions(_)
            | Answer::Tasks(_)
            | Answer::Page(_)
            | Answer::None(_) => None,
        };
    }
    None
}
fn chip_hold(chip: &d::Chip) -> Box<[u8]> {
    match &chip.details.hold {
        Some(hold) => match &hold.words {
            Some(text) => words::short(&text.bytes, text.cut),
            None => words::hold(hold.reason),
        },
        None => match chip.phase {
            d::PhaseLine::Held { reason, .. } => words::hold(reason),
            d::PhaseLine::Waiting { .. }
            | d::PhaseLine::Active(_)
            | d::PhaseLine::Person { .. }
            | d::PhaseLine::Closing(_)
            | d::PhaseLine::Ended => b"".as_slice().into(),
        },
    }
}

fn task_document(answer: &Answer) -> Option<&d::Task> {
    match answer {
        Answer::Document(document) => match document.as_ref() {
            d::Document::Task(task) => Some(task),
            d::Document::Frame(_)
            | d::Document::Chips(_)
            | d::Document::Run(_)
            | d::Document::Tree(_)
            | d::Document::Lines(_)
            | d::Document::Proposal(_)
            | d::Document::Escalation(_)
            | d::Document::Question(_)
            | d::Document::PersonTask(_)
            | d::Document::Result(_)
            | d::Document::Project(_)
            | d::Document::Note(_)
            | d::Document::Inbox(_)
            | d::Document::Terminal { .. }
            | d::Document::Accounts(_)
            | d::Document::Deployment(_)
            | d::Document::Settings(_) => None,
        },
        Answer::Ended(ended) => match &ended.value {
            jig_domain_core::reads::EndedValue::Task { document, .. } => Some(document),
            jig_domain_core::reads::EndedValue::Proposal(_)
            | jig_domain_core::reads::EndedValue::PersonProposal { .. }
            | jig_domain_core::reads::EndedValue::Escalation { .. }
            | jig_domain_core::reads::EndedValue::Question { .. }
            | jig_domain_core::reads::EndedValue::PersonTask { .. }
            | jig_domain_core::reads::EndedValue::Reply(_)
            | jig_domain_core::reads::EndedValue::WithheldTurn(_)
            | jig_domain_core::reads::EndedValue::Turn(_) => None,
        },
        Answer::TaskText(_)
        | Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::Tasks(_)
        | Answer::Page(_)
        | Answer::None(_) => None,
    }
}
fn task_text(
    number_value: u64,
    answer: &Answer,
    report: Option<&Answer>,
    names: &SeeNames,
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    lines: &mut List<Box<[u8]>>,
) {
    let Some(task) = task_document(answer) else {
        lines.push(join(&[b"task ", &number(number_value), b": not read\n"])).expect("unread task");
        return;
    };
    lines
        .push(join(&[
            &task_name(b"task", &task.chip),
            " · ".as_bytes(),
            &kind(task.label, Some(&task.contract), task.executor),
            " · ".as_bytes(),
            &state(&task.chip, &task.waits_on, names),
            b"\n",
        ]))
        .expect("task heading");
    match &task.chip.phase {
        d::PhaseLine::Held { .. } => lines.push(join(&[b"hold: ", &chip_hold(&task.chip), b"\n"])).expect("hold"),
        d::PhaseLine::Waiting { .. }
        | d::PhaseLine::Active(_)
        | d::PhaseLine::Person { .. }
        | d::PhaseLine::Closing(_)
        | d::PhaseLine::Ended => {}
    }
    let requester = match task.requester {
        jig_domain_tasks::Party::Person(n) => words::person(n, names),
        jig_domain_tasks::Party::Task(n) => {
            let mut label = join(&[b"task ", &number(n)]);
            for chip in &task.lineage {
                if chip.number == n {
                    label = task_name(b"task", chip);
                    break;
                }
            }
            label
        }
        jig_domain_tasks::Party::Deployment { .. } => b"the deployment".as_slice().into(),
    };
    lines
        .push(join(&[
            b"requested by: ",
            &requester,
            b"\nexecutor: ",
            &executor(task.executor, task.llm.as_ref(), names, catalogue),
            b"\nspent: ",
            &words::spent(&task.budget.numbers),
            b" of ",
            &words::money(task.budget.numbers.budget),
            b"\n",
        ]))
        .expect("task details");
    lines
        .push(join(&[
            b"delegates: ",
            &count(task.delegates.len(), task.cut),
            b" live\nasked: ",
            &words::short(&task.words.bytes, task.words.cut),
            b"\n",
        ]))
        .expect("task request");
    if let Some(report) = report {
        lines.push(join(&[&report_head(task.chip.number, report, names.project.number), b"\n"])).expect("report");
        if let Some(note) = goal_note(report, task.chip.number, names.project.number) {
            lines
                .push(join(&[&note.body.bytes, if note.body.cut { "…\n".as_bytes() } else { b"\n" }]))
                .expect("report body");
        }
    }
    let result = match answer {
        Answer::Ended(ended) => match &ended.value {
            jig_domain_core::reads::EndedValue::Task { result, .. } => Some(&result.ending),
            jig_domain_core::reads::EndedValue::Proposal(_)
            | jig_domain_core::reads::EndedValue::PersonProposal { .. }
            | jig_domain_core::reads::EndedValue::Escalation { .. }
            | jig_domain_core::reads::EndedValue::Question { .. }
            | jig_domain_core::reads::EndedValue::PersonTask { .. }
            | jig_domain_core::reads::EndedValue::Reply(_)
            | jig_domain_core::reads::EndedValue::WithheldTurn(_)
            | jig_domain_core::reads::EndedValue::Turn(_) => None,
        },
        Answer::Document(_) => task.result.as_ref(),
        Answer::TaskText(_)
        | Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::Tasks(_)
        | Answer::Page(_)
        | Answer::None(_) => None,
    };
    if let Some(result) = result {
        lines.push(join(&[b"result: ", &words::result(result), b"\n"])).expect("result");
    }
}
