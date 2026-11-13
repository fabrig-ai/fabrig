#![allow(clippy::disallowed_macros, reason = "unit assertions match typed requests and capture codec diagnostics")]
use super::{
    accounts::{deliver, document, event, frame, key, opened, sends, started},
    fixtures,
};
use crate::{
    Action, Address, Event, Step,
    pages::{
        self,
        accounts::{SignInState, Way},
        connect_model::{self, Act},
    },
};
use jig_client::contract as c;
use skein_lib::{Queue, Token};
const PROVIDER: u32 = 17;
const NAME: [u8; 16] = [97; 16];
const LOGIN: u32 = 91;

fn terminal_fixture(welcome: bool) -> (crate::Domain, Token) {
    let (mut root, me_watch, watch) = opened(c::HeldRole::Owner);
    let mut me = pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers.get_mut(0).unwrap().name =
        c::DocumentText::complete(c::Text::new(b"anthropic".as_slice().into(), 32).unwrap());
    deliver(&mut root, me_watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))));
    let address = if welcome {
        Address::Welcome { step: Step::Model { way: None } }
    } else {
        Address::ConnectModel { way: None }
    };
    event(&mut root, Event::Went { place: crate::Place::Address(address) });
    let c::WatchEvent::Delivered(c::Delivery::Accounts(mut doc)) = document(false) else { unreachable!() };
    doc.lendable = true;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
    (root, watch)
}
fn login_state(root: &crate::Domain) -> connect_model::LoginState {
    root.pages().connect_model().unwrap().login_state(root.client())
}
fn login_document(root: &crate::Domain) -> Box<c::Accounts> {
    Box::new(pages::accounts::document(root.client()).unwrap().clone())
}
fn lending() -> c::TerminalLogin {
    c::TerminalLogin { number: LOGIN, provider: PROVIDER, name: b"claude-pat".as_slice().into(), cancelled: false }
}
#[test]
fn a_terminal_login_asks_for_claudes_default_name_and_cancels_by_number() {
    use connect_model::LoginState as S;
    for cancel_before_answer in [false, true] {
        let (mut root, watch) = terminal_fixture(true);
        // The card always uses the default, not a name retained from a way's panel.
        act(&mut root, Act::ProgramName { value: b"another-login".as_slice().into() });
        let sent = sends(act(&mut root, Act::LoginProgram { name: NAME })).pop().unwrap();
        assert_eq!(
            sent.2,
            jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                holder: c::AskAccountHolder::Own,
                change: c::AccountChange::LoginProgram { provider: PROVIDER, name: b"claude-pat".as_slice().into() },
            }))
        );
        assert_eq!(login_state(&root), S::Waiting { number: None, cancelled: false });
        assert_eq!(pages::welcome::model_step(&root), Some(pages::welcome::ModelStep::TerminalLogin));
        assert_no_moves(&act(&mut root, Act::LoginProgram { name: NAME }));
        if cancel_before_answer {
            assert!(sends(act(&mut root, Act::CancelLogin)).is_empty());
        }
        let out = answered(&mut root, sent.0, done(c::DocumentOutcome::AccountLoginAsked { number: LOGIN }));
        assert_no_moves(&out);
        let cancel =
            if cancel_before_answer { sends(out) } else { sends(act(&mut root, Act::CancelLogin)) }.pop().unwrap();
        assert_eq!(
            cancel.2,
            jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                holder: c::AskAccountHolder::Own,
                change: c::AccountChange::CancelLogin { number: LOGIN },
            }))
        );
        assert!(sends(act(&mut root, Act::CancelLogin)).is_empty());
        answered(&mut root, cancel.0, done(c::DocumentOutcome::AccountChanged));
        assert_eq!(login_state(&root), S::Waiting { number: Some(LOGIN), cancelled: true });
        let mut doc = login_document(&root);
        doc.lent_end = Some(c::AccountLentEnd { number: LOGIN, end: c::LoginEnd::Cancelled });
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
        assert_eq!(login_state(&root), S::Idle);
        assert_eq!(pages::welcome::model_step(&root), Some(pages::welcome::ModelStep::Choose));
    }
}
#[test]
fn a_reopened_terminal_login_waits_for_its_source_and_only_settings_moves_on() {
    use connect_model::LoginState as S;
    for welcome in [false, true] {
        let (mut root, watch) = terminal_fixture(welcome);
        let mut doc = login_document(&root);
        doc.lending = Some(lending());
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
        assert_eq!(login_state(&root), S::Waiting { number: Some(LOGIN), cancelled: false });
        // No press or answer was observed by this page.
        program_document(&mut root, watch, false, false);
        let mut doc = login_document(&root);
        doc.lent_end = Some(c::AccountLentEnd { number: LOGIN, end: c::LoginEnd::SignedIn { source: 43 } });
        let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc.clone())));
        assert_no_moves(&out);
        assert_eq!(login_state(&root), S::Waiting { number: Some(LOGIN), cancelled: false });
        doc.settled = true;
        doc.program_hosts = skein_lib::List::from_boxed(Box::new([c::ProgramHost {
            name: b"worker".as_slice().into(),
            channel: 1,
            connected: true,
            awaiting_logins: false,
            cut: false,
            programs: skein_lib::List::from_boxed(Box::new([c::ProgramLogin {
                name: b"claude-pat".as_slice().into(),
                state: None,
                sign_out: None,
                cleanup: false,
                cut: false,
            }])),
        }]));
        let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc.clone())));
        assert_no_moves(&out);
        assert_eq!(login_state(&root), S::Waiting { number: Some(LOGIN), cancelled: false });
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().usable = true;
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc.clone())));
        assert_eq!(login_state(&root), S::Waiting { number: Some(LOGIN), cancelled: false });
        doc.program_hosts.clear();
        doc.usable = true;
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().usable = true;
        let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
        if welcome {
            assert_no_moves(&out);
            assert_eq!(login_state(&root), S::Usable { provider: PROVIDER });
            assert_eq!(pages::welcome::model_step(&root), Some(pages::welcome::ModelStep::Usable));
        } else {
            assert_eq!(root.pages().address(), &Address::Accounts);
            assert_eq!(
                root.pages().accounts().unwrap().added,
                Some(pages::accounts::Added { provider: PROVIDER, kind: crate::WayKind::Program })
            );
        }
    }
}
#[test]
fn terminal_login_endings_and_press_refusals_return_to_the_right_form() {
    use connect_model::LoginState as S;
    for (end, expected) in [
        (c::LoginEnd::Cancelled, S::Idle),
        (c::LoginEnd::NotFinished { exit: None }, S::NotFinished),
        (c::LoginEnd::NotFinished { exit: Some(c::LoginExit::Exited { code: 1 }) }, S::NotFinished),
        (
            c::LoginEnd::Refused { refusal: c::AccountRefusal::ProgramNameReserved },
            S::Refused { refusal: c::AccountRefusal::ProgramNameReserved },
        ),
        (c::LoginEnd::SignedIn { source: 43 }, S::NotFinished),
    ] {
        let (mut root, watch) = terminal_fixture(true);
        program_document(&mut root, watch, true, false);
        let mut doc = login_document(&root);
        doc.lendable = false;
        doc.lent_end = Some(c::AccountLentEnd { number: LOGIN, end });
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
        assert_eq!(login_state(&root), expected);
        assert_eq!(pages::welcome::model_step(&root), Some(pages::welcome::ModelStep::Choose));
        // A new press must not read the preceding login's ending.
        let mut doc = login_document(&root);
        doc.lendable = true;
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
        act(&mut root, Act::LoginProgram { name: NAME });
        assert_eq!(login_state(&root), S::Waiting { number: None, cancelled: false });
    }
    for (refusal, waiting, lendable) in [
        (c::Refusal::NoTerminal, false, false),
        (c::Refusal::LoginUnderWay, true, true),
        (c::Refusal::NotOffered, false, true),
    ] {
        let (mut root, watch) = terminal_fixture(true);
        let sent = sends(act(&mut root, Act::LoginProgram { name: NAME })).pop().unwrap();
        answered(&mut root, sent.0, c::Answer::Refused { refusal, at: None });
        assert_eq!(
            match login_state(&root) {
                S::Waiting { .. } => true,
                S::Idle | S::Usable { .. } | S::NotFinished | S::Refused { .. } => false,
            },
            waiting
        );
        assert_eq!(root.pages().connect_model().unwrap().lendable(root.client()), lendable);
        if !lendable {
            let mut doc = login_document(&root);
            doc.lendable = false;
            deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc.clone())));
            doc.lendable = true;
            deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
            assert!(root.pages().connect_model().unwrap().lendable(root.client()));
        }
    }
}

fn go(root: &mut crate::Domain, way: Option<Way>) {
    let way = match way {
        Some(way) => Some(connect_model::model_way(way)),
        None => None,
    };
    event(root, Event::Went { place: crate::Place::Address(Address::ConnectModel { way }) });
}
fn act(root: &mut crate::Domain, act: Act) -> Queue<crate::Request> {
    event(root, Event::Act { action: Action::Pages(pages::Act::ConnectModel(act)) })
}
fn root(way: Way) -> (crate::Domain, Token, Token) {
    let (mut root, me, accounts) = opened(c::HeldRole::Owner);
    go(&mut root, Some(way));
    (root, me, accounts)
}
#[test]
fn a_sign_in_way_sends_one_own_sign_in_request() {
    let way = Way::SignIn { provider: PROVIDER, plan: true };
    let (mut root, _, _) = root(way);
    let sent = sends(act(&mut root, Act::SignIn { name: NAME, way })).pop().unwrap();
    assert_eq!(
        sent.2,
        jig_client::Ask::Jig(Box::new(c::Ask::SignIn { holder: c::AskAccountHolder::Own, provider: PROVIDER }))
    );
    assert!(sends(act(&mut root, Act::SignIn { name: NAME, way })).is_empty());
}
#[test]
fn a_key_way_adds_a_new_account_with_the_value_privately_once() {
    let (mut root, _, _) = root(Way::Key { provider: PROVIDER });
    act(&mut root, Act::Holder { holder: c::AskAccountHolder::Deployment });
    key(&mut root, b"private-once");
    let sent = sends(act(&mut root, Act::Submit { name: NAME })).pop().unwrap();
    assert_eq!(
        sent.2,
        jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AskAccountHolder::Deployment,
            change: c::AccountChange::AddKey {
                provider: PROVIDER,
                join: None,
                digest: skein_lib::sha256::digest(b"private-once")
            }
        }))
    );
    assert_eq!(sent.3.unwrap().expose(), b"private-once");
    assert!(!root.pages().connect_model().unwrap().request.has_key());
    assert!(sends(act(&mut root, Act::Submit { name: NAME })).is_empty());
}
#[test]
fn a_program_way_declares_the_default_name_for_the_person() {
    let (mut root, _, _) = root(Way::Program { provider: PROVIDER });
    assert!(root.pages().connect_model().unwrap().program.is_none());
    let sent = sends(act(&mut root, Act::Submit { name: NAME })).pop().unwrap();
    assert_eq!(
        sent.2,
        jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AskAccountHolder::Own,
            change: c::AccountChange::DeclareProgram {
                provider: PROVIDER,
                join: None,
                name: Box::from(b"claude-pat".as_slice())
            }
        }))
    );
}
#[test]
fn an_invalid_or_overlong_program_name_is_never_declared() {
    for value in [b"".as_slice(), b"Alice", b"bad name", &[b'a'; 33]] {
        let (mut root, _, _) = root(Way::Program { provider: PROVIDER });
        act(&mut root, Act::ProgramName { value: Box::from(value) });
        assert!(sends(act(&mut root, Act::Submit { name: NAME })).is_empty());
    }
}
#[test]
fn the_program_default_uses_the_lowercased_first_name_within_the_name_bound() {
    for (person, expected) in [
        (b"ALICE Example".as_slice(), b"claude-alice".as_slice()),
        (b"abcdefghijklmnopqrstuvwxyz", b"claude-abcdefghijklmnopqrstuvwxy"),
    ] {
        let (mut root, watch, _) = root(Way::Program { provider: PROVIDER });
        let c::WatchEvent::Delivered(c::Delivery::Me(mut me)) = frame(c::HeldRole::Owner, fixtures::PARTY) else {
            unreachable!()
        };
        me.party.name = c::DocumentText::complete(c::Text::new(Box::from(person), 32).unwrap());
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(me)));
        assert_eq!(root.pages().connect_model().unwrap().program_name(root.client()).as_ref(), expected);
    }
}
#[test]
fn a_program_name_at_its_bound_is_declared() {
    let (mut root, _, _) = root(Way::Program { provider: PROVIDER });
    act(&mut root, Act::ProgramName { value: Box::new([b'a'; 32]) });
    let sent = sends(act(&mut root, Act::Submit { name: NAME })).pop().unwrap();
    let jig_client::Ask::Jig(ask) = sent.2 else { panic!("jig request") };
    let c::Ask::AccountChange { change: c::AccountChange::DeclareProgram { name, .. }, .. } = *ask else {
        panic!("program declaration")
    };
    assert_eq!(name.as_ref(), [b'a'; 32]);
}
#[test]
fn a_key_at_capacity_can_create_a_standalone_account() {
    let (mut root, _, _) = root(Way::Key { provider: PROVIDER });
    key(&mut root, &[b'x'; 4096]);
    assert!(sends(act(&mut root, Act::Submit { name: NAME })).pop().is_some());
}
#[test]
fn the_model_draft_is_kept_between_steps_and_dropped_on_leaving() {
    let way = Way::Key { provider: PROVIDER };
    let (mut root, _, _) = root(way);
    act(&mut root, Act::Holder { holder: c::AskAccountHolder::Deployment });
    key(&mut root, b"private-draft");
    for address in
        [Address::ConnectModel { way: None }, Address::ConnectModel { way: Some(connect_model::model_way(way)) }]
    {
        event(&mut root, Event::Went { place: crate::Place::Address(address) });
        let page = root.pages().connect_model().unwrap();
        assert!(page.request.has_key());
        assert_eq!(page.holder, c::AskAccountHolder::Deployment);
    }
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
    assert!(root.pages().connect_model().is_none());
    go(&mut root, Some(way));
    assert!(!root.pages().connect_model().unwrap().request.has_key());
    assert_eq!(root.pages().connect_model().unwrap().holder, c::AskAccountHolder::Own);
}
#[test]
fn unavailable_and_playground_ways_resolve_to_choose() {
    let (mut root, _, _) = root(Way::Key { provider: PROVIDER + 1 });
    assert_eq!(connect_model::way(root.pages(), root.client()), None);
    go(&mut root, Some(Way::Key { provider: PROVIDER }));
    root.pages.playground_provider = Some(PROVIDER);
    assert_eq!(connect_model::way(root.pages(), root.client()), None);
}
#[test]
fn deployment_offers_neither_sign_in_nor_program() {
    let (mut root, _, _) = root(Way::SignIn { provider: PROVIDER, plan: true });
    act(&mut root, Act::Holder { holder: c::AskAccountHolder::Deployment });
    assert_eq!(connect_model::way(root.pages(), root.client()), None);
    assert!(
        sends(act(&mut root, Act::SignIn { name: NAME, way: Way::SignIn { provider: PROVIDER, plan: true } }))
            .is_empty()
    );
    go(&mut root, Some(Way::Program { provider: PROVIDER }));
    assert_eq!(connect_model::way(root.pages(), root.client()), None);
}
#[test]
fn a_sign_in_starts_then_ends_only_from_the_accounts_watch() {
    let way = Way::SignIn { provider: PROVIDER, plan: true };
    let (mut root, _, watch) = root(way);
    act(&mut root, Act::SignIn { name: NAME, way });
    let pending = root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().key;
    started(&mut root, pending);
    assert_eq!(
        root.pages().connect_model().unwrap().request.sign_in_state(root.client()),
        SignInState::Started { source: 43 }
    );
    deliver(&mut root, watch, document(true));
    assert_eq!(
        root.pages().connect_model().unwrap().request.sign_in_state(root.client()),
        SignInState::Signing { source: 43 }
    );
    deliver(&mut root, watch, document(false));
    assert_eq!(
        root.pages().connect_model().unwrap().request.sign_in_state(root.client()),
        SignInState::Ended { source: 43 }
    );
    assert!(root.pages().connect_model().unwrap().request.sign_in_address(root.client()).is_none());
}
#[test]
fn creation_kept_work_belongs_to_the_open_model_flow() {
    for ask in [
        c::Ask::SignIn { holder: c::AskAccountHolder::Own, provider: PROVIDER },
        c::Ask::AccountChange {
            holder: c::AskAccountHolder::Own,
            change: c::AccountChange::DeclareProgram {
                provider: PROVIDER,
                join: None,
                name: Box::from(b"claude-pat".as_slice()),
            },
        },
        c::Ask::AccountChange {
            holder: c::AskAccountHolder::Own,
            change: c::AccountChange::AddKey { provider: PROVIDER, join: None, digest: [0; 32] },
        },
    ] {
        let (mut root, _, _) = root(Way::Key { provider: PROVIDER });
        crate::route::told(
            &mut root,
            &fixtures::env(&fixtures::limits()),
            jig_client::Told::Kept {
                key: jig_client::Key { asker: pages::me_handle().asker, number: 991 },
                request: jig_client::Ask::Jig(Box::new(ask.clone())),
            },
            &mut Queue::uncapped(),
        );
        assert_eq!(root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().ask, ask);
        event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
        assert!(root.pages().accounts().unwrap().request.pending.is_none());
    }
}

#[test]
fn a_choose_sign_in_shows_the_way_in_the_same_flow() {
    let way = Way::SignIn { provider: PROVIDER, plan: true };
    for current in [Address::ConnectModel { way: None }, Address::Welcome { step: Step::Model { way: None } }] {
        let (mut root, _, _) = opened(c::HeldRole::Member);
        event(&mut root, Event::Went { place: crate::Place::Address(current.clone()) });
        let out = act(&mut root, Act::SignIn { name: NAME, way });
        let expected = crate::Request::Show {
            address: connect_model::address(&current, Some(connect_model::model_way(way))),
            replace: false,
        };
        let mut shown = false;
        for request in &out {
            shown |= *request == expected;
        }
        assert!(shown);
    }
}
#[test]
fn a_member_cannot_choose_deployment_accounts() {
    let (mut root, _, _) = opened(c::HeldRole::Member);
    go(&mut root, None);
    act(&mut root, Act::Holder { holder: c::AskAccountHolder::Deployment });
    assert_eq!(root.pages().connect_model().unwrap().holder, c::AskAccountHolder::Own);
}
#[test]
fn losing_the_owner_role_clears_a_deployment_key_draft() {
    let (mut root, me, _) = root(Way::Key { provider: PROVIDER });
    act(&mut root, Act::Holder { holder: c::AskAccountHolder::Deployment });
    key(&mut root, b"private-draft");
    deliver(&mut root, me, frame(c::HeldRole::Member, fixtures::PARTY));
    let page = root.pages().connect_model().unwrap();
    assert!(!page.request.has_key());
    assert_eq!(page.holder, c::AskAccountHolder::Own);
}
#[test]
fn other_accounts_kept_work_belongs_to_management_while_the_flow_is_open() {
    let (mut root, _, _) = root(Way::Key { provider: PROVIDER });
    let ask =
        c::Ask::AccountChange { holder: c::AskAccountHolder::Own, change: c::AccountChange::Resume { source: 43 } };
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Kept {
            key: jig_client::Key { asker: pages::me_handle().asker, number: 991 },
            request: jig_client::Ask::Jig(Box::new(ask.clone())),
        },
        &mut Queue::uncapped(),
    );
    assert!(root.pages().connect_model().unwrap().request.pending.is_none());
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Accounts) });
    assert_eq!(root.pages().accounts().unwrap().request.pending.as_ref().unwrap().ask, ask);
}
#[test]
fn invalid_creation_keys_keep_the_bound_error_and_send_nothing() {
    for (value, broken) in
        [(b"".as_slice(), pages::accounts::Broken::Empty), (&[b'x'; 4097], pages::accounts::Broken::TooLong)]
    {
        let (mut root, _, _) = root(Way::Key { provider: PROVIDER });
        key(&mut root, value);
        assert!(sends(act(&mut root, Act::Submit { name: NAME })).is_empty());
        assert_eq!(root.pages().connect_model().unwrap().request.broken, Some(broken));
    }
}

#[test]
fn a_restored_sign_in_address_is_live_only_while_the_watch_names_its_source() {
    let (mut root, _, watch) = opened(c::HeldRole::Member);
    go(&mut root, Some(Way::SignIn { provider: PROVIDER, plan: true }));
    let pending = jig_client::Key { asker: pages::me_handle().asker, number: 991 };
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Kept {
            key: pending,
            request: jig_client::Ask::Jig(Box::new(c::Ask::SignIn {
                holder: c::AskAccountHolder::Own,
                provider: PROVIDER,
            })),
        },
        &mut Queue::uncapped(),
    );
    started(&mut root, pending);
    deliver(&mut root, watch, document(true));
    let page = root.pages().connect_model().unwrap();
    assert_eq!(page.request.sign_in_state(root.client()), SignInState::Signing { source: 43 });
    assert_eq!(page.request.sign_in_address(root.client()), Some(b"https://issuer/visit".as_slice()));
    let c::WatchEvent::Delivered(c::Delivery::Accounts(mut cut)) = document(false) else { unreachable!() };
    cut.cut = true;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(cut)));
    assert_eq!(root.pages().connect_model().unwrap().request.sign_in_state(root.client()), SignInState::Unknown);
    deliver(&mut root, watch, document(false));
    let page = root.pages().connect_model().unwrap();
    assert_eq!(page.request.sign_in_state(root.client()), SignInState::Ended { source: 43 });
    assert_eq!(page.request.sign_in_address(root.client()), None);
    act(&mut root, Act::SignIn { name: NAME, way: Way::SignIn { provider: PROVIDER, plan: true } });
    let key = root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().key;
    started(&mut root, key);
    let page = root.pages().connect_model().unwrap();
    assert_eq!(page.request.sign_in_state(root.client()), SignInState::Started { source: 43 });
    assert!(page.request.sign_in_address(root.client()).is_some(), "an older watch does not end a new sign-in");
}

#[test]
fn entering_welcome_discards_the_hidden_deployment_holder_and_key() {
    let (mut root, _, _) = root(Way::Key { provider: PROVIDER });
    act(&mut root, Act::Holder { holder: c::AskAccountHolder::Deployment });
    key(&mut root, b"private-draft");
    event(
        &mut root,
        Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let page = root.pages().connect_model().unwrap();
    assert_eq!(page.holder, c::AskAccountHolder::Own);
    assert!(!page.request.has_key());
}

fn claude() -> (crate::Domain, Token) {
    let (mut root, _, watch) = opened(c::HeldRole::Owner);
    event(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Welcome {
                step: Step::Model { way: Some(connect_model::model_way(Way::Program { provider: PROVIDER })) },
            }),
        },
    );
    deliver(&mut root, watch, document(false));
    (root, watch)
}
fn signed_in(root: &mut crate::Domain) -> Queue<crate::Request> {
    act(root, Act::SignedIn { name: root.pages().account_name(), way: Way::Program { provider: PROVIDER } })
}
fn answered(root: &mut crate::Domain, attempt: Token, answer: c::Answer) -> Queue<crate::Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(answer)),
                },
            },
        },
    )
}
fn done(outcome: c::DocumentOutcome) -> c::Answer {
    c::Answer::Done(c::Outcome::Document(outcome))
}
fn checked(root: &crate::Domain) -> connect_model::ProgramCheck {
    root.pages().connect_model().unwrap().check
}
fn program_document(root: &mut crate::Domain, watch: Token, settled: bool, usable: bool) -> Queue<crate::Request> {
    let c::WatchEvent::Delivered(c::Delivery::Accounts(mut doc)) = document(false) else { unreachable!() };
    doc.settled = settled;
    doc.usable = usable;
    let mut sources = skein_lib::List::with_capacity(1);
    sources
        .push(c::AccountSource {
            number: 43,
            place: 0,
            kind: c::AccountKind::Program,
            name: Some(b"claude-pat".as_slice().into()),
            state: c::AccountState::Granted,
            usable,
            expiring: false,
            attention: false,
            spent_until: None,
        })
        .unwrap();
    doc.accounts = skein_lib::List::with_capacity(1);
    doc.accounts
        .push(c::AccountLine {
            list: None,
            facts: skein_lib::List::with_capacity(0),
            number: 29,
            place: 0,
            uses: None,
            plan: true,
            provider: Some(PROVIDER),
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(c::Text::new(b"Claude".as_slice().into(), 32).unwrap()),
            holder: c::AccountHolder::Party(fixtures::PARTY),
            sources,
            cut: false,
        })
        .unwrap();
    deliver(root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)))
}
fn assert_check(ask: jig_client::Ask) {
    assert_eq!(
        ask,
        jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AskAccountHolder::Own,
            change: c::AccountChange::CheckProgram { source: 43 },
        }))
    );
}
#[test]
fn welcome_claude_declares_then_checks_under_a_fresh_request_name() {
    let (mut root, _) = claude();
    let declaration = sends(signed_in(&mut root)).pop().expect("declare first");
    assert_eq!(checked(&root), connect_model::ProgramCheck::Declaring);
    assert_eq!(
        declaration.2,
        jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AskAccountHolder::Own,
            change: c::AccountChange::DeclareProgram {
                provider: PROVIDER,
                join: None,
                name: b"claude-pat".as_slice().into(),
            },
        }))
    );
    assert!(sends(signed_in(&mut root)).is_empty(), "a second press while declaring waits");
    let check = sends(answered(
        &mut root,
        declaration.0,
        done(c::DocumentOutcome::AccountProgramDeclared { source: 43, name: b"claude-pat".as_slice().into() }),
    ))
    .pop()
    .expect("check follows declaration answer");
    assert_check(check.2);
    assert_ne!(declaration.1, check.1, "the chained check has its own idempotency name");
    assert_eq!(checked(&root), connect_model::ProgramCheck::Checking { source: 43 });
    assert!(sends(signed_in(&mut root)).is_empty(), "a second press while checking waits");
}
#[test]
fn welcome_claude_rechecks_a_held_source_only_after_answer_and_settled() {
    let (mut root, watch) = claude();
    program_document(&mut root, watch, false, false);
    let check = sends(signed_in(&mut root)).pop().expect("existing source checked");
    assert_check(check.2);
    answered(&mut root, check.0, done(c::DocumentOutcome::AccountChanged));
    assert_eq!(checked(&root), connect_model::ProgramCheck::Checking { source: 43 });
    assert!(sends(signed_in(&mut root)).is_empty(), "answered is still checking until settled");
    program_document(&mut root, watch, true, false);
    assert_eq!(checked(&root), connect_model::ProgramCheck::NotYet);
    let second = sends(signed_in(&mut root)).pop().expect("not yet can retry");
    assert_check(second.2);
    assert_ne!(check.1, second.1);
}
#[test]
fn welcome_claude_press_replaces_at_once_and_keeps_the_check_on_ready() {
    for existing in [false, true] {
        let (mut root, watch) = claude();
        if existing {
            program_document(&mut root, watch, false, false);
        }
        let out = signed_in(&mut root);
        let mut replaced = false;
        for request in &out {
            replaced |=
                *request == crate::Request::Show { address: Address::Welcome { step: Step::Ready }, replace: true };
        }
        assert!(replaced);
        let check = checked(&root);
        event(&mut root, Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Ready }) });
        assert_eq!(checked(&root), check);
        assert_eq!(pages::welcome::model_counts(&root), Some(true));
        assert!(sends(signed_in(&mut root)).is_empty());
        program_document(&mut root, watch, true, true);
        assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    }
}

#[test]
fn welcome_claude_busy_refusal_keeps_checking_and_other_refusals_remain_visible() {
    let (mut root, watch) = claude();
    program_document(&mut root, watch, false, false);
    let check = sends(signed_in(&mut root)).pop().unwrap();
    answered(
        &mut root,
        check.0,
        c::Answer::Refused {
            refusal: c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::Sources } },
            at: None,
        },
    );
    assert_eq!(checked(&root), connect_model::ProgramCheck::Checking { source: 43 });
    program_document(&mut root, watch, true, false);
    assert_eq!(checked(&root), connect_model::ProgramCheck::NotYet);
    let check = sends(signed_in(&mut root)).pop().unwrap();
    answered(&mut root, check.0, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    assert!(root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().outcome.is_some());
}
#[test]
fn welcome_claude_recovers_an_issued_name_and_resets_when_the_way_changes() {
    for (refusal, recovers) in [(c::Refusal::NameTaken, true), (c::Refusal::NotOffered, false)] {
        let (mut root, watch) = claude();
        let declaration = sends(signed_in(&mut root)).pop().unwrap();
        program_document(&mut root, watch, false, false);
        let mut checks = sends(answered(&mut root, declaration.0, c::Answer::Refused { refusal, at: None }));
        if recovers {
            let check = checks.pop().expect("an issued name resolves to its source");
            assert_check(check.2);
            assert_eq!(checked(&root), connect_model::ProgramCheck::Checking { source: 43 });
            assert!(root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().outcome.is_none());
        } else {
            assert!(checks.is_empty(), "not offered must not check an existing source");
            assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
            assert!(root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().outcome.is_some());
        }
        event(
            &mut root,
            Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
        );
        assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    }
}
#[test]
fn welcome_claude_never_declares_from_a_cut_document_or_another_holders_source() {
    for cut_account in [false, true] {
        let (mut root, watch) = claude();
        program_document(&mut root, watch, true, false);
        let mut doc = pages::accounts::document(root.client()).unwrap().clone();
        if cut_account {
            doc.accounts.get_mut(0).unwrap().cut = true;
            doc.accounts.get_mut(0).unwrap().sources.clear();
        } else {
            doc.accounts.get_mut(0).unwrap().holder = c::AccountHolder::Party(fixtures::PARTY + 1);
            doc.cut = true;
        }
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))));
        assert!(sends(signed_in(&mut root)).is_empty());
    }
}

fn environment_read(
    page: &mut connect_model::ConnectModel,
    address: &Address,
    link: jig_client::Link,
    serial: &mut u64,
) -> Queue<pages::Request> {
    let mut out = Queue::uncapped();
    page.read_environment(address, link, serial, &mut out);
    out
}
fn program_address(welcome: bool) -> Address {
    let way = Some(connect_model::model_way(Way::Program { provider: PROVIDER }));
    if welcome { Address::Welcome { step: Step::Model { way } } } else { Address::ConnectModel { way } }
}
#[test]
fn a_programs_way_reads_the_environment_once() {
    use jig_client::{Event as C, Link, Readable};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    for welcome in [false, true] {
        let mut page = connect_model::ConnectModel::default();
        let mut serial = 0;
        let address = program_address(welcome);
        assert!(environment_read(&mut page, &address, Link::Starting, &mut serial).is_empty());
        let mut out = environment_read(&mut page, &address, Link::Live, &mut serial);
        let Some(pages::Request::Client(C::Read { read: Readable::Family { scope, window, record }, .. })) = out.pop()
        else {
            panic!("one family read")
        };
        assert_eq!(scope, c::Scope::Deployment);
        assert_eq!(window, c::Window { rows: 0, after: None });
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::ProgramEnvironment);
        assert!(out.is_empty());
        assert!(environment_read(&mut page, &address, Link::Live, &mut serial).is_empty());
    }
}
#[test]
fn another_way_reads_nothing() {
    for way in [
        None,
        Some(connect_model::model_way(Way::Key { provider: PROVIDER })),
        Some(connect_model::model_way(Way::SignIn { provider: PROVIDER, plan: true })),
    ] {
        let mut page = connect_model::ConnectModel::default();
        assert!(environment_read(&mut page, &Address::ConnectModel { way }, jig_client::Link::Live, &mut 0).is_empty());
    }
}
#[test]
fn a_refusal_leaves_the_words_empty() {
    use jig_client::{Event as C, Link, Readout, Told};
    for answer in [
        Readout::Family { family: temper_client_protocol_wire::FORGE, record: None },
        Readout::Family { family: temper_client_protocol_wire::FORGE, record: Some(b"bad".as_slice().into()) },
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
        Readout::Limit,
    ] {
        let mut page = connect_model::ConnectModel::default();
        let mut out = environment_read(&mut page, &program_address(false), Link::Live, &mut 0);
        let Some(pages::Request::Client(C::Read { handle, .. })) = out.pop() else { panic!("read") };
        assert!(page.environment_told(&Told::Page { handle, page: answer }));
        assert_eq!(page.program_environment(), b"");
        assert!(!page.program_environment_answered());
    }
}

#[test]
fn a_ways_page_clears_a_stopped_check_and_ready_can_retry() {
    let (mut root, watch) = claude();
    program_document(&mut root, watch, false, false);
    let check = sends(signed_in(&mut root)).pop().unwrap();
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Ready }) });
    answered(&mut root, check.0, done(c::DocumentOutcome::AccountChanged));
    program_document(&mut root, watch, true, false);
    assert_eq!(checked(&root), connect_model::ProgramCheck::NotYet);
    assert_eq!(pages::welcome::model_counts(&root), Some(false));
    let retry = sends(signed_in(&mut root)).pop().expect("Ready retries");
    assert_check(retry.2);
    assert_ne!(check.1, retry.1);
    event(&mut root, Event::Went { place: crate::Place::Address(program_address(true)) });
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
}

#[test]
fn a_welcome_key_moves_once_only_after_it_is_added_and_usable() {
    for (refused, answer_first) in [(false, false), (false, true), (true, true)] {
        let (mut root, _, watch) = root(Way::Key { provider: PROVIDER });
        event(
            &mut root,
            Event::Went {
                place: crate::Place::Address(Address::Welcome {
                    step: Step::Model { way: Some(connect_model::model_way(Way::Key { provider: PROVIDER })) },
                }),
            },
        );
        key(&mut root, b"private-once");
        let attempt = sends(act(&mut root, Act::Submit { name: NAME })).pop().unwrap().0;
        let answer = if refused {
            c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }
        } else {
            done(c::DocumentOutcome::AccountKeyKept { source: 43 })
        };
        if answer_first {
            let out = answered(&mut root, attempt, answer);
            assert_no_moves(&out);
        }
        let c::WatchEvent::Delivered(c::Delivery::Accounts(mut doc)) = document(false) else { unreachable!() };
        doc.usable = true;
        let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc.clone())));
        let mut replaced = 0_usize;
        for request in &out {
            if *request == (crate::Request::Show { address: Address::Welcome { step: Step::Ready }, replace: true }) {
                replaced = replaced.checked_add(1).unwrap();
            }
        }
        assert_eq!(replaced, usize::from(!refused));
        let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(doc)));
        assert_no_moves(&out);
        if refused {
            assert!(root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().outcome.is_some());
        }
    }
}
#[test]
fn signed_in_elsewhere_sends_nothing_and_a_ways_page_clears_a_stop() {
    let (mut root, watch) = claude();
    program_document(&mut root, watch, false, false);
    let attempt = sends(signed_in(&mut root)).pop().unwrap().0;
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Ready }) });
    answered(&mut root, attempt, done(c::DocumentOutcome::AccountChanged));
    program_document(&mut root, watch, true, false);
    assert_eq!(checked(&root), connect_model::ProgramCheck::NotYet);
    event(
        &mut root,
        Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    assert_eq!(checked(&root), connect_model::ProgramCheck::NotYet);
    assert!(sends(signed_in(&mut root)).is_empty(), "Choose is not the tried way");
    event(&mut root, Event::Went { place: crate::Place::Address(program_address(true)) });
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    assert_check(sends(signed_in(&mut root)).pop().expect("the way can check again").2);
}

#[test]
fn a_fresh_sign_in_on_ready_leaves_the_retained_check_behind() {
    let (mut root, _) = claude();
    signed_in(&mut root);
    event(&mut root, Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Ready }) });
    let way = Way::SignIn { provider: PROVIDER, plan: true };
    let out = act(&mut root, Act::SignIn { name: NAME, way });
    let sent = sends(out).pop().expect("Ready’s fresh sign-in does not wait on the retained check");
    assert_eq!(
        sent.2,
        jig_client::Ask::Jig(Box::new(c::Ask::SignIn { holder: c::AskAccountHolder::Own, provider: PROVIDER }))
    );
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    assert_eq!(root.pages().connect_model().unwrap().tried, None);
}

fn assert_no_moves(out: &Queue<crate::Request>) {
    for request in out {
        if let crate::Request::Show { .. } = request {
            panic!("a key moves once; a refusal stays said");
        }
    }
}

#[test]
fn an_empty_environment_answer_confirms_the_default_directories() {
    use jig_client::{Event as C, Link, Readout, Told};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut page = connect_model::ConnectModel::default();
    assert!(!page.program_environment_answered());
    let mut out = environment_read(&mut page, &program_address(false), Link::Live, &mut 0);
    let Some(pages::Request::Client(C::Read { handle, .. })) = out.pop() else { panic!("read") };
    let answer =
        w::ProgramEnvironmentAnswer::new(&w::CEILINGS, w::ProgramEnvironmentAnswerParts { environment: Box::new([]) })
            .unwrap();
    assert!(page.environment_told(&Told::Page {
        handle,
        page: Readout::Family { family: wire::FORGE, record: Some(wire::program_environment_answer(&answer)) },
    }));
    assert_eq!(page.program_environment(), b"");
    assert!(page.program_environment_answered());
}

#[test]
fn connected_matches_the_person_provider_and_source_kind() {
    let (mut root, _, watch) = opened(c::HeldRole::Owner);
    let way = Way::Program { provider: PROVIDER };
    assert!(!connect_model::connected(&root, way));
    program_document(&mut root, watch, true, false);
    assert!(connect_model::connected(&root, way), "a held source counts even when not usable");
    assert!(!connect_model::connected(&root, Way::SignIn { provider: PROVIDER, plan: true }));
    assert!(!connect_model::connected(&root, Way::Program { provider: PROVIDER + 1 }));
    let mut doc = pages::accounts::document(root.client()).unwrap().clone();
    doc.accounts.get_mut(0).unwrap().holder = c::AccountHolder::Party(fixtures::PARTY + 1);
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))));
    assert!(!connect_model::connected(&root, way));
}
#[test]
fn the_default_is_the_first_unconnected_way_or_none() {
    let (mut root, _, watch) = opened(c::HeldRole::Owner);
    let first = Way::Program { provider: PROVIDER };
    let second = Way::SignIn { provider: PROVIDER, plan: true };
    let mut ways = skein_lib::List::with_capacity(2);
    assert_eq!(connect_model::default_way(&ways, &root), None);
    ways.push(first).unwrap();
    ways.push(second).unwrap();
    assert_eq!(connect_model::default_way(&ways, &root), Some(first));
    program_document(&mut root, watch, true, false);
    assert_eq!(connect_model::default_way(&ways, &root), Some(second));
    let mut doc = pages::accounts::document(root.client()).unwrap().clone();
    let mut source = doc.accounts.get(0).unwrap().sources.get(0).unwrap().clone();
    source.kind = c::AccountKind::SignIn;
    source.number = 44;
    source.name = None;
    let mut sources = skein_lib::List::with_capacity(2);
    sources.push(doc.accounts.get(0).unwrap().sources.get(0).unwrap().clone()).unwrap();
    sources.push(source).unwrap();
    doc.accounts.get_mut(0).unwrap().sources = sources;
    deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))));
    assert_eq!(connect_model::default_way(&ways, &root), None);
}
#[test]
fn demo_is_not_a_way_before_the_playground_is_read() {
    let mut providers = skein_lib::List::with_capacity(1);
    providers
        .push(c::ProviderOffer {
            provider: PROVIDER,
            name: c::DocumentText::complete(c::Text::new(b"demo".as_slice().into(), 32).unwrap()),
            key_bytes: 4096,
            sign_in: true,
            signed_in_plan: false,
            key: true,
            programs: skein_lib::List::with_capacity(0),
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .unwrap();
    assert!(pages::accounts::ways(&providers, None, c::AskAccountHolder::Own).is_empty());
}
