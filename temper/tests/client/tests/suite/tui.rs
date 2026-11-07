//! The shipped TUI host on the engine (temper/docs/design/client/README.md, 10).
use jig_client::ApprovalCode;
use jig_client_kit::harness::{ClientId, FrontFault, Settings, World};
use skein_lib::{Duration, Token};
use skein_ui_screen_terminal::Key;
use temper_client_domain::{self as client, Action, Place, approve};
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{self, Joint},
    tui::{ADDRESS, Tui},
};

const TUI: ClientId = ClientId(1_000);
const ELSEWHERE: ClientId = ClientId(1_001);
const SIGN_OUT: Token = Token::new(1);
const CONNECTED: &str = "Connected to 127.0.0.1:8367 as Person";
const SIGN_IN: &str = "Sign in this terminal to 127.0.0.1:8367";
// Walking's one seeded project is party 1's project in `me`.
const PROJECTS: &str = "Projects: Project 1";
const OFFLINE: Duration = Duration::from_millis(300);
const STEP: Duration = Duration::from_millis(50);
const RESTART_SPAN: Duration = Duration::from_secs(2);
// One printed-link sign-in writes its credential once.
const CREDENTIAL_WRITES: usize = 1;

fn approval() -> (World<Temper, Joint>, Tui, ClientId) {
    let mut world = World::new(
        Settings { seed: fixtures::SEED, ..Settings::default() },
        Joint::with_approval(fixtures::SEED, fixtures::domain_limits()),
    );
    world.randomness(2, 0, 0);
    let browser = world.client(fixtures::PARTY, fixtures::inbox());
    world.front().as_browser(browser);
    let mut tui = Tui::start(&mut world, TUI, None);
    assert_eq!(tui.approval_asks, 1, "the first duties ask for approval");
    assert!(tui.opens.is_empty(), "nothing opens before the ask is answered");
    assert!(tui.credential().is_none());
    assert!(tui.text().contains("Asking temper for a code…"), "{}", tui.text());
    tui.run(&mut world, fixtures::WITHIN);
    (world, tui, browser)
}

fn opened_code(tui: &mut Tui) -> ApprovalCode {
    let address = String::from_utf8(tui.opens.last().expect("the host opens the approval page").to_vec())
        .expect("the opened address is UTF-8");
    let shown = address.strip_prefix("http://127.0.0.1:8367/terminal?code=").expect("the TUI's own address");
    let code = ApprovalCode::parse(shown.as_bytes()).expect("the opened address carries an approval code");
    assert_eq!(shown.as_bytes(), code.shown());
    let text = tui.text();
    assert!(text.contains(std::str::from_utf8(&code.shown()).expect("code alphabet")), "{text}");
    assert!(text.contains(&address), "{text}");
    code
}

fn open_approval(world: &mut World<Temper, Joint>, tui: &mut Tui, browser: ClientId, code: ApprovalCode) {
    world.event(browser, client::Event::Went { place: Place::Approve { code: Some(code) } }.into());
    tui.run(world, STEP);
    assert!(matches!(
        world.root(browser).person().approval().expect("the browser's approval page").state(),
        approve::State::Asking { .. }
    ));
}

#[test]
fn a_tuis_approval_approved_by_a_browser_draws_the_first_screen() {
    let (mut world, mut tui, browser) = approval();
    assert_eq!(tui.opens.len(), 1, "each answered ask opens once");
    let code = opened_code(&mut tui);
    let root = world.root(browser).person();
    let party = root.pages().me(root.client()).expect("the browser's party").party.clone();
    open_approval(&mut world, &mut tui, browser, code);
    world.event(browser, client::Event::Act { action: Action::Approve(approve::Act::Approve) }.into());
    tui.run(&mut world, fixtures::WITHIN);
    assert_eq!(world.root(browser).person().approval().expect("the approval page").state(), approve::State::Approved);
    assert!(tui.credential().is_some(), "the host keeps the approval's credential");
    assert_eq!(tui.keeps.len(), CREDENTIAL_WRITES);
    // The take's retry precedes the signed-in stream and its first watches.
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    let name = std::str::from_utf8(party.name.bytes()).expect("party name");
    assert!(text.contains(&format!("Connected to 127.0.0.1:8367 as {name}")), "{text}");
    assert!(tui.status().ends_with("● Live"), "status={:?}; frame={text}", tui.status());
    assert_eq!(tui.opens.len(), 1, "the take opens no second tab");
}

#[test]
fn a_tuis_approval_refused_says_so_and_asks_again() {
    let (mut world, mut tui, browser) = approval();
    let code = opened_code(&mut tui);
    open_approval(&mut world, &mut tui, browser, code);
    world.event(browser, client::Event::Act { action: Action::Approve(approve::Act::Refuse) }.into());
    tui.run(&mut world, fixtures::WITHIN);
    assert_eq!(world.root(browser).person().approval().expect("the approval page").state(), approve::State::Refused);
    let text = tui.text();
    assert!(text.contains("The browser refused this sign-in. Press Enter to ask again."), "{text}");
    assert!(tui.credential().is_none());
    assert_eq!(tui.opens.len(), 1);
    tui.press(&mut world, vec![Key::Enter]);
    assert_eq!(tui.approval_asks, 2, "Enter in the empty field asks again");
    tui.run(&mut world, STEP);
    assert_eq!(tui.opens.len(), 2, "the new ask opens a new tab once");
    assert_ne!(opened_code(&mut tui), code, "the new ask carries a new code");
}

fn signed_in() -> (World<Temper, Joint>, Tui) {
    let mut world = fixtures::tui_world(fixtures::SEED);
    let mut tui = Tui::start(&mut world, TUI, None);
    tui.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    (world, tui)
}

#[test]
fn a_tui_signed_in_by_its_pasted_link_draws_me() {
    let mut world = fixtures::tui_world(fixtures::SEED);
    let mut tui = Tui::start(&mut world, TUI, None);
    assert!(tui.text().contains(SIGN_IN), "{}", tui.text());
    assert!(tui.status().ends_with("● Signed out"), "status={:?}; frame={}", tui.status(), tui.text());
    tui.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    assert!(text.contains(CONNECTED), "{text}");
    assert!(text.contains(PROJECTS), "{text}");
    assert!(tui.status().ends_with("● Live"), "status={:?}; frame={}", tui.status(), tui.text());
    assert_eq!(tui.keeps.len(), CREDENTIAL_WRITES);
    assert!(tui.credential().is_some());
    assert_eq!(tui.address.as_deref(), Some(ADDRESS));
}

#[test]
fn the_engine_restarted_under_a_tui_goes_offline_then_live_and_keeps_me() {
    let (mut world, mut tui) = signed_in();
    let credential = tui.credential().cloned().expect("signed-in TUI");
    world.fault(FrontFault::Restart { offline: OFFLINE });
    let mut offline = false;
    let mut live_after = false;
    let until = world.now().saturating_add(RESTART_SPAN);
    while world.now() < until {
        tui.run(&mut world, STEP);
        let text = tui.text();
        assert!(text.contains(CONNECTED), "{text}");
        assert!(text.contains(PROJECTS), "{text}");
        let status = tui.status();
        offline |= status.ends_with("● Offline");
        live_after |= offline && status.ends_with("● Live");
    }
    assert!(offline, "the TUI must observe the engine's outage");
    assert!(live_after, "the TUI must reconnect after observing Offline");
    assert!(tui.status().ends_with("● Live"), "status={:?}; frame={}", tui.status(), tui.text());
    assert_eq!(tui.keeps, vec![Some(credential)]);
}

#[test]
fn a_tui_signed_out_elsewhere_asks_for_a_link_with_its_token_erased() {
    let (mut world, mut tui) = signed_in();
    let credential = tui.credential().cloned().expect("signed-in TUI");
    world.front().link_client(ELSEWHERE, temper_client_shell_tui::limits::tui().native.client.client.domain);
    let now = world.now();
    world.front().carry(
        ELSEWHERE,
        now,
        jig_client::Request::Exchange {
            exchange: SIGN_OUT,
            credential: Some(credential.clone()),
            message: jig_client::Message::SignOut,
        },
    );
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    assert!(text.contains("Signed out."), "{text}");
    assert!(text.contains(SIGN_IN), "{text}");
    assert!(tui.status().ends_with("● Signed out"), "status={:?}; frame={}", tui.status(), tui.text());
    assert_eq!(tui.keeps, vec![Some(credential), None]);
    assert!(tui.credential().is_none());
}
