//! Finding cards and their forge acts (temper/docs/design/client/pages/needs-you.md, 1).
use super::ask;
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{Decimal, bytes};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act as PageAct,
        findings::{Act, Card, Group, Mode},
    },
};
use temper_client_protocol_wire::v1 as w;

/// Three bars say the severity in the browser (temper/docs/design/client/pages/home.md, 1, Findings, drawn).
pub fn severity(b: &mut Builder, words: &Words, severity: w::Severity) {
    if words.shell == Shell::Terminal {
        return;
    }
    let (style, word) = match severity {
        w::Severity::Low => (look::SEVERITY_LOW, Word::FindingLowSeverity),
        w::Severity::Medium => (look::SEVERITY_MEDIUM, Word::FindingMediumSeverity),
        w::Severity::High => (look::SEVERITY_HIGH, Word::FindingHighSeverity),
    };
    let label = words.say(word, &[]);
    b.open(Role::Text);
    b.style(look::SEVERITY);
    b.style(style);
    b.name(&label);
    b.hint(&label);
    for _ in 0..3_u32 {
        b.open(Role::Text);
        b.close();
    }
    b.close();
}

/// A chip takes its highest severity's ground (temper/docs/design/client/pages/home.md, 1, Findings, drawn).
pub fn chip(b: &mut Builder, words: &Words, severity: w::Severity, label: &[u8]) {
    if words.shell == Shell::Terminal {
        b.text(Run::Plain, label);
        return;
    }
    b.open(Role::Text);
    b.style(look::CHIP);
    b.style(match severity {
        w::Severity::Low => look::CHIP_LOW,
        w::Severity::Medium => look::CHIP_MEDIUM,
        w::Severity::High => look::CHIP_HIGH,
    });
    self::severity(b, words, severity);
    b.text(Run::Plain, label);
    b.close();
}

pub(crate) fn title(root: &Domain, card: &Card, words: &Words) -> Box<[u8]> {
    let project = match super::project_name(root.pages().me(root.client()), card.id.project) {
        Some(name) => name,
        None => words.say(Word::Project, &[Hole::Number(u64::from(card.id.project))]),
    };
    words.say(
        Word::FindingFrom,
        &[
            Hole::Name(project),
            Hole::Number(u64::from(card.id.project)),
            Hole::Number(u64::from(card.id.number)),
            Hole::Name(card.row.title.clone()),
        ],
    )
}
pub(crate) fn from(b: &mut Builder, root: &Domain, card: &Card, words: &Words, times: &mut Times) {
    let age = times.say(words, TimeForm::Span, skein_lib::Wall::from_nanos(card.row.at));
    ask::text(b, Role::Text, &words.say(Word::Finding, &[]), look::CHIP);
    ask::text(
        b,
        Role::Text,
        &bytes::concat(&[&title(root, card, words), &words.say(Word::ToolSeparator, &[]), &age]),
        look::ASK_META,
    );
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    card: &Card,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Slot);
    b.style(look::ASK);
    from(b, root, card, words, times);
    ask::text(b, Role::Heading { level: 1 }, &card.row.title, look::FLOW_Q);
    facts(b, card, words);
    context(b, card, words);
    reason(b, root, card, words);
    acts(b, root, card, words, bindings);
    b.close();
}
fn acts(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::ASK_CHOICES);
    button(b, bindings, words, Word::FindingHand, Act::Show { id: card.id, mode: Mode::Hand }, card.pending.is_some());
    if card.mode == Mode::Hand {
        hand(b, root, card, words, bindings);
    }
    button(
        b,
        bindings,
        words,
        Word::FindingRoute,
        Act::Show { id: card.id, mode: Mode::Route },
        card.pending.is_some(),
    );
    if card.mode == Mode::Route {
        route(b, root, card, words, bindings);
    }
    button(b, bindings, words, Word::FindingBacklog, Act::Backlog { id: card.id }, card.pending.is_some());
    button(b, bindings, words, Word::FindingDrop, Act::Show { id: card.id, mode: Mode::Drop }, card.pending.is_some());
    if card.mode == Mode::Drop {
        field(
            b,
            root,
            card,
            words,
            bindings,
            Word::FindingReason,
            Act::Reason { id: card.id, words: card.reason.clone() },
            &card.reason,
        );
        button(b, bindings, words, Word::FindingDrop, Act::Drop { id: card.id }, !card.can_drop());
    }
    b.close();
}
fn facts(b: &mut Builder, card: &Card, words: &Words) {
    let kind = words.say(kind_word(&card.row), &[]);
    let severity = words.say(severity_word(&card.row.severity), &[]);
    let area = match &card.row.area {
        Some(area) => area.clone(),
        None => words.say(Word::FindingNoArea, &[]),
    };
    ask::text(
        b,
        Role::Paragraph,
        &words.say(Word::FindingFacts, &[Hole::Name(kind), Hole::Name(severity), Hole::Name(area)]),
        look::ASK_CONTEXT,
    );
}
fn context(b: &mut Builder, card: &Card, words: &Words) {
    handed(b, card, words);
    if let Some(refusal) = &card.refusal {
        ask::text(b, Role::Paragraph, &refused(card, refusal, words), look::ASK_CONTEXT);
    }
    if card.pending.is_some() || card.unconfirmed {
        ask::text(b, Role::Paragraph, &words.say(Word::FindingWaiting, &[]), look::ASK_CONTEXT);
    }
}
fn handed(b: &mut Builder, card: &Card, words: &Words) {
    if let (Some(why), Some(hander)) = (&card.row.why, &card.row.hander) {
        let (prefix, number) = match hander {
            w::Filer::Task(task) => (Word::FindingTask, task.number),
            w::Filer::Party(party) => (Word::FindingParty, party.number),
        };
        let who = bytes::concat(&[&words.say(prefix, &[]), Decimal::of(number).as_bytes()]);
        ask::text(
            b,
            Role::Paragraph,
            &words.say(Word::FindingHanded, &[Hole::Name(who), Hole::Name(why.clone())]),
            look::ASK_CONTEXT,
        );
    }
}
fn button(b: &mut Builder, bindings: &mut Bindings, words: &Words, word: Word, act: Act, disabled: bool) {
    b.open(Role::Button);
    b.style(look::ASK_QUIET);
    b.name(&words.say(word, &[]));
    b.bind(bindings.insert(PageAct::Finding(act)));
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
#[expect(clippy::too_many_arguments, reason = "a finding field keeps its label, value and act together")]
fn field(
    b: &mut Builder,
    root: &Domain,
    card: &Card,
    words: &Words,
    bindings: &mut Bindings,
    label: Word,
    act: Act,
    value: &[u8],
) {
    b.open(Role::TextField);
    b.name(&words.say(label, &[]));
    b.value(value, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(PageAct::Finding(act)));
    if card.pending.is_some() {
        b.state(State::Disabled);
    }
    b.close();
}
fn hand(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Disclosure);
    b.name(&words.say(Word::FindingArea, &[]));
    for area in root.pages().needs().findings().areas(card.id) {
        b.open(Role::Button);
        b.name(&area.name);
        b.bind(bindings.insert(PageAct::Finding(Act::Area { id: card.id, words: area.name.clone() })));
        if card.area.as_ref() == area.name.as_ref() {
            b.state(State::Selected);
        }
        if card.pending.is_some() {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    if !card.area.is_empty() {
        ask::text(b, Role::Text, &card.area, look::ASK_CONTEXT);
    }
    field(
        b,
        root,
        card,
        words,
        bindings,
        Word::FindingWhy,
        Act::Why { id: card.id, words: card.why.clone() },
        &card.why,
    );
    button(b, bindings, words, Word::FindingHand, Act::Hand { id: card.id }, !card.can_hand());
}
fn route(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Disclosure);
    b.name(&words.say(Word::Goal, &[]));
    for goal in root.pages().needs().findings().goals(card.id) {
        b.open(Role::Button);
        b.name(goal.chip.title.bytes());
        b.bind(bindings.insert(PageAct::Finding(Act::Goal { id: card.id, task: goal.chip.task })));
        if card.goal == Some(goal.chip.task) {
            b.state(State::Selected);
        }
        if card.pending.is_some() {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    button(
        b,
        bindings,
        words,
        Word::FindingRoute,
        Act::Route { id: card.id },
        card.pending.is_some() || card.goal.is_none(),
    );
}
fn refused(card: &Card, refusal: &w::FindingRefusal, words: &Words) -> Box<[u8]> {
    match refusal {
        w::FindingRefusal::NoArea(areas) => {
            let mut names = Box::<[u8]>::default();
            for area in &areas.areas {
                names = if names.is_empty() { area.clone() } else { bytes::concat(&[&names, b", ", area]) };
            }
            words.say(
                Word::FindingNoAreaRefused,
                &[Hole::Name(card.area.clone()), Hole::Number(u64::from(card.id.project)), Hole::Name(names)],
            )
        }
        w::FindingRefusal::NotYours(held) => {
            let (prefix, number) = match &held.holder {
                w::Holder::Task(task) => (Word::FindingTask, task.number),
                w::Holder::Party(party) => (Word::FindingParty, party.number),
                w::Holder::Role(role) => (Word::FindingRole, u64::from(role.number)),
            };
            let holder = bytes::concat(&[&words.say(prefix, &[]), Decimal::of(number).as_bytes()]);
            words.say(
                Word::FindingNotYours,
                &[
                    Hole::Number(u64::from(card.id.project)),
                    Hole::Number(u64::from(card.id.number)),
                    Hole::Name(holder),
                ],
            )
        }
        w::FindingRefusal::Full(_)
        | w::FindingRefusal::NoFinding
        | w::FindingRefusal::NoGoal
        | w::FindingRefusal::Ended
        | w::FindingRefusal::Words(_) => crate::shared::finding_refusal(words, refusal),
    }
}
fn kind_word(row: &w::FindingRow) -> Word {
    match &row.kind {
        w::FindingKind::Qa => Word::FindingQa,
        w::FindingKind::Flake => Word::FindingFlake,
        w::FindingKind::Red => Word::FindingRed,
        w::FindingKind::Review => Word::FindingReview,
        w::FindingKind::Feedback => Word::FindingFeedback,
    }
}
fn severity_word(severity: &w::Severity) -> Word {
    match severity {
        w::Severity::Low => Word::FindingLow,
        w::Severity::Medium => Word::FindingMedium,
        w::Severity::High => Word::FindingHigh,
    }
}
/// A finding's short line shares the row's words, including severity in the terminal.
pub(crate) fn short(card: &Card, words: &Words, times: &mut Times, kind: bool) -> Box<[u8]> {
    short_row(&card.row, words, times, kind)
}
fn short_row(row: &w::FindingRow, words: &Words, times: &mut Times, kind: bool) -> Box<[u8]> {
    let separator = words.say(Word::ToolSeparator, &[]);
    let mut text: Box<[u8]> = row.title.clone();
    if words.shell == Shell::Terminal {
        text = bytes::concat(&[&text, &separator, &words.say(severity_word(&row.severity), &[])]);
    }
    bytes::concat(&[&text, &separator, &when(row, words, times, kind)])
}
fn when(row: &w::FindingRow, words: &Words, times: &mut Times, kind: bool) -> Box<[u8]> {
    let age = times.say(words, TimeForm::Span, skein_lib::Wall::from_nanos(row.at));
    if kind {
        bytes::concat(&[&words.say(kind_word(row), &[]), &words.say(Word::ToolSeparator, &[]), &age])
    } else {
        age
    }
}
/// One item in the caller's finding list (client/pages/home.md, 1, A finding's row).
pub fn row(
    b: &mut Builder,
    _root: &Domain,
    card: &Card,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    kind: bool,
) {
    finding_row(b, &card.row, Some(Act::Open { id: card.id }), words, bindings, times, kind);
}
fn finding_row(
    b: &mut Builder,
    row: &w::FindingRow,
    open: Option<Act>,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    kind: bool,
) {
    b.open(Role::Item);
    if words.shell == Shell::Terminal {
        b.text(Run::Plain, &short_row(row, words, times, kind));
    } else {
        severity(b, words, row.severity.clone());
        b.open(if open.is_some() { Role::Link } else { Role::Text });
        b.style(look::FINDING_TITLE);
        b.name(&row.title);
        if let Some(open) = open {
            b.bind(bindings.insert(PageAct::Finding(open)));
        }
        b.text(Run::Plain, &row.title);
        b.close();
        ask::text(b, Role::Text, &when(row, words, times, kind), look::FINDING_WHEN);
    }
    b.close();
}
/// The group's from line, shared by Home and the open Needs you card.
pub(crate) fn group_from(
    b: &mut Builder,
    root: &Domain,
    group: &Group<'_>,
    words: &Words,
    times: &mut Times,
    across: bool,
) {
    if across && let Some(name) = super::project_name(root.pages().me(root.client()), group.project) {
        ask::text(b, Role::Text, &name, look::CHIP);
        b.text(Run::Plain, b" ");
    }
    chip(b, words, group.severity(), &group_label(group, words));
    let separator = words.say(Word::ToolSeparator, &[]);
    b.text(Run::Plain, if words.shell == Shell::Terminal { &separator } else { b" " });
    b.text(Run::Plain, &group_when(group, words, times));
}
fn group_label(group: &Group<'_>, words: &Words) -> Box<[u8]> {
    let count = u64::try_from(group.rows.len()).expect("group rows bounded");
    if !group.noun.is_empty() {
        words.say(Word::FindingGroupCount, &[Hole::Number(count), Hole::Name(group.noun.clone())])
    } else if count == 1 {
        words.say(Word::FindingGroupOne, &[])
    } else {
        words.say(Word::FindingGroupMany, &[Hole::Number(count)])
    }
}
fn group_when(group: &Group<'_>, words: &Words, times: &mut Times) -> Box<[u8]> {
    let age = times.say(words, TimeForm::Span, group.oldest());
    let age = if group.rows.len() == 1 { age } else { words.say(Word::FindingGroupOldest, &[Hole::Name(age)]) };
    if group.phrase.is_empty() {
        age
    } else {
        bytes::concat(&[&group.phrase, &words.say(Word::ToolSeparator, &[]), &age])
    }
}
pub(crate) fn group_rows(
    b: &mut Builder,
    group: &Group<'_>,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let mut kind = false;
    if let Some(first) = group.rows.first() {
        for row in &group.rows {
            if row.kind != first.kind {
                kind = true;
                break;
            }
        }
    }
    b.open(Role::List);
    b.style(look::FINDING_ROWS);
    for row in &group.rows {
        finding_row(b, row, None, words, bindings, times, kind);
    }
    b.close();
}
pub(crate) fn group_context(b: &mut Builder, asked: &ask::Asked<'_>) {
    if let Some(parsed) = &asked.parsed
        && !parsed.context.is_empty()
    {
        b.open(Role::Paragraph);
        b.style(look::ASK_CONTEXT);
        b.style(look::GROUP_CONTEXT);
        b.name(&parsed.context);
        b.text(Run::Plain, &parsed.context);
        b.close();
    }
}
pub(crate) fn group_short(b: &mut Builder, group: &Group<'_>, words: &Words, times: &mut Times) {
    severity(b, words, group.severity());
    b.text(Run::Plain, &group_label(group, words));
    if words.shell == Shell::Terminal {
        b.text(Run::Plain, &words.say(Word::ToolSeparator, &[]));
        let severity = severity_word(&group.severity());
        b.text(Run::Plain, &words.say(severity, &[]));
    }
    let separator = words.say(Word::ToolSeparator, &[]);
    b.text(Run::Plain, if words.shell == Shell::Terminal { &separator } else { b" " });
    b.text(Run::Plain, &group_when(group, words, times));
}
fn reason(b: &mut Builder, root: &Domain, card: &Card, words: &Words) {
    if let Some(why) =
        temper_client_domain::pages::findings::why(root, temper_client_domain::pages::findings::Need::Finding(card))
        && let Some(line) = crate::shared::why_words(root, words, &why, false)
    {
        ask::text(b, Role::Paragraph, &line, look::WHY);
    }
}
#[expect(clippy::too_many_arguments, reason = "Home's finding card keeps its scope and reason placement")]
pub(crate) fn preview(
    b: &mut Builder,
    root: &Domain,
    card: &Card,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    across: bool,
    own_reason: bool,
) {
    b.open(Role::Slot);
    b.style(look::NEED_CARD);
    b.open(Role::Paragraph);
    b.style(look::NEED_FROM);
    if across && let Some(name) = super::project_name(root.pages().me(root.client()), card.id.project) {
        ask::text(b, Role::Text, &name, look::CHIP);
        b.text(Run::Plain, b" ");
    }
    chip(b, words, card.row.severity.clone(), &words.say(kind_word(&card.row), &[]));
    b.text(Run::Plain, b" ");
    let area = match &card.row.area {
        Some(area) => words.say(Word::FindingInArea, &[Hole::Name(area.clone())]),
        None => words.say(Word::FindingNoArea, &[]),
    };
    b.text(Run::Plain, &area);
    b.close();
    b.open(Role::List);
    b.style(look::FINDING_ROWS);
    row(b, root, card, words, bindings, times, false);
    b.close();
    context(b, card, words);
    if own_reason {
        reason(b, root, card, words);
    }
    b.open(Role::Slot);
    b.style(look::NEED_ACTS);
    acts(b, root, card, words, bindings);
    b.close();
    if words.shell == Shell::Browser {
        ask::button(
            b,
            bindings,
            &words.say(Word::HomeCardMore, &[]),
            PageAct::Finding(Act::Open { id: card.id }),
            look::NEED_MORE,
        );
    }
    b.close();
}
