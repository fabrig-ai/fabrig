//! Paste QA through the existing joined client world and its public acts.
use jig_client_world::harness::App;
use jig_domain_accounts as a;
use skein_lib::{Queue, Secret};
use skein_ui::{Act, Role, State};
use temper_client_domain::{Address, Place, address, pages};
use temper_client_world::{
    app::{Root, Temper, View},
    fixtures as f,
    front::{Joint, account_acts as h},
};

const FIELD: &[u8] = b"Address of the page that couldn't be reached";
const FINISH: &[u8] = b"Finish connecting";
const ELSEWHERE: &[u8] = "This browser isn't on temper's computer ›".as_bytes();
// Inert fake issuer only. Callback fixtures are generated locally and never formatted.
const ISSUER: &[u8] = b"https://issuer.example/qa-paste";

fn at(path: &[u8]) -> Address {
    let Place::Address(address) = address::parse(path) else { panic!("page address") };
    address
}

fn shown(root: &Root) -> View {
    let (_, mut view) = Temper::new(f::SEED);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, root, &mut patches);
    view
}

fn offered(root: &Root, role: Role, name: &[u8]) -> bool {
    Temper::tree(&shown(root)).nodes().iter().any(|node| node.role == role && node.name.as_deref() == Some(name))
}

fn has(root: &Root, expected: &[u8]) -> bool {
    Temper::tree(&shown(root)).nodes().iter().any(|node| {
        node.name.as_deref().is_some_and(|name| name.windows(expected.len()).any(|part| part == expected))
            || node.text.as_slice().iter().any(|run| run.text.windows(expected.len()).any(|part| part == expected))
    })
}

fn press(front: &mut Joint, root: &mut Root, name: &[u8]) {
    let mut view = shown(root);
    let node = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.name.as_deref() == Some(name) && matches!(node.role, Role::Button | Role::Link))
        .expect("offered control")
        .id;
    let event = Temper::act(&mut view, node, Act::Press).expect("bound control");
    h::event(front, root, event);
}

fn account(front: &mut Joint, root: &mut Root, event: a::Event) {
    front.engine.send(temper_engine_domain::Event::Account(event));
    front.engine.settle();
    h::settle(front, root);
}

fn visit(front: &mut Joint, root: &mut Root, source: u32) {
    account(front, root, a::Event::Visited { source, address: ISSUER.into() });
}

fn begin(path: &[u8], departing: bool, elsewhere: bool) -> (Joint, Root, u32) {
    let (mut front, mut root) = h::plan_fixture(f::SEED);
    if departing {
        h::reopen(&mut front, &mut root, at(path), true);
    } else {
        h::event(&mut front, &mut root, Temper::went(at(path)));
    }
    press(&mut front, &mut root, if elsewhere { ELSEWHERE } else { b"Continue with ChatGPT" });
    let source = h::signing(&front);
    visit(&mut front, &mut root, source);
    (front, root, source)
}

fn paste(front: &mut Joint, root: &mut Root, bytes: &[u8]) {
    h::event(front, root, temper_client_domain::Event::SignInAddress { value: Secret::new(bytes.into()) }.into());
}

fn finish(front: &mut Joint, root: &mut Root, name: u8) {
    h::event(
        front,
        root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(pages::Act::ConnectModel(pages::connect_model::Act::Finish {
                name: [name; 16],
            })),
        }
        .into(),
    );
}

fn disabled(root: &Root) -> bool {
    Temper::tree(&shown(root))
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(FINISH))
        .unwrap_or_else(|| {
            panic!(
                "finish control; field={}, sign-in={}, ended={}, ready={}",
                offered(root, Role::TextField, FIELD),
                offered(root, Role::Button, b"Continue with ChatGPT"),
                has(root, b"This sign-in has ended."),
                has(root, b"Signed in to ChatGPT"),
            )
        })
        .states
        .as_slice()
        .contains(&State::Disabled)
}

fn callback() -> Vec<u8> {
    let mut bytes = b"http://127.0.0.1:1455/auth/callback?code=".to_vec();
    bytes.extend_from_slice(b"qa-code&state=qa-state&client_id=qa-client");
    bytes
}

// client/README.md 3.5 From another computer: no Return or departure,
// for both the main flow and the welcome's shared Model card.
#[test]
fn elsewhere_keeps_connect_and_welcome_in_the_page() {
    for path in [b"/accounts/connect".as_slice(), b"/welcome?at=model"] {
        let (front, root, _) = begin(path, true, true);
        assert!(front.engine.protocol.returns.is_empty(), "no native tab return for elsewhere");
        assert!(front.leaves.is_empty(), "the sign-in remains in this tab");
        assert!(has(&root, "Open ChatGPT's sign-in ›".as_bytes()), "elsewhere link has the promised words");
        assert!(
            Temper::tree(&shown(&root))
                .nodes()
                .iter()
                .any(|node| { node.role == Role::Link && node.external.as_deref() == Some(ISSUER) })
        );
        assert!(offered(&root, Role::TextField, FIELD));
        assert!(disabled(&root), "an empty field cannot submit");
    }
}

// client/README.md 3.5 Signing in and 6.5: the ordinary browser still departs;
// the terminal remains in place and offers the same paste.
#[test]
fn native_browser_and_terminal_keep_their_departure_policies() {
    for departing in [false, true] {
        let (front, root, _) = begin(b"/accounts/connect", departing, false);
        assert_eq!(front.engine.protocol.returns.len(), usize::from(departing));
        assert_eq!(front.leaves.len(), usize::from(departing));
        assert!(offered(&root, Role::TextField, FIELD));
    }
}

// client/README.md 3.5 The paste: surrounding whitespace alone is empty.
#[test]
fn whitespace_only_paste_stays_disabled_and_sends_nothing() {
    let (mut front, mut root, _) = begin(b"/accounts/connect", false, false);
    let before = front.engine.protocol.accounts.len();
    paste(&mut front, &mut root, b" \t\r\n ");
    assert!(disabled(&root));
    finish(&mut front, &mut root, 91);
    assert_eq!(front.engine.protocol.accounts.len(), before);
    h::assert_no_carried(&front);
}

// client/README.md 3.5, 5.5: an oversized paste is refused locally,
// but a later correctly sized paste can be sent.
#[test]
fn oversized_paste_is_local_and_editing_it_recovers() {
    let (mut front, mut root, _) = begin(b"/accounts/connect", false, false);
    let before = front.engine.protocol.accounts.len();
    let oversized = vec![b'x'; 4097];
    paste(&mut front, &mut root, &oversized);
    assert!(disabled(&root));
    assert!(has(&root, b"That's longer than a sign-in's address."));
    finish(&mut front, &mut root, 92);
    assert_eq!(front.engine.protocol.accounts.len(), before);
    h::assert_no_carried(&front);
    paste(&mut front, &mut root, &callback());
    assert!(!disabled(&root));
}

// client/README.md 3.5 and 3.6: the secret is absent from the semantic
// tree and address, including before sending. Assertions never format it.
#[test]
fn typed_callback_never_appears_in_the_view_or_page_address() {
    let (mut front, mut root, _) = begin(b"/accounts/connect", false, false);
    let value = callback();
    paste(&mut front, &mut root, &value);
    assert!(!has(&root, &value), "the semantic page must not echo the pasted secret");
    let view = shown(&root);
    for node in Temper::tree(&view).nodes() {
        for text in
            [node.name.as_deref(), node.description.as_deref(), node.value.as_ref().map(|value| value.text.as_ref())]
                .into_iter()
                .flatten()
        {
            assert!(!text.windows(value.len()).any(|part| part == value), "no pasted secret in node metadata");
        }
    }
    let field =
        Temper::tree(&view).nodes().iter().find(|node| node.name.as_deref() == Some(FIELD)).expect("paste field");
    assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
    let printed = address::print(root.person().pages().address());
    assert!(!printed.windows(value.len()).any(|part| part == value), "page address contains no pasted secret");
    assert!(!disabled(&root));
}

// client/README.md 3.5 The paste: changing page or way drops the value.
#[test]
fn leaving_the_page_or_way_drops_the_paste() {
    for destination in [b"/accounts".as_slice(), b"/accounts/connect?provider=17&way=key"] {
        let (mut front, mut root, source) = begin(b"/accounts/connect", false, false);
        paste(&mut front, &mut root, &callback());
        h::event(&mut front, &mut root, Temper::went(at(destination)));
        let returned = format!("/accounts/connect?provider=17&way=sign-in&source={source}");
        h::event(&mut front, &mut root, Temper::went(at(returned.as_bytes())));
        assert!(disabled(&root), "a returned page asks for a fresh paste");
    }
}

// client/README.md 3.5 and 3.6: repeated submission while out joins
// the pending finish, even when supplied under another name.
#[test]
fn duplicate_finish_sends_one_request_and_clears_the_field() {
    let (mut front, mut root, _) = begin(b"/accounts/connect", false, false);
    paste(&mut front, &mut root, &callback());
    let before = front.engine.protocol.accounts.len();
    finish(&mut front, &mut root, 93);
    finish(&mut front, &mut root, 94);
    assert_eq!(front.engine.protocol.accounts.len(), before + 1, "one request reaches the accounts host");
    assert!(has(&root, "Finishing…".as_bytes()));
    assert!(!has(&root, &callback()), "sending clears the secret");
}

// client/README.md 3.5 and 5.5: the accounts watch owns the final result,
// including when a finish was already submitted.
#[test]
fn watch_success_replaces_a_pending_paste_with_ready() {
    let (mut front, mut root, source) = begin(b"/accounts/connect", false, false);
    paste(&mut front, &mut root, &callback());
    finish(&mut front, &mut root, 95);
    account(
        &mut front,
        &mut root,
        a::Event::SignedIn {
            source,
            subject: b"qa-paste-person".as_slice().into(),
            display: Box::new([]),
            declined: false,
        },
    );
    assert!(has(&root, b"Signed in to ChatGPT"));
    assert!(!offered(&root, Role::TextField, FIELD));
    assert!(offered(&root, Role::Link, b"Start a chat"));
}

// client/README.md 3.5: reloading retains no pasted draft.
#[test]
fn reload_of_the_waiting_source_drops_the_pasted_value() {
    let (mut front, mut root, source) = begin(b"/accounts/connect", false, false);
    paste(&mut front, &mut root, &callback());
    let returned = format!("/accounts/connect?provider=17&way=sign-in&source={source}");
    h::reopen(&mut front, &mut root, at(returned.as_bytes()), true);
    assert!(disabled(&root));
    assert!(front.leaves.is_empty());
}

// client/README.md 3.5 From another computer: the ordinary paste is folded;
// choosing elsewhere presents it already open.
#[test]
fn ordinary_paste_is_folded_and_elsewhere_paste_is_open() {
    const DISCLOSURE: &[u8] = b"Signed in, but the page couldn't be reached?";
    for elsewhere in [false, true] {
        let (_, root, _) = begin(b"/accounts/connect", false, elsewhere);
        let view = shown(&root);
        let disclosure = Temper::tree(&view)
            .nodes()
            .iter()
            .find(|node| node.role == Role::Disclosure && node.name.as_deref() == Some(DISCLOSURE));
        if elsewhere {
            assert!(disclosure.is_none_or(|node| node.states.as_slice().contains(&State::Expanded)));
        } else {
            assert!(!disclosure.expect("ordinary paste disclosure").states.as_slice().contains(&State::Expanded));
        }
    }
}

// client/README.md 3.5 Accounts: the existing account's sign-in offers
// the same paste and finishes from the accounts watch.
#[test]
fn accounts_row_sign_in_offers_the_paste_and_observes_success() {
    let (mut front, mut root, source) = begin(b"/accounts/connect", false, false);
    account(
        &mut front,
        &mut root,
        a::Event::SignedIn {
            source,
            subject: b"qa-row-person".as_slice().into(),
            display: Box::new([]),
            declined: false,
        },
    );
    h::reopen(&mut front, &mut root, Address::Accounts, false);
    account(
        &mut front,
        &mut root,
        a::Event::Component(a::FromComponent::Failed { account: source, failure: a::Failure::SignInAgain }),
    );
    press(&mut front, &mut root, b"Continue with ChatGPT");
    let source = h::signing(&front);
    visit(&mut front, &mut root, source);
    assert!(offered(&root, Role::TextField, FIELD));
    assert!(disabled(&root));
    paste(&mut front, &mut root, &callback());
    assert!(!disabled(&root));
    press(&mut front, &mut root, FINISH);
    account(
        &mut front,
        &mut root,
        a::Event::SignedIn {
            source,
            subject: b"qa-row-second-person".as_slice().into(),
            display: Box::new([]),
            declined: false,
        },
    );
    assert!(has(&root, b"Signed in to ChatGPT"));
    assert!(!offered(&root, Role::TextField, FIELD));
    assert!(front.leaves.is_empty());
}

#[test]
#[ignore = "needs: joined-world public finish-refusal and taken replies correlated to the pending finish"]
fn each_finish_reply_clears_the_value_and_shows_its_promised_words() {}

#[test]
#[ignore = "needs: joined-world public client small-store observation for the root-only fixture"]
fn lost_finish_is_never_replayed_and_a_manual_retry_keeps_its_name() {}
