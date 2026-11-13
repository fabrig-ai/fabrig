//! The welcome's Model shapes (temper/docs/design/client/pages/README.md).
use super::{accounts, connect_model as model, sign_in_wait};
use crate::{
    accounts::{Bindings, Meaning},
    flow::{self, Control},
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain, Step,
    pages::{
        accounts::{self as domain_accounts, Way},
        connect_model::Act,
        welcome::{self, ModelStep},
    },
};

pub(super) fn said(w: &Words, root: &Domain, word: Word) -> Box<[u8]> {
    let Some(page) = root.pages().connect_model() else { return w.say(word, &[]) };
    if [Word::WelcomePasteOpen, Word::WelcomeAwayTitle, Word::WelcomeAwayOwn].contains(&word) {
        w.say(word, &[Hole::Name(crate::words::shown(accounts::paste_plan(root, &page.request), w))])
    } else {
        w.say(word, &[])
    }
}
fn text(b: &mut Builder, root: &Domain, w: &Words, role: Role, word: Word) {
    crate::cards::text(b, role, &said(w, root, word));
}
fn acts(b: &mut Builder) {
    b.open(Role::Slot);
    b.style(look::WELCOME_ACTS);
}
fn panel(b: &mut Builder) {
    b.open(Role::Slot);
    b.style(look::WELCOME_PANEL);
}
fn note(b: &mut Builder, root: &Domain, w: &Words) {
    b.open(Role::Slot);
    b.style(look::WELCOME_AWAY);
    if w.shell == Shell::Browser {
        b.open(Role::Slot);
        b.style(look::WELCOME_SPIN);
        b.close();
    }
    b.open(Role::Slot);
    let title = said(w, root, Word::WelcomeFinishing);
    b.open(Role::Status);
    b.name(&title);
    b.open(Role::Text);
    b.run(Run::Strong, &title);
    b.close();
    b.close();
    b.close();
    b.close();
}
fn usable(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    text(b, root, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
    b.open(Role::Slot);
    b.style(look::WELCOME_FORGE_STATUS);
    b.style(look::FLOW_DONE);
    let account = match accounts::first_usable_short(root, w) {
        Some(account) => account,
        None => crate::choice::new_chat_model(root, w),
    };
    b.open(Role::Status);
    b.name(&account);
    if w.shell == Shell::Browser {
        b.open(Role::Slot);
        b.style(look::WELCOME_OK);
        b.text(Run::Plain, "✓".as_bytes());
        b.close();
    } else {
        b.text(Run::Plain, "✓ ".as_bytes());
    }
    if w.shell == Shell::Browser {
        let kind = match accounts::first_usable_kind(root, w) {
            Some(kind) => kind,
            None => account.clone(),
        };
        b.open(Role::Text);
        b.run(Run::Strong, &kind);
        b.close();
        if let Some(rest) = account.strip_prefix(kind.as_ref())
            && !rest.is_empty()
        {
            b.open(Role::Slot);
            b.style(look::FLOW_MUTED);
            b.text(Run::Plain, rest.strip_prefix(" · ".as_bytes()).unwrap_or(rest));
            b.close();
        }
    } else {
        b.text(Run::Plain, &account);
    }
    b.close();
    b.close();
    acts(b);
    if let Some(to) = bindings.insert(Meaning::Go(Address::Welcome { step: Step::Ready })) {
        flow::control(b, Control::PrimaryLink { name: &w.say(Word::WelcomeContinue, &[]), to });
    }
    b.close();
}
fn key(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, way: Way, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    let busy = page.request.busy(root.client());
    let Some(me) = domain_accounts::current_me(root.client()) else { return };
    let (brand, _) = model::names(me, way);
    crate::cards::text(b, Role::Heading { level: 1 }, &model::key_heading(brand, w));
    panel(b);
    crate::cards::text(b, Role::Paragraph, &crate::accounts::words::key_description(brand, w));
    b.open(Role::Form);
    b.style(look::WELCOME_FIELD);
    model::key_field(b, root, bindings, busy, &w.say(Word::WelcomeKeyField, &[]));
    b.close();
    acts(b);
    model::button(
        b,
        bindings,
        &w.say(Word::ModelAddKey, &[]),
        Act::Submit { name: root.pages().account_name() },
        busy || !page.request.has_key(),
        true,
    );
    b.close();
    b.open(Role::Paragraph);
    b.style(look::FLOW_FACTS);
    w.text(b, Word::WelcomeKeyFacts, &[Hole::Name(crate::words::shown(brand, w))]);
    b.close();
    accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
    b.close();
}
fn sign_in(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, way: Way, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = domain_accounts::current_me(root.client()) else { return };
    crate::cards::text(b, Role::Heading { level: 1 }, &model::signing_heading(root, w, way));
    panel(b);
    let ways = domain_accounts::ways(&me.providers, root.forge().playground_provider(), page.holder);
    if model::ready(root, way) {
        model::ready_body(b, root, w, bindings, way, &ways);
    } else {
        model::choice(b, root, w, bindings, way, &ways);
        if model::request_for_way(page, way) {
            accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
        }
    }
    acts(b);
    b.close();
    b.close();
}
#[expect(clippy::match_like_matches_macro, reason = "the strict subset keeps the request pattern explicit")]
fn program(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, way: Way, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    text(b, root, w, Role::Heading { level: 1 }, Word::WelcomeClaudeTitle);
    panel(b);
    text(b, root, w, Role::Paragraph, Word::WelcomeClaudeBody);
    crate::cards::text(b, Role::Paragraph, &model::command_prompt(root, w));
    super::welcome_copy::block(
        b,
        root,
        w,
        bindings,
        &model::printed_command(root, w, &page.program_name(root.client())),
    );
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    w.text(b, Word::WelcomeClaudeNote, &[]);
    b.close();
    let check_outcome = if let Some(pending) = &page.request.pending {
        if let jig_client::contract::Ask::AccountChange {
            change: jig_client::contract::AccountChange::CheckProgram { .. },
            ..
        } = pending.ask
        {
            true
        } else {
            false
        }
    } else {
        false
    };
    if !check_outcome {
        accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
    }
    acts(b);
    model::button(
        b,
        bindings,
        &w.say(Word::WelcomeClaudeSignedIn, &[]),
        Act::SignedIn { name: root.pages().account_name(), way },
        page.request.busy(root.client()),
        true,
    );
    b.close();
    b.close();
}

pub(super) fn step(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    let Some(state) = welcome::model_step(root) else { return };
    let Some(page) = root.pages().connect_model() else { return };
    match state {
        ModelStep::Usable => usable(b, root, w, bindings),
        ModelStep::TerminalLogin => {
            text(b, root, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
            super::claude_login::note(b, root, w, bindings);
        }
        ModelStep::Choose => {
            text(b, root, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
            b.open(Role::Paragraph);
            b.style(look::WELCOME_LEAD);
            b.name(&w.say(Word::WelcomeModelLead, &[]));
            w.text(b, Word::WelcomeModelLead, &[]);
            b.close();
            model::choose_body(b, root, w, bindings, text_per_node);
        }
        ModelStep::Away { .. } => {
            text(b, root, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
            sign_in_wait::waiting(b, root, w, bindings, &page.request, sign_in_wait::tab_refused(page), text_per_node);
        }
        ModelStep::Finishing { .. } => {
            text(b, root, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
            note(b, root, w);
        }
        ModelStep::Key { way } => key(b, root, w, bindings, way, text_per_node),
        ModelStep::SignIn { way } => sign_in(b, root, w, bindings, way, text_per_node),
        ModelStep::Program { way } => program(b, root, w, bindings, way, text_per_node),
    }
}
