//! A welcome trial on the hosted worker and fake Claude Code
//! (temper/docs/design/domain/trial.md, 2, 6; temper/docs/design/testing/real-loop.md).
use temper_real_loop_world::{
    Exit, Scratch,
    calls::{self, D1},
    claude, fixtures, forge, run_forge_on, terminal,
};

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
                // The trial's 64-byte report and the fixture's project and
                // period/pool room fit this story.
                document.jig_over("tasks.result_bytes", 64);
                document.jig_over("tasks.funders", 6);
                document.jig_over("people.projects", 3);
                document.jig_over("people.holdings", 2);
                document.jig_over("authority.projects", 2);
            },
            ..forge::fixture()
        },
        |_| Box::new([]),
        || {
            let mut steps = vec![calls::sign_in(D1)];
            steps.extend(fixtures::change_settings(
                D1,
                "show-welcome",
                serde_json::json!([
                    {"setting":{"kind":"welcome"},"set":{"shown":true}}
                ]),
            ));
            steps.extend([
                terminal::open_welcome(),
                terminal::see(b"Use your Claude subscription"),
                terminal::type_on(b"Use your Claude subscription", b""),
                terminal::see("I’ve signed in".as_bytes()),
                terminal::type_on("I’ve signed in".as_bytes(), b""),
                terminal::see(b"Create your project"),
                terminal::see(b"answered in "),
                terminal::see(b"you@example.com"),
                terminal::close(),
                // Ready observes the answer before the trial finishes closing.
                calls::trial_ended(),
            ]);
            steps
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
