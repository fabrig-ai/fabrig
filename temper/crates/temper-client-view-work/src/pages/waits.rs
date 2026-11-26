//! Work's live Waits page (temper/docs/design/client/pages/waits.md, 1).
use super::{changes::count_named, work};
use crate::{
    bindings::{self, Bindings},
    look,
    words::{Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, State, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain, WaitsFilter,
    pages::{Act, waits::Problem},
};

const HOUR: u32 = 3600;

pub fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    project: u32,
    filter: WaitsFilter,
    times: &mut Times,
) {
    let Some(waits) = root.pages().waits() else { return };
    let page = waits.page();
    let me = root.pages().me(root.client());
    let name =
        super::project_name(me, project).unwrap_or(words.say(Word::Project, &[Hole::Number(u64::from(project))]));
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::WORK_PAGE);
    b.open(Role::Slot);
    b.style(look::WORK_INTRO);
    text(b, Role::Paragraph, look::WORK_KICKER, &words.say(Word::WorkKicker, &[Hole::Name(name.clone())]));
    b.open(Role::Slot);
    b.style(look::WORK_SECTION_HEAD);
    if let Some(page) = page {
        text(b, Role::Heading { level: 1 }, look::WORK_HEADLINE, &heading(page, words));
    } else {
        work::held_heading(b, words, 1, look::WORK_HEADLINE);
    }
    text(b, Role::Paragraph, look::WORK_BYLINE, &words.say(Word::WaitsOrder, &[]));
    b.close();
    filters(b, words, bindings, project, filter);
    button(b, words, bindings, Word::WaitsRefresh, Act::RefreshWaits, false);
    b.close();
    b.open(Role::Slot);
    b.style(look::WORK_SECTION);
    if let Some(problem) = waits.problem() {
        let word = match problem {
            Problem::Busy => Word::WaitsBusy,
            Problem::Refused => Word::WaitsRefused,
            Problem::Unavailable => Word::WaitsUnavailable,
        };
        text(b, Role::Alert, look::GOALS_EMPTY, &words.say(word, &[]));
    }
    if let Some(page) = page {
        if page.rows.is_empty() && page.after.is_none() {
            let empty = if filter == WaitsFilter::ALL {
                words.say(Word::WaitsOnNothing, &[Hole::Name(name)])
            } else {
                words.say(Word::WaitsNoMatch, &[])
            };
            text(b, Role::Paragraph, look::GOALS_EMPTY, &empty);
        } else {
            for row in &page.rows {
                draw_row(b, root, words, bindings, project, times, row);
            }
        }
        if page.after.is_some() {
            button(b, words, bindings, Word::WaitsMore, Act::MoreWaits, waits.reading());
        }
    }
    b.close();
    b.close();
}

fn heading(page: &c::WaitPage, words: &Words) -> Box<[u8]> {
    let count = u64::from(page.rows.len());
    if page.after.is_some() {
        return words.say(Word::WaitsMoreCount, &[Hole::Number(count)]);
    }
    match count {
        0 => words.say(Word::WaitsNothing, &[]),
        1 => words.say(Word::WaitsOne, &[]),
        2..=u64::MAX => words.say(Word::WaitsMany, &[Hole::Name(count_named(count, words))]),
    }
}

fn filters(b: &mut Builder, words: &Words, bindings: &mut Bindings, project: u32, here: WaitsFilter) {
    b.open(Role::Navigation);
    b.name(&words.say(Word::WaitsFilters, &[]));
    b.style(look::WORK_FILTERS);
    for (word, filter, active) in [
        (Word::GoalsAll, WaitsFilter::ALL, here == WaitsFilter::ALL),
        (Word::WaitsMine, WaitsFilter { mine: true, ..WaitsFilter::ALL }, here.mine),
        (Word::WaitsDueHour, WaitsFilter { due: Some(HOUR), ..WaitsFilter::ALL }, here.due == Some(HOUR)),
    ] {
        work::filter_chip(b, bindings, &words.say(word, &[]), Address::Waits { project, filter }, active);
    }
    if let Some(kind) = here.kind {
        let filter = WaitsFilter { kind: Some(kind), ..WaitsFilter::ALL };
        work::filter_chip(b, bindings, &words.say(kind_word(kind), &[]), Address::Waits { project, filter }, true);
    }
    b.close();
}

fn draw_row(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
    row: &c::WaitRow,
) {
    let task = row.chip.task;
    let title = if row.chip.title.bytes().is_empty() {
        words.say(Word::DelegateNumber, &[Hole::Number(task)])
    } else {
        crate::common::chat::row_title(row.chip.title.bytes())
    };
    let separator = words.say(Word::ToolSeparator, &[]);
    b.open(Role::Paragraph);
    b.style(look::WAIT_ROW);
    link(b, bindings, &title, Address::Task { task });
    b.text(Run::Plain, &separator);
    let reader = root.client().link().party;
    let members = root.pages().wait_members(project);
    let on_box = match root.forge().deployment() {
        Some(deployment) => deployment.owned,
        None => false,
    };
    b.text(Run::Plain, &words.wait(&row.wait, times, reader, members, on_box, true));
    b.text(Run::Plain, &separator);
    let kind = row.wait.kind;
    let filter = WaitsFilter { kind: Some(kind), ..WaitsFilter::ALL };
    link(b, bindings, &words.say(kind_word(kind), &[]), Address::Waits { project, filter });
    match row.wait.bound {
        c::waits::Bound::At { at, then: c::AtBound::Overdue } if at <= times.now => {
            b.text(Run::Plain, &separator);
            text(b, Role::Text, look::WAIT_OVERDUE, &words.say(Word::WaitsOverdue, &[]));
        }
        c::waits::Bound::At { .. } | c::waits::Bound::None(_) => {}
    }
    b.close();
}

fn kind_word(kind: c::WaitKind) -> Word {
    match kind {
        c::WaitKind::Party => Word::WaitsKindParty,
        c::WaitKind::Question => Word::Question,
        c::WaitKind::Dependency => Word::WaitsKindDependency,
        c::WaitKind::Resource => Word::WaitsKindResource,
        c::WaitKind::Account => Word::WaitsKindAccount,
        c::WaitKind::Host => Word::WaitsKindHost,
        c::WaitKind::Delegate => Word::WaitsKindDelegate,
        c::WaitKind::News => Word::WaitsKindNews,
        c::WaitKind::Step => Word::WaitsKindStep,
        c::WaitKind::Room => Word::WaitsKindRoom,
        c::WaitKind::Turn => Word::WaitsKindTurn,
        c::WaitKind::SignOff => Word::WaitsKindSignOff,
    }
}

fn link(b: &mut Builder, bindings: &mut Bindings, label: &[u8], address: Address) {
    bindings::link(b, label, bindings.insert(Act::Go { address }));
    b.text(Run::Plain, label);
    b.close();
}

fn text(b: &mut Builder, role: Role, style: Style, label: &[u8]) {
    b.open(role);
    b.style(style);
    b.name(label);
    if role == Role::Text {
        b.run(Run::Plain, label);
    } else {
        b.text(Run::Plain, label);
    }
    b.close();
}

fn button(b: &mut Builder, words: &Words, bindings: &mut Bindings, word: Word, act: Act, reading: bool) {
    b.open(Role::Button);
    b.style(look::GOALS_MORE);
    b.name(&words.say(word, &[]));
    if reading {
        b.state(State::Disabled);
    } else {
        b.bind(bindings.insert(act));
    }
    b.close();
}
