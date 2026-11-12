//! The terminal login note shared by Model and ConnectModel (client/pages/welcome.md, 1).
use crate::{
    accounts::{Bindings, words::{self, Word as W}},
    look,
    words::{Shell, Word, Words},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{Domain, pages::connect_model::{Act, ConnectModel, LoginState}};

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
    super::connect_model::button(b, bindings, &words::say(w, W::Cancel, &[]), Act::CancelLogin, cancelled, false);
    b.close();
    b.close();
    if w.shell == Shell::Browser {
        b.open(Role::Disclosure);
        b.style(look::FLOW_MUTED);
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
pub(super) fn owns_request(page: &ConnectModel) -> bool {
    let Some(pending) = &page.request.pending else { return false };
    if let jig_client::contract::Ask::AccountChange {
        change: jig_client::contract::AccountChange::LoginProgram { .. } | jig_client::contract::AccountChange::CancelLogin { .. }, ..
    } = &pending.ask {
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
            super::accounts::refused(b, w, refusal);
            b.close();
        }
        LoginState::Idle => {
            if owns_request(page) {
                if let Some(pending) = &page.request.pending
                    && let Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(
                        jig_client::contract::Answer::Refused { refusal: jig_client::contract::Refusal::NoTerminal, .. }
                    ))) = &pending.outcome
                {
                    return;
                }
                super::accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
            }
        }
        LoginState::Waiting { .. } | LoginState::Usable { .. } => {}
    }
}
