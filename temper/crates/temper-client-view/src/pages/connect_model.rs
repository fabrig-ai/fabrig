//! The standalone model connection flow and parts used by the welcome.
use crate::{
    accounts::{
        Bindings, Meaning,
        words::{self, Word as W},
    },
    flow::{self, Control},
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain, WayKind,
    pages::{
        accounts::{self, SignInState, Way},
        connect_model::{self as model, Act},
    },
};

fn say(w: &Words, word: W) -> Box<[u8]> {
    words::say(w, word, &[])
}
fn named(w: &Words, word: W, name: &[u8]) -> Box<[u8]> {
    words::say(w, word, &[Hole::Name(crate::choice::shown(name, w))])
}
fn text(b: &mut Builder, w: &Words, role: Role, word: W) {
    crate::cards::text(b, role, &say(w, word));
}
pub(crate) fn names(me: &c::Me, way: Way) -> (&[u8], &[u8]) {
    for offer in &me.providers {
        if offer.provider == way.provider() {
            return words::provider_words(offer.name.bytes());
        }
    }
    (b"", b"")
}
pub(crate) fn label(me: &c::Me, way: Way, ways: &skein_lib::List<Way>, w: &Words) -> Box<[u8]> {
    let (brand, plan) = names(me, way);
    match way {
        Way::SignIn { plan: true, .. } => named(w, W::Continue, plan),
        Way::SignIn { plan: false, .. } => named(w, W::SignInBrand, brand),
        Way::Program { .. } => named(w, W::SubscriptionWay, plan),
        Way::Key { .. } => {
            let mut keys = 0_u32;
            for way in ways {
                if model::model_way(*way).kind == WayKind::Key {
                    keys = keys.checked_add(1).expect("provider ways fit");
                }
            }
            if keys == 1 {
                say(w, W::KeyWayOnly)
            } else {
                let word = match brand.first() {
                    Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => W::KeyWayAn,
                    _ => W::KeyWay,
                };
                named(w, word, brand)
            }
        }
    }
}
fn go(b: &mut Builder, bindings: &mut Bindings, name: &[u8], address: Address, primary: bool) {
    if let Some(to) = bindings.insert(Meaning::Go(address)) {
        let action = if primary { Control::PrimaryLink { name, to } } else { Control::SecondaryLink { name, to } };
        flow::control(b, action);
    }
}
pub(super) fn button(b: &mut Builder, bindings: &mut Bindings, name: &[u8], act: Act, busy: bool, primary: bool) {
    if busy {
        flow::control(b, if primary { Control::PrimaryDisabled { name } } else { Control::Disabled { name } });
    } else if let Some(to) = bindings.insert(Meaning::Model(act)) {
        flow::control(b, if primary { Control::PrimaryButton { name, to } } else { Control::Button { name, to } });
    }
}
pub(super) fn choice(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    way: Way,
    ways: &skein_lib::List<Way>,
) {
    let Some(me) = accounts::current_me(root.client()) else { return };
    let (brand, plan) = names(me, way);
    let label = label(me, way, ways, w);
    let heading = match way {
        Way::Key { .. } => named(
            w,
            match brand.first() {
                Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => W::KeyTitle,
                _ => W::KeyTitleA,
            },
            brand,
        ),
        Way::Program { .. } => named(w, W::SubscriptionWay, plan),
        Way::SignIn { .. } => crate::choice::shown(plan, w),
    };
    let (description, note) = match way {
        Way::SignIn { plan: true, .. } => (
            words::say(
                w,
                W::ChatDescription,
                &[Hole::Name(crate::choice::shown(plan, w)), Hole::Name(crate::choice::shown(plan, w))],
            ),
            W::BrowserMinute,
        ),
        Way::SignIn { plan: false, .. } => (named(w, W::SignInDescription, brand), W::BrowserMinute),
        Way::Key { .. } => (named(w, W::KeyDescription, brand), W::Pasted),
        Way::Program { .. } => (say(w, W::ProgramDescription), W::ProgramNote),
    };
    let meaning = if model::model_way(way).kind == WayKind::SignIn {
        Meaning::Model(Act::SignIn { name: root.pages().account_name(), way })
    } else {
        Meaning::Go(model::address(root.pages().address(), Some(model::model_way(way))))
    };
    let Some(to) = bindings.insert(meaning) else { return };
    let control = if model::model_way(way).kind == WayKind::SignIn {
        Control::PrimaryButton { name: &label, to }
    } else {
        Control::SecondaryLink { name: &label, to }
    };
    flow::choice_card(b, &heading);
    crate::cards::text(b, Role::Paragraph, &description);
    b.open(Role::Paragraph);
    b.style(crate::look::FLOW_MUTED);
    b.name(&say(w, note));
    b.text(Run::Plain, &say(w, note));
    b.close();
    b.open(Role::Slot);
    b.style(crate::look::FLOW_ACTIONS);
    flow::control(b, control);
    b.close();
    b.close();
}
/// The welcome's Model step: a card for each plan its card names, `openai`'s
/// and `anthropic`'s, its sign-in or subscription alone, a sign-in the
/// default; no key and no other provider's ways
/// (`temper/docs/design/client/README.md`, 3.5).
fn welcome_choices(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, ways: &skein_lib::List<Way>) {
    let Some(me) = accounts::current_me(root.client()) else { return };
    let mut shown = false;
    b.open(Role::Slot);
    b.style(crate::look::MODEL_CHOICES);
    for offer in &me.providers {
        if offer.name.bytes() != b"openai" && offer.name.bytes() != b"anthropic" {
            continue;
        }
        // A plan alone: a key is added from the accounts page (3.5).
        let mut main = None;
        for way in ways {
            if way.provider() == offer.provider
                && let Way::SignIn { .. } | Way::Program { .. } = way
            {
                main = main.or(Some(*way));
            }
        }
        let Some(way) = main else { continue };
        shown = true;
        let (_, plan) = names(me, way);
        flow::choice_card(b, &crate::choice::shown(plan, w));
        let name = [Hole::Name(crate::choice::shown(plan, w))];
        let (description, note) = match way {
            Way::Program { .. } => {
                (w.say(Word::WelcomeModelSubscription, &name), w.say(Word::WelcomeModelProgramTime, &[]))
            }
            Way::SignIn { .. } | Way::Key { .. } => {
                (w.say(Word::WelcomeModelPlan, &name), w.say(Word::WelcomeModelSignInTime, &[]))
            }
        };
        crate::cards::text(b, Role::Paragraph, &description);
        b.open(Role::Paragraph);
        b.style(crate::look::FLOW_MUTED);
        b.name(&note);
        b.text(Run::Plain, &note);
        b.close();
        b.open(Role::Slot);
        b.style(crate::look::FLOW_ACTIONS);
        let label = label(me, way, ways, w);
        let meaning = match way {
            Way::SignIn { .. } => Meaning::Model(Act::SignIn { name: root.pages().account_name(), way }),
            Way::Key { .. } | Way::Program { .. } => {
                Meaning::Go(model::address(root.pages().address(), Some(model::model_way(way))))
            }
        };
        if let Some(to) = bindings.insert(meaning) {
            flow::control(
                b,
                match way {
                    Way::SignIn { .. } => Control::PrimaryButton { name: &label, to },
                    Way::Key { .. } | Way::Program { .. } => Control::SecondaryLink { name: &label, to },
                },
            );
        }
        b.close();
        b.close();
    }
    b.close();
    if !shown {
        choices(b, root, w, bindings, ways);
    }
}
fn choices(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, ways: &skein_lib::List<Way>) {
    b.open(Role::Slot);
    b.style(crate::look::MODEL_CHOICES);
    for way in ways {
        choice(b, root, w, bindings, *way, ways);
    }
    b.close();
}
pub(super) fn request_for_way(page: &model::ConnectModel, way: Way) -> bool {
    if page.request.returned() && model::model_way(way).kind == WayKind::SignIn {
        return true;
    }
    let Some(pending) = &page.request.pending else { return false };
    match (&pending.ask, way) {
        (c::Ask::SignIn { provider, .. }, Way::SignIn { provider: chosen, .. }) => *provider == chosen,
        (
            c::Ask::AccountChange { holder, change: c::AccountChange::AddKey { provider, join: None, .. } },
            Way::Key { provider: chosen },
        )
        | (
            c::Ask::AccountChange { holder, change: c::AccountChange::DeclareProgram { provider, join: None, .. } },
            Way::Program { provider: chosen },
        ) => *provider == chosen && *holder == page.holder,
        _ => false,
    }
}
pub(super) fn ready(root: &Domain, way: Way) -> bool {
    let Some(page) = root.pages().connect_model() else { return false };
    if !request_for_way(page, way) {
        return false;
    }
    if page.request.returned() {
        return match page.request.sign_in_state(root.client()) {
            SignInState::Ended { .. } => true,
            SignInState::Unknown | SignInState::Started { .. } | SignInState::Signing { .. } => false,
        };
    }
    if let Some(p) = &page.request.pending {
        match (&p.ask, &p.outcome, way) {
            (
                c::Ask::AccountChange { change: c::AccountChange::AddKey { provider, join: None, .. }, .. },
                Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                    c::DocumentOutcome::AccountKeyKept { .. },
                ))))),
                Way::Key { provider: chosen },
            )
            | (
                c::Ask::AccountChange { change: c::AccountChange::DeclareProgram { provider, join: None, .. }, .. },
                Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                    c::DocumentOutcome::AccountProgramDeclared { .. },
                ))))),
                Way::Program { provider: chosen },
            ) => *provider == chosen,
            (c::Ask::SignIn { provider, .. }, _, Way::SignIn { provider: chosen, .. }) => {
                *provider == chosen
                    && match page.request.sign_in_state(root.client()) {
                        SignInState::Ended { .. } => true,
                        SignInState::Started { .. } | SignInState::Signing { .. } | SignInState::Unknown => false,
                    }
            }
            _ => false,
        }
    } else {
        false
    }
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    pages: &mut crate::frame::Bindings,
    project: Option<u32>,
    text_per_node: u32,
) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    b.style(crate::look::PAGE);
    super::settings::path(b, root, w, pages, project, super::settings::FlowTab::Accounts, Word::PathConnectAccount);
    crate::cards::text(b, Role::Heading { level: 1 }, &w.say(Word::ConnectModel, &[]));
    let way = model::way(root.pages(), root.client());
    let middle = if match way {
        Some(Way::SignIn { .. }) => true,
        Some(Way::Key { .. } | Way::Program { .. }) | None => false,
    } {
        W::SigningIn
    } else {
        W::Details
    };
    let current = match way {
        Some(way) if ready(root, way) => 2,
        Some(_) => 1,
        None => 0,
    };
    flow::steps(b, w, &[&say(w, W::Choose), &say(w, middle), &say(w, W::Ready)], current);
    body(b, root, w, bindings, text_per_node);
    b.close();
}
fn welcome(root: &Domain) -> bool {
    match root.pages().address() {
        Address::Welcome { .. } => true,
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Accounts
        | Address::ConnectModel { .. }
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
}
fn muted(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(crate::look::FLOW_MUTED);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn description(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(crate::look::MODEL_DESCRIPTION);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
pub(super) fn signing_heading(root: &Domain, w: &Words, way: Way) -> Box<[u8]> {
    let Some(me) = accounts::current_me(root.client()) else { return say(w, W::SigningIn) };
    let (_, plan) = names(me, way);
    w.say(Word::ModelSigningIn, &[Hole::Name(crate::choice::shown(plan, w))])
}
fn ending_heading(root: &Domain, w: &Words, way: Way) -> Box<[u8]> {
    if model::model_way(way).kind == WayKind::SignIn {
        if let Some(page) = root.pages().connect_model()
            && let SignInState::Ended { source } = page.request.sign_in_state(root.client())
            && let Some(doc) = accounts::document(root.client())
            && let Some(last) = &doc.last_end
            && last.source == source
            && let c::AccountEnd::Made { account, declined: false } = last.end
        {
            return made_words(root, w, way, account, false);
        }
        return signing_heading(root, w, way);
    }
    say(w, W::Ready)
}
pub(super) fn key_field(b: &mut Builder, root: &Domain, bindings: &mut Bindings, busy: bool, word: &[u8]) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(to) = bindings.insert(Meaning::Key) else { return };
    b.open(Role::TextField);
    b.style(crate::look::FLOW_FIELD);
    b.name(word);
    b.entry(skein_ui::EntryKind::Secret);
    b.value(b"", page.request.key_written());
    b.bind(to);
    if busy {
        b.state(State::Disabled);
    }
    b.close();
}
fn standalone_choose_body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let ways = accounts::ways(&me.providers, root.forge().playground_provider(), page.holder);
    let busy = if let Some(p) = &page.request.pending { p.outcome.is_none() } else { false };
    flow::card(b, &say(w, W::Choose));
    if me.deployment_role == Some(c::HeldRole::Owner) {
        for (holder, word) in
            [(c::AskAccountHolder::Own, W::OwnChoice), (c::AskAccountHolder::Deployment, W::DeploymentChoice)]
        {
            if let Some(to) = bindings.insert(Meaning::Model(Act::Holder { holder })) {
                b.open(Role::Button);
                b.name(&say(w, word));
                b.bind(to);
                if holder == page.holder {
                    b.state(State::Pressed);
                }
                if busy {
                    b.state(State::Disabled);
                }
                b.close();
            }
        }
    }
    choices(b, root, w, bindings, &ways);
    super::accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
    b.close();
}
pub(super) fn choose_body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let ways = accounts::ways(&me.providers, root.forge().playground_provider(), page.holder);
    welcome_choices(b, root, w, bindings, &ways);
    super::accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
}
pub(crate) fn body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let playground = root.forge().playground_provider();
    let ways = accounts::ways(&me.providers, playground, page.holder);
    let way = model::way(root.pages(), root.client());
    let busy = match &page.request.pending {
        Some(p) => p.outcome.is_none(),
        None => false,
    };
    let Some(way) = way else {
        standalone_choose_body(b, root, w, bindings, text_per_node);
        return;
    };
    if let Some(to) = bindings.insert(Meaning::Go(model::address(root.pages().address(), None))) {
        flow::control(b, Control::Link { name: &say(w, W::AllWays), to });
    }
    if let Some(trial) = super::model_check::trial(root, w) {
        trial_body(b, root, w, bindings, &trial, way);
        return;
    }
    if ready(root, way) {
        flow::details(b, &ending_heading(root, w, way));
        ready_body(b, root, w, bindings, way, &ways);
        b.close();
        return;
    }
    if request_for_way(page, way)
        && let SignInState::Started { .. } | SignInState::Signing { .. } = page.request.sign_in_state(root.client())
    {
        super::sign_in_wait::waiting(
            b,
            root,
            w,
            bindings,
            &page.request,
            super::sign_in_wait::tab_refused(),
            text_per_node,
        );
        return;
    }
    match way {
        Way::SignIn { .. } => {
            flow::details(b, &signing_heading(root, w, way));
            if !request_for_way(page, way) || page.request.sign_in_state(root.client()) == SignInState::Unknown {
                choice(b, root, w, bindings, way, &ways);
            }
        }
        Way::Key { .. } => key_details(b, root, w, bindings, way, busy),
        Way::Program { .. } => {
            let (_, plan) = names(me, way);
            flow::details(b, &w.say(Word::ModelProgramTitle, &[Hole::Name(crate::choice::shown(plan, w))]));
            program_body(b, root, w, bindings, way, busy);
        }
    }
    if request_for_way(page, way) {
        super::accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
    }
    if let Some(pending) = &page.request.pending
        && let Some(jig_client::Outcome::Lost | jig_client::Outcome::Unsent | jig_client::Outcome::Full { .. }) =
            &pending.outcome
    {
        button(b, bindings, &say(w, W::Cancel), Act::Cancel, false, false);
    }
    b.close();
}
/// The way's card while a try is shown (temper/docs/design/client/README.md, 3.5).
pub(crate) fn trial_body(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    trial: &super::model_check::Trial,
    way: Way,
) {
    flow::details(b, &ending_heading(root, w, way));
    super::model_check::status(b, w, trial);
    match trial.state {
        super::model_check::State::Answered { .. } => ready_primary(b, root, w, bindings),
        super::model_check::State::Stopped { .. } => {
            super::model_check::stopped_acts(b, root, w, bindings, trial, way, ready_address(root));
        }
        super::model_check::State::Running { .. } => {}
    }
    b.close();
}
pub(super) fn program_body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, way: Way, busy: bool) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let (_, plan) = names(me, way);
    description(b, &w.say(Word::ModelProgramDescription, &[]));
    let name = page.program_name(root.client());
    b.open(Role::Form);
    b.name(&say(w, W::Details));
    if let Some(to) = bindings.insert(Meaning::Program) {
        b.open(Role::TextField);
        b.name(&w.say(Word::ModelProgramName, &[]));
        b.value(&name, page.request.key_written());
        b.bind(to);
        if !c::valid_program_name(&name) {
            b.state(State::Invalid);
        }
        if busy {
            b.state(State::Disabled);
        }
        b.close();
    }
    command(b, root, w, &name, false);
    muted(b, &say(w, W::ProgramNote));
    button(
        b,
        bindings,
        &named(w, W::SubscriptionWay, plan),
        Act::Submit { name: root.pages().account_name() },
        busy || !c::valid_program_name(&name),
        false,
    );
    b.close();
}
/// Opens the key's details card; the shared body closes it after its outcome.
fn key_details(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, way: Way, busy: bool) {
    let Some(me) = accounts::current_me(root.client()) else { return };
    let (brand, _) = names(me, way);
    let title = match brand.first() {
        Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => W::KeyTitle,
        _ => W::KeyTitleA,
    };
    flow::card(b, &named(w, title, brand));
    b.style(crate::look::MODEL_KEY);
    description(b, &w.say(Word::ModelKeyDescription, &[Hole::Name(crate::choice::shown(brand, w))]));
    b.open(Role::Form);
    b.name(&say(w, W::Details));
    key_field(b, root, bindings, busy, &w.say(Word::ModelKeyField, &[]));
    button(b, bindings, &w.say(Word::ModelAddKey, &[]), Act::Submit { name: root.pages().account_name() }, busy, true);
    b.close();
}
/// The whole command for NAME (client/README.md, 4.6, *The printed command*).
pub(crate) fn printed_command(root: &Domain, w: &Words, name: &[u8]) -> Box<[u8]> {
    let (environment, answered) = match root.pages().connect_model() {
        Some(page) => (page.program_environment(), page.program_environment_answered()),
        None => (b"".as_slice(), false),
    };
    if answered {
        if environment.is_empty() {
            named(w, W::EnvironmentCommand, name)
        } else {
            words::join(&[environment, b" ", &named(w, W::ProgramLogin, name)])
        }
    } else if let Some(answer) = root.forge().deployment()
        && let Some(line) = &answer.program_login
    {
        words::join(&[line, b" ", &crate::choice::shown(name, w)])
    } else {
        named(w, W::Command, name)
    }
}
/// The same command with the words saying where to run it.
pub(crate) fn command_line(root: &Domain, w: &Words, name: &[u8]) -> Box<[u8]> {
    words::join(&[&command_prompt(root, w), &printed_command(root, w, name)])
}
fn command_prompt(root: &Domain, w: &Words) -> Box<[u8]> {
    let word = match root.forge().deployment() {
        Some(answer) if answer.program_login.is_some() => W::ProgramCommandHere,
        Some(_) | None => W::ProgramCommand,
    };
    named(w, word, b"")
}
pub(crate) fn command(b: &mut Builder, root: &Domain, w: &Words, name: &[u8], welcome: bool) {
    description(b, &command_prompt(root, w));
    b.open(Role::Code);
    b.style(if welcome { crate::look::WELCOME_CMD } else { crate::look::FLOW_CODE });
    b.text(Run::Plain, &printed_command(root, w, name));
    b.close();
}
pub(super) fn ready_body(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    way: Way,
    ways: &skein_lib::List<Way>,
) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let (brand, plan) = names(me, way);
    match way {
        Way::Key { .. } => {
            crate::cards::text(b, Role::Status, &named(w, W::KeyAdded, brand));
            text(b, w, Role::Paragraph, W::KeyRuns);
        }
        Way::Program { .. } => {
            let Some(pending) = &page.request.pending else { return };
            let name = match &pending.outcome {
                Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                    c::Outcome::Document(c::DocumentOutcome::AccountProgramDeclared { name, .. }),
                )))) => name.as_ref(),
                _ => b"",
            };
            let label = words::say(
                w,
                W::ProgramDeclared,
                &[Hole::Name(crate::choice::shown(plan, w)), Hole::Name(crate::choice::shown(name, w))],
            );
            crate::cards::text(b, Role::Status, &label);
            text(b, w, Role::Paragraph, W::ProgramRun);
            command(b, root, w, name, false);
            text(b, w, Role::Paragraph, W::ProgramNote);
        }
        Way::SignIn { .. } => {
            let mut ending = None;
            if let SignInState::Ended { source } = page.request.sign_in_state(root.client())
                && let Some(doc) = accounts::document(root.client())
                && let Some(last) = &doc.last_end
                && last.source == source
            {
                ending = Some(&last.end);
            }
            if let Some(end) = ending {
                if !sign_in_end(b, root, w, way, end) {
                    button(
                        b,
                        bindings,
                        &label(me, way, ways, w),
                        Act::SignIn { name: root.pages().account_name(), way },
                        false,
                        true,
                    );
                    if declined(end) {
                        for key in ways {
                            if model::model_way(*key).kind == WayKind::Key && key.provider() == way.provider() {
                                go(
                                    b,
                                    bindings,
                                    &label(me, *key, ways, w),
                                    model::address(root.pages().address(), Some(model::model_way(*key))),
                                    false,
                                );
                            }
                        }
                    }
                    return;
                }
            } else {
                super::accounts::paste(b, root, w, bindings, &page.request, false);
                return;
            }
            crate::cards::text(b, Role::Paragraph, &named(w, W::PlanRuns, plan));
        }
    }
    if model::model_way(way).kind != WayKind::Program {
        facts(b, root, w, way);
    }
    b.open(Role::Slot);
    b.style(crate::look::MODEL_ACTIONS);
    ready_primary(b, root, w, bindings);
    manage_usage(b, root, w, way);
    if model::model_way(way).kind == WayKind::SignIn {
        button(
            b,
            bindings,
            &label(me, way, ways, w),
            Act::SignIn { name: root.pages().account_name(), way },
            false,
            false,
        );
    }
    b.close();
}
fn ready_address(root: &Domain) -> Address {
    if welcome(root) {
        Address::Welcome { step: temper_client_domain::Step::Ready }
    } else {
        Address::NewChat { draft: false }
    }
}
fn ready_primary(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let name = if welcome(root) { w.say(Word::WelcomeContinue, &[]) } else { say(w, W::StartChat) };
    go(b, bindings, &name, ready_address(root), true);
}
/// Ready reads the last durable ending even after the signing tab has returned.
pub(super) fn welcome_end(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    let Some(doc) = accounts::document(root.client()) else { return };
    let Some(last) = &doc.last_end else { return };
    let declined = match last.end {
        c::AccountEnd::Made { declined: true, .. } | c::AccountEnd::Renewed { declined: true, .. } => true,
        c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => false,
        c::AccountEnd::Made { declined: false, .. } | c::AccountEnd::Renewed { declined: false, .. } => return,
    };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let ways = accounts::ways(&me.providers, root.forge().playground_provider(), c::AskAccountHolder::Own);
    // These endings are ChatGPT's: last_end has no provider on a Failed ending.
    let mut offered = None;
    for way in &ways {
        if let Way::SignIn { plan: true, .. } = way
            && names(me, *way).1 == b"ChatGPT"
        {
            offered = Some(*way);
            break;
        }
    }
    let Some(way) = offered else { return };
    b.open(Role::Slot);
    b.style(crate::look::MODEL_SIGN_IN);
    sign_in_end(b, root, w, way, &last.end);
    b.open(Role::Slot);
    b.style(crate::look::MODEL_ACTIONS);
    button(
        b,
        bindings,
        &label(me, way, &ways, w),
        Act::SignIn { name: root.pages().account_name(), way },
        false,
        false,
    );
    if declined && ways.as_slice().contains(&Way::Key { provider: way.provider() }) {
        go(
            b,
            bindings,
            &say(w, W::KeyWayOnly),
            Address::Welcome {
                step: temper_client_domain::Step::Model {
                    way: Some(model::model_way(Way::Key { provider: way.provider() })),
                },
            },
            false,
        );
    }
    b.close();
    b.close();
}
pub(super) fn sign_in_end(b: &mut Builder, root: &Domain, w: &Words, way: Way, end: &c::AccountEnd) -> bool {
    match end {
        c::AccountEnd::Made { declined: true, .. } | c::AccountEnd::Renewed { declined: true, .. } => {
            text(b, w, Role::Alert, W::DeclinedPlan);
        }
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Held { .. } } => {
            text(b, w, Role::Alert, W::HeldPlan);
        }
        c::AccountEnd::Failed { failure: c::AccountFailure::Misconfigured } => {
            text(b, w, Role::Alert, W::MisconfiguredPlan);
        }
        c::AccountEnd::Made { account, .. } | c::AccountEnd::Renewed { account, .. } => {
            let renewed = match end {
                c::AccountEnd::Renewed { .. } => true,
                c::AccountEnd::Made { .. } | c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => false,
            };
            let text = made_words(root, w, way, *account, renewed);
            if renewed || !model::open(root.pages().address()) {
                crate::cards::text(b, Role::Status, &text);
            }
            return !renewed;
        }
        c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => {
            if let Some(doc) = accounts::document(root.client()) {
                super::accounts::end(b, w, end, &doc.accounts);
            }
        }
    }
    false
}
fn pair(b: &mut Builder, w: &Words, name: W, value: &[u8]) {
    b.open(Role::Slot);
    b.name(&say(w, name));
    b.open(Role::Term);
    b.style(crate::look::MODEL_FACT);
    b.name(&say(w, name));
    b.text(Run::Plain, &say(w, name));
    b.close();
    b.open(Role::Description);
    b.style(crate::look::MODEL_FACT);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
    b.close();
}
fn facts(b: &mut Builder, root: &Domain, w: &Words, way: Way) {
    let Some(page) = root.pages().connect_model() else { return };
    let doc = accounts::document(root.client());
    let Some(me) = accounts::current_me(root.client()) else { return };
    let (brand, plan) = names(me, way);
    let outcome = match &page.request.pending {
        Some(pending) => pending.outcome.as_ref(),
        None => None,
    };
    let account = match doc {
        Some(doc) => match outcome {
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountKeyKept { source },
            ))))) => {
                let mut found = None;
                for row in &doc.accounts {
                    for entry in &row.sources {
                        if entry.number == *source {
                            found = Some(row);
                        }
                    }
                }
                found
            }
            _ => match &doc.last_end {
                Some(c::AccountLastEnd { end: c::AccountEnd::Made { account, .. }, .. }) => {
                    let mut found = None;
                    for row in &doc.accounts {
                        if row.number == *account {
                            found = Some(row);
                        }
                    }
                    found
                }
                _ => None,
            },
        },
        None => None,
    };
    let display = match account {
        Some(row) => row.display.bytes(),
        None => b"",
    };
    let name = if model::model_way(way).kind == WayKind::Key {
        named(w, W::ApiKeyBrand, brand)
    } else {
        words::join(&[plan, if display.is_empty() { b"" } else { b" \xc2\xb7 " }, display])
    };
    b.open(Role::Slot);
    b.style(crate::look::MODEL_FACTS);
    pair(b, w, W::ReadyAccount, &name);
    // The pinned contract has no resolved-default preview. An explicit chat
    // template choice is known without making another read.
    if let Some(project) = root.pages().project()
        && let Some(template) = jig_client::shapes::template(me, project, jig_client::shapes::CHAT)
        && let Some(choice) = &template.llm
    {
        pair(b, w, W::ReadyModel, &crate::choice::shown(choice.main.model.bytes(), w));
    }
    pair(
        b,
        w,
        W::PaysFor,
        &say(
            w,
            if is_plan(way) {
                W::PlanPays
            } else if page.holder == c::AskAccountHolder::Own {
                W::OwnPays
            } else {
                W::DeploymentPays
            },
        ),
    );
    b.close();
}
fn manage_usage(b: &mut Builder, root: &Domain, w: &Words, way: Way) {
    let Some(doc) = accounts::document(root.client()) else { return };
    let Some(c::AccountLastEnd { end: c::AccountEnd::Made { account, declined: false }, .. }) = &doc.last_end else {
        return;
    };
    let mut found = None;
    for row in &doc.accounts {
        if row.number == *account {
            found = Some(row);
            break;
        }
    }
    if is_plan(way)
        && let Some(account) = found
        && let Some(address) = &account.manage_usage
        && c::valid_manage_usage(address)
        && skein_ui::markdown::safe_url(address)
    {
        b.open(Role::Link);
        b.style(crate::look::FLOW_QUIET);
        let name = w.say(Word::Usage, &[]);
        b.name(&name);
        b.text(Run::Plain, &name);
        b.external(address);
        b.close();
    }
}

fn declined(end: &c::AccountEnd) -> bool {
    match end {
        c::AccountEnd::Made { declined, .. } | c::AccountEnd::Renewed { declined, .. } => *declined,
        c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => false,
    }
}
fn is_plan(way: Way) -> bool {
    match way {
        Way::SignIn { plan, .. } => plan,
        Way::Key { .. } | Way::Program { .. } => false,
    }
}

fn made_words(root: &Domain, w: &Words, way: Way, account: u32, renewed: bool) -> Box<[u8]> {
    let Some(me) = accounts::current_me(root.client()) else { return say(w, W::Unknown) };
    let (_, plan) = names(me, way);
    let mut display = b"".as_slice();
    if let Some(doc) = accounts::document(root.client()) {
        for row in &doc.accounts {
            if row.number == account {
                display = row.display.bytes();
                break;
            }
        }
    }
    let fallback = words::join(&[b"your ", plan, b" account"]);
    if renewed {
        words::say(
            w,
            W::RenewedNamed,
            &[
                Hole::Name(skein_lib::bytes::copy_of(plan)),
                Hole::Name(skein_lib::bytes::copy_of(if display.is_empty() { &fallback } else { display })),
                Hole::Name(skein_lib::bytes::copy_of(plan)),
            ],
        )
    } else if display.is_empty() {
        named(w, W::MadePlan, plan)
    } else {
        words::say(
            w,
            W::MadeNamed,
            &[Hole::Name(skein_lib::bytes::copy_of(plan)), Hole::Name(skein_lib::bytes::copy_of(display))],
        )
    }
}
