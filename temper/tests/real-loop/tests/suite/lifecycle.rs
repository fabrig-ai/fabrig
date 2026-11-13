//! Standalone startup and termination share the real kernel and checked ownership.
#[test]
fn a_started_service_drains_on_termination() {
    use temper_real_loop_world::{End, run, scripts};
    let ran = run(|| scripts::chat_scripts(1), Vec::new);
    assert_eq!(ran.served(), Some(&End::Engine(temper_service::EngineEnd::Drained)), "link {:?}", ran.link());
}

#[test]
fn a_service_stops_its_held_claude_turn_before_ending() {
    use smith_fake_claude::{Seen, Step, Story, Turn};
    use temper_real_loop_world::{
        End, Exit, Scratch, argv,
        calls::{D1, call, ends_leaving, task},
        claude,
        ending::{Left, Leftover},
        forge, run_forge_on,
    };

    let scratch = Scratch::new("stop-program");
    let story = Story { turns: vec![Turn::new(vec![Step::Hang])], ..Story::default() };
    claude::source(&scratch, claude::SOURCE, &story).expect("signed-in fake source");
    let ran = run_forge_on(
        scratch,
        forge::fixture(),
        |_| Box::new([]),
        || {
            let mut steps = claude::sign_in();
            steps.extend([
                call(D1, argv!["chat", "start", "--project", "1", "--key", "held-chat", "Hello"]),
                call(D1, argv!["task", "show", task(7), "--until", "phase:running"]),
                temper_real_loop_world::calls::Step::ClaudeTurn { source: claude::SOURCE },
                ends_leaving(&[Leftover {
                    task: 1,
                    may_be: &[Left::HostedActive, Left::Running],
                    why: "the held Claude Code turn the stop ends (protocol/README.md, 6)",
                }]),
            ]);
            steps
        },
    );
    for called in ran.calls() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "{called:?}");
    }
    assert_eq!(ran.served(), Some(&End::Engine(temper_service::EngineEnd::Drained)));
    let bytes =
        std::fs::read(ran.path().join("programs").join(claude::SOURCE).join(smith_fake_claude::OBSERVATIONS_FILE))
            .expect("fake observations");
    let seen = smith_fake_claude::seen(&bytes);
    assert!(seen.contains(&Seen::Interrupted), "agent's cancel must interrupt its program: {seen:?}");
    assert!(claude::ended(&ran), "no hosted Claude survives serve");
}
