//! Settings' Models tab (temper/docs/design/client/pages/README.md, The `Models` tab).
use super::settings::{self, Tab};
use crate::{
    accounts::words::{join, provider_words},
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, accounts, choice, models},
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
) {
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
            section(b, root, w, bindings, provider, offered.as_slice(), default, times);
        }
        for member in &me.projects {
            if member.project == project && member.left_out != 0 {
                b.open(Role::Paragraph);
                b.style(look::SETTINGS_FACTS);
                w.text(b, Word::ModelsLeftOut, &[Hole::Number(u64::from(member.left_out))]);
                b.close();
            }
        }
        add(b, root, w, bindings, me, project);
        ask(b, w, bindings, project);
        b.close();
    }
    b.close();
    b.close();
}

#[expect(clippy::too_many_arguments, reason = "provider section carries its offer, default and clock together")]
fn section(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    provider: &c::ProviderOffer,
    models: &[&c::MeModel],
    default: Option<&[u8]>,
    times: &mut Times,
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
        let stock = source(root, provider.provider, name, w, times);
        let line = if paid.is_empty() { stock } else { join(&[&paid, b" ", &stock]) };
        b.open(Role::Text);
        b.style(look::SETTINGS_DETAIL);
        b.name(&line);
        b.text(Run::Plain, &line);
        b.close();
        let page = root.pages().models();
        if models::readable(root.client(), provider.provider) {
            b.open(Role::Button);
            b.style(look::SETTINGS_SMALL);
            b.name(
                &w.say(if page.reading(provider.provider) { Word::ModelsReading } else { Word::ModelsReadAgain }, &[]),
            );
            b.bind(bindings.insert(Act::Models(models::Act::Read(provider.provider))));
            if page.reading(provider.provider) {
                b.state(State::Disabled);
            }
            b.close();
        }
        b.close();
        if let Some(outcome) = page.read_refused(provider.provider) {
            b.open(Role::Slot);
            b.style(look::SETTINGS_REFUSAL);
            crate::confirm::outcome_words(b, w, outcome);
            b.close();
        }
        for model in models {
            row(b, root, w, bindings, model, default == Some(model.name.bytes()), times);
        }
    }
    b.close();
}

fn source(root: &Domain, provider: u32, name: &[u8], w: &Words, times: &mut Times) -> Box<[u8]> {
    let mut read = None;
    let mut failed = None;
    if let Some(me) = root.pages().me(root.client())
        && let Some(document) = accounts::document(root.client())
    {
        for account in &document.accounts {
            if account.provider != Some(provider) || account.holder != c::AccountHolder::Party(me.party.number) {
                continue;
            }
            let mut usable = false;
            for source in &account.sources {
                usable |= source.usable;
            }
            if !usable {
                continue;
            }
            if let Some(list) = &account.list {
                read = read.max(list.read);
                if list.failure.is_some() {
                    failed = failed.max(list.tried);
                }
            }
        }
    }
    let (word, at) = if let Some(at) = failed {
        (Word::ModelsReadFailed, at)
    } else if let Some(at) = read {
        (Word::ModelsReadFrom, at)
    } else {
        return w.say(Word::ModelsStock, &[]);
    };
    let age = times.public_time(w, TimeForm::Ago, at);
    w.say(word, &[Hole::Name(name.into()), Hole::Name(age)])
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

fn row(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    model: &c::MeModel,
    is_default: bool,
    times: &mut Times,
) {
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
    if times.model_new(model) {
        detail = join(&[&detail, " · ".as_bytes(), &w.say(Word::ModelsNew, &[])]);
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
    let name = w.say(Word::ModelsAskPage, &[]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Ask { project } }));
    b.text(Run::Plain, &name);
    b.close();
    b.close();
}

fn add(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, me: &c::Me, project: u32) {
    if me.deployment_role != Some(c::HeldRole::Owner) {
        return;
    }
    let page = root.pages().models();
    let playground = root.forge().playground_provider();
    let selected = page.provider(me, playground);
    settings::section_head(b, &w.say(Word::ModelsAddHeading, &[]));
    b.close();
    b.open(Role::Slot);
    b.style(look::MODELS_ADD);
    b.open(Role::Select);
    b.name(&w.say(Word::ModelsProvider, &[]));
    if page.add_pending() || selected.is_none() {
        b.state(State::Disabled);
    }
    let mut provider_name: Box<[u8]> = Box::default();
    for provider in &me.providers {
        if Some(provider.provider) == playground {
            continue;
        }
        let name = add_provider_name(provider.name.bytes());
        b.open(Role::Option);
        b.name(name);
        b.bind(bindings.insert(Act::Models(models::Act::Provider(provider.provider))));
        if selected == Some(provider.provider) {
            b.state(State::Selected);
            provider_name = name.into();
        }
        b.close();
    }
    b.close();
    b.open(Role::TextField);
    b.state(State::SubmitsOnEnter);
    b.name(&w.say(Word::ModelsName, &[]));
    b.placeholder(&w.say(Word::ModelsNameHint, &[]));
    b.value(page.name(), temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(Act::Models(models::Act::Name(Box::default()))));
    if page.add_pending() {
        b.state(State::Disabled);
    }
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::ModelsAdd, &[]));
    b.style(look::SETTINGS_SMALL);
    b.bind(bindings.insert(Act::Models(models::Act::Add)));
    if !page.can_add(me, project, playground) {
        b.state(State::Disabled);
    }
    b.close();
    b.close();
    detail(b, &w.say(Word::ModelsAddHelp, &[]));
    if page.duplicate(me, project, playground) {
        detail(b, &w.say(Word::ModelsDuplicate, &[Hole::Name(provider_name)]));
    }
    if let Some(provider) = page.added() {
        let mut offered = false;
        if let Some(models) = choice::offer(me, project) {
            for model in models {
                offered |= model.provider == Some(provider);
            }
        }
        if !offered {
            for offer in &me.providers {
                if offer.provider == provider {
                    detail(
                        b,
                        &w.say(Word::ModelsAddedWaiting, &[Hole::Name(add_provider_name(offer.name.bytes()).into())]),
                    );
                }
            }
        }
    }
    if let Some(outcome) = page.add_refused() {
        b.open(Role::Slot);
        b.style(look::SETTINGS_REFUSAL);
        crate::confirm::outcome_words(b, w, outcome);
        b.close();
    }
    b.close();
}
fn detail(b: &mut Builder, words: &[u8]) {
    b.open(Role::Text);
    b.style(look::SETTINGS_DETAIL);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}

fn add_provider_name(name: &[u8]) -> &[u8] {
    if name == b"anthropic" { b"Claude" } else { provider_words(name).0 }
}
