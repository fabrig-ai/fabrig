//! A welcome trial on the hosted worker and fake Claude Code
//! (temper/docs/design/domain/trial.md, 2, 6; temper/docs/design/domain/testing.md, 2.4).
use temper_real_loop_world::{Exit, Scratch, claude, forge, run_forge_on, terminal};

const SOURCE: &str = "claude-owner";

#[test]
fn a_trial_runs_through_the_worker_and_the_fake_model() {
    let scratch = Scratch::new("model-check");
    let story = smith_fake_claude::Story {
        turns: vec![smith_fake_claude::Turn::new(vec![smith_fake_claude::Step::Text("Yes".into())])],
        ..smith_fake_claude::Story::default()
    };
    claude::source(&scratch, SOURCE, &story).expect("signed-in fake source");
    let ran = run_forge_on(
        scratch,
        forge::Fixture {
            configure: |document, _, _| {
                document["application"]["welcome"] = true.into();
                // The trial's 64-byte report and the period/pool pairs for
                // project 0, the seed and Create's project fit this story.
                let limits = &mut document["engine"]["limits"]["overrides"];
                limits["tasks.result_bytes"] = 64.into();
                limits["tasks.funders"] = 6.into();
                limits["people.projects"] = 3.into();
                limits["people.holdings"] = 2.into();
                limits["authority.projects"] = 2.into();
            },
            ..forge::fixture()
        },
        |_| Box::new([]),
        || {
            vec![
                terminal::open_welcome(),
                terminal::see(b"Use your Claude subscription"),
                terminal::type_on(b"Use your Claude subscription", b""),
                terminal::see("I’ve signed in".as_bytes()),
                terminal::type_on("I’ve signed in".as_bytes(), b""),
                terminal::see(b"Create your project"),
                terminal::see(b"answered in "),
                terminal::see(b"you@example.com"),
                terminal::type_on(b"Create your project", b""),
                // To begin has its ready headline; the everyday headline follows Skip.
                terminal::see(b"Optional Skip"),
                terminal::type_on(b"Optional Skip", b""),
                terminal::see(b"Nothing needs you."),
                terminal::see(b"No chats yet."),
                // Work has no Chats filter until a chat starts work.
                terminal::type_on(b"Work", b""),
                terminal::see(b"Nothing in flight yet."),
                terminal::close(),
            ]
        },
    );
    let read = ran.terminal();
    assert_eq!(read.exit, Some(Exit::Code(0)), "terminal drained");
    let ready = read.pages.iter().find(|page| page.commands == 2).expect("Ready follows the press");
    let text = std::str::from_utf8(&ready.bytes).expect("Ready text");
    assert!(text.contains("Ready") && text.contains("Create your project"), "{text}");
    assert!(!text.contains("Almost ready"), "{text}");
    let answered = read
        .pages
        .iter()
        .find_map(|page| {
            let text = std::str::from_utf8(&page.bytes).expect("printed text");
            text.split_once("answered in ").map(|(_, after)| after.to_owned())
        })
        .expect("the try answered");
    let seconds = answered.split_whitespace().next().expect("elapsed seconds").parse::<u64>().expect("seconds");
    assert!(seconds >= 1 && answered.starts_with(&format!("{seconds} s")), "{answered}");
    let home = read
        .pages
        .iter()
        .find(|page| {
            page.commands == 4 && page.bytes.windows(b"No chats yet.".len()).any(|part| part == b"No chats yet.")
        })
        .expect("Home has read its chats");
    let text = std::str::from_utf8(&home.bytes).expect("Home text");
    assert!(text.contains("Nothing needs you."), "Home has no needs: {text}");
    assert!(ran.llm().is_empty(), "the program's model answers, with no direct-key call");
    let bytes = std::fs::read(ran.path().join("programs").join(SOURCE).join(smith_fake_claude::OBSERVATIONS_FILE))
        .expect("fake program observations");
    let prints = smith_fake_claude::observations(&bytes);
    assert!(
        prints.iter().any(|seen| seen.arguments.windows(2).any(|args| args == ["--model", "claude-opus-5-5"])),
        "the fake program runs a stock model name"
    );
    assert_eq!(prints.iter().filter(|seen| seen.arguments.first().is_some_and(|arg| arg == "-p")).count(), 1);
    assert_eq!(
        smith_fake_claude::selections(&bytes).first(),
        Some(&smith_fake_claude::Seen::Selected { script: smith_fake_claude::Selection::Otherwise, turn: 0 })
    );
}
