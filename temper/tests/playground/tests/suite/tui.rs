//! Both shipped hosts in one world (temper/docs/design/client/journey-walks.md, 3.3).
use skein_fake_person::Find;
use skein_lib::Duration;
use skein_ui::{Act, Role};
use skein_ui_screen_terminal::Key;
use temper_client_walk_world::{
    Native, facts,
    web::{Failed, Page},
};
use temper_playground::programs::Process;

const WITHIN: Duration = Duration::from_secs(30);
const APPROVAL: &[u8] = b"then press Approve.";
const LAUNCHPAD: &[u8] = b"Setting temper up makes your first project";

fn start() -> Page {
    Page::start(facts::start(None).expect("fixed start"), b"a").expect("native page starts")
}
fn text(page: &mut Page) -> String {
    String::from_utf8(page.tui().expect("started").frame().text().into_vec()).expect("UTF-8 frame")
}
fn awaiting(page: &mut Page) {
    page.tui_start().expect("terminal starts");
    page.tui_wait(APPROVAL, WITHIN).expect("terminal asks for approval");
}
fn approved(page: &mut Page) {
    awaiting(page);
    page.tui_approve(WITHIN).expect("browser approves terminal");
    page.tui_wait(LAUNCHPAD, WITHIN).expect("signed-in launchpad");
}
fn playground(page: &mut Page) {
    let playground = page
        .wait(&Find::exactly_named(Role::Button, b"Try the playground"), Some(&Act::Press), WITHIN)
        .expect("welcome");
    page.act(playground, Act::Press).expect("make playground project");
    page.wait(&Find::text_box(b"What do you want done?"), None, WITHIN).expect("playground ready");
}
fn type_words(page: &mut Page, words: &str) {
    for key in words.chars().map(Key::Char) {
        page.tui_key(key).expect("character key");
    }
    page.tui_key(Key::Enter).expect("enter");
}

#[test]
fn started_terminal_asks_to_browse_the_service() {
    let mut page = start();
    assert!(page.tui().is_none());
    assert!(!page.world().processes().iter().any(|process| matches!(process, Process::Other(Native::Terminal(_)))));
    awaiting(&mut page);
    assert!(text(&mut page).contains("Sign in this terminal to 127.0.0.1:10001"));
    assert!(
        page.tui().expect("started").browsed().expect("browsed").starts_with(b"http://127.0.0.1:10001/terminal?code=")
    );
    assert_eq!(
        page.world()
            .processes()
            .iter()
            .filter(|process| matches!(process, Process::Other(Native::Terminal(_))))
            .count(),
        1
    );
}

#[test]
fn browser_approval_signs_in_and_restores_its_address() {
    let mut page = start();
    let address = page.address();
    approved(&mut page);
    assert_eq!(page.address(), address);
    assert!(!text(&mut page).contains("then press Approve."));
    page.wait(&Find::exactly_named(Role::Button, b"Try the playground"), Some(&Act::Press), WITHIN)
        .expect("browser is still signed in at welcome");
}

#[test]
fn without_approval_the_terminal_spends_its_bound_still_asking() {
    let mut page = start();
    awaiting(&mut page);
    let began = page.world().now();
    assert_eq!(page.tui_wait(LAUNCHPAD, WITHIN), Err(Failed::Bound));
    assert!(page.world().now() >= began.saturating_add(WITHIN));
    assert!(text(&mut page).contains("then press Approve."));
}

#[test]
fn stopped_terminal_keeps_its_credential_for_the_next_start() {
    let mut page = start();
    approved(&mut page);
    playground(&mut page);
    page.tui_wait(b"/change", WITHIN).expect("first terminal's project launchpad");
    page.tui_stop();
    assert!(page.tui().is_none());
    page.tui_start().expect("second terminal starts");
    page.tui_wait(b"/change", WITHIN).expect("credential signs in without approval");
    assert!(page.tui().expect("started").browsed().is_none());
    type_words(&mut page, "/change Add CSV export to widget");
    let result = page.tui_wait("I’ll delegate this change in playground/widget.".as_bytes(), WITHIN);
    assert_eq!(result, Ok(()), "{}", text(&mut page));
    page.tui_stop();
    page.tui_start().expect("third terminal starts");
    page.tui_wait(b"/change", WITHIN).expect("third terminal's launchpad");
    assert!(page.tui().expect("started").browsed().is_none());
    type_words(&mut page, "/change Another request");
    let result = page.tui_wait(b"I am the scripted model of temper's playground", WITHIN);
    assert_eq!(result, Ok(()), "{}", text(&mut page));
}

#[test]
fn identical_terminal_calls_draw_identical_bytes() {
    assert_eq!(facts::tui_seed(0), 8);
    assert_eq!(facts::tui_seed(1), 9);
    assert_eq!(facts::tui_seed(u64::MAX), facts::SEED);
    let mut first = start();
    let mut second = start();
    awaiting(&mut first);
    awaiting(&mut second);
    assert_eq!(text(&mut first), text(&mut second));
    first.tui_approve(WITHIN).expect("first approval");
    second.tui_approve(WITHIN).expect("second approval");
    first.tui_wait(LAUNCHPAD, WITHIN).expect("first launchpad");
    second.tui_wait(LAUNCHPAD, WITHIN).expect("second launchpad");
    assert_eq!(text(&mut first), text(&mut second));
}

#[test]
fn a_closed_terminal_refuses_later_lines_until_stopped_but_the_browser_continues() {
    let mut page = start();
    page.tui_wait(APPROVAL, WITHIN).expect("wait starts a terminal");
    page.tui_key(Key::Control(b'c')).expect("close terminal");
    assert_eq!(page.tui_key(Key::Enter), Err(Failed::Closed));
    assert_eq!(page.tui_wait(APPROVAL, WITHIN), Err(Failed::Closed));
    assert_eq!(page.tui_approve(WITHIN), Err(Failed::Closed));
    page.wait(&Find::exactly_named(Role::Button, b"Try the playground"), Some(&Act::Press), WITHIN)
        .expect("the browser still runs");
    page.tui_stop();
    page.tui_wait(APPROVAL, WITHIN).expect("stop permits a new terminal");
}

#[test]
fn approval_without_a_browsed_address_spends_one_bound() {
    let mut page = start();
    approved(&mut page);
    page.tui_stop();
    page.tui_wait(LAUNCHPAD, WITHIN).expect("implicit restart keeps the credential");
    let began = page.world().now();
    assert_eq!(page.tui_approve(WITHIN), Err(Failed::NotBrowsed));
    assert!(page.world().now() >= began.saturating_add(WITHIN));
}

#[test]
fn a_chat_reply_reaches_the_terminal_over_its_native_stream() {
    let mut page = start();
    playground(&mut page);
    awaiting(&mut page);
    page.tui_approve(WITHIN).expect("approve for playground");
    page.tui_wait(b"/change", WITHIN).expect("project launchpad");
    type_words(&mut page, "/change Add CSV export to widget");
    let result = page.tui_wait("I’ll delegate this change in playground/widget.".as_bytes(), WITHIN);
    assert_eq!(result, Ok(()), "{}", text(&mut page));
    assert!(
        page.world().processes().iter().any(|process| {
            if let Process::Other(Native::Terminal(connection)) = process {
                connection.opened_streams() > 0
            } else {
                false
            }
        }),
        "the terminal carried OpenStream over the native transport"
    );
}
