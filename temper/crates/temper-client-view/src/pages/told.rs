//! Telling you in Deployment's column (temper/docs/design/client/pages/told.md).
use super::{remembered::text, settings};
use crate::{
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use skein_ui::{Builder, Role, Style, phrases::Hole};
use temper_client_domain::{
    Domain,
    forge::devices,
    pages::{
        Act as PageAct,
        told::{Act, Refusal, State},
    },
};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, times: &mut Times) {
    settings::main(b);
    settings::open_alone(b, &w.say(Word::ToldTitle, &[]), &w.say(Word::ToldLede, &[]));
    if let Some(page) = root.pages().told() {
        let push = match root.forge().deployment() {
            Some(answer) => Some(answer.push),
            None => None,
        };
        let state = page.state(push);
        if state == State::NotSetUp {
            b.open(Role::Slot);
            b.style(look::SETTINGS_BAND);
            b.style(look::SETTINGS_QUIET_BAND);
            said(b, w, Word::ToldNotSetUp, look::SETTINGS_TAG);
            crate::cards::text(b, Role::Paragraph, &w.say(Word::ToldNotSetUpLine, &[]));
            b.close();
        } else if state != State::Unknown {
            b.open(Role::Slot);
            b.style(look::REMEMBERED_ROWS);
            b.open(Role::Slot);
            b.open(Role::Slot);
            crate::cards::text(b, Role::Text, &w.say(Word::ToldBrowser, &[]));
            let (word, act) = match state {
                State::Cannot { home_screen } => {
                    said(b, w, Word::ToldCannot, look::SETTINGS_DETAIL);
                    if home_screen {
                        said(b, w, Word::ToldHomeScreen, look::SETTINGS_DETAIL);
                    }
                    (None, None)
                }
                State::Blocked => (Some(Word::ToldBlocked), None),
                State::Off => (Some(Word::ToldOff), Some((Word::ToldTurnOn, Act::TurnOn))),
                State::TurningOn => (Some(Word::ToldTurningOn), None),
                State::On { at, .. } => {
                    let day = times.say(w, TimeForm::Day, at);
                    text(b, Role::Text, &w.say(Word::ToldOn, &[Hole::Name(day)]), look::SETTINGS_DETAIL);
                    (None, Some((Word::ToldTurnOff, Act::TurnOff)))
                }
                State::Unknown | State::NotSetUp => (None, None),
            };
            if let Some(word) = word {
                said(b, w, word, look::SETTINGS_DETAIL);
            }
            b.close();
            if let Some((word, act)) = act {
                button(b, w, bindings, word, act);
            }
            if let Some(refusal) = page.refusal() {
                let word = match refusal {
                    Refusal::Off => Word::ToldNotSetUpLine,
                    Refusal::Service => Word::ToldService,
                    Refusal::TooMany => Word::ToldTooMany,
                    Refusal::Malformed => Word::ToldMalformed,
                    Refusal::Person => Word::ToldPerson,
                    Refusal::Blocked => Word::ToldBlocked,
                    Refusal::Failed => Word::ToldFailed,
                    Refusal::Unknown => Word::ToldUnknown,
                };
                // The blocked state already supplies the permission refusal's line.
                if refusal != Refusal::Blocked {
                    said(b, w, word, look::SETTINGS_REFUSAL);
                }
            }
            b.close();
            b.close();
            b.open(Role::Heading { level: 2 });
            b.name(&w.say(Word::ToldDevices, &[]));
            b.text(skein_ui::Run::Plain, &w.say(Word::ToldDevices, &[]));
            b.close();
            if let Some(rows) = page.rows() {
                if rows.is_empty() {
                    crate::cards::text(b, Role::Paragraph, &w.say(Word::ToldEmpty, &[]));
                } else {
                    b.open(Role::Slot);
                    b.style(look::REMEMBERED_ROWS);
                    for row in rows {
                        b.open(Role::Slot);
                        b.key(skein_ui::Key { owner: crate::frame::owner(), kind: 71, a: u64::from(row.number), b: 0 });
                        b.open(Role::Slot);
                        crate::cards::text(b, Role::Text, &row.label);
                        if let Some(current) = page.current()
                            && current.number == row.number
                        {
                            said(b, w, Word::ToldBrowser, look::SETTINGS_TAG);
                        }
                        let day = times.say(w, TimeForm::Day, row.at);
                        let when = w.say(
                            Word::ToldDeviceWhen,
                            &[Hole::Name(devices::host(&row.endpoint).into()), Hole::Name(day)],
                        );
                        text(b, Role::Text, &when, look::SETTINGS_DETAIL);
                        b.close();
                        button(b, w, bindings, Word::ToldRemove, Act::Remove { number: row.number });
                        b.close();
                    }
                    b.close();
                }
            }
        }
    }
    b.close();
    b.close();
}
fn said(b: &mut Builder, w: &Words, word: Word, style: Style) {
    text(b, Role::Text, &w.say(word, &[]), style);
}
fn button(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, act: Act) {
    b.open(Role::Slot);
    b.style(look::REMEMBERED_ACTS);
    b.open(Role::Button);
    b.style(look::SETTINGS_BTN);
    b.style(look::SETTINGS_SMALL);
    b.name(&w.say(word, &[]));
    b.bind(bindings.insert(PageAct::Told(act)));
    b.close();
    b.close();
}
