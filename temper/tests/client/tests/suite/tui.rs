//! The shipped TUI host on the engine (temper/docs/design/client/README.md, 10).
use jig_client::ApprovalCode;
use jig_client_kit::harness::{ClientId, FrontFault, Settings, World};
use skein_lib::{Duration, Secret, Token, Wall};
use skein_ui_screen_terminal::Key;
use temper_client_domain::{self as client, Action, Place, approve};
use temper_client_tui::Note;
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{self, Joint},
    tui::{ADDRESS, File, Tui},
};

const TUI: ClientId = ClientId(1_000);
const ELSEWHERE: ClientId = ClientId(1_001);
const SECOND_TUI: ClientId = ClientId(1_002);
const SIGN_OUT: Token = Token::new(1);
const CONNECTED: &str = "temper · Project 1";
const SIGN_IN: &str = "Sign in this terminal to 127.0.0.1:8367";
// Walking's one seeded project is party 1's project in `me`.
const PROJECTS: &str = "/ for commands";
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
    open_approval(&mut world, &mut tui, browser, code);
    world.event(browser, client::Event::Act { action: Action::Approve(approve::Act::Approve) }.into());
    tui.run(&mut world, fixtures::WITHIN);
    assert_eq!(world.root(browser).person().approval().expect("the approval page").state(), approve::State::Approved);
    assert!(tui.credential().is_some(), "the host keeps the approval's credential");
    assert_eq!(tui.keeps.len(), CREDENTIAL_WRITES);
    // The take's retry precedes the signed-in stream and its first watches.
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    assert!(text.contains(CONNECTED), "{text}");
    assert!(text.contains(PROJECTS), "{text}");
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
fn a_tuis_launchpad_asks_temper_and_draws_the_chat() {
    const QUESTION: &str = "Which checks should we run?";
    const PARTS: &[&[u8]] = &[b"Run the focused checks", b" before the fuzzy suite."];
    const ANSWER: &str = "Run the focused checks before the fuzzy suite.";
    let mut world = fixtures::tui_asking_world(fixtures::SEED);
    world.front().reply_in_pieces(PARTS);
    let mut tui = Tui::start(&mut world, TUI, None);
    tui.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    for words in [CONNECTED, "Ask temper", PROJECTS, "● Live"] {
        assert!(text.contains(words), "missing {words:?}: {text}");
    }

    tui.type_words(&mut world, "/");
    tui.run(&mut world, STEP);
    let text = tui.text();
    for command in ["/model", "/new", "/chats", "/theme", "/help", "/exit"] {
        assert!(text.contains(command), "missing {command}: {text}");
    }
    assert!(!text.contains("/font"), "{text}");
    tui.press(&mut world, vec![Key::Escape]);
    tui.run(&mut world, STEP);
    let text = tui.text();
    assert!(text.contains("❯ Ask temper"), "Esc clears the box: {text}");
    assert!(!text.contains("/exit"), "Esc closes the commands: {text}");
    assert!(text.contains(PROJECTS), "{text}");

    tui.type_words(&mut world, QUESTION);
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    let front = world.front();
    let assignment = front.assignments.last().expect("Ask temper run assigned");
    assert_eq!(assignment.charter, 11);
    let task = front.engine.task(assignment.task).expect("Ask temper task");
    assert_eq!(task.spec.words.as_ref(), QUESTION.as_bytes());
    let heading =
        format!("Project 1 / Which checks should… T{} on {}", assignment.task, core::str::from_utf8(ADDRESS).unwrap());
    let text = tui.text();
    assert!(text.contains(&heading), "{text}");
    assert!(text.contains(&format!("❯ {QUESTION}")), "{text}");
    assert!(text.contains("✦ Run the focused checks"), "{text}");
    assert!(text.contains("✦ Working"), "{text}");
    assert!(!text.contains(ANSWER), "only the first piece is released: {text}");

    assert!(world.front().release_reply(), "release the second piece without committing");
    tui.run(&mut world, STEP);
    let text = tui.text();
    assert!(text.contains(ANSWER), "{text}");
    assert!(text.contains("✦ Working"), "the whole reply is still provisional: {text}");
    assert!(world.front().release_reply(), "commit the whole answer");
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    assert!(text.contains(&format!("✦ {ANSWER}")), "{text}");
    assert!(!text.contains("✦ Working"), "the run has ended: {text}");
    assert!(text.contains(PROJECTS), "{text}");
    assert!(text.contains("● Live"), "{text}");
    assert!(!world.front().release_reply(), "no reply remains held after commit");

    tui.type_words(&mut world, "/exit");
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, STEP);
    assert!(tui.closed, "/exit asks the shipped host to close");
}

fn file_secret_is_in_no_frame(tui: &Tui, link: &[u8]) {
    let url = std::str::from_utf8(link).expect("fixture URL");
    let (_, secret) = url.split_once("?secret=").expect("fixture secret field");
    assert!(!tui.frames.is_empty(), "the host drew frames");
    for frame in &tui.frames {
        assert!(!frame.contains(url), "the file's URL must not be drawn");
        assert!(!frame.contains(secret), "the file's secret must not be drawn");
    }
}

#[test]
fn a_fresh_tui_signs_in_from_the_engines_link_with_no_key() {
    let mut world = fixtures::tui_world(fixtures::SEED);
    let link = world.front().local_link().expect("the engine's local link");
    assert!(link != front::printed_link(), "the file's link must differ from the printed link");
    assert!(Some(&link) != world.front().latest_link().as_ref(), "the file holds only the local link");
    let mut tui = Tui::with_file(&mut world, TUI, None, File::Engine, None);
    assert!(tui.text().contains("Connecting to 127.0.0.1:8367"));
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.text().contains(CONNECTED));
    assert!(tui.text().contains(PROJECTS));
    assert!(tui.status().ends_with("● Live"));
    assert!(tui.credential().is_some(), "the file's sign-in keeps a credential");
    assert_eq!(tui.keeps.len(), CREDENTIAL_WRITES);
    assert_eq!(tui.reads, 1);
    assert_eq!(tui.sign_in_sends, 1);
    assert_eq!(tui.approval_asks, 0);
    assert!(tui.opens.is_empty());
    assert!(matches!(tui.notes.last(), Some(Note::Until(until)) if *until > world.front().engine.env.wall));
    assert!(
        world.front().local_link().is_some_and(|next| next != link),
        "the file's sign-in draws the next local link"
    );
    file_secret_is_in_no_frame(&tui, &link);
}

#[test]
fn a_tui_from_the_file_and_a_browser_by_the_printed_link_sign_in_in_either_order() {
    for (row, order) in [
        ("printed first", [ELSEWHERE, TUI].as_slice()),
        ("file first", [TUI, ELSEWHERE].as_slice()),
        ("a second TUI", [TUI, SECOND_TUI, ELSEWHERE].as_slice()),
    ] {
        let mut world = fixtures::tui_world(fixtures::SEED);
        let mut file_links = Vec::new();
        let mut hosts = Vec::new();
        for &who in order {
            let link = world.front().local_link().expect("the engine's local link");
            assert!(link != front::printed_link(), "{row}: the local link is never the printed link");
            assert!(
                world.front().latest_link() == Some(front::printed_link()),
                "{row}: the printed link stays unchanged"
            );
            // A shipped host pastes the printed link as a browser would.
            let mut host = if who == ELSEWHERE {
                let mut host = Tui::start(&mut world, who, None);
                host.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
                host
            } else {
                assert!(
                    file_links.iter().all(|previous| previous != &link),
                    "{row}: each TUI reads the next local link"
                );
                file_links.push(link.clone());
                Tui::with_file(&mut world, who, None, File::Engine, None)
            };
            host.run(&mut world, fixtures::WITHIN);
            if who != ELSEWHERE {
                assert_eq!(host.reads, 1, "{row}: the file signs in at its first read");
                assert_eq!(host.approval_asks, 0, "{row}: the file needs no approval");
                assert!(host.opens.is_empty(), "{row}: the file opens nothing");
            }
            assert_eq!(host.sign_in_sends, 1, "{row}: each host sends one sign-in");
            assert!(host.credential().is_some(), "{row}: each host keeps a credential");
            assert_eq!(host.keeps.len(), CREDENTIAL_WRITES, "{row}");
            assert!(host.text().contains(CONNECTED), "{row}: the launchpad is connected");
            assert!(host.text().contains(PROJECTS), "{row}: the launchpad draws its commands");
            assert!(host.status().ends_with("● Live"), "{row}: each host is live");
            if who != ELSEWHERE {
                assert!(
                    world.front().local_link().is_some_and(|next| next != link),
                    "{row}: spending the file's link draws the next"
                );
            }
            assert!(
                world.front().latest_link() == Some(front::printed_link()),
                "{row}: signing in draws no printed link"
            );
            assert!(
                world.front().local_link().is_some_and(|next| next != front::printed_link()),
                "{row}: the next local link is never printed"
            );
            hosts.push(host);
        }
        for host in &hosts {
            for link in &file_links {
                file_secret_is_in_no_frame(host, link);
            }
        }
    }
}

#[test]
fn a_stale_files_link_falls_back_with_nothing_alarming() {
    const UNKNOWN: &[u8] =
        b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    const ELSEWHERE_LINK: &[u8] =
        b"http://127.0.0.1:9000/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    for (link, sends) in [(UNKNOWN, 1), (ELSEWHERE_LINK, 0)] {
        let mut world = fixtures::tui_world(fixtures::SEED);
        let mut tui = Tui::with_file(&mut world, TUI, None, File::Bytes(Secret::new(link.into())), None);
        tui.run(&mut world, fixtures::WITHIN);
        assert_eq!(tui.sign_in_sends, sends, "only a link at this address is sent");
        assert_eq!(tui.reads, 1);
        assert_eq!(tui.approval_asks, 1);
        assert!(tui.credential().is_none());
        assert!(tui.text().contains(SIGN_IN));
        for frame in &tui.frames {
            for refusal in ["isn't from this temper", "already used", "refused"] {
                assert!(!frame.contains(refusal), "a file's fallback draws no paste refusal");
            }
        }
    }
}

#[test]
fn an_expired_sign_in_is_renewed_from_the_file() {
    let mut world = fixtures::tui_world(fixtures::SEED);
    let link = world.front().local_link().expect("the engine's local link");
    // This world's wall starts at the epoch, with no representable hour before it.
    let expired = Note::Until(Wall::from_nanos(1));
    let credential = Secret::new(Box::new([0; 40]));
    let mut tui = Tui::with_file(&mut world, TUI, Some(credential), File::Engine, Some(expired));
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.keeps.first().is_some_and(Option::is_none), "erase the unknown credential first");
    assert_eq!(tui.keeps.len(), 2, "erase, then keep the new sign-in");
    assert!(tui.credential().is_some());
    assert!(tui.text().contains(CONNECTED));
    assert!(tui.text().contains(PROJECTS));
    assert!(tui.status().ends_with("● Live"));
    assert_eq!(tui.reads, 1);
    assert_eq!(tui.approval_asks, 0);
    assert!(tui.opens.is_empty());
    assert!(matches!(tui.notes.last(), Some(Note::Until(until)) if *until > world.front().engine.env.wall));
    assert!(world.front().local_link().is_some_and(|next| next != link), "renewal spends the file's local link");
    file_secret_is_in_no_frame(&tui, &link);
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
    let mut world = fixtures::tui_world(fixtures::SEED);
    let mut tui = Tui::with_file(&mut world, TUI, None, File::Engine, Some(Note::SignedOut));
    assert_eq!(tui.reads, 0);
    tui.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    assert!(matches!(tui.notes.last(), Some(Note::Until(_))));
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
    assert_eq!(tui.notes.last(), Some(&Note::SignedOut));
    assert_eq!(tui.reads, 0, "a sign-out does not read the file");

    let note = tui.notes.last().copied();
    let mut restarted = Tui::with_file(&mut world, ClientId(1_002), None, File::Engine, note);
    restarted.run(&mut world, fixtures::WITHIN);
    assert_eq!(restarted.reads, 0, "the retained signed-out note forbids renewal");
    assert_eq!(restarted.approval_asks, 1);
    assert!(restarted.text().contains(SIGN_IN));
    // A person's paste may try the spent link and draw the next, then use that one.
    restarted.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
    restarted.run(&mut world, STEP);
    let fresh = world.front().latest_link().expect("trying the spent link draws a fresh one");
    restarted.press(&mut world, vec![Key::Paste(fresh), Key::Enter]);
    restarted.run(&mut world, fixtures::WITHIN);
    assert!(restarted.credential().is_some());
    assert!(restarted.status().ends_with("● Live"));
    assert!(matches!(restarted.notes.last(), Some(Note::Until(_))));
    assert_eq!(restarted.reads, 0, "the paste alone signs the fresh host in");
}
