//! The terminal login note shared by Model and `ConnectModel` (client/pages/welcome.md, 1).
use crate::{
    accounts::{
        Bindings, Meaning,
        words::{self, Word as W},
    },
    look,
    words::{Shell, Word, Words},
};
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::connect_model::{Act, ConnectModel, LoginState},
};

pub(super) fn waiting(root: &Domain) -> bool {
    let Some(page) = root.pages().connect_model() else { return false };
    match page.login_state(root.client()) {
        LoginState::Waiting { .. } => true,
        LoginState::Idle | LoginState::Usable { .. } | LoginState::NotFinished | LoginState::Refused { .. } => false,
    }
}
pub(super) fn note(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let Some(page) = root.pages().connect_model() else { return };
    let cancelled = match page.login_state(root.client()) {
        LoginState::Waiting { cancelled, .. } => cancelled,
        LoginState::Idle | LoginState::Usable { .. } | LoginState::NotFinished | LoginState::Refused { .. } => return,
    };
    b.open(Role::Slot);
    b.style(look::WELCOME_AWAY);
    if w.shell == Shell::Browser {
        b.open(Role::Slot);
        b.style(look::WELCOME_SPIN);
        b.close();
    }
    b.open(Role::Slot);
    let title = w.say(Word::WelcomeClaudeWaiting, &[]);
    b.open(Role::Status);
    b.name(&title);
    b.open(Role::Text);
    b.run(Run::Strong, &title);
    b.close();
    b.close();
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    let body = w.say(Word::WelcomeClaudeApprove, &[]);
    b.name(&body);
    b.text(Run::Plain, &body);
    b.close();
    super::connect_model::connect_control(
        b,
        root,
        bindings,
        &words::say(w, W::Cancel, &[]),
        Role::Button,
        Meaning::Model(Act::CancelLogin),
        false,
        cancelled,
    );
    b.close();
    b.close();
    if w.shell == Shell::Browser {
        b.open(Role::Disclosure);
        b.style(look::FLOW_MUTED);
        b.style(look::WELCOME_PASTE);
        b.name(&w.say(Word::WelcomeClaudeDisclosure, &[]));
    }
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    let help = w.say(Word::WelcomeClaudeHelp, &[]);
    b.name(&help);
    b.text(Run::Plain, &help);
    b.close();
    if w.shell == Shell::Browser {
        b.close();
    }
}
#[expect(clippy::match_like_matches_macro, reason = "the strict subset keeps the request pattern explicit")]
pub(super) fn owns_request(page: &ConnectModel) -> bool {
    let Some(pending) = &page.request.pending else { return false };
    if let c::Ask::AccountChange {
        change: c::AccountChange::LoginProgram { .. } | c::AccountChange::CancelLogin { .. },
        ..
    } = &pending.ask
    {
        true
    } else {
        false
    }
}
pub(super) fn ending(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    match page.login_state(root.client()) {
        LoginState::NotFinished => {
            b.open(Role::Paragraph);
            b.style(look::FLOW_MUTED);
            let line = w.say(Word::WelcomeClaudeNotFinished, &[]);
            b.name(&line);
            b.text(Run::Plain, &line);
            b.close();
        }
        LoginState::Refused { refusal } => {
            b.open(Role::Paragraph);
            b.style(look::FLOW_MUTED);
            b.open(Role::Text);
            match refusal {
                c::AccountRefusal::ProgramNameReserved => {
                    b.text(Run::Plain, &crate::cards::refusal_words(w, &c::Refusal::ProgramNameReserved));
                }
                c::AccountRefusal::NotLimited
                | c::AccountRefusal::Uses
                | c::AccountRefusal::NotKept { .. }
                | c::AccountRefusal::Limit { .. }
                | c::AccountRefusal::Held { .. }
                | c::AccountRefusal::Provider { .. }
                | c::AccountRefusal::Subject
                | c::AccountRefusal::Unknown
                | c::AccountRefusal::Numbers
                | c::AccountRefusal::Kind { .. }
                | c::AccountRefusal::Closing
                | c::AccountRefusal::NotStarted
                | c::AccountRefusal::NotSignedIn
                | c::AccountRefusal::NotOurs
                | c::AccountRefusal::Cancelled
                | c::AccountRefusal::Moving => super::accounts::refused(b, w, refusal),
            }
            b.close();
            b.close();
        }
        LoginState::Idle => {
            if !owns_request(page) {
                return;
            }
            let Some(pending) = &page.request.pending else { return };
            if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal, ..
            }))) = &pending.outcome
            {
                if *refusal == c::Refusal::NoTerminal || *refusal == c::Refusal::LoginUnderWay {
                    return;
                }
                let line = match super::accounts::act_refusal(&pending.ask, refusal) {
                    Some((word, name)) => {
                        let hole = match name {
                            Some(name) => Some(Hole::Name(skein_lib::bytes::copy_of(name))),
                            None => None,
                        };
                        words::say(w, word, hole.as_slice())
                    }
                    None => crate::cards::refusal_words(w, refusal),
                };
                b.open(Role::Paragraph);
                b.style(look::FLOW_MUTED);
                b.name(&line);
                b.text(Run::Plain, &line);
                b.close();
            } else if let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_)))) =
                &pending.outcome
            {
                // An acknowledgement is not the login's end; Cancelled leaves no line.
            } else if pending.outcome.is_some() {
                super::accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
            }
        }
        LoginState::Waiting { .. } | LoginState::Usable { .. } => {}
    }
}
