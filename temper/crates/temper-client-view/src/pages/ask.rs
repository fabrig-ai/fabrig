//! Shared card words and parts (temper/docs/design/client/pages/README.md).
pub(crate) use crate::common::ask::{Asked, Kind, need_kind, read};
use crate::common::ask::{person_in_words, read_parts};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::ReadValue;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, inbox::Card, needs},
};

pub(crate) fn bounded<'a>(card: &'a Card, w: &Words) -> Asked<'a> {
    read_parts(card, w, false)
}
/// Context, evidence and the way to finish reading a cut question (client/pages/README.md).
pub(crate) fn whole_parts(b: &mut Builder, asked: &Asked<'_>, w: &Words, bindings: &mut Bindings) {
    if let Some(parsed) = &asked.parsed {
        if !parsed.context.is_empty() {
            text(b, Role::Paragraph, &parsed.context, look::ASK_CONTEXT);
        }
        if let Some(saw) = &parsed.saw {
            b.open(Role::Disclosure);
            b.style(look::ASK_SAW);
            b.name(&w.say(Word::NeedsSaw, &[]));
            text(b, Role::Paragraph, saw, look::ASK_CONTEXT);
            b.close();
        }
    }
    if asked.cut {
        b.open(Role::Paragraph);
        b.style(look::ASK_CONTEXT);
        b.text(Run::Plain, &w.say(Word::NeedsCut, &[]));
        b.text(Run::Plain, b" ");
        if let Some(task) = asked.source {
            link(b, bindings, &w.say(Word::NeedsCutRest, &[]), Address::Task { task });
        }
        b.close();
    }
}

pub(crate) fn card_kind(card: &Card) -> Word {
    if person_in_words(card).is_some() { Word::HomeQuestion } else { need_kind(card.group) }
}
pub(crate) fn need_when(card: &Card, w: &Words, times: &mut Times) -> Box<[u8]> {
    let kind = if let Some(ReadValue::Escalation(held)) = &card.page
        && let Some(hold) = &held.task.hold
        && hold.reason == jig_client::contract::HoldCategory::Budget
    {
        Word::NeedsKindFunds
    } else {
        card_kind(card)
    };
    when(card, kind, w, times)
}
fn when(card: &Card, kind: Word, w: &Words, times: &mut Times) -> Box<[u8]> {
    let kind = w.say(kind, &[]);
    let since = match &card.page {
        Some(page) => super::inbox::waiting_since(page),
        None => None,
    };
    match since {
        Some(since) => crate::accounts::words::join(&[
            &kind,
            &w.say(Word::ToolSeparator, &[]),
            &times.say(w, TimeForm::Span, since),
        ]),
        None => kind,
    }
}
pub(crate) fn answered_label(w: &Words, label: &needs::Label) -> Box<[u8]> {
    match label {
        needs::Label::Choice(words) => words.clone(),
        needs::Label::Reply(words) => w.say(Word::AnsweredReply, &[Hole::Name(words.clone())]),
    }
}
pub(crate) fn answered_lines(
    b: &mut Builder,
    root: &Domain,
    lines: &[&needs::Answered],
    w: &Words,
    bindings: &mut Bindings,
) {
    for line in lines {
        b.open(Role::Slot);
        b.style(look::ANSWERED);
        let title = if line.title.is_empty() {
            match super::inbox::source_task(line.object) {
                Some(task) => w.say(Word::Task, &[Hole::Number(task)]),
                None => w.say(Word::HomeQuestion, &[]),
            }
        } else {
            line.title.clone()
        };
        if w.shell == Shell::Terminal {
            b.open(Role::Paragraph);
        }
        b.open(Role::Text);
        if let Some(task) = super::inbox::source_task(line.object) {
            link(b, bindings, &title, Address::Task { task });
        } else {
            b.text(Run::Plain, &title);
        }
        b.text(Run::Plain, if w.shell == Shell::Terminal { b": " } else { b":" });
        b.close();
        text(b, Role::Text, &answered_label(w, &line.label), look::ANSWERED_LABEL);
        if w.shell == Shell::Terminal {
            b.close();
        }
        if root.pages().needs().held(line.object) {
            button(
                b,
                bindings,
                &w.say(Word::ToastUndo, &[]),
                Act::Needs(needs::Act::Undo { object: line.object }),
                look::ANSWERED_UNDO,
            );
        }
        b.close();
    }
}
pub(crate) fn text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
pub(crate) fn button(b: &mut Builder, bindings: &mut Bindings, words: &[u8], act: Act, style: skein_ui::Style) {
    b.open(Role::Button);
    b.style(style);
    b.name(words);
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn link(b: &mut Builder, bindings: &mut Bindings, words: &[u8], address: Address) {
    crate::bindings::link(b, words, bindings.insert(Act::Go { address }));
    b.text(Run::Plain, words);
    b.close();
}
