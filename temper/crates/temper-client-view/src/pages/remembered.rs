//! The person's own preferences (temper/docs/design/client/pages/remembered.md).
use super::settings;
use crate::{
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, Shortcut, State, Style, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act as PageAct,
        remembered::{Act, Refusal, Remembered, Row},
    },
};

// Both deployment profiles admit twelve party lines (client/pages/remembered.md).
const PARTY_ENTRIES: u32 = 12;

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, times: &mut Times) {
    settings::main(b);
    settings::open_alone(b, &w.say(Word::RememberedTitle, &[]), &w.say(Word::RememberedLede, &[]));
    if let Some(page) = root.pages().remembered()
        && page.answered()
    {
        let open = match page.field() {
            Some((note, _)) => Some(note),
            None => None,
        };
        if open.is_none() {
            bindings.remembered_return = bindings.remembered_field;
        }
        bindings.remembered_field = open;
        rows(b, root, page, w, bindings, times);
    }
    b.close();
    b.close();
}

fn rows(b: &mut Builder, root: &Domain, page: &Remembered, w: &Words, bindings: &mut Bindings, times: &mut Times) {
    let mut count = 0_u32;
    for row in page.rows() {
        let (note, line, chat, changed) = match row {
            Row::Task { note, line, chat, changed, .. } => (note, line, Some(chat), changed),
            Row::Party { note, line, changed, .. } => (note, line, None, changed),
        };
        if count == 0 {
            b.open(Role::Slot);
            b.style(look::REMEMBERED_ROWS);
        }
        count = count.checked_add(1).expect("party lines fit");
        b.open(Role::Slot);
        b.key(skein_ui::Key { owner: crate::frame::owner(), kind: 70, a: note, b: 0 });
        b.open(Role::Slot);
        crate::cards::text(b, Role::Text, line);
        let date = match changed {
            Some(at) => times.say(w, TimeForm::Day, at),
            None => Box::default(),
        };
        let byline = match chat {
            Some(Some(chat)) => w.say(Word::RememberedChat, &[Hole::Number(chat), Hole::Name(date)]),
            Some(None) => w.say(Word::RememberedAsk, &[Hole::Name(date)]),
            None => w.say(Word::RememberedParty, &[Hole::Name(date)]),
        };
        text(b, Role::Text, &byline, look::SETTINGS_DETAIL);
        b.close();
        b.open(Role::Slot);
        b.style(look::REMEMBERED_ACTS);
        button(b, w, bindings, Word::RememberedChange, Act::Change { note });
        button(b, w, bindings, Word::RememberedDelete, Act::Delete { note });
        b.close();
        if let Some((open, words)) = page.field()
            && open == note
        {
            field(b, root, w, bindings, words);
        }
        if let Some((refused, refusal)) = page.refusal()
            && refused == note
        {
            match refusal {
                Refusal::Moved => text(b, Role::Paragraph, &w.say(Word::RememberedMoved, &[]), look::SETTINGS_REFUSAL),
                Refusal::TooLong { bound } => text(
                    b,
                    Role::Paragraph,
                    &w.say(Word::RememberedTooLong, &[Hole::Number(u64::from(*bound))]),
                    look::SETTINGS_REFUSAL,
                ),
                Refusal::Other(refusal) => crate::cards::refusal(b, w, refusal),
            }
        }
        b.close();
    }
    if count > 0 {
        b.close();
    }
    if count == 0 {
        b.open(Role::Slot);
        b.style(look::SETTINGS_BAND);
        b.style(look::SETTINGS_QUIET_BAND);
        text(b, Role::Text, &w.say(Word::RememberedEmpty, &[]), look::SETTINGS_TAG);
        crate::cards::text(b, Role::Paragraph, &w.say(Word::RememberedEmptyLine, &[]));
        b.close();
    }
    if count >= PARTY_ENTRIES {
        text(
            b,
            Role::Paragraph,
            &w.say(Word::RememberedFull, &[Hole::Number(u64::from(PARTY_ENTRIES))]),
            look::SETTINGS_FACTS,
        );
    }
}

fn field(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, words: &[u8]) {
    b.open(Role::Form);
    b.name(&w.say(Word::RememberedChange, &[]));
    b.open(Role::TextField);
    b.name(&w.say(Word::RememberedField, &[]));
    b.style(look::SETTINGS_TEXT);
    b.state(State::SubmitsOnEnter);
    b.value(words, temper_client_domain::pages::shown(root.pages()));
    b.focus(0);
    b.bind(bindings.insert(PageAct::Remembered(Act::Words { words: Box::default() })));
    b.close();
    b.open(Role::Slot);
    b.style(look::REMEMBERED_ACTS);
    button(b, w, bindings, Word::RememberedSave, Act::Save);
    button(b, w, bindings, Word::RememberedCancel, Act::Cancel);
    b.close();
    b.close();
}
fn button(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, act: Act) {
    b.open(Role::Button);
    b.style(look::SETTINGS_BTN);
    b.style(look::SETTINGS_SMALL);
    b.name(&w.say(word, &[]));
    let returning = match &act {
        Act::Change { note } => bindings.remembered_return == Some(*note),
        Act::Words { .. } | Act::Save | Act::Cancel | Act::Delete { .. } => false,
    };
    if returning {
        b.focus(0);
    }
    let cancel = act == Act::Cancel;
    b.bind(bindings.insert(PageAct::Remembered(act)));
    if cancel {
        b.shortcut(Shortcut::ESCAPE);
    }
    b.close();
}
pub(super) fn text(b: &mut Builder, role: Role, words: &[u8], style: Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
