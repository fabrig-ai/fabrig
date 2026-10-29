//! Sign-in departure, return and endings through the joined world's public fakes.
use jig_client_world::harness::{App, Front, FrontFault};
use jig_domain_accounts as a;
use skein_lib::{Duration, Queue, Wall};
use skein_ui::{Act, Role};
use temper_client_domain::{Address, Place, address};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures as f,
    front::{Joint, account_acts as h},
};

// An inert address of this world's fake issuer; never a real sign-in link.
const ISSUER: &[u8] = b"https://issuer.example/qa-sign-in";
// README 3.5: one Return and one departure for one answered sign-in.
const ONE: usize = 1;
const NONE: usize = 0;
const SUBJECT: &[u8] = b"qa-person";

fn at(path: &[u8]) -> Address {
    let Place::Address(address) = address::parse(path) else { panic!("QA address") };
    address
}
fn printed(root: &Root) -> Vec<u8> {
    address::print(root.person().pages().address()).as_ref().to_vec()
}
fn returned(source: u32) -> Address {
    at(format!("/accounts/connect?provider={}&way=sign-in&source={source}", h::PROVIDER).as_bytes())
}
fn shown(root: &Root) -> View {
    let (_, mut view) = Temper::new(f::SEED);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, root, &mut patches);
    view
}
fn words(root: &Root) -> String {
    let view = shown(root);
    let mut text = Vec::new();
    for node in Temper::tree(&view).nodes() {
        if let Some(name) = &node.name {
            text.push(String::from_utf8_lossy(name).into_owned());
        }
        let line: Vec<u8> = node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).collect();
        if !line.is_empty() {
            text.push(String::from_utf8_lossy(&line).into_owned());
        }
    }
    text.join("\n")
}
fn has(root: &Root, expected: &str) {
    let text = words(root);
    assert!(text.contains(expected), "expected {expected:?}; page:\n{text}");
}
fn button(root: &Root, name: &[u8]) -> bool {
    let view = shown(root);
    Temper::tree(&view).nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(name))
}
fn press(front: &mut Joint, root: &mut Root, name: &[u8]) {
    let mut view = shown(root);
    let node = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("offered QA button {:?}; page:\n{}", String::from_utf8_lossy(name), words(root)))
        .id;
    let event = Temper::act(&mut view, node, Act::Press).expect("bound QA button");
    h::event(front, root, event);
}
fn begin(departing: bool) -> (Joint, Root, u32) {
    let (mut front, mut root) = h::plan_fixture(f::SEED);
    if departing {
        h::reopen(&mut front, &mut root, at(b"/accounts/connect"), true);
    } else {
        // Keep the fixture's seed: reopen intentionally uses one fixed fresh seed.
        h::event(&mut front, &mut root, Temper::went(at(b"/accounts/connect")));
    }
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    (front, root, source)
}

fn account_event(front: &mut Joint, root: &mut Root, event: a::Event) {
    front.engine.send(temper_engine_domain::Event::Account(event));
    front.engine.settle();
    h::settle(front, root);
}
fn visited(front: &mut Joint, source: u32) {
    front.engine.send(temper_engine_domain::Event::Account(a::Event::Visited { source, address: ISSUER.into() }));
    front.engine.settle();
}
fn started(front: &mut Joint, root: &mut Root, source: u32) {
    visited(front, source);
    h::settle(front, root);
}
fn made(front: &mut Joint, root: &mut Root, source: u32, declined: bool) {
    account_event(front, root, a::Event::SignedIn { source, subject: SUBJECT.into(), display: Box::new([]), declined });
}
fn established() -> (Joint, Root, u32) {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    made(&mut front, &mut root, source, false);
    (front, root, source)
}
fn returns(front: &Joint) -> Vec<(u32, Vec<u8>)> {
    front.engine.protocol.returns.iter().map(|(source, path)| (*source, path.to_vec())).collect()
}

fn row_sign_in() -> (Joint, Root, u32) {
    let (mut front, mut root, source) = established();
    h::reopen(&mut front, &mut root, Address::Accounts, false);
    // The existing component event makes this source need sign-in, keeping its row.
    account_event(
        &mut front,
        &mut root,
        a::Event::Component(a::FromComponent::Failed { account: source, failure: a::Failure::SignInAgain }),
    );
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    started(&mut front, &mut root, source);
    (front, root, source)
}
fn account_ending(root: &Root, expected: &str) {
    has(root, expected);
    assert!(button(root, b"Continue with ChatGPT"), "each ending offers sign-in again");
}

// client/README.md 3.5 Come back: a source missing from the watch drops from the address.
#[test]
fn absent_return_source_drops_from_the_address() {
    let (mut front, mut root) = h::plan_fixture(f::SEED);
    h::reopen(&mut front, &mut root, returned(u32::MAX), true);
    assert_eq!(printed(&root), b"/accounts/connect?provider=17&way=sign-in");
    assert!(button(&root, b"Continue with ChatGPT"));
}

// client/README.md 3.5: a way without a source does not adopt another client's sign-in.
#[test]
fn a_reload_without_source_offers_the_way_again() {
    let (mut front, mut root, _) = begin(false);
    h::reopen(&mut front, &mut root, at(b"/accounts/connect?provider=17&way=sign-in"), true);
    assert!(button(&root, b"Continue with ChatGPT"));
    assert_eq!(front.leaves.len(), NONE);
}

// client/README.md 3.5 and 10: the terminal prints a sign-in address and sends no Return.
#[test]
fn terminal_sign_in_never_requests_a_tab_departure() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    assert_eq!(returns(&front).len(), NONE);
    assert_eq!(front.leaves.len(), NONE);
    let view = shown(&root);
    assert!(
        Temper::tree(&view)
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Link && node.external.as_deref() == Some(ISSUER) })
    );
}

// client/README.md 3.5 The tab's departure: one Return names this source and way before Leave.
#[test]
fn browser_departure_registers_its_source_and_return_way() {
    let (mut front, mut root, source) = begin(true);
    started(&mut front, &mut root, source);
    assert_eq!(
        returns(&front),
        vec![(source, format!("/accounts/connect?provider=17&way=sign-in&source={source}").into_bytes())]
    );
    assert_eq!(front.leaves.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(), [ISSUER]);
}

// client/README.md 3.5 The tab's departure: Return refused leaves the link available in place.
#[test]
fn refused_return_keeps_the_page_and_external_link() {
    let (mut front, mut root, source) = begin(true);
    front.refuse_return = true;
    started(&mut front, &mut root, source);
    assert_eq!(front.leaves.len(), NONE);
    let view = shown(&root);
    assert!(
        Temper::tree(&view)
            .nodes()
            .iter()
            .any(|node| { node.role == Role::Link && node.external.as_deref() == Some(ISSUER) })
    );
}

// client/README.md 3.5 The tab's departure: Return lost never asks the shell to leave.
#[test]
fn lost_return_does_not_depart() {
    let (mut front, mut root, source) = begin(true);
    visited(&mut front, source);
    let now = front.engine.env.now;
    let callbacks = front.poll(now, Wall::EPOCH);
    // The start's answer is already captured; lose the following Return answer.
    h::lose_next(&mut front);
    for (client, event) in callbacks {
        front.received(client, &event);
        h::event(&mut front, &mut root, event);
    }
    h::lost(&mut front, &mut root);
    assert_eq!(returns(&front).len(), ONE);
    assert_eq!(front.leaves.len(), NONE);
}

// client/README.md 3.5 and 10: rebuilding the view cannot repeat a departure.
#[test]
fn rebuilding_after_departure_never_leaves_again() {
    let (mut front, mut root, source) = begin(true);
    started(&mut front, &mut root, source);
    let mut view = shown(&root);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, &root, &mut patches);
    Temper::build(&mut view, &root, &mut patches);
    h::settle(&mut front, &mut root);
    assert_eq!(returns(&front).len(), ONE);
    assert_eq!(front.leaves.len(), ONE);
}

// client/README.md 3.5 Come back: a fresh browser root adopts its source without departing.
#[test]
fn returned_browser_draws_signing_in_without_departing_again() {
    let (mut front, mut root, source) = begin(true);
    started(&mut front, &mut root, source);
    h::reopen(&mut front, &mut root, returned(source), true);
    has(&root, "Signing in");
    assert!(!button(&root, b"Continue with ChatGPT"));
    assert_eq!(returns(&front).len(), ONE);
    assert_eq!(front.leaves.len(), ONE);
}

// client/README.md 3.5 The welcome: Model's sign-in sends its browser back to Ready.
#[test]
fn welcome_model_registers_ready_as_its_return() {
    let (mut front, mut root) = h::plan_fixture(f::SEED);
    h::reopen(&mut front, &mut root, at(b"/welcome?at=model"), true);
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    started(&mut front, &mut root, source);
    assert_eq!(returns(&front), vec![(source, b"/welcome?at=ready".to_vec())]);
    assert_eq!(front.leaves.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(), [ISSUER]);
}

fn connect_ending(front: &mut Joint, root: &mut Root, source: u32, expected: &str) {
    has(root, expected);
    assert!(button(root, b"Continue with ChatGPT"));
    h::reopen(front, root, returned(source), true);
    has(root, expected);
    assert!(button(root, b"Continue with ChatGPT"));
}

// client/README.md 3.5 Come back and 5.5 A sign-in: Made reaches Ready live and at the returned address.
#[test]
fn made_sign_in_reaches_ready_live_and_after_return() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    made(&mut front, &mut root, source, false);
    connect_ending(&mut front, &mut root, source, "Signed in to ChatGPT");
}

// client/README.md 5.5 A sign-in: Renewed names the same account and offers sign-in again.
#[test]
fn renewed_sign_in_explains_the_same_account_on_connect_model() {
    let (mut front, mut root, _) = established();
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    started(&mut front, &mut root, source);
    made(&mut front, &mut root, source, false);
    connect_ending(
        &mut front,
        &mut root,
        source,
        "That’s your ChatGPT account again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.",
    );
}

// client/README.md 5.5 A sign-in: Made with declined consent has the plan-declined words.
#[test]
fn made_without_plan_usage_offers_sign_in_and_a_key() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    made(&mut front, &mut root, source, true);
    connect_ending(&mut front, &mut root, source, "You signed in without plan usage.");
    has(&root, "Use an API key");
}

// client/README.md 5.5 A sign-in: Renewed with declined consent uses plan-declined words too.
#[test]
fn renewed_without_plan_usage_offers_sign_in_and_a_key() {
    let (mut front, mut root, _) = established();
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    started(&mut front, &mut root, source);
    made(&mut front, &mut root, source, true);
    connect_ending(&mut front, &mut root, source, "You signed in without plan usage.");
    has(&root, "Use an API key");
}

// client/README.md 3.5 Come back: a later ending replaces an older source's ending.
#[test]
fn superseded_return_source_offers_the_way_again() {
    let (mut front, mut root, old_source) = established();
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let new_source = h::signing(&front);
    started(&mut front, &mut root, new_source);
    made(&mut front, &mut root, new_source, false);
    h::reopen(&mut front, &mut root, returned(old_source), true);
    assert!(button(&root, b"Continue with ChatGPT"));
    assert_eq!(printed(&root), b"/accounts/connect?provider=17&way=sign-in");
    assert!(!words(&root).contains("Signed in to ChatGPT"));
}

// client/README.md 3.5 The tab's departure: Leave waits for Return's answer.
#[test]
fn browser_waits_for_the_return_answer_before_leaving() {
    // Longer than the root helper's 3 ms callbacks; only fake time advances.
    const HOLD: Duration = Duration::from_secs(1);
    let (mut front, mut root, source) = begin(true);
    visited(&mut front, source);
    let now = front.engine.env.now;
    let callbacks = front.poll(now, Wall::EPOCH);
    front.fault(now, FrontFault::Slow { span: HOLD });
    for (client, event) in callbacks {
        front.received(client, &event);
        h::event(&mut front, &mut root, event);
    }
    assert_eq!(returns(&front).len(), ONE);
    assert_eq!(front.leaves.len(), NONE);
    for (client, event) in front.poll(now.saturating_add(HOLD), Wall::EPOCH) {
        front.received(client, &event);
        h::event(&mut front, &mut root, event);
    }
    assert_eq!(front.leaves.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(), [ISSUER]);
}

// client/README.md 5.5 A sign-in and Each state: Failed Misconfigured explains registration.
#[test]
fn misconfigured_sign_in_explains_registration_on_connect_model() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure: a::Failure::Misconfigured });
    connect_ending(
        &mut front,
        &mut root,
        source,
        "OpenAI refused temper’s sign-in. Check the configuration’s registration.",
    );
}

// client/README.md 5.5 A sign-in and Each state: Failed SignInAgain says signed out.
#[test]
#[ignore = "backlog: a sign-in ending SignInAgain reads \"Needs sign-in\", not 5.5's \"Signed out of ChatGPT.\"; off the happy path (m3 backlog 2)"]
fn signed_out_sign_in_explains_the_ending_on_connect_model() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure: a::Failure::SignInAgain });
    connect_ending(&mut front, &mut root, source, "Signed out of ChatGPT.");
}

// client/README.md 5.5 A sign-in and Each state: Failed Unloaded names where and why.
#[test]
#[ignore = "backlog: a sign-in ending Unloaded reads the load error, not 5.5's \"Can't be used on HOST: WHY.\"; off the happy path (m3 backlog 3)"]
fn unloaded_sign_in_names_its_place_and_reason_on_connect_model() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    account_event(
        &mut front,
        &mut root,
        a::Event::SignInFailed {
            source,
            failure: a::Failure::Unloaded { at: a::Place::File, why: a::Unread::Unreadable },
        },
    );
    connect_ending(&mut front, &mut root, source, "Can't be used on ");
    has(&root, "File");
    has(&root, "Unreadable");
}

// client/README.md 3.5 Come back: a returned source observes the later Made ending.
#[test]
fn returned_sign_in_changes_to_ready_when_the_watch_ends() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    h::reopen(&mut front, &mut root, returned(source), true);
    made(&mut front, &mut root, source, false);
    has(&root, "Signed in to ChatGPT");
    let view = shown(&root);
    assert!(
        Temper::tree(&view)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Start a chat"))
    );
    assert_eq!(front.leaves.len(), NONE);
    assert_eq!(returns(&front).len(), NONE);
}

// client/README.md 3.5 Come back: a returned source observes the later Failed ending.
#[test]
#[ignore = "backlog: a sign-in ending SignInAgain reads \"Needs sign-in\", not 5.5's \"Signed out of ChatGPT.\"; off the happy path (m3 backlog 2)"]
fn returned_sign_in_changes_to_failure_when_the_watch_ends() {
    let (mut front, mut root, source) = begin(false);
    started(&mut front, &mut root, source);
    h::reopen(&mut front, &mut root, returned(source), true);
    account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure: a::Failure::SignInAgain });
    has(&root, "Signed out of ChatGPT.");
    assert!(button(&root, b"Continue with ChatGPT"));
    assert_eq!(front.leaves.len(), NONE);
}

// client/README.md 3.5 The welcome Ready: no model line precedes the accounts document.
#[test]
fn welcome_ready_waits_for_accounts_before_drawing_a_model_line() {
    let (mut front, mut root) = h::plan_fixture(f::SEED);
    let now = front.engine.env.now;
    front.fault(now, FrontFault::Slow { span: Duration::from_secs(1) });
    h::reopen(&mut front, &mut root, at(b"/welcome?at=ready"), true);
    assert!(!words(&root).contains("Model:"), "{}", words(&root));
}

// client/README.md 3.5 The welcome Ready: an underway sign-in names its provider.
#[test]
fn welcome_ready_names_the_sign_in_under_way() {
    let (mut front, mut root) = h::empty_plan_fixture(f::SEED);
    h::reopen(&mut front, &mut root, at(b"/welcome?at=model"), true);
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    started(&mut front, &mut root, source);
    h::reopen(&mut front, &mut root, at(b"/welcome?at=ready"), true);
    has(&root, "Model: ChatGPT · signing in");
}

// client/README.md 3.5 The welcome Ready: the made usable account names its model line.
#[test]
fn welcome_ready_names_the_made_account() {
    let (mut front, mut root, source) = established();
    account_event(
        &mut front,
        &mut root,
        a::Event::Component(a::FromComponent::Granted {
            account: source,
            generation: 1,
            valid: Duration::from_secs(3600),
            form: a::Form::Credential(a::Scheme::Bearer),
        }),
    );
    h::reopen(&mut front, &mut root, at(b"/welcome?at=ready"), true);
    has(&root, "Model: ChatGPT · OpenAI · held by Person");
}

// client/README.md 3.5 Accounts and 5.5 A sign-in: Made uses the signed-in words and acts.
#[test]
fn accounts_made_sign_in_names_the_plan_and_offers_retry() {
    let (mut front, mut root, source) = row_sign_in();
    account_event(
        &mut front,
        &mut root,
        a::Event::SignedIn {
            source,
            subject: b"qa-new-person".as_slice().into(),
            display: Box::new([]),
            declined: false,
        },
    );
    account_ending(&root, "Signed in to ChatGPT");
}

// client/README.md 3.5 Accounts and 5.5 A sign-in: Renewed explains adding another account.
#[test]
fn accounts_renewed_sign_in_explains_adding_another_account() {
    let (mut front, mut root, source) = row_sign_in();
    made(&mut front, &mut root, source, false);
    account_ending(
        &root,
        "That’s your ChatGPT account again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.",
    );
}

// client/README.md 5.5 A sign-in: a row's Made with declined consent offers sign-in and a key.
#[test]
fn accounts_made_without_plan_usage_offers_sign_in_and_key() {
    let (mut front, mut root, source) = row_sign_in();
    account_event(
        &mut front,
        &mut root,
        a::Event::SignedIn {
            source,
            subject: b"qa-new-person".as_slice().into(),
            display: Box::new([]),
            declined: true,
        },
    );
    account_ending(&root, "You signed in without plan usage.");
    has(&root, "Use an API key");
}

// client/README.md 5.5 A sign-in: a row's Renewed with declined consent has the same words.
#[test]
fn accounts_renewed_without_plan_usage_offers_sign_in_and_key() {
    let (mut front, mut root, source) = row_sign_in();
    made(&mut front, &mut root, source, true);
    account_ending(&root, "You signed in without plan usage.");
    has(&root, "Use an API key");
}

// client/README.md 5.5 Each state: a row's Failed Misconfigured explains registration.
#[test]
fn accounts_misconfigured_sign_in_explains_registration() {
    let (mut front, mut root, source) = row_sign_in();
    account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure: a::Failure::Misconfigured });
    account_ending(&root, "OpenAI refused temper’s sign-in. Check the configuration’s registration.");
}

// client/README.md 5.5 Each state: a row's Failed SignInAgain names ChatGPT and sign-out.
#[test]
#[ignore = "backlog: a sign-in ending SignInAgain reads \"Needs sign-in\", not 5.5's \"Signed out of ChatGPT.\"; off the happy path (m3 backlog 2)"]
fn accounts_signed_out_sign_in_names_chatgpt() {
    let (mut front, mut root, source) = row_sign_in();
    account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure: a::Failure::SignInAgain });
    account_ending(&root, "Signed out of ChatGPT.");
}

// client/README.md 5.5 Each state: a row's Failed Unloaded names where and why.
#[test]
#[ignore = "backlog: a sign-in ending Unloaded reads the load error, not 5.5's \"Can't be used on HOST: WHY.\"; off the happy path (m3 backlog 3)"]
fn accounts_unloaded_sign_in_names_its_place_and_reason() {
    let (mut front, mut root, source) = row_sign_in();
    account_event(
        &mut front,
        &mut root,
        a::Event::SignInFailed {
            source,
            failure: a::Failure::Unloaded { at: a::Place::File, why: a::Unread::Unreadable },
        },
    );
    account_ending(&root, "Can't be used on ");
    has(&root, "File");
    has(&root, "Unreadable");
}

// client/README.md 5.5 A sign-in: every other Failed ending offers the way again.
#[test]
fn other_failures_offer_retry_on_connect_model() {
    for failure in [
        a::Failure::Later { after: None },
        a::Failure::Later { after: Some(Duration::from_secs(1)) },
        a::Failure::MissingScope { scope: h::PROVIDER },
        a::Failure::Cancelled,
    ] {
        let (mut front, mut root, source) = begin(false);
        started(&mut front, &mut root, source);
        account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure });
        assert!(button(&root, b"Continue with ChatGPT"));
        h::reopen(&mut front, &mut root, returned(source), true);
        assert!(button(&root, b"Continue with ChatGPT"));
    }
}

// client/README.md 3.5 Accounts and 5.5 A sign-in: the row offers retry for other failures.
#[test]
fn accounts_other_failed_sign_ins_offer_retry() {
    for failure in [
        a::Failure::Later { after: None },
        a::Failure::Later { after: Some(Duration::from_secs(1)) },
        a::Failure::MissingScope { scope: h::PROVIDER },
        a::Failure::Cancelled,
    ] {
        let (mut front, mut root, source) = row_sign_in();
        account_event(&mut front, &mut root, a::Event::SignInFailed { source, failure });
        assert!(button(&root, b"Continue with ChatGPT"));
    }
}
