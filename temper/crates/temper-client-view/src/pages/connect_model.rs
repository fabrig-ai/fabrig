//! The standalone model connection flow and parts used by the welcome.
use crate::{
    accounts::{
        Bindings, Meaning,
        words::{self, Word as W},
    },
    flow::{self, Control},
    look,
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
pub(super) fn key_heading(brand: &[u8], w: &Words) -> Box<[u8]> {
    named(
        w,
        match brand.first() {
            Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => W::KeyTitle,
            _ => W::KeyTitleA,
        },
        brand,
    )
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
        Way::Key { .. } => key_heading(brand, w),
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
    if welcome(root) {
        flow::choice_card(b, &heading);
    }
    crate::cards::text(b, Role::Paragraph, &description);
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    b.name(&say(w, note));
    b.text(Run::Plain, &say(w, note));
    b.close();
    b.open(Role::Slot);
    b.style(if welcome(root) { look::FLOW_ACTIONS } else { look::CONNECT_ACTS });
    if welcome(root) {
        flow::control(b, control);
    } else {
        connect_control(
            b,
            root,
            bindings,
            &label,
            Role::Button,
            Meaning::Model(Act::SignIn { name: root.pages().account_name(), way }),
            true,
            false,
        );
        other_ways(b, root, w, bindings);
    }
    b.close();
    if welcome(root) {
        b.close();
    }
}
/// Shared plan cards (temper/docs/design/client/pages/README.md, `ConnectModel` and Model).
fn cards(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, ways: &skein_lib::List<Way>) {
    let Some(me) = accounts::current_me(root.client()) else { return };
    let mut plans = skein_lib::List::with_capacity(ways.len());
    for name in [b"anthropic".as_slice(), b"openai"] {
        for offer in &me.providers {
            if offer.name.bytes() != name {
                continue;
            }
            for way in ways {
                if way.provider() == offer.provider {
                    match way {
                        Way::SignIn { .. } | Way::Program { .. } => {
                            plans.push(*way).expect("one card per provider");
                            break;
                        }
                        Way::Key { .. } => {}
                    }
                }
            }
        }
    }
    if plans.is_empty() && welcome(root) {
        b.open(Role::Slot);
        b.style(look::MODEL_CHOICES);
        for way in ways {
            choice(b, root, w, bindings, *way, ways);
        }
        b.close();
        return;
    }
    if plans.is_empty() {
        for way in ways {
            match way {
                Way::SignIn { .. } | Way::Program { .. } => plans.push(*way).expect("offered plans fit"),
                Way::Key { .. } => {}
            }
        }
    }
    let default = model::default_way(&plans, root);
    if plans.is_empty() {
        return;
    }
    b.open(Role::Slot);
    b.style(if welcome(root) { look::MODEL_CHOICES } else { look::CONNECT_CARDS });
    for way in &plans {
        plan_card(b, root, w, bindings, *way, ways, default);
    }
    b.close();
}
fn plan_card(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    way: Way,
    ways: &skein_lib::List<Way>,
    default: Option<Way>,
) {
    let Some(me) = accounts::current_me(root.client()) else { return };
    let (_, plan) = names(me, way);
    let heading = crate::choice::shown(plan, w);
    b.open(Role::Region);
    b.name(&heading);
    if welcome(root) {
        b.style(look::FLOW_CARD);
        b.style(look::FLOW_CHOICE);
    } else {
        b.style(look::CONNECT_CARD);
        if default == Some(way) {
            b.style(look::CONNECT_DEFAULT);
        }
    }
    crate::cards::text(b, Role::Heading { level: 2 }, &heading);
    let name = [Hole::Name(crate::choice::shown(plan, w))];
    let connected = model::connected(root, way);
    let lendable = match root.pages().connect_model() {
        Some(page) => page.lendable(root.client()),
        None => false,
    };
    let terminal = model::model_way(way).kind == WayKind::Program && plan == b"Claude" && !connected && lendable;
    let (description, note) = match way {
        Way::Program { .. } => {
            (w.say(Word::WelcomeModelSubscription, &name), w.say(if terminal { Word::WelcomeClaudeTerminal } else { Word::WelcomeModelProgramTime }, &[]))
        }
        Way::SignIn { .. } | Way::Key { .. } => {
            (w.say(Word::WelcomeModelPlan, &name), w.say(Word::WelcomeModelSignInTime, &[]))
        }
    };
    crate::cards::text(b, Role::Paragraph, &description);
    b.open(Role::Paragraph);
    b.style(if welcome(root) { look::FLOW_MUTED } else { look::CONNECT_MUTED });
    if connected {
        b.open(Role::Text);
        b.style(look::CONNECT_YES);
        b.name(&say(w, W::Connected));
        b.text(Run::Plain, &say(w, W::Connected));
        b.close();
    } else {
        b.name(&note);
        b.text(Run::Plain, &note);
    }
    b.close();
    b.open(Role::Slot);
    b.style(if welcome(root) { look::FLOW_ACTIONS } else { look::CONNECT_ACTS });
    let label = if connected { say(w, W::AnotherPlan) } else if terminal { w.say(Word::WelcomeClaudeLogin, &[]) } else { label(me, way, ways, w) };
    let (role, meaning) = if terminal {
        (Role::Button, Meaning::Model(Act::LoginProgram { name: root.pages().account_name() }))
    } else { match way {
        Way::SignIn { .. } => (Role::Button, Meaning::Model(Act::SignIn { name: root.pages().account_name(), way })),
        Way::Key { .. } | Way::Program { .. } => {
            (Role::Link, Meaning::Go(model::address(root.pages().address(), Some(model::model_way(way)))))
        }
    }
    };
    connect_control(b, root, bindings, &label, role, meaning, default == Some(way), false);
    b.close();
    b.close();
}

#[expect(clippy::too_many_arguments, reason = "the connection control keeps its role, meaning and state together")]
fn connect_control(
    b: &mut Builder,
    root: &Domain,
    bindings: &mut Bindings,
    name: &[u8],
    role: Role,
    meaning: Meaning,
    primary: bool,
    disabled: bool,
) {
    let Some(to) = bindings.insert(meaning) else { return };
    b.open(role);
    b.style(if welcome(root) {
        if primary { look::FLOW_PRIMARY } else { look::FLOW_SECONDARY }
    } else if primary {
        look::CONNECT_PRIMARY
    } else {
        look::CONNECT_SECONDARY
    });
    b.name(name);
    if role == Role::Link {
        b.link(to);
        b.text(Run::Plain, name);
    }
    b.bind(to);
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
pub(super) fn other_ways(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    if welcome(root) {
        return;
    }
    if let Some(to) = bindings.insert(Meaning::Go(model::address(root.pages().address(), None))) {
        crate::link(b, &say(w, W::AllWays), to);
        b.style(look::CONNECT_QUIET);
        b.text(Run::Plain, &say(w, W::AllWays));
        b.close();
    }
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
    b.style(look::MAIN);
    b.style(look::SETTINGS);
    b.open(Role::Slot);
    b.style(look::SETTINGS_COL);
    b.style(look::CONNECT);
    let way = model::way(root.pages(), root.client());
    if let Some(way) = way
        && let Some(me) = accounts::current_me(root.client())
    {
        let (brand, plan) = names(me, way);
        let name = match way {
            Way::SignIn { plan: true, .. } | Way::Program { .. } => crate::choice::shown(plan, w),
            Way::SignIn { plan: false, .. } => crate::choice::shown(brand, w),
            Way::Key { .. } => named(w, W::ApiKeyBrand, brand),
        };
        super::settings::way_path(
            b,
            root,
            w,
            pages,
            project,
            super::settings::FlowTab::Accounts,
            &super::settings::FlowWay {
                choices: Word::PathConnectAccount,
                address: Address::ConnectModel { way: None },
                name: &name,
            },
        );
    } else {
        super::settings::path(b, root, w, pages, project, super::settings::FlowTab::Accounts, Word::PathConnectAccount);
    }
    let heading = match way {
        Some(way) => way_heading(root, w, way),
        None => w.say(Word::ConnectModel, &[]),
    };
    b.open(Role::Heading { level: 1 });
    b.style(look::SETTINGS_HEADLINE);
    b.name(&heading);
    b.text(Run::Plain, &heading);
    b.close();
    if way.is_none() && !super::claude_login::waiting(root) {
        b.open(Role::Paragraph);
        b.style(look::SETTINGS_LEDE);
        b.name(&say(w, W::ModelLead));
        b.text(Run::Plain, &say(w, W::ModelLead));
        b.close();
    }
    body(b, root, w, bindings, text_per_node);
    b.close();
    b.close();
}
fn way_heading(root: &Domain, w: &Words, way: Way) -> Box<[u8]> {
    let Some(me) = accounts::current_me(root.client()) else { return say(w, W::Unknown) };
    let (brand, plan) = names(me, way);
    match way {
        Way::SignIn { plan: true, .. } => named(w, W::YourPlan, plan),
        Way::SignIn { plan: false, .. } => signing_heading(root, w, way),
        Way::Program { .. } => w.say(Word::ModelProgramTitle, &[Hole::Name(crate::choice::shown(plan, w))]),
        Way::Key { .. } => key_heading(brand, w),
    }
}

fn welcome(root: &Domain) -> bool {
    match root.pages().address() {
        Address::Welcome { .. } => true,
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        // page-view: draw Remembered.
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Chat { .. }
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
        | Address::Ask { .. }
        | Address::Adopt { .. } => false,
    }
}
fn description(b: &mut Builder, words: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::MODEL_DESCRIPTION);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
pub(super) fn signing_heading(root: &Domain, w: &Words, way: Way) -> Box<[u8]> {
    let Some(me) = accounts::current_me(root.client()) else { return say(w, W::SigningIn) };
    let (_, plan) = names(me, way);
    w.say(Word::ModelSigningIn, &[Hole::Name(crate::choice::shown(plan, w))])
}
pub(super) fn key_field(b: &mut Builder, root: &Domain, bindings: &mut Bindings, busy: bool, word: &[u8]) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(to) = bindings.insert(Meaning::Key) else { return };
    b.open(Role::TextField);
    b.style(if welcome(root) { look::FLOW_FIELD } else { look::SETTINGS_TEXT });
    b.name(word);
    b.entry(skein_ui::EntryKind::Secret);
    b.value(b"", page.request.key_written());
    b.bind(to);
    if busy {
        b.state(State::Disabled);
    }
    b.close();
}
pub(super) fn choose_body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    let Some(page) = root.pages().connect_model() else { return };
    let Some(me) = accounts::current_me(root.client()) else { return };
    let ways = accounts::ways(&me.providers, root.forge().playground_provider(), page.holder);
    super::claude_login::ending(b, root, w, bindings, text_per_node);
    cards(b, root, w, bindings, &ways);
    if !welcome(root) {
        b.open(Role::Slot);
        b.style(look::CONNECT_MORE);
        text(b, w, Role::Heading { level: 2 }, W::OrKey);
        b.open(Role::Slot);
        b.style(look::CONNECT_ROWS);
        for way in &ways {
            match way {
                Way::SignIn { .. } | Way::Program { .. } => continue,
                Way::Key { .. } => {}
            }
            let (brand, _) = names(me, *way);
            b.open(Role::Slot);
            b.style(look::CONNECT_ROW);
            b.name(&named(w, W::ApiKeyBrand, brand));
            b.open(Role::Slot);
            b.open(Role::Text);
            b.run(Run::Strong, &named(w, W::ApiKeyBrand, brand));
            b.close();
            b.open(Role::Text);
            b.text(Run::Plain, &say(w, W::KeyBilling));
            b.close();
            b.close();
            connect_control(
                b,
                root,
                bindings,
                &say(w, W::AddKey),
                Role::Link,
                Meaning::Go(model::address(root.pages().address(), Some(model::model_way(*way)))),
                false,
                false,
            );
            b.close();
        }
        b.close();
        b.close();
        if me.deployment_role == Some(c::HeldRole::Owner) {
            let (holder, word) = if page.holder == c::AskAccountHolder::Own {
                (c::AskAccountHolder::Deployment, W::DeploymentChoice)
            } else {
                (c::AskAccountHolder::Own, W::OwnChoice)
            };
            if let Some(to) = bindings.insert(Meaning::Model(Act::Holder { holder })) {
                crate::link(b, &say(w, word), to);
                b.style(look::CONNECT_QUIET);
                b.text(Run::Plain, &say(w, word));
                if page.request.busy(root.client()) {
                    b.state(State::Disabled);
                }
                b.close();
            }
        }
    }
    if !super::claude_login::owns_request(page) {
        super::accounts::outcome(b, root, w, bindings, &page.request, text_per_node);
    }
}
pub(crate) fn body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, text_per_node: u32) {
    if super::claude_login::waiting(root) {
        super::claude_login::note(b, root, w, bindings);
        return;
    }
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
        choose_body(b, root, w, bindings, text_per_node);
        return;
    };
    if ready(root, way) {
        b.open(Role::Slot);
        b.style(look::CONNECT_PANEL);
        ready_body(b, root, w, bindings, way, &ways);
        b.close();
        return;
    }
    if request_for_way(page, way)
        && let SignInState::Started { .. } | SignInState::Signing { .. } = page.request.sign_in_state(root.client())
    {
        b.open(Role::Slot);
        b.style(look::CONNECT_PANEL);
        super::sign_in_wait::waiting(
            b,
            root,
            w,
            bindings,
            &page.request,
            super::sign_in_wait::tab_refused(page),
            text_per_node,
        );
        b.close();
        return;
    }
    b.open(Role::Slot);
    b.style(look::CONNECT_PANEL);
    match way {
        Way::SignIn { .. } => {
            if !request_for_way(page, way) || page.request.sign_in_state(root.client()) == SignInState::Unknown {
                choice(b, root, w, bindings, way, &ways);
            }
        }
        Way::Key { .. } => key_details(b, root, w, bindings, way, busy),
        Way::Program { .. } => program_body(b, root, w, bindings, way, busy),
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
pub(super) fn program_body(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, _way: Way, busy: bool) {
    let Some(page) = root.pages().connect_model() else { return };
    crate::cards::text(b, Role::Paragraph, &w.say(Word::WelcomeClaudeBody, &[]));
    b.open(Role::Slot);
    b.style(look::CONNECT_CMD);
    b.open(Role::Code);
    b.text(Run::Plain, &printed_command(root, w, &page.program_name(root.client())));
    b.close();
    if w.shell == crate::words::Shell::Browser {
        let word = match root.copy_label() {
            temper_client_domain::copy::Label::Copy => Word::WelcomeCopy,
            temper_client_domain::copy::Label::Copied => Word::WelcomeCopied,
            temper_client_domain::copy::Label::NotCopied => Word::WelcomeNotCopied,
        };
        if let Some(to) =
            bindings.insert(Meaning::Copy { words: printed_command(root, w, &page.program_name(root.client())) })
        {
            b.open(Role::Button);
            b.name(&w.say(word, &[]));
            b.bind(to);
            b.close();
        }
    }
    b.close();
    b.open(Role::Paragraph);
    b.style(look::CONNECT_MUTED);
    b.text(Run::Plain, &w.say(Word::WelcomeClaudeNote, &[]));
    b.close();
    b.open(Role::Disclosure);
    b.name(&say(w, W::AnotherLogin));
    b.open(Role::Slot);
    b.open(Role::Slot);
    if let Some(to) = bindings.insert(Meaning::Program) {
        b.open(Role::TextField);
        b.style(look::SETTINGS_TEXT);
        b.name(&w.say(Word::ModelProgramName, &[]));
        b.value(&page.program_name(root.client()), page.request.key_written());
        b.bind(to);
        if !c::valid_program_name(&page.program_name(root.client())) {
            b.state(State::Invalid);
        }
        if busy {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.close();
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::CONNECT_ACTS);
    connect_control(
        b,
        root,
        bindings,
        &w.say(Word::WelcomeClaudeSignedIn, &[]),
        Role::Button,
        Meaning::Model(Act::Submit { name: root.pages().account_name() }),
        true,
        busy || !c::valid_program_name(&page.program_name(root.client())),
    );
    other_ways(b, root, w, bindings);
    b.close();
}
fn key_details(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, way: Way, busy: bool) {
    let Some(me) = accounts::current_me(root.client()) else { return };
    let Some(page) = root.pages().connect_model() else { return };
    let (brand, _) = names(me, way);
    crate::cards::text(b, Role::Paragraph, &words::key_description(brand, w));
    b.open(Role::Form);
    b.name(&say(w, W::Details));
    key_field(b, root, bindings, busy, &w.say(Word::ModelKeyField, &[]));
    b.close();
    b.open(Role::Slot);
    b.style(look::CONNECT_ACTS);
    connect_control(
        b,
        root,
        bindings,
        &w.say(Word::ModelAddKey, &[]),
        Role::Button,
        Meaning::Model(Act::Submit { name: root.pages().account_name() }),
        true,
        busy || !page.request.has_key(),
    );
    other_ways(b, root, w, bindings);
    b.close();
}
/// The whole command for NAME (client/forge/records.md, *The printed command*).
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
pub(super) fn command_prompt(root: &Domain, w: &Words) -> Box<[u8]> {
    let word = match root.forge().deployment() {
        Some(answer) if answer.program_login.is_some() => W::ProgramCommandHere,
        Some(_) | None => W::ProgramCommand,
    };
    named(w, word, b"")
}
pub(crate) fn command(b: &mut Builder, root: &Domain, w: &Words, name: &[u8], welcome: bool) {
    description(b, &command_prompt(root, w));
    b.open(Role::Code);
    b.style(if welcome { look::WELCOME_CMD } else { look::FLOW_CODE });
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
    let (_, plan) = names(me, way);
    match way {
        Way::Key { .. } | Way::Program { .. } => return,
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
    b.style(look::MODEL_ACTIONS);
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
    b.style(look::MODEL_SIGN_IN);
    sign_in_end(b, root, w, way, &last.end);
    b.open(Role::Slot);
    b.style(look::MODEL_ACTIONS);
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
    b.style(look::MODEL_FACT);
    b.name(&say(w, name));
    b.text(Run::Plain, &say(w, name));
    b.close();
    b.open(Role::Description);
    b.style(look::MODEL_FACT);
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
        Some(row) => words::who(row.email(), row.display.bytes()),
        None => b"",
    };
    let name = if model::model_way(way).kind == WayKind::Key {
        named(w, W::ApiKeyBrand, brand)
    } else {
        words::join(&[plan, if display.is_empty() { b"" } else { b" \xc2\xb7 " }, display])
    };
    b.open(Role::Slot);
    b.style(look::MODEL_FACTS);
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
        b.style(look::FLOW_QUIET);
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
                display = words::who(row.email(), row.display.bytes());
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
