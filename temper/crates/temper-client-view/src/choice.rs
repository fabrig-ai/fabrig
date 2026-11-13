//! Supplied next-run choices and account waits (client/pages/README.md and client/pages/accounts.md, 2).
pub(crate) mod picker;
use crate::{
    frame::Bindings,
    words::{TimeForm, Times, Word as W, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
pub(crate) use picker::{closed_line, picker, ran_line};
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        choice::{self},
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
    if root.approval().is_some()
        || crate::frame::unavailable(root).is_some()
        || match root.pages().address() {
            // page-view: draw Remembered.
            Address::Told
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::ConnectModel { .. }
            | Address::Welcome { .. } => true,
            Address::Forges
            | Address::ConnectForge { .. }
            | Address::AllHome
            | Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Chat { .. }
            | Address::Talk { .. }
            | Address::Task { .. }
            | Address::Home { .. }
            | Address::Changes { .. }
            | Address::Team { .. }
            | Address::Settings { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Role { .. }
            | Address::Rule { .. }
            | Address::Repositories { .. }
            | Address::Work { .. }
            | Address::Goals { .. }
            | Address::Ask { .. } | Address::Tell { .. }
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
/// Composers use the playground or the supplied real-account readiness (temper/docs/design/client/forge/family.md, 7).
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
        if task.result.is_some() || crate::pages::chats::ended(&task.chip) {
            return;
        }
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
        // Paragraphs flatten their controls in the terminal (client/line.md, 5).
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
    picker(b, root, task, w, bindings);
    feedback(b, root, task, w, bindings, times);
    b.close();
}
pub(crate) fn changeable(root: &Domain, task: Option<&c::Task>) -> bool {
    let draft = root.pages().choice();
    can_send(root)
        && match task {
            Some(task) => {
                draft.pending.is_some()
                    || task.result.is_none() && !crate::pages::chats::ended(&task.chip) && editable(task)
            }
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
/// Feedback and the no-model offer stay above the composer (client/pages/accounts.md, 2).
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
    if let Some(task) = task {
        waiting(b, root, task, w, bindings, times);
    }
}

/// Display names keep the offer's bounded presentation.
fn model_shown(model: &c::MeModel, w: &Words) -> Box<[u8]> {
    if model.name.bytes() == crate::words::SCRIPTED_MODEL {
        return shown(&w.model_name(model.name.bytes()), w);
    }
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
struct UsingLine {
    text: Box<[u8]>,
    usage: Option<Box<[u8]>>,
}
fn selected_line(w: &Words, word: W, holes: &[Hole]) -> UsingLine {
    UsingLine { text: w.say(word, holes), usage: None }
}
pub(crate) fn using(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words) {
    let line = using_words(root, task, root.pages().choice().project(), w);
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
fn using_words(root: &Domain, task: Option<&c::Task>, project: Option<u32>, w: &Words) -> UsingLine {
    let draft = root.pages().choice();
    let choice = match task {
        Some(_) => draft.original.as_ref(),
        None => draft.requested.as_ref(),
    };
    match model_words(root, choice, project, w) {
        Some((name, provider, _offered)) => using_line(root, task, choice, provider, name, w),
        None => selected_line(w, W::UsingDefault, &[]),
    }
}

/// Ready uses the same model and effort words as a new chat's composer.
pub(crate) fn new_chat_model(root: &Domain, w: &Words) -> Box<[u8]> {
    closed_line(root, None, w)
}

fn model_words(
    root: &Domain,
    choice: Option<&c::LlmChoice>,
    project: Option<u32>,
    w: &Words,
) -> Option<(Box<[u8]>, Option<u32>, bool)> {
    let me = temper_client_domain::pages::accounts::current_me(root.client());
    let template = match root.pages().choice().context {
        Some(choice::Context::Template { template, .. } | choice::Context::Task { template, .. }) => template,
        None => jig_client::shapes::CHAT,
    };
    let choice = match choice {
        Some(choice) => choice,
        None => jig_client::shapes::resolved_default(me?, project?, template)?,
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
    let effort = choice.main.effort.as_ref().or(match model {
        Some(model) => model.effort.as_ref(),
        None => None,
    });
    if let Some(effort) = effort {
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
        return selected_line(w, W::PlaygroundUsing, &[Hole::Name(w.say(W::Default, &[]))]);
    }
    let (Some(me), Some(doc)) = (me, doc) else {
        return selected_line(w, W::UsingModel, &[Hole::Name(name)]);
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
            selected_line(w, W::UsingModel, &[Hole::Name(name)])
        } else {
            match source {
                SourceChoice::Chosen(index) => {
                    let source = account.sources.get(index).expect("chosen source exists");
                    using_account(root, account, source, name, w)
                }
                SourceChoice::Wait | SourceChoice::Undecidable => selected_line(w, W::UsingModel, &[Hole::Name(name)]),
            }
        }
    } else {
        selected_line(w, W::UsingModel, &[Hole::Name(name)])
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
        return selected_line(w, W::UsingModel, &[Hole::Name(name)]);
    };
    let mut line = match source.kind {
        c::AccountKind::Key => {
            selected_line(w, W::UsingKey, &[Hole::Name(skein_lib::bytes::copy_of(provider_name.0)), Hole::Name(name)])
        }
        c::AccountKind::SignIn => {
            selected_line(w, W::UsingPlan, &[Hole::Name(skein_lib::bytes::copy_of(provider_name.1)), Hole::Name(name)])
        }
        c::AccountKind::Program => selected_line(
            w,
            W::UsingProgram,
            &[
                Hole::Name(skein_lib::bytes::copy_of(provider_name.1)),
                Hole::Name(program_name(root, account.provider, w)),
                Hole::Name(name),
            ],
        ),
        c::AccountKind::None => selected_line(w, W::UsingModel, &[Hole::Name(name)]),
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

pub(crate) fn provider_line(
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
    b.open(Role::Paragraph);
    b.style(crate::look::ACCOUNT_LINE);
    b.name(name);
    b.text(skein_ui::Run::Plain, name);
    b.close();
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
/// A wait for a host, in a box's words on a box (temper/docs/design/client/pages/accounts.md, 2).
fn host_wait(root: &Domain) -> W {
    if crate::forge::connect::owned(root) { W::WaitCapacityBox } else { W::WaitCapacity }
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
            let text = refused(w, times, why);
            b.open(Role::Paragraph);
            b.style(crate::look::ACCOUNT_LINE);
            b.name(&text);
            b.text(skein_ui::Run::Plain, &text);
            b.close();
        }
        Some(c::TaskRunWait::Host) => line(b, w, host_wait(root), &[]),
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
        c::TaskWaitRefused::Bounds { kind, why, figures } => bound_refusal(w, times, kind, why, figures, None),
        c::TaskWaitRefused::Hello { kind, at, why, figures } => {
            match why {
                c::HelloRefusal::Agent { why } if figures.is_some() => {
                    return bound_refusal(w, times, kind, Some(why), figures, Some(at));
                }
                c::HelloRefusal::Agent { .. }
                | c::HelloRefusal::JobKinds
                | c::HelloRefusal::ReservedChannel
                | c::HelloRefusal::Position { .. }
                | c::HelloRefusal::Workers { .. }
                | c::HelloRefusal::StopBound { .. } => {}
            }
            let word = match kind {
                c::HostKind::Engine => W::RefusedEngineAt,
                c::HostKind::Worker => W::RefusedWorkerAt,
            };
            let first = w.say(word, &[Hole::Name(times.say(w, TimeForm::Clock, at))]);
            let cause = sentences(w, first, hello_refusal(w, why));
            w.say(W::WaitRefused, &[Hole::Name(cause)])
        }
    }
}

/// Bound figures keep their units and the fleet's largest offer (temper/docs/design/client/pages/accounts.md, 2).
fn bound_refusal(
    w: &Words,
    times: &mut Times,
    kind: c::HostKind,
    why: Option<c::AgentShortfall>,
    figures: Option<c::Figures>,
    at: Option<skein_lib::Wall>,
) -> Box<[u8]> {
    use c::AgentShortfall as S;
    let (lead, alone) = match kind {
        c::HostKind::Worker => (W::BoundWorker, W::BoundNoWorker),
        c::HostKind::Engine => (W::BoundEngine, W::BoundNoEngine),
    };
    let Some(why) = why else {
        return w.say(lead, &[Hole::Name(w.say(W::BoundDifferent, &[]))]);
    };
    let Some(figures) = figures else {
        return sentences(w, w.say(alone, &[]), agent_shortfall(w, why));
    };
    let charter = match why {
        S::Model { charter, .. }
        | S::Window { charter, .. }
        | S::Output { charter, .. }
        | S::Turns { charter }
        | S::Time { charter }
        | S::Waiting { charter }
        | S::Checks { charter } => charter,
        S::Spend | S::Message | S::Inbox | S::Unacknowledged => 0,
    };
    let charter = match at {
        Some(_) => w.say(W::BoundHelloCharter, &[Hole::Number(u64::from(charter))]),
        None => w.say(W::BoundCharter, &[]),
    };
    let required = bound_figure(w, why, figures.required);
    let offered = bound_figure(w, why, figures.offered);
    let clause = match why {
        S::Turns { .. } => w.say(W::BoundTurns, &[Hole::Name(charter), Hole::Name(required)]),
        S::Time { .. } => w.say(W::BoundTime, &[Hole::Name(charter), Hole::Name(required)]),
        S::Waiting { .. } => w.say(W::BoundWaiting, &[Hole::Name(charter), Hole::Name(required)]),
        S::Checks { .. } => w.say(W::BoundChecks, &[Hole::Name(charter), Hole::Name(required)]),
        S::Window { model, .. } => {
            w.say(W::BoundWindow, &[Hole::Name(charter), Hole::Number(u64::from(model)), Hole::Name(required)])
        }
        S::Output { model, .. } => {
            w.say(W::BoundOutput, &[Hole::Name(charter), Hole::Number(u64::from(model)), Hole::Name(required)])
        }
        S::Spend => w.say(W::BoundSpend, &[Hole::Name(required)]),
        S::Message => w.say(W::BoundMessage, &[Hole::Name(required)]),
        S::Inbox => w.say(W::BoundInbox, &[Hole::Name(required)]),
        S::Unacknowledged => w.say(W::BoundUnacknowledged, &[Hole::Name(required)]),
        S::Model { .. } => return sentences(w, w.say(alone, &[]), agent_shortfall(w, why)),
    };
    let tail = match kind {
        c::HostKind::Engine => w.say(W::BoundTakes, &[Hole::Name(offered)]),
        c::HostKind::Worker => match at {
            Some(at) => w.say(W::BoundRefusedAt, &[Hole::Name(times.say(w, TimeForm::Clock, at)), Hole::Name(offered)]),
            None => {
                let tail = match why {
                    S::Checks { .. } => W::BoundQuickest,
                    S::Model { .. }
                    | S::Window { .. }
                    | S::Output { .. }
                    | S::Turns { .. }
                    | S::Spend
                    | S::Time { .. }
                    | S::Waiting { .. }
                    | S::Message
                    | S::Inbox
                    | S::Unacknowledged => W::BoundLargest,
                };
                w.say(tail, &[Hole::Name(offered)])
            }
        },
    };
    w.say(lead, &[Hole::Name(sentences(w, clause, tail))])
}
fn bound_figure(w: &Words, why: c::AgentShortfall, value: u64) -> Box<[u8]> {
    use c::AgentShortfall as S;
    match why {
        S::Time { .. } | S::Waiting { .. } | S::Checks { .. } => {
            w.time_span(value.checked_div(60_000).expect("minute is nonzero"))
        }
        S::Model { .. }
        | S::Window { .. }
        | S::Output { .. }
        | S::Turns { .. }
        | S::Spend
        | S::Message
        | S::Inbox
        | S::Unacknowledged => crate::words::grouped(value),
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
        S::Turns { .. } => W::AgentTurns,
        S::Spend => W::AgentSpend,
        S::Time { .. } => W::AgentTime,
        S::Waiting { .. } => W::AgentWaiting,
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
        S::Turns { .. } | S::Spend | S::Time { .. } | S::Waiting { .. } | S::Message | S::Inbox | S::Unacknowledged => {
            w.say(word, &[])
        }
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

/// The first explanatory line of a run wait, with no command (temper/docs/design/domain/trial.md, 6.4).
pub(crate) fn wait_why(root: &Domain, w: &Words, wait: &c::TaskRunWait) -> Box<[u8]> {
    use crate::accounts::words::{self, Word as A};
    use c::{TaskSourceState as S, TaskWaitWhy as R};
    let (word, until) = match *wait {
        c::TaskRunWait::Credential => return w.say(W::WaitCredential, &[]),
        c::TaskRunWait::Host => return w.say(host_wait(root), &[]),
        c::TaskRunWait::Refused(why) => {
            let at = match why {
                c::TaskWaitRefused::Hello { at, .. } => at,
                c::TaskWaitRefused::Bounds { .. } => skein_lib::Wall::EPOCH,
            };
            return refused(w, &mut Times::new(at), why);
        }
        c::TaskRunWait::Account { account, why, .. } => match why {
            R::Refreshing => return w.say(W::WaitRefreshing, &[]),
            R::Host => {
                let row = match account {
                    Some(number) => account_row(root, number),
                    None => None,
                };
                let name = match row {
                    Some(row) => skein_lib::bytes::copy_of(row.display.bytes()),
                    None => w.say(W::AccountRegion, &[]),
                };
                return w.say(W::WaitHost, &[Hole::Name(name)]);
            }
            R::Empty => return w.say(W::WaitEmpty, &[]),
            R::Invalid(_) => return w.say(W::WaitInvalid, &[]),
            R::Source(S::Moving) => return w.say(W::WaitMoving, &[]),
            R::Source(S::Asking) => (A::Asking, None),
            R::Source(S::Granted) => (A::Granted, None),
            R::Source(S::Limited { until }) => (A::Limited, until),
            R::Source(S::Later { until }) => (A::Later, until),
            R::Source(S::NeedsSignIn) => (A::NeedsSignIn, None),
            R::Source(S::Refused) => (A::Refused, None),
            R::Source(S::Misconfigured) => (A::Misconfigured, None),
            R::Source(S::Cancelled) => (A::Cancelled, None),
            R::Source(S::Closing) => (A::Closing, None),
            R::Source(S::Unloaded { at, why }) => {
                let place = match at {
                    c::AccountPlace::Directory => A::Directory,
                    c::AccountPlace::File => A::File,
                };
                let why = match why {
                    c::AccountUnread::Unreadable => A::Unreadable,
                    c::AccountUnread::Refused => A::Refused,
                    c::AccountUnread::Failed => A::Failed,
                    c::AccountUnread::Stalled => A::Stalled,
                    c::AccountUnread::TooLarge => A::TooLarge,
                };
                return words::join(&[
                    &words::say(w, A::Unloaded, &[]),
                    &words::say(w, place, &[]),
                    b": ",
                    &words::say(w, why, &[]),
                ]);
            }
        },
    };
    let first = words::say(w, word, &[]);
    match until {
        Some(until) => words::join(&[&first, &words::say(w, A::Until, &[Hole::Wall(until)])]),
        None => first,
    }
}
/// Only waits that a moment can settle leave the try running (temper/docs/design/domain/trial.md, 6).
pub(crate) const fn wait_settles(wait: &c::TaskRunWait) -> bool {
    use c::{TaskSourceState as S, TaskWaitWhy as R};
    match wait {
        c::TaskRunWait::Account { why: R::Refreshing | R::Source(S::Asking | S::Granted | S::Moving), .. } => true,
        c::TaskRunWait::Account {
            why:
                R::Host
                | R::Empty
                | R::Invalid(_)
                | R::Source(
                    S::Later { .. }
                    | S::Limited { .. }
                    | S::Refused
                    | S::NeedsSignIn
                    | S::Misconfigured
                    | S::Unloaded { .. }
                    | S::Cancelled
                    | S::Closing,
                ),
            ..
        }
        | c::TaskRunWait::Credential
        | c::TaskRunWait::Host
        | c::TaskRunWait::Refused(_) => false,
    }
}
