//! The shipped TUI host on the engine (temper/docs/design/client/testing.md).
use jig_client::ApprovalCode;
use jig_client_kit::harness::{App, ClientId, FrontFault, Settings, World};
use skein_lib::{Duration, Queue, Secret, Token, Wall};
use skein_ui_screen_terminal::Key;
use temper_client_domain::{self as client, Action, Place, approve};
use temper_client_tui::{BoxLine, Note};
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{self, Joint, account_acts},
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
    for command in ["/model", "/new", "/chats", "/theme", "/help", "/quit"] {
        assert!(text.contains(command), "missing {command}: {text}");
    }
    assert!(!text.contains("/font"), "{text}");
    tui.press(&mut world, vec![Key::Escape]);
    tui.run(&mut world, STEP);
    let text = tui.text();
    assert!(text.contains("❯ Ask for a change, or ask temper.  / for commands"), "Esc clears the box: {text}");
    assert!(!text.contains("/quit"), "Esc closes the commands: {text}");
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
    assert_said_before_answer(&mut tui, QUESTION, ANSWER);
    assert!(!text.contains("✦ Working"), "the run has ended: {text}");
    assert!(tui.status().starts_with("Project 1 · "), "{text}");
    assert!(text.contains("● Live"), "{text}");
    assert!(!world.front().release_reply(), "no reply remains held after commit");

    tui.type_words(&mut world, "/quit");
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, STEP);
    assert!(tui.closed, "/quit asks the shipped host to close");
}

#[test]
fn a_need_the_engine_asks_is_walked_from_the_launchpad_and_answered() {
    const QUESTION: &str = "Comma or semicolon?";
    const WORDS: &[u8] = b"Comma or semicolon?\nChoice: Semicolon\nChoice: Comma (recommended)";
    let (mut world, chat) = fixtures::tui_question_world(fixtures::SEED, WORDS);
    let (question, delegate) = {
        let task = world.front().engine.task(chat).expect("question's chat");
        let question = task
            .inbox
            .iter()
            .find(|message| message.kind == jig_domain_tasks::MessageKind::Question && message.words.as_ref() == WORDS)
            .expect("the engine committed the question");
        let jig_domain_tasks::Party::Task(delegate) = question.from else { panic!("a delegate asks") };
        let number = world
            .front()
            .engine
            .task(delegate)
            .expect("asking delegate")
            .questions
            .first()
            .expect("delegate's question")
            .number;
        (number, delegate)
    };
    let mut tui = Tui::hello_launchpad(&mut world, TUI);
    tui.run(&mut world, fixtures::WITHIN);
    let rows = tui.rows();
    let needs =
        rows.iter().position(|row| row == "Needs you").unwrap_or_else(|| panic!("Needs you heading: {}", tui.text()));
    let need = rows
        .iter()
        .position(|row| row.contains("? Project 2") && row.contains(QUESTION))
        .unwrap_or_else(|| panic!("the question's row: {}", tui.text()));
    assert!(needs < need, "the question stands under Needs you: {}", tui.text());
    assert!(tui.text().contains("1 needs you · n to walk"), "{}", tui.text());

    tui.walk_needs(&mut world);
    let rows = tui.rows();
    let prompt = rows.iter().position(|row| row == QUESTION).expect("the walk's question head");
    assert_eq!(
        rows.get(prompt + 1).map(String::as_str),
        Some("Reached you because it was asked of you directly."),
        "{}",
        tui.text()
    );
    assert_eq!(rows.get(prompt + 2).map(String::as_str), Some("❯ 1. Comma"), "{}", tui.text());
    assert_eq!(rows.get(prompt + 3).map(String::as_str), Some("2. Semicolon"), "{}", tui.text());
    tui.press(&mut world, vec![Key::Enter]);
    // Needs you holds a choice for Undo before sending it to the engine.
    tui.run(&mut world, client::pages::needs::HOLD.saturating_add(fixtures::WITHIN));
    let answers: Vec<_> = world
        .front()
        .engine
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                jig_domain_tasks::Stored::Activity(line),
            )) = record
                && line.task == delegate
                && let jig_domain_tasks::Line::Words {
                    kind: jig_domain_tasks::MessageKind::Answer { question: number },
                    words,
                    ..
                } = &line.line
                && *number == question
            {
                Some(words.as_ref())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(answers, [b"Comma".as_slice()], "the engine records the recommended choice's words: {}", tui.text());
    let rows = tui.rows();
    assert!(!rows.iter().any(|row| row == "Needs you" || row.contains("? Project 2")), "{}", tui.text());
    assert!(rows.iter().any(|row| row == "Nothing else needs you."), "{}", tui.text());
}

#[test]
fn a_change_asked_from_the_launchpad_starts_a_chat() {
    const REQUEST: &str = "Add a CSV export";
    let mut world = fixtures::tui_asking_world(fixtures::SEED);
    let mut tui = Tui::hello_launchpad(&mut world, TUI);
    let reader = world.client(fixtures::PARTY, fixtures::inbox());
    tui.run(&mut world, fixtures::WITHIN);
    let root = world.root(reader).person();
    let me = root.pages().me(root.client()).expect("the engine's project templates");
    let template = jig_client::shapes::template(me, 1, jig_client::shapes::CHAT).expect("project's chat template");
    let jig_client::contract::TemplateExecutor::Charter(charter) = template.executor else {
        panic!("the chat template supplies its charter")
    };
    assert_ne!(charter, 11, "the project's chat and Ask temper have distinct charters");
    let before = fixtures::chats(&world.front().engine);
    tui.type_words(&mut world, &format!("/change {REQUEST}"));
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    assert_eq!(fixtures::chats(&world.front().engine), before + 1, "one new chat");
    let front = world.front();
    let assignment = front.assignments.last().expect("the new chat's run");
    let task = front.engine.task(assignment.task).expect("the engine's new chat");
    assert_eq!(task.executor, jig_domain_tasks::Executor::Agent { charter });
    assert_eq!(task.spec.words.as_ref(), REQUEST.as_bytes());
    let heading =
        format!("Hello / Add a CSV export T{} on {}", assignment.task, core::str::from_utf8(ADDRESS).expect("address"));
    assert!(tui.rows().iter().any(|row| row.contains(&heading)), "{}", tui.text());
    assert!(tui.rows().iter().any(|row| row == &format!("❯ {REQUEST}")), "{}", tui.text());
}

#[test]
fn telling_from_the_launchpad_files_feedback_in_hello() {
    const WORDS: &str = "The export button is grey";
    let mut world = fixtures::tui_asking_world(fixtures::SEED);
    let mut tui = Tui::hello_launchpad(&mut world, TUI);
    tui.type_words(&mut world, &format!("/tell {WORDS}"));
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    let front = world.front();
    let findings: Vec<_> = front
        .engine
        .store
        .rows
        .values()
        .filter_map(|record| {
            let temper_engine_domain::Record::Forge { row } = record else { return None };
            let temper_engine_domain_forge::Stored::Finding(finding) = row.as_ref() else { return None };
            Some(finding)
        })
        .collect();
    assert_eq!(findings.len(), 1, "one word files one finding");
    let finding = findings[0];
    assert_eq!(finding.project, 1);
    assert_eq!(finding.kind, temper_engine_domain_forge::FindingKind::Feedback);
    assert_eq!(finding.title.as_ref(), WORDS.as_bytes());
    let number = finding.number;
    assert!(tui.text().contains(&format!("Filed as finding {number} on Hello.")), "{}", tui.text());
}

const FAST_ANSWER: &str = "Hello from the local fake model.";

// client/tui/pages/talk.md, 1: template 7's role talks through the shipped host.
#[test]
fn a_person_talks_with_a_standing_role_in_the_tui_and_leaves() {
    const WORDS: &str = "Why is the port waiting?";
    const REPLY: &[u8] = b"The handoff fix touches the same three functions.";
    let (mut front, task) = super::person_task_tools::parked_role();
    front.answer_runs = false;
    let mut world: World<Temper, Joint> =
        World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front);
    world.randomness(0, 0, 0);
    let mut tui = Tui::role_launchpad(&mut world, TUI);
    assert!(tui.text().contains("/talk"), "{}", tui.text());
    tui.type_words(&mut world, "/talk");
    tui.press(&mut world, vec![Key::Enter]);
    assert!(tui.text().contains("Talk with a role on Hello"), "{}", tui.text());
    assert!(tui.text().contains("Owns the tests"), "{}", tui.text());
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.text().contains("talking with Owns the tests"), "{}", tui.text());
    tui.type_words(&mut world, WORDS);
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    let role = world.front().engine.task(task).expect("standing role");
    assert!(role.talk.is_some(), "the first words open the talk");
    super::person_task_tools::scripted_talk_reply(world.front(), task, REPLY);
    tui.run(&mut world, fixtures::WITHIN);
    let shown = tui.text();
    assert!(shown.contains(WORDS), "{shown}");
    assert!(shown.contains(core::str::from_utf8(REPLY).unwrap()), "{shown}");
    tui.press(&mut world, vec![Key::Escape]);
    tui.run(&mut world, fixtures::WITHIN);
    let ended = world.front().engine.store.rows.values().any(|record| {
        matches!(record,
            temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Activity(line)))
            if line.task == task && matches!(line.line, jig_domain_tasks::Line::TalkEnded { lapsed: false, .. })
        )
    });
    assert!(ended, "Esc ends the engine's talk");
    assert!(world.front().engine.task(task).unwrap().talk.is_none());
    let shown = tui.text();
    assert!(shown.contains("Left Owns the tests after 1 min"), "{shown}");
    assert!(shown.contains("It writes its Now."), "{shown}");
    assert!(!shown.contains("talking with Owns the tests"), "back to the launchpad: {shown}");
}

fn assert_said_before_answer(tui: &mut Tui, question: &str, answer: &str) {
    let text = tui.text();
    let said = text.lines().position(|line| line.trim() == format!("❯ {question}"));
    let reply = text.lines().position(|line| line.trim() == format!("✦ {answer}"));
    assert!(said.is_some(), "the Said row is separate from the title: {text}");
    assert!(reply.is_some(), "the completed reply is visible: {text}");
    assert!(said < reply, "the opening question precedes the reply: {text}");
}

fn resize_tui(tui: &mut Tui, columns: u16, rows: u16) {
    let duties =
        tui.host.report(temper_client_shell_tui::host::Report::Resize(skein_ui_screen::Size { columns, rows }));
    assert!(duties.asked.is_empty(), "resizing keeps the settled conversation");
}

fn complete_fast_chat(world: &mut World<Temper, Joint>, tui: &mut Tui, question: &str) {
    world.front().reply_at_once(FAST_ANSWER.as_bytes());
    let assigned = world.front().assignments.len();
    tui.type_words(world, question);
    assert!(tui.text().contains(&format!("❯ {question}")), "the question is typed before Enter");
    tui.press(world, vec![Key::Enter]);
    for _ in 0..100 {
        tui.run(world, STEP);
        if world.front().assignments.len() > assigned {
            break;
        }
    }
    assert_eq!(world.front().assignments.len(), assigned + 1, "{}", tui.text());
    assert_eq!(
        world.front().runs_answered,
        u32::try_from(assigned + 1).expect("four assigned chats"),
        "the reply commits at assignment"
    );
    tui.run(world, fixtures::WITHIN);
    tui.run(world, Duration::from_secs(1));
    assert_said_before_answer(tui, question, FAST_ANSWER);
    assert!(!tui.text().contains("✦ Working"), "the fast reply has completed");
}

#[test]
fn a_tuis_fast_completed_chats_keep_the_opening_said_row() {
    let mut world = fixtures::tui_asking_world(fixtures::SEED);
    let mut tui = Tui::start(&mut world, TUI, None);
    tui.press(&mut world, vec![Key::Paste(front::printed_link()), Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    resize_tui(&mut tui, 120, 40);
    let questions = ["What can you tell me about Hello?", "Which checks?", "What landed?", "What is next?"];
    for question in questions {
        complete_fast_chat(&mut world, &mut tui, question);
        tui.type_words(&mut world, "/new");
        tui.press(&mut world, vec![Key::Enter]);
        tui.run(&mut world, STEP);
        let text = tui.text();
        assert!(
            text.contains("❯ Ask for a change, or ask temper.  / for commands"),
            "new returns to the launchpad: {text}"
        );
    }
    tui.type_words(&mut world, "/chats");
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, STEP);
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    assert_said_before_answer(&mut tui, "What is next?", FAST_ANSWER);
    tui.press(&mut world, vec![Key::PageUp]);
    tui.run(&mut world, STEP);
    assert_said_before_answer(&mut tui, "What is next?", FAST_ANSWER);
    resize_tui(&mut tui, 80, 24);
    assert_said_before_answer(&mut tui, "What is next?", FAST_ANSWER);
}

fn file_secret_is_in_no_frame(tui: &Tui, link: &[u8]) {
    let url = std::str::from_utf8(link).expect("fixture URL");
    let (_, secret) = url.split_once("?secret=").expect("fixture secret field");
    assert!(!tui.frames.is_empty(), "the host drew frames");
    for frame in &tui.frames {
        let frame: String = frame.lines().map(str::trim).collect();
        assert!(!frame.contains(url), "the file's URL must not be drawn");
        assert!(!frame.contains(secret), "the file's secret must not be drawn");
    }
}

#[test]
fn a_tui_in_front_signs_in_by_its_handed_link_and_ticks_the_browsers_sign_in() {
    let mut world = fixtures::tui_first_run_world(fixtures::SEED);
    let handed = world.front().local_link().expect("the handed link");
    let printed = front::printed_link();
    assert!(handed != printed, "the handed and printed links differ");
    let mut tui = Tui::front(&mut world, TUI, None);
    tui.boxed(&mut world, BoxLine::Link { url: Secret::new(printed.clone()) });
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.credential().is_some(), "the handed link keeps a credential");
    assert_eq!(tui.keeps.len(), CREDENTIAL_WRITES);
    assert!(tui.status().ends_with("● Live"));
    assert_eq!(tui.reads, 1);
    assert_eq!(tui.sign_in_sends, 1);
    assert_eq!(tui.approval_asks, 0);
    assert_eq!(tui.opens.len(), 0);
    let text = tui.text();
    // The 92-byte link wraps at the host's 80 columns.
    let unwrapped: String = text.lines().map(str::trim).collect();
    assert!(
        unwrapped.contains(std::str::from_utf8(&printed).expect("the printed link is UTF-8")),
        "the welcome draws the printed link whole"
    );
    assert!(text.contains("In the browser"), "the welcome shows the browser's steps");
    assert!(!text.contains("Signed in as the owner"), "the handed link does not tick the browser's sign-in");

    // A second shipped host pastes the printed link as the browser would.
    let mut browser = Tui::start(&mut world, ELSEWHERE, None);
    browser.press(&mut world, vec![Key::Paste(printed), Key::Enter]);
    browser.run(&mut world, fixtures::WITHIN);
    assert!(browser.credential().is_some(), "the printed link signed the browser in");
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    assert!(text.contains("✓ Signed in as the owner"), "the browser's sign-in reaches the welcome");
    assert!(text.contains("▸ 1 Welcome"), "Welcome is the current step");
    assert_eq!(tui.reads, 1, "the handed link signs in without reading again");
    assert!(tui.opens.is_empty(), "the welcome never opens the printed link");
    file_secret_is_in_no_frame(&tui, &handed);
    file_secret_is_in_no_frame(&browser, &handed);
}

#[test]
fn a_playground_the_browser_starts_shows_in_the_tuis_header_and_quiet_line() {
    let (front, mut browser) = account_acts::tui_playground_fixture(fixtures::SEED);
    let mut world = World::new(Settings { seed: fixtures::SEED, ..Settings::default() }, front);
    world.randomness(2, 0, 0);
    let mut tui = Tui::with_file(&mut world, TUI, None, File::Engine, None);
    tui.run(&mut world, fixtures::WITHIN);
    let before = tui.text();
    assert!(before.contains(CONNECTED), "the TUI starts on the playground project's launchpad");
    assert!(!tui.status().contains("playground"));
    let (_, mut view) = Temper::new(fixtures::SEED);
    let mut patches = Queue::with_capacity(Temper::patch_limit(&view));
    Temper::build(&mut view, &browser, &mut patches);
    let button = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Try the playground"))
        .expect("the browser offers Try the playground")
        .id;
    let temper_client_world::app::View::Person(view) = &mut view else { panic!("the browser's view") };
    let action = temper_client_view::act(view, button, skein_ui::Act::Press).expect("the browser's start action");
    account_acts::event(world.front(), &mut browser, client::Event::Act { action }.into());
    tui.run(&mut world, fixtures::WITHIN);
    assert_eq!(world.front().playground_starts.len(), 1, "the browser starts the playground once");
    let text = tui.text();
    let header = text.lines().next().expect("the header");
    assert!(header.contains("playground · 127.0.0.1:8367"), "the playground precedes the place");
    let words = text.lines().map(|line| line.trim_matches(['│', ' '])).collect::<Vec<_>>().join(" ");
    assert!(words.contains("This is the playground: a scripted model and in-memory forge. Set temper up in your browser when you're ready to leave it."));
    assert!(tui.status().ends_with("playground"));
    assert!(!text.contains("Nothing needs you."), "the playground leaves out Home's headline");
}

#[test]
fn a_tui_in_front_shows_the_boxs_state_and_its_log() {
    let mut world = fixtures::tui_world(fixtures::SEED);
    let handed = world.front().local_link().expect("the handed link");
    let mut tui = Tui::front(&mut world, TUI, None);
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.credential().is_some());
    tui.boxed(&mut world, BoxLine::Restarting { step: b"forgejo-3".as_slice().into() });
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.status().contains("forgejo-3 restarting"));
    let lines = [
        "10:41:02 forgejo exited 1: database is locked",
        "10:41:03 forgejo restarting, 2 of 5 within 10 min",
        "10:41:05 forgejo listening on 127.0.0.1:8368",
    ];
    for line in lines {
        tui.boxed(&mut world, BoxLine::Log { line: line.as_bytes().into() });
    }
    tui.type_words(&mut world, "/log");
    tui.press(&mut world, vec![Key::Enter]);
    tui.run(&mut world, fixtures::WITHIN);
    let text = tui.text();
    let mut note = text.split_once(" /log\n").expect("the /log note").1;
    for line in lines {
        note = note.split_once(line).expect("the log's lines are oldest first").1;
    }
    tui.boxed(&mut world, BoxLine::Running { step: b"forgejo-3".as_slice().into() });
    tui.run(&mut world, fixtures::WITHIN);
    assert!(!tui.status().contains("restarting"));
    tui.boxed(&mut world, BoxLine::GaveUp { step: b"forgejo-3".as_slice().into() });
    tui.run(&mut world, fixtures::WITHIN);
    assert!(tui.text().contains("The box gave up: forgejo-3. /log shows why; Ctrl+C closes."));
    tui.stage(&mut world, temper_client_tui::BoxStage::Ended { end: temper_client_tui::BoxEnd::Exited { code: 1 } });
    assert!(!tui.closed, "the front stays open after the box gives up and ends");
    tui.press(&mut world, vec![Key::Control(b'c')]);
    assert!(tui.closed, "Ctrl+C closes at once after give-up");
    assert!(!tui.text().contains("Stop the box?"), "nothing asks after give-up");
    file_secret_is_in_no_frame(&tui, &handed);
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
    assert_eq!(tui.opens.len(), 0);
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
    assert_eq!(tui.opens.len(), 0);
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
