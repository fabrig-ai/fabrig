//! The model and effort at the composer (client/pages/README.md).
use super::{
    Act, Address, Bindings, Box, Builder, Domain, Hole, Role, State, W, Words, account_link, account_row, c, can_send,
    changeable, choice, control, line, model_shown, provider_shown, shown,
};
use crate::look;
use jig_client::thread::Thread;
use skein_ui::Run;
use temper_client_domain::pages::choice::Act as C;

fn provider_line(
    b: &mut Builder,
    root: &Domain,
    provider: &c::ProviderOffer,
    w: &Words,
    bindings: &mut Bindings,
    terminal: bool,
) {
    use crate::accounts::words::{self, Word as A};
    let configured = shown(&provider_shown(root, Some(provider.provider), provider.name.bytes(), w), w);
    let name = words::provider_words(&configured).0;
    let name = if provider.name.bytes() == b"anthropic" { b"Claude" } else { name };
    if terminal {
        b.open(Role::MenuItem);
        b.name(name);
    } else {
        b.open(Role::Slot);
        b.style(look::MODEL_FROM);
        b.text(Run::Strong, name);
    }
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let doc = temper_client_domain::pages::accounts::document(root.client());
    let mut held_account = false;
    if let Some(doc) = doc {
        for account in &doc.accounts {
            if account.provider != Some(provider.provider) || account.holder != c::AccountHolder::Party(me.party.number)
            {
                continue;
            }
            held_account = true;
            let Some(source) = account.sources.get(0) else { continue };
            let fix = match source.state {
                c::AccountState::SignInAgain => Some(match source.kind {
                    c::AccountKind::Program => words::say(w, A::CheckAgain, &[]),
                    c::AccountKind::Key => words::say(w, A::Replace, &[]),
                    c::AccountKind::SignIn | c::AccountKind::None => w.say(W::WelcomeSignInAgain, &[]),
                }),
                c::AccountState::Misconfigured | c::AccountState::Unloaded { .. }
                    if source.kind == c::AccountKind::Key =>
                {
                    Some(words::say(w, A::Replace, &[]))
                }
                c::AccountState::Later { .. } => Some(words::say(w, A::Resume, &[])),
                c::AccountState::Moving
                | c::AccountState::Asking
                | c::AccountState::Granted
                | c::AccountState::Misconfigured
                | c::AccountState::Unloaded { .. }
                | c::AccountState::Cancelled
                | c::AccountState::Closing => None,
            };
            if let Some(fix) = fix {
                let fix = words::join(&[&fix, " ›".as_bytes()]);
                if terminal {
                    b.description(&fix);
                    b.bind(bindings.insert(Act::Go { address: Address::Accounts }));
                } else {
                    account_link(b, bindings, &fix);
                }
                b.close();
                return;
            }
        }
    }
    if held_account {
        if terminal {
            b.description(&w.say(W::AccountReadinessUnknown, &[]));
            b.state(State::Disabled);
        } else {
            line(b, w, W::AccountReadinessUnknown, &[]);
        }
    } else if terminal {
        b.description(&w.say(W::ConnectPlanOrKey, &[]));
        b.bind(bindings.insert(Act::Go { address: Address::ConnectModel { way: None } }));
    } else {
        control(
            b,
            bindings,
            Role::Link,
            &w.say(W::ConnectPlanOrKey, &[]),
            Act::Go { address: Address::ConnectModel { way: None } },
            false,
        );
    }
    b.close();
}

/// The disclosure, fixed words, or nothing without a usable model.
pub(crate) fn picker(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings) {
    if let Some(chat) = root.pages().chat()
        && chat.talk().is_some()
    {
        return;
    }
    if !can_send(root) {
        return;
    }
    let line = closed_line(root, task, w);
    if changeable(root, task) {
        b.open(Role::Disclosure);
        b.style(look::MODEL);
        b.name(&w.say(W::Model, &[]));
        b.description(&line);
        b.state(State::FoldsAway);
        b.open(Role::Slot);
        b.style(look::MODEL_MENU);
        menu(b, root, task, w, bindings);
        b.close();
    } else {
        b.open(Role::Paragraph);
        b.style(look::MODEL);
        b.style(look::MODEL_FIXED);
        b.name(&line);
        b.text(Run::Plain, &line);
    }
    b.close();
}

/// The model and effort the next run takes (client/pages/README.md).
pub(crate) fn closed_line(root: &Domain, task: Option<&c::Task>, w: &Words) -> Box<[u8]> {
    let me = temper_client_domain::pages::accounts::current_me(root.client());
    let draft = root.pages().choice();
    let choice = match me {
        Some(me) => draft.chosen(me),
        None => None,
    }
    .or(match task {
        Some(task) => task.llm.as_ref(),
        None => None,
    });
    // Ready has no choice context; it previews the same chat template.
    let choice = choice.or(match (me, root.pages().project()) {
        (Some(me), Some(project)) => match jig_client::shapes::template(me, project, jig_client::shapes::CHAT) {
            Some(template) => {
                template.llm.as_ref().or(jig_client::shapes::resolved_default(me, project, jig_client::shapes::CHAT))
            }
            None => None,
        },
        (Some(_), None) | (None, _) => None,
    });
    let Some(choice) = choice else { return w.say(W::DefaultModel, &[]) };
    let models = match (me, root.pages().project()) {
        (Some(me), Some(project)) => choice::offer(me, project),
        (Some(_), None) | (None, _) => None,
    };
    let mut selected = None;
    if let (Some(me), Some(models)) = (me, models) {
        if let Some(index) = draft.selected(me) {
            selected = models.get(index);
        }
        if selected.is_none() {
            for model in models {
                if model.name == choice.main.model {
                    selected = Some(model);
                    break;
                }
            }
        }
    }
    let effort = choice.main.effort.as_ref().or(match selected {
        Some(model) => model.effort.as_ref(),
        None => None,
    });
    shown_line(root, selected, choice.main.model.bytes(), effort, w)
}
#[expect(clippy::manual_find, reason = "the programming model uses bounded loops, not closures")]
fn offered_model<'a>(models: Option<&'a skein_lib::List<c::MeModel>>, name: &c::Text) -> Option<&'a c::MeModel> {
    for model in models? {
        if model.name == *name {
            return Some(model);
        }
    }
    None
}
/// One model and effort in the picker's shown names (client/pages/README.md): the offered
/// name, "Claude " before an `anthropic` model's, and no effort for the playground's.
fn shown_line(
    root: &Domain,
    offered: Option<&c::MeModel>,
    model: &[u8],
    effort: Option<&c::Text>,
    w: &Words,
) -> Box<[u8]> {
    let mut name = match offered {
        Some(model) => model_shown(model, w),
        None => shown(&w.model_name(model), w),
    };
    let mut playground = false;
    if let Some(model) = offered {
        playground = model.provider.is_some() && model.provider == root.forge().playground_provider();
        if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) {
            for provider in &me.providers {
                if model.provider == Some(provider.provider) && provider.name.bytes() == b"anthropic" {
                    name = crate::accounts::words::join(&[b"Claude ", &name]);
                }
            }
        }
    }
    if !playground && let Some(effort) = effort {
        name = w.say(W::ModelEffort, &[Hole::Name(name), Hole::Name(shown(effort.bytes(), w))]);
    }
    name
}
/// What a page with no composer says it ran on (client/pages/README.md); none until its
/// conversation is read to its end.
pub(crate) fn ran_line(root: &Domain, task: &c::Task, thread: &Thread, w: &Words) -> Option<Box<[u8]>> {
    if !thread.caught_up() {
        return None;
    }
    let me = temper_client_domain::pages::accounts::current_me(root.client());
    let models = match me {
        Some(me) => choice::offer(me, task.project),
        None => None,
    };
    if let Some(llm) = &thread.latest_llm {
        return Some(shown_line(root, offered_model(models, &llm.model), llm.model.bytes(), llm.effort.as_ref(), w));
    }
    let mut chosen = task.llm.as_ref();
    if chosen.is_none()
        && let Some(me) = me
    {
        chosen = root.pages().choice().chosen(me);
    }
    let Some(chosen) = chosen else { return Some(w.say(W::UsingDefault, &[])) };
    let offered = offered_model(models, &chosen.main.model);
    let effort = chosen.main.effort.as_ref().or(match offered {
        Some(model) => model.effort.as_ref(),
        None => None,
    });
    Some(shown_line(root, offered, chosen.main.model.bytes(), effort, w))
}

fn menu(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings) {
    let draft = root.pages().choice();
    if task.is_some() || draft.outcome.is_some() {
        b.open(Role::Paragraph);
        b.style(look::MODEL_NOTE);
        if draft.pending.is_some() {
            b.name(&w.say(W::ModelChanging, &[]));
            w.text(b, W::ModelChanging, &[]);
        } else if let Some(outcome) = &draft.outcome {
            crate::pages::chats::outcome_text(b, w, outcome, 0);
        } else {
            b.name(&w.say(W::ModelNextTurn, &[]));
            w.text(b, W::ModelNextTurn, &[]);
        }
        b.close();
    }
    if draft.all {
        all_models(b, root, w, bindings);
        return;
    }
    groups(b, root, w, bindings, false);
    effort(b, root, w, bindings);
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let Some(models) = choice::offer(me, draft.project().expect("picker has project")) else { return };
    let mut fewer = false;
    for model in models {
        fewer |= !model.picker;
    }
    if fewer {
        b.open(Role::Button);
        b.style(look::MODEL_ALL);
        b.name(&w.say(W::AllModels, &[]));
        b.bind(bindings.insert(Act::Choice(C::All)));
        if w.shell == crate::words::Shell::Terminal {
            w.text(b, W::AllModels, &[]);
            w.text(b, W::ToolSeparator, &[]);
        }
        b.open(Role::Slot);
        b.style(look::MODEL_TAG);
        b.text(Run::Plain, skein_lib::Decimal::of(u64::from(models.len())).as_bytes());
        b.close();
        b.close();
    }
}
fn all_models(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    b.open(Role::Button);
    b.style(look::MODEL_BACK);
    b.name(&w.say(W::ModelsBack, &[]));
    b.bind(bindings.insert(Act::Choice(C::Back)));
    b.close();
    let draft = root.pages().choice();
    b.open(Role::TextField);
    b.style(look::MODEL_FIND);
    b.name(&w.say(W::FindModel, &[]));
    b.placeholder(&w.say(W::FindModel, &[]));
    // Consume the form's native Enter. This field's Submit has no page action.
    b.state(State::SubmitsOnEnter);
    b.value(&draft.find, 0);
    b.bind(bindings.insert(Act::Choice(C::Find { words: Box::default() })));
    b.close();
    b.open(Role::Slot);
    b.style(look::MODEL_LONG);
    groups(b, root, w, bindings, true);
    b.close();
    crate::link(
        b,
        &w.say(W::ModelsAccounts, &[]),
        bindings.insert(Act::Go { address: Address::Models { project: draft.project().expect("picker has project") } }),
    );
    b.style(look::MODEL_ALL);
    w.text(b, W::ModelsAccounts, &[]);
    b.close();
}
#[expect(clippy::manual_find, reason = "the programming model uses bounded loops, not closures")]
fn provider(root: &Domain, number: Option<u32>) -> Option<&c::ProviderOffer> {
    let me = temper_client_domain::pages::accounts::current_me(root.client())?;
    for provider in &me.providers {
        if number == Some(provider.provider) {
            return Some(provider);
        }
    }
    None
}
fn pays(root: &Domain, number: u32, provider: Option<&c::ProviderOffer>, w: &Words) -> Box<[u8]> {
    let Some(account) = account_row(root, number) else { return Box::default() };
    let mut source = account.sources.get(0);
    for candidate in &account.sources {
        if candidate.usable {
            source = Some(candidate);
            break;
        }
    }
    let Some(source) = source else { return Box::default() };
    match source.kind {
        c::AccountKind::Program => w.say(W::ModelSubscription, &[]),
        c::AccountKind::SignIn => match provider {
            Some(provider) if provider.name.bytes() == b"anthropic" => w.say(W::ModelSubscription, &[]),
            Some(provider) => crate::accounts::words::join(&[
                &w.say(W::ModelPlan, &[]),
                &shown(crate::accounts::words::provider_words(provider.name.bytes()).1, w),
                &w.say(W::ModelPlanSuffix, &[]),
            ]),
            None => Box::default(),
        },
        c::AccountKind::Key => w.say(W::ModelApiKey, &[]),
        c::AccountKind::None => Box::default(),
    }
}
fn payer(
    root: &Domain,
    account: u32,
    provider: Option<&c::ProviderOffer>,
    accounts: &skein_lib::List<u32>,
    w: &Words,
) -> Box<[u8]> {
    let mut clause = pays(root, account, provider, w);
    for other in accounts {
        if *other != account
            && clause == pays(root, *other, provider, w)
            && let Some(row) = account_row(root, account)
        {
            clause = shown(crate::accounts::words::who(row.email(), row.display.bytes()), w);
            break;
        }
    }
    clause
}
fn heading(
    b: &mut Builder,
    root: &Domain,
    number: Option<u32>,
    account: Option<u32>,
    accounts: &skein_lib::List<u32>,
    w: &Words,
) {
    let provider = provider(root, number);
    b.open(Role::Slot);
    b.style(look::MODEL_FROM);
    if let Some(provider) = provider {
        let name = shown(&provider_shown(root, number, provider.name.bytes(), w), w);
        b.text(
            Run::Strong,
            if provider.name.bytes() == b"anthropic" {
                b"Claude"
            } else {
                crate::accounts::words::provider_words(&name).0
            },
        );
    }
    if let Some(account) = account {
        b.text(Run::Plain, &payer(root, account, provider, accounts, w));
    }
    b.close();
}
pub(crate) fn terminal_models(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    for choice in &crate::shared::models(root, None, w) {
        terminal_option(b, root, choice, true, bindings);
    }
    provider_lines(b, root, w, bindings, true);
}
fn groups(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, all: bool) {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let draft = root.pages().choice();
    let Some(models) = choice::offer(me, draft.project().expect("picker has project")) else { return };
    for group in &model_groups(models) {
        let accounts = group_accounts(models, group.0);
        heading(b, root, group.0, group.1, &accounts, w);
        rows(b, root, *group, accounts.len() > 1, all, w, bindings);
    }
    provider_lines(b, root, w, bindings, false);
}
fn provider_lines(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, terminal: bool) {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let Some(project) = root.pages().choice().project() else { return };
    let Some(models) = choice::offer(me, project) else { return };
    let ways = temper_client_domain::pages::accounts::ways(
        &me.providers,
        root.forge().playground_provider(),
        c::AskAccountHolder::Own,
    );
    for offer in &me.providers {
        if offer.name.bytes() == crate::words::SCRIPTED_MODEL && !root.forge().playground_running() {
            continue;
        }
        let mut offered = false;
        for model in models {
            offered |= model.provider == Some(offer.provider);
        }
        let mut connectable = false;
        for way in &ways {
            connectable |= way.provider() == offer.provider;
        }
        if !offered && connectable {
            provider_line(b, root, offer, w, bindings, terminal);
        }
    }
}
fn rows(
    b: &mut Builder,
    root: &Domain,
    group: (Option<u32>, Option<u32>),
    multi: bool,
    all: bool,
    w: &Words,
    bindings: &mut Bindings,
) {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let draft = root.pages().choice();
    let Some(models) = choice::offer(me, draft.project().expect("picker has project")) else { return };
    b.open(Role::List);
    for (model, index) in models.iter().zip(0u32..) {
        let fits = match group.1 {
            Some(account) => model.accounts.as_slice().contains(&account),
            None => model.accounts.is_empty(),
        };
        if model.provider == group.0 && fits && (all || model.picker) && (!all || draft.finds(model)) {
            row(b, root, model, index, if multi { group.1 } else { None }, group.1, w, bindings, all);
        }
    }
    b.close();
}
#[expect(clippy::too_many_arguments, reason = "a row carries the model, account heading and selection together")]
fn row(
    b: &mut Builder,
    root: &Domain,
    model: &c::MeModel,
    index: u32,
    preference: Option<u32>,
    account: Option<u32>,
    w: &Words,
    bindings: &mut Bindings,
    all: bool,
) {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let draft = root.pages().choice();
    let chosen = match draft.chosen(me) {
        Some(choice) => choice.account.or(model.accounts.get(0).copied()),
        None => None,
    };
    b.open(Role::Item);
    b.open(Role::Button);
    b.name(&model_shown(model, w));
    b.bind(bindings.insert(Act::Choice(C::Model { index, account: preference })));
    if draft.pending.is_some() {
        b.state(State::Disabled);
    }
    if draft.selected(me) == Some(index) && chosen == account {
        b.state(State::Pressed);
    }
    if w.shell == crate::words::Shell::Terminal {
        b.text(Run::Plain, &model_shown(model, w));
    }
    if draft.default_row(me) == Some(index) && account == model.accounts.get(0).copied() {
        b.open(Role::Slot);
        b.style(look::MODEL_TAG);
        if w.shell == crate::words::Shell::Terminal {
            w.text(b, W::ToolSeparator, &[]);
        }
        w.text(b, W::ModelDefault, &[]);
        b.close();
    }
    if all && bindings.model_times.model_new(model) {
        b.open(Role::Slot);
        b.style(look::MODEL_TAG);
        if w.shell == crate::words::Shell::Terminal {
            w.text(b, W::ToolSeparator, &[]);
        }
        w.text(b, W::ModelsNew, &[]);
        b.close();
    }
    b.close();
    b.close();
}
pub(crate) fn terminal_efforts(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    for choice in &crate::shared::efforts(root, None, w) {
        terminal_option(b, root, choice, false, bindings);
    }
}
fn effort(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    efforts_at(b, root, w, bindings);
}
fn efforts_at(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let draft = root.pages().choice();
    let Some(index) = draft.selected(me) else { return };
    let Some(models) = choice::offer(me, draft.project().expect("picker has project")) else { return };
    let Some(model) = models.get(index) else { return };
    if model.efforts.is_empty() {
        return;
    }
    b.open(Role::Group);
    b.style(look::MODEL_EFFORT);
    b.name(&w.say(W::Effort, &[]));
    let label = w.say(W::Effort, &[]);
    b.text(
        Run::Plain,
        &if w.shell == crate::words::Shell::Terminal { crate::accounts::words::join(&[&label, b":"]) } else { label },
    );
    b.open(Role::Slot);
    for (effort, index) in model.efforts.iter().zip(0u32..) {
        let selected = match draft.chosen(me) {
            Some(choice) => {
                let chosen = choice.main.effort.as_ref();
                chosen == Some(effort)
            }
            None => false,
        };
        control(b, bindings, Role::Button, effort.bytes(), Act::Choice(C::Effort { index }), selected);
    }
    b.close();
    b.close();
}

// Account groups follow their first appearance in the offer (client/pages/README.md, The work's account and model).
fn model_groups(models: &skein_lib::List<c::MeModel>) -> skein_lib::List<(Option<u32>, Option<u32>)> {
    let mut capacity = models.len();
    for model in models {
        capacity = capacity.checked_add(model.accounts.len()).expect("wire offer fits");
    }
    let mut groups = skein_lib::List::with_capacity(capacity);
    for model in models {
        if model.accounts.is_empty() {
            let group = (model.provider, None);
            if !groups.as_slice().contains(&group) {
                groups.push(group).expect("offer groups bounded");
            }
        } else {
            for account in &model.accounts {
                let group = (model.provider, Some(*account));
                if !groups.as_slice().contains(&group) {
                    groups.push(group).expect("offer groups bounded");
                }
            }
        }
    }
    groups
}
fn group_accounts(models: &skein_lib::List<c::MeModel>, provider: Option<u32>) -> skein_lib::List<u32> {
    let mut capacity = 0_u32;
    for model in models {
        capacity = capacity.checked_add(model.accounts.len()).expect("wire offer fits");
    }
    let mut accounts = skein_lib::List::with_capacity(capacity);
    for model in models {
        if model.provider == provider {
            for account in &model.accounts {
                if !accounts.as_slice().contains(account) {
                    accounts.push(*account).expect("wire accounts bounded");
                }
            }
        }
    }
    accounts
}
/// The options of the terminal model list (temper/docs/design/client/pages/README.md).
pub(crate) fn model_choices(root: &Domain, w: &Words) -> Box<[crate::shared::Choice]> {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return Box::default() };
    let draft = root.pages().choice();
    let Some(project) = draft.project() else { return Box::default() };
    let Some(models) = choice::offer(me, project) else { return Box::default() };
    let mut capacity = models.len();
    for model in models {
        capacity = capacity.checked_add(model.accounts.len()).expect("wire offer fits");
    }
    let mut choices = skein_lib::List::with_capacity(capacity);
    for group in &model_groups(models) {
        let accounts = group_accounts(models, group.0);
        for (model, index) in models.iter().zip(0_u32..) {
            let fits = match group.1 {
                Some(account) => model.accounts.as_slice().contains(&account),
                None => model.accounts.is_empty(),
            };
            if model.provider != group.0 || !fits || !model.picker {
                continue;
            }
            let chosen = match draft.chosen(me) {
                Some(choice) => choice.account.or(model.accounts.get(0).copied()),
                None => None,
            };
            let mut description = match group.1 {
                Some(account) => payer(root, account, provider(root, model.provider), &accounts, w),
                None => Box::default(),
            };
            if draft.default_row(me) == Some(index) && group.1 == model.accounts.get(0).copied() {
                description = if description.is_empty() {
                    w.say(W::ModelDefault, &[])
                } else {
                    crate::accounts::words::join(&[&description, " · ".as_bytes(), &w.say(W::ModelDefault, &[])])
                };
            }
            choices
                .push(crate::shared::Choice {
                    name: model_shown(model, w),
                    words: description,
                    selected: draft.selected(me) == Some(index) && chosen == group.1,
                    act: Act::Choice(C::Model { index, account: if accounts.len() > 1 { group.1 } else { None } }),
                })
                .expect("choices fit offered account rows");
        }
    }
    choices.into_boxed()
}
/// The options of the terminal effort list (temper/docs/design/client/pages/README.md).
pub(crate) fn effort_choices(root: &Domain) -> Box<[crate::shared::Choice]> {
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return Box::default() };
    let draft = root.pages().choice();
    let Some(index) = draft.selected(me) else { return Box::default() };
    let Some(project) = draft.project() else { return Box::default() };
    let Some(models) = choice::offer(me, project) else { return Box::default() };
    let Some(model) = models.get(index) else { return Box::default() };
    let mut choices = skein_lib::List::with_capacity(model.efforts.len());
    for (effort, index) in model.efforts.iter().zip(0_u32..) {
        let selected = match draft.chosen(me) {
            Some(choice) => choice.main.effort.as_ref().or(model.effort.as_ref()) == Some(effort),
            None => false,
        };
        choices
            .push(crate::shared::Choice {
                name: effort.bytes().into(),
                words: Box::default(),
                selected,
                act: Act::Choice(C::Effort { index }),
            })
            .expect("choices fit offered efforts");
    }
    choices.into_boxed()
}
fn terminal_option(
    b: &mut Builder,
    root: &Domain,
    choice: &crate::shared::Choice,
    described: bool,
    bindings: &mut Bindings,
) {
    b.open(Role::MenuItem);
    b.name(&choice.name);
    if described {
        b.description(&choice.words);
    }
    b.bind(bindings.insert(choice.act.clone()));
    if choice.selected {
        b.state(State::Selected);
    }
    if described && root.pages().choice().pending.is_some() {
        b.state(State::Disabled);
    }
    b.close();
}
