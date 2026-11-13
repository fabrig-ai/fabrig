//! Finding cards and their forge acts (temper/docs/design/client/pages/needs-you.md, 1).
use super::ask;
use crate::{
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{Decimal, bytes};
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act as PageAct,
        findings::{Act, Card, Mode},
    },
};
use temper_client_protocol_wire::v1 as w;

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
            Hole::Name(card.row.title().into()),
        ],
    )
}
pub(crate) fn from(b: &mut Builder, root: &Domain, card: &Card, words: &Words, times: &mut Times) {
    let age = times.say(words, TimeForm::Span, skein_lib::Wall::from_nanos(card.row.at()));
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
    ask::text(b, Role::Heading { level: 1 }, card.row.title(), look::FLOW_Q);
    facts(b, card, words);
    if let Some(refusal) = &card.refusal {
        ask::text(b, Role::Paragraph, &refused(card, refusal, words), look::ASK_CONTEXT);
    }
    if card.pending.is_some() || card.unconfirmed {
        ask::text(b, Role::Paragraph, &words.say(Word::FindingWaiting, &[]), look::ASK_CONTEXT);
    }
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
    b.close();
}
fn facts(b: &mut Builder, card: &Card, words: &Words) {
    let kind = words.say(
        match card.row.kind() {
            w::FindingKind::Qa => Word::FindingQa,
            w::FindingKind::Flake => Word::FindingFlake,
            w::FindingKind::Red => Word::FindingRed,
            w::FindingKind::Review => Word::FindingReview,
            w::FindingKind::Feedback => Word::FindingFeedback,
        },
        &[],
    );
    let severity = words.say(
        match card.row.severity() {
            w::Severity::Low => Word::FindingLow,
            w::Severity::Medium => Word::FindingMedium,
            w::Severity::High => Word::FindingHigh,
        },
        &[],
    );
    let area = match card.row.area() {
        Some(area) => area.clone(),
        None => words.say(Word::FindingNoArea, &[]),
    };
    ask::text(
        b,
        Role::Paragraph,
        &words.say(Word::FindingFacts, &[Hole::Name(kind), Hole::Name(severity), Hole::Name(area)]),
        look::ASK_CONTEXT,
    );
    if let (Some(why), Some(hander)) = (card.row.why(), card.row.hander()) {
        let (prefix, number) = match hander {
            w::Filer::Task(task) => (Word::FindingTask, task.number()),
            w::Filer::Party(party) => (Word::FindingParty, party.number()),
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
        b.name(area.name());
        b.bind(bindings.insert(PageAct::Finding(Act::Area { id: card.id, words: area.name().into() })));
        if card.area.as_ref() == area.name() {
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
            for area in areas.areas() {
                names = if names.is_empty() { area.clone() } else { bytes::concat(&[&names, b", ", area]) };
            }
            words.say(
                Word::FindingNoAreaRefused,
                &[Hole::Name(card.area.clone()), Hole::Number(u64::from(card.id.project)), Hole::Name(names)],
            )
        }
        w::FindingRefusal::NotYours(held) => {
            let (prefix, number) = match held.holder() {
                w::Holder::Task(task) => (Word::FindingTask, task.number()),
                w::Holder::Party(party) => (Word::FindingParty, party.number()),
                w::Holder::Role(role) => (Word::FindingRole, u64::from(role.number())),
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
pub(crate) fn preview(
    b: &mut Builder,
    root: &Domain,
    card: &Card,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Slot);
    b.style(look::NEED_CARD);
    from(b, root, card, words, times);
    ask::text(b, Role::Paragraph, card.row.title(), look::ASK_QUESTION);
    ask::button(
        b,
        bindings,
        &words.say(Word::HomeOpen, &[]),
        PageAct::Finding(Act::Open { id: card.id }),
        look::ASK_QUIET,
    );
    b.close();
}
