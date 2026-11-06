//! Supplied next-run choices and account waits (client/README.md, 3.5 and 5.5).
use crate::{
    frame::Bindings,
    words::{TimeForm, Times, Word as W, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        choice::{self, Act as C},
    },
};

fn line(b: &mut Builder, w: &Words, word: W, holes: &[Hole]) {
    let text = w.say(word, holes);
    b.open(Role::Paragraph);
    if word != W::AccountBanner {
        b.style(crate::look::ACCOUNT_LINE);
    }
    b.name(&text);
    b.text(skein_ui::Run::Plain, &text);
    b.close();
}
fn control(b: &mut Builder, bindings: &mut Bindings, role: Role, name: &[u8], act: Act, pressed: bool) {
    if role == Role::Link {
        crate::link(b, name, bindings.insert(act));
        b.text(skein_ui::Run::Plain, name);
    } else {
        b.open(role);
        b.name(name);
        b.bind(bindings.insert(act));
    }
    if pressed {
        b.state(State::Pressed);
    }
    b.close();
}
fn account_link(b: &mut Builder, bindings: &mut Bindings, name: &[u8]) {
    control(b, bindings, Role::Link, name, Act::Go { address: Address::Accounts }, false);
}
pub(crate) fn banner(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    if crate::frame::unavailable(root).is_some()
        || match root.pages().address() {
            Address::Accounts | Address::ConnectModel { .. } | Address::Welcome { .. } => true,
            Address::Forges
            | Address::ConnectForge { .. }
            | Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Home { .. }
            | Address::Changes { .. }
            | Address::Settings { .. }
            | Address::Builders { .. }
            | Address::Repositories { .. }
            | Address::Goals { .. }
            | Address::Configure { .. }
            | Address::Adopt { .. } => false,
        }
        || (root.forge().playground_running() || root.forge().leaving())
            && match root.forge().playground() {
                Some(playground) => playground.project.is_some() && playground.project == root.pages().project(),
                None => false,
            }
        || root.pages().availability(root.client()) != Some(false)
    {
        return;
    }
    b.open(Role::Region);
    b.name(&w.say(W::AccountRegion, &[]));
    b.style(crate::look::ACCOUNTS);
    line(b, w, W::AccountBanner, &[]);
    if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) {
        let playground = root.forge().playground_provider();
        let ways = temper_client_domain::pages::accounts::ways(&me.providers, playground, c::AskAccountHolder::Own);
        for way in &ways {
            let name = crate::pages::connect_model::label(me, *way, &ways, w);
            control(
                b,
                bindings,
                Role::Link,
                &name,
                Act::Go {
                    address: Address::ConnectModel {
                        way: Some(temper_client_domain::pages::connect_model::model_way(*way)),
                    },
                },
                false,
            );
        }
    }
    b.close();
}
/// Composers use the playground or the supplied real-account readiness (temper/docs/design/client/README.md, 4.8).
pub(crate) fn can_send(root: &Domain) -> bool {
    root.forge().playground().is_some() || temper_client_domain::pages::welcome::model_usable(root) != Some(false)
}
pub(crate) fn composer(
    b: &mut Builder,
    root: &Domain,
    task: Option<&c::Task>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    if let Some(task) = task {
        match task.chip.executor {
            c::ChipExecutor::Agent { .. } => {}
            c::ChipExecutor::Job { .. }
            | c::ChipExecutor::Procedure { .. }
            | c::ChipExecutor::Party { .. }
            | c::ChipExecutor::Role { .. } => return,
        }
    }
    b.open(Role::Slot);
    b.style(crate::look::CHOICE);
    if !can_send(root) {
        bindings.block_submissions();
        // Paragraphs flatten their controls in the terminal (client/README.md, 6.5).
        b.open(Role::Slot);
        b.style(crate::look::ACCOUNT_LINE);
        w.text(b, W::ComposerNoModel, &[]);
        control(
            b,
            bindings,
            Role::Link,
            &w.say(W::ConnectModel, &[]),
            Act::Go { address: Address::ConnectModel { way: None } },
            false,
        );
        b.close();
        feedback(b, root, task, w, bindings, times);
        b.close();
        return;
    }
    using(b, root, task, w);
    if offered(root, task) {
        b.open(Role::Disclosure);
        b.name(&w.say(W::ChangeModel, &[]));
        panel(b, root, task, w, bindings);
        b.close();
    }
    feedback(b, root, task, w, bindings, times);
    b.close();
}
fn offered(root: &Domain, task: Option<&c::Task>) -> bool {
    let draft = root.pages().choice();
    can_send(root)
        && draft.pending.is_none()
        && match task {
            Some(task) => task.result.is_none() && !crate::pages::chats::ended(&task.chip) && editable(task),
            None => true,
        }
        && match (temper_client_domain::pages::accounts::current_me(root.client()), draft.project()) {
            (Some(me), Some(project)) => match choice::offer(me, project) {
                Some(models) => !models.is_empty() || !me.providers.is_empty(),
                None => false,
            },
            (Some(_), None) | (None, _) => false,
        }
}
fn panel(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings) {
    let draft = root.pages().choice();
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    let Some(project) = draft.project() else { return };
    let Some(models) = choice::offer(me, project) else { return };
    b.open(Role::List);
    b.style(crate::look::ROWS);
    let selected_model = draft.selected(me);
    for provider in &me.providers {
        let mut has_rows = false;
        for model in models {
            has_rows |= model.provider == Some(provider.provider);
        }
        if !has_rows && Some(provider.provider) == root.forge().playground_provider() {
            continue;
        }
        b.open(Role::Item);
        b.style(crate::look::ROW);
        provider_line(b, root, provider, has_rows, w, bindings);
        for (model, index) in models.iter().zip(0u32..) {
            if model.provider == Some(provider.provider) && model.picker {
                model_button(b, draft, model, index, selected_model, w, bindings);
            }
        }
        b.close();
    }
    for (model, index) in models.iter().zip(0u32..) {
        if model.provider.is_none() && model.picker {
            b.open(Role::Item);
            b.style(crate::look::ROW);
            model_button(b, draft, model, index, selected_model, w, bindings);
            b.close();
        }
    }
    b.close();
    control(b, bindings, Role::Button, &w.say(W::ChoiceReset, &[]), Act::Choice(C::Default), false);
    if task.is_some() && draft.requested != draft.original {
        match &draft.requested {
            Some(choice) => {
                line(b, w, W::ChoiceDraft, &[Hole::Name(choice_shown(root, project, choice.main.model.bytes(), w))]);
            }
            None => line(b, w, W::ChoiceDraftDefault, &[]),
        }
        if draft.valid(me) {
            control(b, bindings, Role::Button, &w.say(W::ChoiceApply, &[]), Act::Choice(C::Apply), false);
        }
    }
}

fn model_button(
    b: &mut Builder,
    draft: &choice::Draft,
    model: &c::MeModel,
    index: u32,
    selected_model: Option<u32>,
    w: &Words,
    bindings: &mut Bindings,
) {
    let selected = selected_model == Some(index);
    control(b, bindings, Role::Button, &model_shown(model, w), Act::Choice(C::Model { index }), selected);
    if selected {
        for (effort, index) in model.efforts.iter().zip(0u32..) {
            let selected = match &draft.requested {
                Some(choice) => choice.main.effort.as_ref() == Some(effort),
                None => false,
            };
            control(b, bindings, Role::Button, &shown(effort.bytes(), w), Act::Choice(C::Effort { index }), selected);
        }
    }
}

/// The chat bar's account and model, with the chooser behind it (client/README.md, 3.5).
pub(crate) fn byline(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings) {
    let line = using_words(root, task, root.pages().choice().project(), w, LineForm::Byline);
    let text = match task {
        Some(task) => w.say(W::BylineTask, &[Hole::Number(task.chip.task), Hole::Name(line.text)]),
        None => line.text,
    };
    if offered(root, task) {
        b.open(Role::Disclosure);
        b.style(crate::look::BYLINE);
        b.name(&w.say(W::ChangeModel, &[]));
        b.description(&text);
        b.state(State::FoldsAway);
        b.open(Role::Slot);
        b.style(crate::look::CHOICE);
        panel(b, root, task, w, bindings);
        usage_link(b, w, line.usage.as_deref());
        b.close();
    } else {
        b.open(Role::Paragraph);
        b.style(crate::look::BYLINE);
        b.name(&text);
        b.text(skein_ui::Run::Plain, &text);
    }
    b.close();
}

/// The composer's line's words for a byline with nothing behind it (client/README.md, 3.5).
pub(crate) fn byline_words(root: &Domain, task: Option<&c::Task>, project: u32, w: &Words) -> Box<[u8]> {
    if task.is_none() {
        let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else {
            return Box::default();
        };
        let Some(models) = choice::offer(me, project) else { return Box::default() };
        if models.is_empty() {
            return Box::default();
        }
    }
    using_words(root, task, Some(project), w, LineForm::Byline).text
}

/// Feedback and the no-model offer stay above the composer (client/README.md, 5.5).
pub(crate) fn above_composer(
    b: &mut Builder,
    root: &Domain,
    task: Option<&c::Task>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Slot);
    b.style(crate::look::CHOICE);
    if !can_send(root) {
        bindings.block_submissions();
        b.open(Role::Slot);
        b.style(crate::look::ACCOUNT_LINE);
        w.text(b, W::ComposerNoModel, &[]);
        control(
            b,
            bindings,
            Role::Link,
            &w.say(W::ConnectModel, &[]),
            Act::Go { address: Address::ConnectModel { way: None } },
            false,
        );
        b.close();
    }
    feedback(b, root, task, w, bindings, times);
    b.close();
}

fn feedback(
    b: &mut Builder,
    root: &Domain,
    task: Option<&c::Task>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let draft = root.pages().choice();
    if draft.pending.is_some() {
        line(b, w, W::ChoicePending, &[]);
    }
    if let Some(outcome) = &draft.outcome {
        crate::pages::chats::outcome_text(b, w, outcome, 0);
    }
    if let Some(task) = task {
        waiting(b, root, task, w, bindings, times);
    }
}

/// Display names keep the offer's bounded presentation.
fn model_shown(model: &c::MeModel, w: &Words) -> Box<[u8]> {
    match &model.shown {
        Some(name) => shown(name.bytes(), w),
        None => shown(&w.model_name(model.name.bytes()), w),
    }
}
fn choice_shown(root: &Domain, project: u32, name: &[u8], w: &Words) -> Box<[u8]> {
    if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client())
        && let Some(models) = choice::offer(me, project)
    {
        for model in models {
            if model.name.bytes() == name {
                return model_shown(model, w);
            }
        }
    }
    shown(&w.model_name(name), w)
}

pub(crate) fn provider_shown(root: &Domain, number: Option<u32>, name: &[u8], w: &Words) -> Box<[u8]> {
    if number.is_some() && number == root.forge().playground_provider() {
        w.say(W::Playground, &[])
    } else {
        skein_lib::bytes::copy_of(name)
    }
}

/// Includes the UTF-8 ellipsis; smaller phrasebooks retain their joined Name bound.
pub(crate) const MODEL_SHOWN: u32 = 256;
pub(crate) fn shown(name: &[u8], w: &Words) -> Box<[u8]> {
    let Ok(bound) = usize::try_from(MODEL_SHOWN.min(w.name_bytes)) else { return Box::default() };
    if name.len() <= bound {
        return skein_lib::bytes::copy_of(name);
    }
    let mark = "…".as_bytes();
    let Some(prefix_bound) = bound.checked_sub(mark.len()) else { return Box::default() };
    let Some(scan) = prefix_bound.checked_add(1) else { return Box::default() };
    // Text arrived validated; a continuation byte cannot begin the cut suffix.
    let mut end = 0;
    for (index, byte) in name.iter().enumerate().take(scan) {
        if byte & 0xc0 != 0x80 {
            end = index;
        }
    }
    let Some(prefix) = name.get(..end) else { return Box::default() };
    let Some(bytes) = end.checked_add(mark.len()) else { return Box::default() };
    let mut out = skein_lib::Writer::new(bytes);
    if out.put(prefix).is_err() || out.put(mark).is_err() {
        return Box::default();
    }
    out.finish()
}

/// Only work funded by the viewer can name their accounts.
fn payer(me: &c::Me, task: Option<&c::Task>) -> Option<c::AccountUse> {
    match task {
        None => Some(c::AccountUse::Party(me.party.number)),
        Some(task) => match task.budget.funder {
            c::Funder::Pool { person, .. } => Some(c::AccountUse::Party(person)),
            c::Funder::Period { project, .. } | c::Funder::Recurring { project, .. } => {
                Some(c::AccountUse::Project(project))
            }
            c::Funder::Task(_) => None,
        },
    }
}
// Translation of jig-domain-core coverage.rs and jig-domain-accounts selection.rs.
// The own-party slice alone preserves jig's payer/explicit/deployment ordering.
fn eligible(row: &c::AccountLine, viewer: u64, provider: Option<u32>) -> bool {
    if row.provider != provider || row.holder != c::AccountHolder::Party(viewer) {
        return false;
    }
    // In the own-holder slice, plan and non-plan coverage coincide.
    match &row.uses {
        Some(uses) => uses.as_slice().contains(&c::AccountUse::Party(viewer)),
        None => true,
    }
}
fn pinned(doc: &c::Accounts, viewer: u64, provider: Option<u32>, pins: [Option<u32>; 2]) -> Option<&c::AccountLine> {
    for number in pins.into_iter().flatten() {
        for row in &doc.accounts {
            if row.number == number && eligible(row, viewer, provider) {
                return Some(row);
            }
        }
    }
    None
}
fn program_endpoint(me: &c::Me, provider: Option<u32>) -> Option<bool> {
    for offer in &me.providers {
        if Some(offer.provider) == provider {
            if offer.programs_cut {
                return None;
            }
            if offer.programs.is_empty() {
                return Some(false);
            }
            // Me has no per-model endpoint: mixed offers cannot establish its kind.
            return if offer.key || offer.sign_in || offer.none { None } else { Some(true) };
        }
    }
    None
}
fn held(state: c::AccountState) -> Option<bool> {
    match state {
        c::AccountState::Asking | c::AccountState::Granted => Some(false),
        // The public Later state also carries jig's Limited state.
        c::AccountState::Later { .. } => None,
        c::AccountState::Moving
        | c::AccountState::SignInAgain
        | c::AccountState::Misconfigured
        | c::AccountState::Unloaded { .. }
        | c::AccountState::Cancelled
        | c::AccountState::Closing => Some(true),
    }
}
enum SourceChoice {
    Chosen(u32),
    Wait,
    Undecidable,
}
fn chosen_source(row: &c::AccountLine, program: bool) -> SourceChoice {
    // An unusable "later" program source is jig's Later, which goes on to the
    // account's next program source, or its Limited, which waits: a source
    // chosen after it is undecidable.
    let mut later = false;
    for (source, index) in row.sources.iter().zip(0u32..) {
        let fits = match source.kind {
            c::AccountKind::Program => program,
            c::AccountKind::Key | c::AccountKind::SignIn | c::AccountKind::None => !program,
        };
        if !fits {
            continue;
        }
        match held(source.state) {
            Some(true) => return SourceChoice::Wait,
            None => {
                if source.usable {
                    return SourceChoice::Undecidable;
                }
                if !program {
                    return SourceChoice::Wait;
                }
                later = true;
                continue;
            }
            Some(false) => {}
        }
        if source.usable {
            return if later { SourceChoice::Undecidable } else { SourceChoice::Chosen(index) };
        }
        if !program {
            return SourceChoice::Wait;
        }
    }
    SourceChoice::Wait
}
fn selected_account(doc: &c::Accounts, viewer: u64, provider: Option<u32>, program: bool) -> Option<&c::AccountLine> {
    for row in &doc.accounts {
        if !eligible(row, viewer, provider) {
            continue;
        }
        if row.cut {
            return Some(row);
        }
        match chosen_source(row, program) {
            SourceChoice::Chosen(_) => return Some(row),
            SourceChoice::Wait => {}
            SourceChoice::Undecidable => return None,
        }
    }
    None
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum LineForm {
    Composer,
    Byline,
}
struct UsingLine {
    text: Box<[u8]>,
    usage: Option<Box<[u8]>>,
}
fn selected_line(w: &Words, word: W, holes: &[Hole], form: LineForm) -> UsingLine {
    let word = if form == LineForm::Composer {
        word
    } else if word == W::UsingModel {
        W::BylineModel
    } else if word == W::UsingKey {
        W::BylineKey
    } else if word == W::UsingPlan {
        W::BylinePlan
    } else if word == W::UsingProgram {
        W::BylineProgram
    } else if word == W::PlaygroundUsing {
        W::BylinePlayground
    } else {
        word
    };
    UsingLine { text: w.say(word, holes), usage: None }
}
pub(crate) fn using(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words) {
    let line = using_words(root, task, root.pages().choice().project(), w, LineForm::Composer);
    b.open(Role::Paragraph);
    b.style(crate::look::ACCOUNT_LINE);
    b.name(&line.text);
    b.text(skein_ui::Run::Plain, &line.text);
    if line.usage.is_some() {
        b.text(skein_ui::Run::Plain, &w.say(W::ToolSeparator, &[]));
    }
    usage_link(b, w, line.usage.as_deref());
    b.close();
}
fn usage_link(b: &mut Builder, w: &Words, address: Option<&[u8]>) {
    if let Some(address) = address {
        b.open(Role::Link);
        let text = w.say(W::Usage, &[]);
        b.name(&text);
        b.text(skein_ui::Run::Plain, &text);
        b.external(address);
        b.close();
    }
}
fn using_words(root: &Domain, task: Option<&c::Task>, project: Option<u32>, w: &Words, form: LineForm) -> UsingLine {
    let draft = root.pages().choice();
    let choice = match task {
        Some(_) => draft.original.as_ref(),
        None => draft.requested.as_ref(),
    };
    match model_words(root, task, choice, project, w) {
        Some((name, provider, offered)) => {
            if !offered {
                return selected_line(w, W::ModelNeedsAccount, &[Hole::Name(name)], form);
            }
            using_line(root, task, choice, provider, name, w, form)
        }
        None => selected_line(w, W::UsingDefault, &[], form),
    }
}

/// Ready uses the same model and effort words as a new chat's composer.
pub(crate) fn new_chat_model(root: &Domain, w: &Words) -> Box<[u8]> {
    let project = root.pages().project();
    let choice = match (temper_client_domain::pages::accounts::current_me(root.client()), project) {
        (Some(me), Some(project)) => match jig_client::shapes::template(me, project, jig_client::shapes::CHAT) {
            Some(template) => {
                template.llm.as_ref().or(jig_client::shapes::resolved_default(me, project, jig_client::shapes::CHAT))
            }
            None => None,
        },
        _ => None,
    };
    let choice = if let Address::NewChat { .. } = root.pages().address()
        && root.pages().choice().project() == project
    {
        root.pages().choice().requested.as_ref().or(choice)
    } else {
        choice
    };
    match model_words(root, None, choice, project, w) {
        Some((name, _, _)) => name,
        None => w.say(W::DefaultModel, &[]),
    }
}
fn model_words(
    root: &Domain,
    task: Option<&c::Task>,
    choice: Option<&c::LlmChoice>,
    project: Option<u32>,
    w: &Words,
) -> Option<(Box<[u8]>, Option<u32>, bool)> {
    let me = temper_client_domain::pages::accounts::current_me(root.client());
    let template = match root.pages().choice().context {
        Some(choice::Context::Template { template, .. }) => template,
        Some(choice::Context::Task { .. }) | None => jig_client::shapes::CHAT,
    };
    let choice = match choice {
        Some(choice) => choice,
        None => {
            if task.is_some() {
                return None;
            }
            jig_client::shapes::resolved_default(me?, project?, template)?
        }
    };
    let models = match (me, project) {
        (Some(me), Some(project)) => choice::offer(me, project),
        (Some(_), None) | (None, _) => None,
    };
    let mut model = None;
    if let Some(models) = models {
        for offered in models {
            if offered.name == choice.main.model {
                model = Some(offered);
                break;
            }
        }
    }
    let mut name = match model {
        Some(model) => model_shown(model, w),
        None => shown(&w.model_name(choice.main.model.bytes()), w),
    };
    if let Some(effort) = &choice.main.effort {
        name = w.say(W::ModelEffort, &[Hole::Name(name), Hole::Name(shown(effort.bytes(), w))]);
    }
    let provider = match model {
        Some(model) => model.provider,
        None => None,
    };
    Some((name, provider, model.is_some()))
}

fn using_line(
    root: &Domain,
    task: Option<&c::Task>,
    choice: Option<&c::LlmChoice>,
    provider: Option<u32>,
    name: Box<[u8]>,
    w: &Words,
    form: LineForm,
) -> UsingLine {
    let me = temper_client_domain::pages::accounts::current_me(root.client());
    let doc = temper_client_domain::pages::accounts::document(root.client());
    let payer = match me {
        Some(me) => payer(me, task),
        None => None,
    };
    let preferred = match choice {
        Some(choice) => choice.account,
        None => None,
    };
    let kept = match task {
        Some(task) => task.kept_account,
        None => None,
    };
    let own = match me {
        Some(me) => payer == Some(c::AccountUse::Party(me.party.number)),
        None => false,
    };
    let pin = match (me, doc) {
        (Some(me), Some(doc)) if own => pinned(doc, me.party.number, provider, [preferred, kept]),
        (Some(_), Some(_)) | (None, _) | (_, None) => None,
    };
    // The playground provider is public even when its deployment-held account is outside this slice.
    if pin.is_none() && provider.is_some() && provider == root.forge().playground_provider() {
        return selected_line(w, W::PlaygroundUsing, &[Hole::Name(w.say(W::Default, &[]))], form);
    }
    let (Some(me), Some(doc)) = (me, doc) else {
        return selected_line(w, W::UsingModel, &[Hole::Name(name)], form);
    };
    let program = program_endpoint(me, provider);
    let account = if own
        && !doc.cut
        && let Some(program) = program
    {
        match pin {
            Some(row) => Some(row),
            None => selected_account(doc, me.party.number, provider, program),
        }
    } else {
        None
    };
    if let Some(account) = account {
        let source = chosen_source(account, program.expect("selected with a known endpoint"));
        // The offer supplies usability; document readiness cannot withdraw a row.
        if account.cut {
            selected_line(w, W::UsingModel, &[Hole::Name(name)], form)
        } else {
            match source {
                SourceChoice::Chosen(index) => {
                    let source = account.sources.get(index).expect("chosen source exists");
                    using_account(root, account, source, name, w, form)
                }
                SourceChoice::Wait | SourceChoice::Undecidable => {
                    selected_line(w, W::UsingModel, &[Hole::Name(name)], form)
                }
            }
        }
    } else {
        selected_line(w, W::UsingModel, &[Hole::Name(name)], form)
    }
}
fn program_name(root: &Domain, provider: Option<u32>, w: &Words) -> Box<[u8]> {
    if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) {
        for offer in &me.providers {
            if Some(offer.provider) == provider
                && let Some(program) = offer.programs.get(0)
            {
                return shown(if program.bytes() == b"claude_code" { b"Claude Code" } else { program.bytes() }, w);
            }
        }
    }
    Box::default()
}
fn using_account(
    root: &Domain,
    account: &c::AccountLine,
    source: &c::AccountSource,
    name: Box<[u8]>,
    w: &Words,
    form: LineForm,
) -> UsingLine {
    let mut provider_name = None;
    if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) {
        for offer in &me.providers {
            if Some(offer.provider) == account.provider {
                provider_name = Some(crate::accounts::words::provider_words(offer.name.bytes()));
                break;
            }
        }
    }
    let Some(provider_name) = provider_name else {
        return selected_line(w, W::UsingModel, &[Hole::Name(name)], form);
    };
    let mut line = match source.kind {
        c::AccountKind::Key => selected_line(
            w,
            W::UsingKey,
            &[Hole::Name(skein_lib::bytes::copy_of(provider_name.0)), Hole::Name(name)],
            form,
        ),
        c::AccountKind::SignIn => selected_line(
            w,
            W::UsingPlan,
            &[Hole::Name(skein_lib::bytes::copy_of(provider_name.1)), Hole::Name(name)],
            form,
        ),
        c::AccountKind::Program => selected_line(
            w,
            W::UsingProgram,
            &[
                Hole::Name(skein_lib::bytes::copy_of(provider_name.1)),
                Hole::Name(program_name(root, account.provider, w)),
                Hole::Name(name),
            ],
            form,
        ),
        c::AccountKind::None => selected_line(w, W::UsingModel, &[Hole::Name(name)], form),
    };
    if source.kind == c::AccountKind::SignIn
        && let Some(address) = &account.manage_usage
        && skein_ui::markdown::safe_url(address)
        && c::valid_manage_usage(address)
    {
        line.usage = Some(skein_lib::bytes::copy_of(address));
    }
    line
}
#[expect(clippy::manual_find, reason = "the programming model uses bounded loops, not closures")]
fn account_row(root: &Domain, account: u32) -> Option<&c::AccountLine> {
    let doc = temper_client_domain::pages::accounts::document(root.client())?;
    for row in &doc.accounts {
        if row.number == account {
            return Some(row);
        }
    }
    None
}
fn account_line(b: &mut Builder, root: &Domain, w: &Words, account: u32, labels: (W, W)) {
    match account_row(root, account) {
        Some(row) => {
            let name = if row.provider.is_some() && row.provider == root.forge().playground_provider() {
                w.say(W::PlaygroundModelName, &[])
            } else {
                skein_lib::bytes::copy_of(row.display.bytes())
            };
            line(b, w, labels.1, &[Hole::Name(name)]);
        }
        None => line(b, w, labels.0, &[Hole::Number(u64::from(account))]),
    }
}

fn provider_line(
    b: &mut Builder,
    root: &Domain,
    provider: &c::ProviderOffer,
    has_rows: bool,
    w: &Words,
    bindings: &mut Bindings,
) {
    use crate::accounts::words::{self, Word as A};
    let configured = provider_shown(root, Some(provider.provider), provider.name.bytes(), w);
    let name = words::provider_words(&configured).0;
    line(b, w, W::ChoiceProvider, &[Hole::Name(skein_lib::bytes::copy_of(name))]);
    if has_rows {
        return;
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
                account_link(b, bindings, &words::join(&[&fix, " ›".as_bytes()]));
                return;
            }
        }
    }
    if held_account {
        line(b, w, W::AccountReadinessUnknown, &[]);
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
}

fn editable(task: &c::Task) -> bool {
    match jig_client::facts::offer(task.acts.as_slice(), c::ActKind::Amend) {
        Some(c::Admission::Allowed | c::Admission::Bounded { .. } | c::Admission::Proposed { .. }) => true,
        Some(c::Admission::Refused(_)) | None => false,
    }
}
pub(crate) fn waiting(
    b: &mut Builder,
    root: &Domain,
    task: &c::Task,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    match task.run_wait {
        None => {}
        Some(c::TaskRunWait::Credential) => {
            line(b, w, W::WaitCredential, &[]);
            control(b, bindings, Role::Link, &w.say(W::ForgesTitle, &[]), Act::Go { address: Address::Forges }, false);
        }
        Some(c::TaskRunWait::Refused(why)) => {
            line(b, w, W::WaitRefused, &[Hole::Name(refused(w, times, why))]);
        }
        Some(c::TaskRunWait::Host) => line(b, w, W::WaitCapacity, &[]),
        Some(c::TaskRunWait::Account { .. }) => {
            account_wait(b, root, task, w, bindings);
        }
    }
}
fn sentences(w: &Words, first: Box<[u8]>, next: Box<[u8]>) -> Box<[u8]> {
    w.say(W::RefusalSentences, &[Hole::Name(first), Hole::Name(next)])
}
fn refused(w: &Words, times: &mut Times, why: c::TaskWaitRefused) -> Box<[u8]> {
    match why {
        c::TaskWaitRefused::Hello { kind, at, why } => {
            let word = match kind {
                c::HostKind::Engine => W::RefusedEngineAt,
                c::HostKind::Worker => W::RefusedWorkerAt,
            };
            let first = w.say(word, &[Hole::Name(times.say(w, TimeForm::Clock, at))]);
            sentences(w, first, hello_refusal(w, why))
        }
    }
}
fn hello_refusal(w: &Words, why: c::HelloRefusal) -> Box<[u8]> {
    match why {
        c::HelloRefusal::JobKinds => w.say(W::HelloJobKinds, &[]),
        c::HelloRefusal::ReservedChannel => w.say(W::HelloReservedChannel, &[]),
        c::HelloRefusal::Position { worker } => w.say(W::HelloPosition, &[Hole::Number(u64::from(worker))]),
        c::HelloRefusal::Workers { limit } => {
            sentences(w, w.say(W::HelloWorkers, &[]), w.say(W::HelloLimit, &[Hole::Number(u64::from(limit))]))
        }
        c::HelloRefusal::StopBound { grace } => {
            sentences(w, w.say(W::HelloStopBound, &[]), w.say(W::HelloAllowedGrace, &[Hole::Span(grace)]))
        }
        c::HelloRefusal::Agent { why } => agent_shortfall(w, why),
    }
}
fn agent_shortfall(w: &Words, why: c::AgentShortfall) -> Box<[u8]> {
    use c::AgentShortfall as S;
    let word = match why {
        S::Model { .. } => W::AgentModel,
        S::Window { .. } => W::AgentWindow,
        S::Output { .. } => W::AgentOutput,
        S::Turns => W::AgentTurns,
        S::Spend => W::AgentSpend,
        S::Time => W::AgentTime,
        S::Waiting => W::AgentWaiting,
        S::Checks { .. } => W::AgentChecks,
        S::Message => W::AgentMessage,
        S::Inbox => W::AgentInbox,
        S::Unacknowledged => W::AgentUnacknowledged,
    };
    match why {
        S::Checks { charter } => w.say(word, &[Hole::Number(u64::from(charter))]),
        S::Model { charter, model } | S::Window { charter, model } | S::Output { charter, model } => sentences(
            w,
            w.say(word, &[Hole::Number(u64::from(charter))]),
            w.say(W::AgentModelPosition, &[Hole::Number(u64::from(model))]),
        ),
        S::Turns | S::Spend | S::Time | S::Waiting | S::Message | S::Inbox | S::Unacknowledged => w.say(word, &[]),
    }
}
fn account_wait(b: &mut Builder, root: &Domain, task: &c::Task, w: &Words, bindings: &mut Bindings) {
    use crate::accounts::words::{self, Word as A};
    use c::{TaskSourceState as S, TaskWaitWhy as R};
    let Some(c::TaskRunWait::Account { account, source, why }) = task.run_wait else { return };
    b.open(Role::Status);
    b.style(crate::look::STATUS_WAITING);
    let name = w.say(W::AccountWaiting, &[]);
    b.name(&name);
    b.text(skein_ui::Run::Plain, &name);
    b.close();
    if let Some(account) = account {
        account_line(b, root, w, account, (W::WaitingAccount, W::WaitingAccountName));
    }
    if let Some(source) = source {
        b.open(Role::Paragraph);
        b.text(skein_ui::Run::Plain, &words::say(w, A::Source, &[]));
        b.text(skein_ui::Run::Plain, &words::say(w, A::Number, &[Hole::Number(u64::from(source))]));
        b.close();
    }
    match why {
        R::Source(S::Moving) => line(b, w, W::WaitMoving, &[]),
        R::Refreshing => line(b, w, W::WaitRefreshing, &[]),
        R::Host => {
            let row = match account {
                Some(number) => account_row(root, number),
                None => None,
            };
            let name = match row {
                Some(row) => skein_lib::bytes::copy_of(row.display.bytes()),
                None => w.say(W::AccountRegion, &[]),
            };
            line(b, w, W::WaitHost, &[Hole::Name(name)]);
            if let Some(row) = row {
                for entry in &row.sources {
                    if Some(entry.number) == source
                        && entry.kind == c::AccountKind::Program
                        && let Some(name) = &entry.name
                    {
                        crate::pages::connect_model::command(b, root, w, name, false);
                    }
                }
            }
            account_link(b, bindings, &w.say(W::SettingsAccountsTab, &[]));
        }
        R::Empty => {
            match &task.llm {
                Some(choice) => line(
                    b,
                    w,
                    W::WaitEmptyModel,
                    &[Hole::Name(choice_shown(root, task.project, choice.main.model.bytes(), w))],
                ),
                None => line(b, w, W::WaitEmpty, &[]),
            }
            control(
                b,
                bindings,
                Role::Link,
                &w.say(W::ConnectModel, &[]),
                Act::Go { address: Address::ConnectModel { way: None } },
                false,
            );
        }
        R::Invalid(why) => {
            line(b, w, W::WaitInvalid, &[]);
            if let Some(account) = account {
                w.refusal(b, &c::Refusal::Account { account, why }, 0);
            }
        }
        R::Source(S::Limited { until }) => {
            source_wait(b, w, A::Limited, until);
            account_link(b, bindings, &words::say(w, A::Resume, &[]));
        }
        R::Source(S::Later { until }) => source_wait(b, w, A::Later, until),
        R::Source(S::Unloaded { at, why }) => {
            b.open(Role::Paragraph);
            crate::pages::accounts::unloaded(b, w, at, why);
            b.close();
        }
        R::Source(S::Asking) => source_wait(b, w, A::Asking, None),
        R::Source(S::Granted) => source_wait(b, w, A::Granted, None),
        R::Source(S::Refused) => {
            source_wait(b, w, A::Refused, None);
            account_link(b, bindings, &w.say(W::AccountKey, &[]));
        }
        R::Source(S::NeedsSignIn) => {
            source_wait(b, w, A::NeedsSignIn, None);
            account_link(b, bindings, &w.say(W::AccountSignIn, &[]));
            account_link(b, bindings, &w.say(W::AccountKey, &[]));
        }
        R::Source(S::Misconfigured) => source_wait(b, w, A::Misconfigured, None),
        R::Source(S::Cancelled) => source_wait(b, w, A::Cancelled, None),
        R::Source(S::Closing) => source_wait(b, w, A::Closing, None),
    }
}
fn source_wait(b: &mut Builder, w: &Words, word: crate::accounts::words::Word, until: Option<skein_lib::Wall>) {
    use crate::accounts::words::{self, Word as A};
    b.open(Role::Paragraph);
    b.text(skein_ui::Run::Plain, &words::say(w, word, &[]));
    if let Some(until) = until {
        b.text(skein_ui::Run::Plain, &words::say(w, A::Until, &[Hole::Wall(until)]));
    }
    b.close();
}
