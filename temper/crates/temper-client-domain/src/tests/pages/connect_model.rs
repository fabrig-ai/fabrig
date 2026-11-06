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
fn welcome_claude_ready_replaces_the_pressed_step_and_clears_the_check() {
    let (mut root, watch) = claude();
    program_document(&mut root, watch, true, false);
    let check = sends(signed_in(&mut root)).pop().expect("source checked");
    answered(&mut root, check.0, done(c::DocumentOutcome::AccountChanged));
    let out = program_document(&mut root, watch, true, true);
    let mut shown = false;
    for request in &out {
        shown |= *request == crate::Request::Show { address: Address::Welcome { step: Step::Ready }, replace: true };
    }
    assert!(shown);
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    for request in &program_document(&mut root, watch, true, true) {
        if let crate::Request::Show { .. } = request {
            panic!("a completed check never moves twice");
        }
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
    let (mut root, watch) = claude();
    let declaration = sends(signed_in(&mut root)).pop().unwrap();
    program_document(&mut root, watch, false, false);
    let check =
        sends(answered(&mut root, declaration.0, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }))
            .pop()
            .expect("an issued name resolves to its source");
    assert_check(check.2);
    event(
        &mut root,
        Event::Went { place: crate::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
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

#[test]
fn sign_in_again_returns_only_a_stopped_welcome_check_to_the_panel() {
    let (mut root, watch) = claude();
    program_document(&mut root, watch, true, false);
    let check = sends(signed_in(&mut root)).pop().unwrap();
    act(&mut root, Act::SignInAgain);
    assert_eq!(checked(&root), connect_model::ProgramCheck::Checking { source: 43 });
    answered(&mut root, check.0, done(c::DocumentOutcome::AccountChanged));
    program_document(&mut root, watch, true, false);
    assert_eq!(checked(&root), connect_model::ProgramCheck::NotYet);
    assert!(sends(act(&mut root, Act::SignInAgain)).is_empty());
    assert_eq!(checked(&root), connect_model::ProgramCheck::Idle);
    assert!(root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().outcome.is_some());
    assert_check(sends(signed_in(&mut root)).pop().expect("panel checks again").2);
}
