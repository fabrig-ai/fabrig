//! QA pass 2 (agents' roles/owners.md, 4.6): the browser shell, milestone 1, as Chromium shows it
//! against `temper serve` (docs/design/client/browser.md). Each test is a person's script; the harness fails
//! it on a policy violation, an exception or a native dialog. Run by `--profile qa`, which the gate does not.
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Setup, Step};
use skein_lib::Duration;
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, Tier, WITHIN, start_chat};

fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: WITHIN })
}
fn gone(find: Find) -> Step {
    Step::Person(PersonStep::Gone { find, within: ABSENT })
}
fn go(address: &[u8]) -> Step {
    Step::Person(PersonStep::Go { address: address.into() })
}
fn at(path: &[u8]) -> Step {
    Step::Address(path.into())
}
fn live() -> Find {
    Find::exactly_named(Role::Status, b"Live")
}
fn inbox() -> Find {
    Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")
}
fn composer() -> Find {
    Find::exactly_named(Role::TextArea, b"Start a chat")
}
/// The first chat's heading, whatever its title.
fn chat() -> Find {
    Find::named(Role::Heading { level: 1 }, b"T1 \xc2\xb7 ")
}
fn played(setup: Setup<'_>, steps: Vec<Step>) {
    let mut entered = vec![see(Find::exactly_named(Role::Link, b"Set up temper")), go(b"/chats")];
    entered.extend(steps);
    let mut script = Script::new(entered.into_boxed_slice());
    assert_eq!(Harness::new(setup).run(&mut [&mut script]), Outcome::Done);
}
fn play(name: &str, steps: Vec<Step>) {
    let tier = Tier::new(name);
    played(tier.setup(), steps);
}

/// browser.md 2.3: "A path that is no page opens the inbox, and the address is replaced with `/`."
#[test]
fn a_path_that_is_no_page_opens_the_inbox_at_slash() {
    let rows: [&[u8]; 14] = [
        b"/nowhere",
        b"/chats/abc",
        b"/chats/-1",
        b"/chats/99999999999999999999999999",
        b"/chats/1/extra",
        b"/chats/%zz",
        b"/chats//1",
        b"/tasks/x",
        b"/projects/x/adopt",
        b"/projects//adopt",
        b"/CHATS",
        b"/test/other",
        b"/test/panic/",
        b"/test/Panic",
    ];
    let mut steps = vec![see(live())];
    for row in rows {
        steps.extend([go(row), see(inbox()), at(b"/")]);
    }
    play("qa-shell-no-page", steps);
}

/// browser.md 2.3: the codec is "total and bounded": a number at the edge of a type, or of a task nobody made,
/// opens a page, or the inbox, with the page live and no trouble.
#[test]
fn an_address_at_the_edge_of_a_number_opens_live() {
    let rows: [&[u8]; 9] = [
        b"/chats/0",
        b"/chats/999",
        b"/chats/4294967295",
        b"/chats/4294967296",
        b"/chats/18446744073709551615",
        b"/chats/18446744073709551616",
        b"/tasks/1",
        b"/projects/1/adopt",
        b"/projects/65536/adopt",
    ];
    let mut steps = vec![see(live())];
    for row in rows {
        steps.extend([go(row), see(live())]);
    }
    play("qa-shell-edges", steps);
}

/// browser.md 2.3, as a person meets it: "a link pressed is a move ... with a new history entry; back and
/// forward are moves too", across an address the page replaced.
#[test]
fn back_crosses_an_address_the_page_replaced() {
    play(
        "qa-shell-replaced",
        vec![
            see(live()),
            go(b"/chats"),
            see(composer()),
            at(b"/chats"),
            go(b"/nowhere"),
            see(inbox()),
            at(b"/"),
            Step::Back,
            see(composer()),
            at(b"/chats"),
            Step::Forward,
            see(inbox()),
            at(b"/"),
        ],
    );
}

/// browser.md 2.4: "A reload ... signs in with `Transport`, by the cookie", and jig's sign-in part takes only a
/// 32-byte base64url secret at the link route: any other `secret=` is no link, so the load signs in by the
/// cookie, still live, and temper replaces the address with `/`.
fn no_link(name: &str, link: &[u8]) {
    play(name, vec![see(live()), go(link), see(live()), see(inbox()), at(b"/")]);
}
#[test]
fn a_garbled_secret_is_no_link_and_the_cookie_signs_in() {
    no_link("qa-shell-garbled", b"/jig/v1/sign-in/link?secret=garbled");
}
#[test]
fn an_empty_secret_is_no_link_and_the_cookie_signs_in() {
    no_link("qa-shell-empty", b"/jig/v1/sign-in/link?secret=");
}
#[test]
fn a_secret_of_escapes_is_no_link_and_the_cookie_signs_in() {
    no_link("qa-shell-escapes", b"/jig/v1/sign-in/link?secret=%00%ff");
}
#[test]
fn a_secret_of_three_hundred_bytes_is_no_link_and_the_cookie_signs_in() {
    let long = format!("/jig/v1/sign-in/link?secret={}", "a".repeat(300));
    no_link("qa-shell-long-secret", long.as_bytes());
}

/// The control for the rows above: the printed link opened again is a link, now spent, and the person is
/// signed out (browser.md 2.4).
#[test]
fn the_printed_link_opened_again_signs_the_person_out() {
    let tier = Tier::new("qa-shell-spent");
    played(
        tier.setup(),
        vec![see(live()), Step::Reopen, see(tier.states().signed_out), at(b"/welcome"), gone(Find::text(b"Needs you"))],
    );
}

/// browser.md 2.4: "Signed out, by the banner's 'Sign out'": the page, with no sign-in of its own to make,
/// raises no trouble (the harness's strict policy fails a console error, as a 401 answered to a request).
#[test]
fn signing_out_raises_no_trouble() {
    let tier = Tier::new("qa-shell-sign-out-quiet");
    played(
        tier.setup(),
        vec![
            see(live()),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign out") }),
            see(tier.states().signed_out),
        ],
    );
}

/// browser.md 2.4: "(jig's sign-out clears the cookie)": a reload or an address typed signs in by no cookie.
/// The 401 each load's stream is answered with is expected here: the client end hears it as signed out.
#[test]
fn signing_out_clears_the_cookie_for_every_later_load() {
    let tier = Tier::new("qa-shell-sign-out");
    let states = tier.states();
    let mut setup = tier.setup();
    setup.expected = vec![Box::from(b"401".as_slice())];
    played(
        setup,
        vec![
            see(live()),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Button, b"Sign out") }),
            see(states.signed_out.clone()),
            Step::Person(PersonStep::Reload),
            see(states.signed_out.clone()),
            gone(live()),
            go(b"/chats/1"),
            see(states.signed_out.clone()),
            gone(live()),
            go(b"/"),
            see(states.signed_out),
            gone(live()),
        ],
    );
}

/// browser.md 2.4: "Each load of the page is a client of its own, with its own stream"; the tier's engine
/// bounds a party's watchers (`views.watchers_per_party`, 16): a stream a load leaves behind would end it
/// within eighteen loads. The harness's own deadline is about twenty seconds, so no more are played.
#[test]
fn many_loads_in_a_row_each_hold_a_stream_of_their_own() {
    let mut steps = vec![see(live())];
    for _ in 0..18 {
        steps.extend([Step::Person(PersonStep::Reload), see(live())]);
    }
    for address in [&b"/chats"[..], b"/", b"/chats", b"/nowhere"].repeat(2) {
        steps.extend([go(address), see(live())]);
    }
    play("qa-shell-loads", steps);
}

/// browser.md 2.4: reloads in quick succession, each a new client before the last was live.
#[test]
fn reloads_in_quick_succession_end_in_one_live_page() {
    let reload = || Step::Person(PersonStep::Reload);
    play(
        "qa-shell-quick",
        vec![see(live()), reload(), reload(), reload(), reload(), reload(), see(live()), at(b"/chats")],
    );
}

/// browser.md 2.5, 4.1 (3): "Enter starts a chat with the words once, and Shift+Enter makes a new line":
/// not an empty composer's Enter, not Shift+Enter, and a second Enter makes no second chat.
#[test]
fn enter_starts_one_chat_and_empty_or_shift_enter_none() {
    let tier = Tier::new("qa-shell-composer").with_model();
    let words = |find: Find, words: &[u8]| Step::Type { find, words: words.into() };
    played(
        tier.setup(),
        vec![
            see(live()),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") }),
            see(composer()),
            words(composer(), b""),
            Step::Key(Key::Enter),
            gone(chat()),
            words(composer(), b"one"),
            Step::KeyWithShift(Key::Enter),
            Step::KeyWithShift(Key::Enter),
            gone(chat()),
            see(composer()),
            Step::Key(Key::Enter),
            Step::Key(Key::Enter),
            see(chat()),
            go(b"/chats"),
            see(composer()),
            see(Find::text(FIRST_CHAT)),
            gone(Find::text(b"T2")),
        ],
    );
}

/// browser.md 2.5: the composer's words "are counted within [their] bound" (README.md 5.1, 5.4): words far
/// past any bound leave the page standing and live, also after a reload.
#[test]
fn words_far_past_the_composers_bound_leave_the_page_live() {
    let long = "x".repeat(10_000);
    play(
        "qa-shell-long",
        vec![
            see(live()),
            Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") }),
            see(composer()),
            Step::Type { find: composer(), words: long.as_bytes().into() },
            Step::Key(Key::Enter),
            see(live()),
            Step::Person(PersonStep::Reload),
            see(live()),
        ],
    );
}

/// browser.md 2.4: "A reload ... signs in with `Transport`, by the cookie" holds through the server's own
/// restart: the banner says "Offline", then "Live", and a chat started before it is there at its address.
#[test]
fn a_restart_shows_offline_then_live_with_the_chat_in_place() {
    let tier = Tier::reserved("qa-shell-restart").with_model();
    let mut setup = tier.setup();
    setup.expected = vec![
        Box::from(b"net::ERR_CONNECTION_REFUSED".as_slice()),
        Box::from(b"net::ERR_INCOMPLETE_CHUNKED_ENCODING".as_slice()),
    ];
    let mut steps = start_chat(b"hello");
    steps.extend([
        Step::StopServer,
        see(Find::exactly_named(Role::Status, b"Offline")),
        Step::StartServer,
        see(live()),
        go(b"/chats/1"),
        see(chat()),
        at(b"/chats/1"),
        Step::Person(PersonStep::Pause { span: Duration::from_millis(300) }),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(setup).run(&mut [&mut script]), Outcome::Done);
}
