//! The browser's approval column (temper/docs/design/client/pages/README.md and temper/docs/design/client/view.md, 1 and 6;
//! temper/docs/design/client/web/look.md).
use crate::{
    bindings::Bindings,
    look, outside,
    sign_in::{self, Ways},
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Domain, address,
    approve::{self, Act, State},
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    page: &approve::Page,
    words: &Words,
    ways: &Ways,
    bindings: &mut Bindings,
    times: &mut Times,
) {
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
    let title = words.say(title, &[]);
    outside::column(b, &title, &[look::APPROVE]);
    match page.state() {
        State::Typing { unread } => {
            outside::words(b, words, Word::ApproveTyping, &[]);
            b.open(Role::Form);
            b.name(&words.say(Word::ApproveCode, &[]));
            b.open(Role::TextField);
            b.style(look::APPROVE_FIELD);
            b.name(&words.say(Word::ApproveCode, &[]));
            b.state(skein_ui::State::SubmitsOnEnter);
            b.value(page.words(), temper_client_domain::shown(root));
            b.bind(bindings.insert_approve(Act::Words { words: Box::default() }));
            b.close();
            button(b, words, Word::ApproveCheck, Act::Check, look::OUTSIDE_PRIMARY, bindings);
            if unread {
                b.open(Role::Alert);
                b.name(&words.say(Word::ApproveUnread, &[]));
                words.text(b, Word::ApproveUnread, &[]);
                b.close();
            }
            b.close();
        }
        State::Reading => outside::words(b, words, Word::ApproveReading, &[Hole::Name(shown_code(page))]),
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
            b.style(look::OUTSIDE_ACTS);
            if let State::Deciding { approve, .. } = page.state() {
                let status = if approve { Word::ApproveApproving } else { Word::ApproveRefusing };
                b.open(Role::Status);
                b.name(&words.say(status, &[]));
                words.text(b, status, &[]);
                b.close();
            } else {
                button(b, words, Word::ApproveYes, Act::Approve, look::OUTSIDE_PRIMARY, bindings);
                button(b, words, Word::ApproveNo, Act::Refuse, look::OUTSIDE_SECONDARY, bindings);
            }
            b.close();
        }
        State::Approved => {
            code(b, page);
            outside::words(b, words, Word::ApproveWaiting, &[]);
        }
        State::SignedIn => outside::words(b, words, Word::ApproveSignedInBody, &[]),
        State::Refused => outside::words(b, words, Word::ApproveRefusedBody, &[]),
        State::Expired => outside::words(b, words, Word::ApproveExpiredBody, &[]),
        State::Used => outside::words(b, words, Word::ApproveUsedBody, &[]),
        State::Unknown => {
            outside::words(b, words, Word::ApproveUnknownBody, &[]);
            button(b, words, Word::ApproveType, Act::Type, look::OUTSIDE_PRIMARY, bindings);
        }
        State::SignedOut => {
            if ways.google && ways.origin.is_some() {
                outside::words(b, words, Word::ApproveSignedOutGoogle, &[]);
                let path = address::print_approve(page.code());
                let origin = ways.origin.as_deref().expect("Google's way has an origin");
                b.open(Role::Link);
                b.style(look::OUTSIDE_PRIMARY);
                b.name(&words.say(Word::ContinueGoogle, &[]));
                b.external(&sign_in::google(origin, &path));
                words.text(b, Word::ContinueGoogle, &[]);
                b.close();
            } else {
                outside::words(b, words, Word::ApproveSignedOutBody, &[]);
            }
        }
        State::Busy => {
            outside::words(b, words, Word::ApproveBusyBody, &[]);
            button(b, words, Word::ApproveAgain, Act::Again, look::OUTSIDE_PRIMARY, bindings);
        }
    }
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
fn button(b: &mut Builder, words: &Words, word: Word, act: Act, style: Style, bindings: &mut Bindings) {
    b.open(Role::Button);
    b.style(style);
    b.name(&words.say(word, &[]));
    b.bind(bindings.insert_approve(act));
    b.close();
}
