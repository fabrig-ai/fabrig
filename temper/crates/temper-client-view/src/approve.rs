//! The browser's approval card (temper/docs/design/client/README.md, 3.5 and 5.12;
//! temper/docs/design/client/browser.md, 2.5).
use crate::{
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Domain,
    approve::{self, Act, State},
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    page: &approve::Page,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SIGN_IN);
    b.style(look::APPROVE);
    b.open(Role::Slot);
    b.style(look::CARD);
    b.open(Role::Paragraph);
    b.style(look::BRAND);
    words.text(b, Word::Brand, &[]);
    b.close();
    let title = match page.state() {
        State::Typing { .. } | State::Reading | State::Asking { .. } | State::Deciding { .. } => Word::ApproveTitle,
        State::Approved => Word::ApproveApproved,
        State::SignedIn => Word::ApproveSignedIn,
        State::Refused => Word::ApproveRefused,
        State::Expired => Word::ApproveExpired,
        State::Used => Word::ApproveUsed,
        State::Unknown => Word::ApproveUnknown,
        State::SignedOut => Word::ApproveSignedOut,
        State::Busy => Word::ApproveBusy,
    };
    b.open(Role::Heading { level: 1 });
    b.name(&words.say(title, &[]));
    words.text(b, title, &[]);
    b.close();
    match page.state() {
        State::Typing { unread } => {
            paragraph(b, words, Word::ApproveTyping, &[]);
            b.open(Role::Form);
            b.style(look::APPROVE_ACTS);
            b.open(Role::TextField);
            b.name(&words.say(Word::ApproveCode, &[]));
            b.state(skein_ui::State::SubmitsOnEnter);
            b.value(page.words(), temper_client_domain::shown(root));
            b.bind(bindings.insert_approve(Act::Words { words: Box::default() }));
            b.close();
            button(b, words, Word::ApproveCheck, Act::Check, bindings);
            b.close();
            if unread {
                b.open(Role::Alert);
                b.name(&words.say(Word::ApproveUnread, &[]));
                words.text(b, Word::ApproveUnread, &[]);
                b.close();
            }
        }
        State::Reading => paragraph(b, words, Word::ApproveReading, &[Hole::Name(shown_code(page))]),
        State::Asking { asked } | State::Deciding { asked, .. } => {
            code(b, page);
            let ago = times.say(words, TimeForm::Ago, asked);
            b.open(Role::Paragraph);
            b.style(look::APPROVE_ASKED);
            words.text(b, Word::ApproveAsked, &[Hole::Name(ago)]);
            b.close();
            b.open(Role::Paragraph);
            b.style(look::APPROVE_ASKED);
            words.text(b, Word::ApproveWarning, &[]);
            b.close();
            b.open(Role::Slot);
            b.style(look::APPROVE_ACTS);
            if let State::Deciding { approve, .. } = page.state() {
                let status = if approve { Word::ApproveApproving } else { Word::ApproveRefusing };
                b.open(Role::Status);
                b.name(&words.say(status, &[]));
                words.text(b, status, &[]);
                b.close();
            } else {
                button(b, words, Word::ApproveYes, Act::Approve, bindings);
                button(b, words, Word::ApproveNo, Act::Refuse, bindings);
            }
            b.close();
        }
        State::Approved => {
            code(b, page);
            paragraph(b, words, Word::ApproveWaiting, &[]);
        }
        State::SignedIn => paragraph(b, words, Word::ApproveSignedInBody, &[]),
        State::Refused => paragraph(b, words, Word::ApproveRefusedBody, &[]),
        State::Expired => paragraph(b, words, Word::ApproveExpiredBody, &[]),
        State::Used => paragraph(b, words, Word::ApproveUsedBody, &[]),
        State::Unknown => {
            paragraph(b, words, Word::ApproveUnknownBody, &[]);
            b.open(Role::Slot);
            b.style(look::APPROVE_ACTS);
            button(b, words, Word::ApproveType, Act::Type, bindings);
            b.close();
        }
        State::SignedOut => paragraph(b, words, Word::ApproveSignedOutBody, &[]),
        State::Busy => {
            paragraph(b, words, Word::ApproveBusyBody, &[]);
            b.open(Role::Slot);
            b.style(look::APPROVE_ACTS);
            button(b, words, Word::ApproveAgain, Act::Again, bindings);
            b.close();
        }
    }
    b.close();
    b.close();
}
fn shown_code(page: &approve::Page) -> Box<[u8]> {
    Box::from(page.code().expect("the approval state carries a code").shown())
}
fn code(b: &mut Builder, page: &approve::Page) {
    let shown = shown_code(page);
    b.open(Role::Paragraph);
    b.style(look::APPROVE_CODE);
    b.name(&shown);
    b.text(Run::Plain, &shown);
    b.close();
}
fn paragraph(b: &mut Builder, words: &Words, word: Word, holes: &[Hole]) {
    b.open(Role::Paragraph);
    words.text(b, word, holes);
    b.close();
}
fn button(b: &mut Builder, words: &Words, word: Word, act: Act, bindings: &mut Bindings) {
    b.open(Role::Button);
    if act == Act::Approve {
        b.style(look::APPROVE_PRIMARY);
    }
    b.name(&words.say(word, &[]));
    b.bind(bindings.insert_approve(act));
    b.close();
}
