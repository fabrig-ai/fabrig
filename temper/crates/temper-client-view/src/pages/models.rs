//! Settings' Models tab (temper/docs/design/client/README.md, 3.5, The `Models` tab).
use super::settings::{self, Tab};
use crate::{
    accounts::words::{join, provider_words},
    frame::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, accounts, choice, models},
};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    settings::main(b);
    settings::open(b, root, w, bindings, project, Tab::Models);
    if !settings::missing(b, root, w, bindings, project)
        && let Some(me) = root.pages().me(root.client())
        && let Some(models) = choice::offer(me, project)
    {
        let mut default = None;
        for member in &me.projects {
            if member.project == project {
                for preview in &member.defaults {
                    if preview.template == jig_client::shapes::CHAT {
                        default = Some(preview.choice.main.model.bytes());
                    }
                }
            }
        }
        b.open(Role::Slot);
        b.style(look::SETTINGS_BODY);
        for provider in &me.providers {
            let mut offered = List::uncapped();
            for model in models {
                if model.provider == Some(provider.provider) {
                    offered.push(model).expect("uncapped offered models");
                }
            }
            section(b, root, w, bindings, provider, offered.as_slice(), default);
        }
        for member in &me.projects {
            if member.project == project && member.left_out != 0 {
                b.open(Role::Paragraph);
                b.style(look::SETTINGS_FACTS);
                w.text(b, Word::ModelsLeftOut, &[Hole::Number(u64::from(member.left_out))]);
                b.close();
            }
        }
        ask(b, w, bindings, project);
        b.close();
    }
    b.close();
    b.close();
}

fn section(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    provider: &c::ProviderOffer,
    models: &[&c::MeModel],
    default: Option<&[u8]>,
) {
    if models.is_empty() {
        b.open(Role::Slot);
        b.style(look::SETTINGS_SECTION);
        let mut shown = provider.clone();
        if provider.name.bytes() == b"anthropic" {
            shown.name = c::DocumentText::complete(
                c::Text::new(provider_words(provider.name.bytes()).1.into(), 32).expect("Claude display fits"),
            );
        }
        crate::choice::provider_line(b, root, &shown, false, w, bindings);
    } else {
        let name = crate::choice::provider_shown(root, Some(provider.provider), provider.name.bytes(), w);
        let name = if name.as_ref() == b"anthropic" { b"Claude".as_slice() } else { provider_words(&name).0 };
        settings::section_head(b, name);
        b.close();
        b.open(Role::Slot);
        b.style(look::MODELS_SOURCE);
        let paid = pays(root, provider.provider, w);
        let stock = w.say(Word::ModelsStock, &[]);
        let line = if paid.is_empty() { stock } else { join(&[&paid, b" ", &stock]) };
        b.open(Role::Text);
        b.style(look::SETTINGS_DETAIL);
        b.name(&line);
        b.text(Run::Plain, &line);
        b.close();
        b.close();
        for model in models {
            row(b, root, w, bindings, model, default == Some(model.name.bytes()));
        }
    }
    b.close();
}

fn pays(root: &Domain, provider: u32, w: &Words) -> Box<[u8]> {
    let Some(me) = root.pages().me(root.client()) else { return Box::default() };
    let Some(document) = accounts::document(root.client()) else { return Box::default() };
    let mut clauses: Box<[u8]> = Box::default();
    for account in &document.accounts {
        if account.provider != Some(provider) || account.holder != c::AccountHolder::Party(me.party.number) {
            continue;
        }
        let mut usable = false;
        let mut program = false;
        for source in &account.sources {
            usable |= source.usable;
            program |= source.usable && source.kind == c::AccountKind::Program;
        }
        if !usable {
            continue;
        }
        let name = match &account.provider_name {
            Some(name) => name.bytes(),
            None => b"".as_slice(),
        };
        let clause = if program || account.plan && name == b"anthropic" {
            w.say(Word::ModelsSubscription, &[Hole::Name(provider_words(name).1.into())])
        } else if account.plan {
            w.say(Word::ModelsPlan, &[Hole::Name(provider_words(name).1.into())])
        } else {
            w.say(Word::ModelsKey, &[])
        };
        clauses = if clauses.is_empty() { clause } else { join(&[&clauses, b" ", &clause]) };
    }
    clauses
}

fn row(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, model: &c::MeModel, is_default: bool) {
    let terminal = w.shell == Shell::Terminal;
    b.open(if terminal { Role::Paragraph } else { Role::Slot });
    b.style(look::SETTINGS_SETTING);
    b.open(Role::Slot);
    let shown = match &model.shown {
        Some(shown) => shown.bytes(),
        None => model.name.bytes(),
    };
    crate::cards::text(b, Role::Text, shown);
    if terminal {
        b.text(Run::Plain, b" ");
    }
    let mut detail = skein_lib::bytes::copy_of(model.name.bytes());
    if !model.efforts.is_empty()
        && let Some(effort) = &model.effort
    {
        detail = join(&[&detail, " · ".as_bytes(), effort.bytes()]);
    }
    if is_default {
        detail = join(&[&detail, &w.say(Word::ModelsDefault, &[])]);
    }
    match model.source {
        c::ModelSource::Person => detail = join(&[&detail, &w.say(Word::ModelsAdded, &[])]),
        c::ModelSource::Stock | c::ModelSource::List => {}
    }
    if model.estimated {
        detail = join(&[&detail, &w.say(Word::ModelsEstimated, &[])]);
    }
    b.open(Role::Text);
    b.style(look::SETTINGS_DETAIL);
    b.name(&detail);
    b.text(Run::Plain, &detail);
    b.close();
    b.close();
    if terminal {
        b.text(Run::Plain, b" ");
    }
    b.open(Role::Slot);
    b.style(look::SETTINGS_TOGGLE_ROW);
    crate::cards::text(b, Role::Text, &w.say(if model.picker { Word::SettingsOn } else { Word::SettingsOff }, &[]));
    if let Some(me) = root.pages().me(root.client())
        && me.deployment_role == Some(c::HeldRole::Owner)
    {
        b.open(Role::Check);
        b.name(&w.say(Word::ModelsPicker, &[Hole::Name(shown.into())]));
        b.style(look::SETTINGS_SWITCH);
        b.bind(bindings.insert(Act::Models(models::Act::Picker { number: model.number, on: !model.picker })));
        if model.picker {
            b.state(State::Pressed);
        }
        if root.pages().models().pending(model.number) {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    if let Some(outcome) = root.pages().models().refused_at(model.number) {
        b.open(Role::Slot);
        b.style(look::SETTINGS_REFUSAL);
        crate::confirm::outcome_words(b, w, outcome);
        b.close();
    }
    b.close();
}

fn ask(b: &mut Builder, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Paragraph);
    b.style(look::SETTINGS_FACTS);
    b.style(look::MODELS_ASK);
    b.text(Run::Plain, &w.say(Word::ModelsAsk, &[]));
    let name = w.say(Word::ModelsConfigure, &[]);
    crate::link(b, &name, bindings.insert(Act::Go { address: Address::Configure { project } }));
    b.text(Run::Plain, &name);
    b.close();
    b.close();
}
