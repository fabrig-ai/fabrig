//! The shipped TUI host on the engine (temper/docs/design/client/README.md, 10).
use jig_client_kit::harness::{ClientId, FrontFault, World};
use skein_lib::{Duration, Token};
use skein_ui_screen_terminal::Key;
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
